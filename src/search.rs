use crate::database::StringId;
use crate::measures::Measure;
use crate::Database;
use lasso::Spur;
use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum SearchError {
    #[error(
        "Search threshold alpha must be between 0.0 (exclusive) and 1.0 (inclusive), but was {0}"
    )]
    InvalidThreshold(f64),
}

pub struct Searcher<'db, M: Measure> {
    db: &'db dyn Database,
    measure: M,
}

impl<'db, M: Measure> Searcher<'db, M> {
    pub fn new(db: &'db dyn Database, measure: M) -> Self {
        Self { db, measure }
    }

    pub fn search<'a>(
        &'a self,
        query_string: &str,
        alpha: f64,
    ) -> Result<Vec<&'a str>, SearchError> {
        let (candidate_ids, _) = self.search_candidates(query_string, alpha)?;

        let mut results: Vec<&'a str> = candidate_ids
            .par_iter()
            .filter_map(|&id| self.db.get_string(id))
            .collect();

        results.sort_unstable();
        Ok(results)
    }

    pub fn ranked_search<'a>(
        &'a self,
        query_string: &str,
        alpha: f64,
    ) -> Result<Vec<(&'a str, f64)>, SearchError> {
        let (candidate_ids, query_features) = self.search_candidates(query_string, alpha)?;

        let mut results_with_scores: Vec<(&'a str, f64)> = candidate_ids
            .par_iter()
            .filter_map(|&id| {
                if let (Some(candidate_str), Some(candidate_features)) =
                    (self.db.get_string(id), self.db.get_features(id))
                {
                    let score = self.measure.similarity(&query_features, candidate_features);
                    if score >= alpha {
                        Some((candidate_str, score))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        results_with_scores.sort_unstable_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(b.0))
        });

        Ok(results_with_scores)
    }

    fn search_candidates(
        &self,
        query_string: &str,
        alpha: f64,
    ) -> Result<(FxHashSet<StringId>, Vec<Spur>), SearchError> {
        if !(alpha > 0.0 && alpha <= 1.0) {
            return Err(SearchError::InvalidThreshold(alpha));
        }

        // Create a binding for the Arc to extend its lifetime.
        let interner_arc = self.db.interner();
        // Now, lock the long-lived Arc. The guard's lifetime is valid.
        let mut interner = interner_arc.lock().unwrap();

        let mut query_features = self
            .db
            .feature_extractor()
            .features(query_string, &mut interner);
        // FIX: If features are sorted by insertions. this sort and dedup maybe redundant?
        query_features.sort_unstable();
        query_features.dedup();

        let candidate_ids = self.search_for_ids(&query_features, alpha);
        Ok((candidate_ids, query_features))
    }

    fn search_for_ids(&self, query_features: &[Spur], alpha: f64) -> FxHashSet<StringId> {
        let query_size = query_features.len();
        if query_size == 0 {
            return FxHashSet::default();
        }

        let min_feat_size = self.measure.min_feature_size(query_size, alpha);
        let max_feat_size = self.measure.max_feature_size(query_size, alpha, self.db);

        (min_feat_size..=max_feat_size)
            .into_par_iter()
            .flat_map(|candidate_size| {
                let tau =
                    self.measure
                        .minimum_common_feature_count(query_size, candidate_size, alpha);
                self.overlap_join(query_features, tau, candidate_size)
            })
            // FIX: Investigate if this multistaged conversion is even necessary
            .collect::<Vec<_>>()
            .into_iter()
            .collect()
    }

    fn overlap_join(
        &self,
        query_features: &[Spur],
        tau: usize,
        candidate_size: usize,
    ) -> Vec<StringId> {
        // FIX: Hmmm, maybe feature_indices and results can be pre-allocated?
        let mut feature_indices: Vec<usize> = (0..query_features.len()).collect();
        // TODO: Can this sorting logic be improved?
        feature_indices.sort_unstable_by_key(|&i| {
            self.db
                .lookup_strings(candidate_size, query_features[i])
                .map_or(usize::MAX, |s| s.len())
        });

        let mut candidate_counts: FxHashMap<StringId, usize> = FxHashMap::default();
        let mut results = Vec::new();
        let q_len = query_features.len();

        for &idx in &feature_indices[..q_len.saturating_sub(tau) + 1] {
            let feature = query_features[idx];
            if let Some(ids) = self.db.lookup_strings(candidate_size, feature) {
                for &id in ids {
                    *candidate_counts.entry(id).or_insert(0) += 1;
                }
            }
        }

        if tau == 1 {
            return candidate_counts.keys().cloned().collect();
        }

        for (&candidate_id, &initial_count) in &candidate_counts {
            let mut count = initial_count;
            if count >= tau {
                results.push(candidate_id);
                continue;
            }
            // TODO: Should early termination happen if ==> count + (q_len - (q_len.saturating_sub(tau) + 1)) < tau
            for (i, &idx) in feature_indices
                .iter()
                .enumerate()
                .skip(q_len.saturating_sub(tau) + 1)
            {
                let feature = query_features[idx];
                if let Some(ids) = self.db.lookup_strings(candidate_size, feature) {
                    if ids.contains(&candidate_id) {
                        count += 1;
                    }
                }

                if count >= tau {
                    results.push(candidate_id);
                    break;
                }

                let remaining_features = q_len - 1 - i;
                if count + remaining_features < tau {
                    break;
                }
            }
        }

        results
    }
}

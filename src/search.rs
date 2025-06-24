use crate::database::StringId;
use crate::measures::Measure;
use crate::Database;
use ahash::{AHashMap, AHashSet};
use rayon::prelude::*;
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

    /// Performs a search and returns all matching strings without ranking them by similarity score.
    pub fn search(&self, query_string: &str, alpha: f64) -> Result<Vec<String>, SearchError> {
        let candidate_ids = self.search_candidates(query_string, alpha)?;

        let mut results: Vec<String> = candidate_ids
            .par_iter()
            .filter_map(|&id| self.db.get_string(id).map(|s| s.to_string()))
            .collect();

        // Sort for deterministic output
        results.sort_unstable();

        Ok(results)
    }

    /// Performs a search and returns matching strings ranked by their similarity score.
    pub fn ranked_search(
        &self,
        query_string: &str,
        alpha: f64,
    ) -> Result<Vec<(String, f64)>, SearchError> {
        let candidate_ids = self.search_candidates(query_string, alpha)?;
        let query_features = self.db.feature_extractor().features(query_string);

        let mut results_with_scores: Vec<(String, f64)> = candidate_ids
            .par_iter()
            .filter_map(|&id| {
                if let (Some(candidate_str), Some(candidate_features)) =
                    (self.db.get_string(id), self.db.get_features(id))
                {
                    let score = self.measure.similarity(&query_features, candidate_features);
                    if score >= alpha {
                        Some((candidate_str.to_string(), score))
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
                .then_with(|| a.0.cmp(&b.0))
        });

        Ok(results_with_scores)
    }

    /// Private helper to get all candidate IDs that meet the threshold criteria.
    fn search_candidates(
        &self,
        query_string: &str,
        alpha: f64,
    ) -> Result<AHashSet<StringId>, SearchError> {
        if !(alpha > 0.0 && alpha <= 1.0) {
            return Err(SearchError::InvalidThreshold(alpha));
        }
        let query_features = self.db.feature_extractor().features(query_string);
        Ok(self.search_for_ids(&query_features, alpha))
    }

    fn search_for_ids(&self, query_features: &[String], alpha: f64) -> AHashSet<StringId> {
        let query_size = query_features.len();
        if query_size == 0 {
            return AHashSet::default();
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
            .collect::<Vec<_>>()
            .into_iter()
            .collect()
    }

    fn overlap_join(
        &self,
        query_features: &[String],
        tau: usize,
        candidate_size: usize,
    ) -> Vec<StringId> {
        let mut sorted_features = query_features.to_vec();

        sorted_features.sort_unstable_by_key(|f| {
            self.db
                .lookup_strings(candidate_size, f)
                .map_or(usize::MAX, |s| s.len())
        });

        let mut candidate_counts: AHashMap<StringId, usize> = AHashMap::default();
        let mut results = Vec::new();
        let q_len = query_features.len();

        for feature in &sorted_features[..q_len.saturating_sub(tau) + 1] {
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

            for (i, feature) in sorted_features
                .iter()
                .enumerate()
                .skip(q_len.saturating_sub(tau) + 1)
            {
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

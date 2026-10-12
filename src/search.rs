use crate::Database;
use crate::database::StringId;
use crate::measures::Measure;
use lasso::Spur;
use rayon::prelude::*;
use rustc_hash::FxHashSet;
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

    pub fn search(&self, query_string: &str, alpha: f64) -> Result<Vec<&str>, SearchError> {
        let candidates = self.search_candidates(query_string, alpha)?;
        let mut results: Vec<_> = candidates
            .into_iter()
            .filter_map(|(id, _)| self.db.get_string(id))
            .collect();
        results.sort_unstable();
        Ok(results)
    }

    pub fn ranked_search(
        &self,
        query_string: &str,
        alpha: f64,
    ) -> Result<Vec<(&str, f64)>, SearchError> {
        let candidates = self.search_candidates(query_string, alpha)?;
        let mut results: Vec<_> = candidates
            .into_iter()
            .filter_map(|(id, score)| {
                let text = self.db.get_string(id)?;
                Some((text, score))
            })
            .collect();
        results.sort_unstable_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(b.0)));
        Ok(results)
    }

    fn search_candidates(
        &self,
        text: &str,
        alpha: f64,
    ) -> Result<Vec<(StringId, f64)>, SearchError> {
        if !(alpha > 0.0 && alpha <= 1.0) {
            return Err(SearchError::InvalidThreshold(alpha));
        }
        let interner = self.db.interner();
        let mut features = Vec::new();
        // Unknown features contribute to the denominator, but never to an overlap.
        let mut unknown = FxHashSet::default();
        self.db
            .feature_extractor()
            .visit_features(text, &mut |feature| {
                if let Some(id) = interner.get(feature) {
                    features.push(id);
                } else {
                    unknown.insert(feature.to_owned());
                }
            });
        features.sort_unstable();
        features.dedup();
        Ok(self.search_for_ids(&features, features.len() + unknown.len(), alpha))
    }

    fn search_for_ids(
        &self,
        features: &[Spur],
        query_size: usize,
        alpha: f64,
    ) -> Vec<(StringId, f64)> {
        if features.is_empty() {
            return Vec::new();
        }
        let min = self.measure.min_feature_size(query_size, alpha);
        let max = self.measure.max_feature_size(query_size, alpha, self.db);
        let sizes = self.db.feature_sizes();
        let start = sizes.partition_point(|&size| size < min);
        let end = sizes.partition_point(|&size| size <= max);
        if start >= end {
            return Vec::new();
        }
        let sizes = &sizes[start..end];
        let join = |&size: &usize| {
            let tau = self
                .measure
                .minimum_common_feature_count(query_size, size, alpha)
                .max(1);
            if tau > features.len() {
                return Vec::new();
            }
            self.overlap_join(features, query_size, tau, size, alpha)
        };
        // Small joins cost less than scheduling a Rayon task. Parallelize only large
        // databases; independent callers can always search concurrently.
        if self.db.total_strings() >= 100_000 && sizes.len() > 4 {
            sizes.par_iter().flat_map_iter(join).collect()
        } else {
            sizes.iter().flat_map(join).collect()
        }
    }

    fn overlap_join(
        &self,
        features: &[Spur],
        query_size: usize,
        tau: usize,
        size: usize,
        alpha: f64,
    ) -> Vec<(StringId, f64)> {
        if tau == 0 || tau > features.len() {
            return Vec::new();
        }
        let mut postings: Vec<_> = features
            .iter()
            .filter_map(|&feature| self.db.lookup_strings(size, feature))
            .filter(|ids| !ids.is_empty())
            .collect();
        if postings.len() < tau {
            return Vec::new();
        }
        postings.sort_unstable_by_key(|ids| ids.len());
        // Every match must occur in this prefix: only tau - 1 lists remain outside it.
        // Missing query features cannot contribute and need not enlarge the prefix.
        let prefix_len = postings.len() - tau + 1;
        let mut candidates = Vec::new();
        for ids in &postings[..prefix_len] {
            candidates.extend_from_slice(ids);
        }
        candidates.sort_unstable();
        candidates
            .chunk_by(|a, b| a == b)
            .filter_map(|occurrences| {
                let id = occurrences[0];
                let mut count = occurrences.len();
                for (index, ids) in postings[prefix_len..].iter().enumerate() {
                    count += usize::from(ids.binary_search(&id).is_ok());
                    if count + postings.len() - prefix_len - index - 1 < tau {
                        return None;
                    }
                }
                let score = self.measure.similarity_from_counts(query_size, size, count);
                (score >= alpha).then_some((id, score))
            })
            .collect()
    }
}

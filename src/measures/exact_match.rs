use crate::SimStringDB;
use std::collections::HashSet;

use super::SimilarityMeasure;

pub struct ExactMatch;

impl Default for ExactMatch {
    fn default() -> Self {
        Self::new()
    }
}

impl ExactMatch {
    pub fn new() -> Self {
        ExactMatch
    }
}

impl SimilarityMeasure for ExactMatch {
    fn minimum_feature_size(&self, query_size: i64, _alpha: f64) -> i64 {
        query_size
    }

    fn maximum_feature_size<TMeasure: SimilarityMeasure>(
        &self,
        _db: &impl SimStringDB<TMeasure>,
        query_size: i64,
        _alpha: f64,
    ) -> i64 {
        query_size
    }

    fn similarity_score(&self, x: &[(String, i32)], y: &[(String, i32)]) -> f64 {
        let set_x: HashSet<_> = x.iter().collect();
        let set_y: HashSet<_> = y.iter().collect();

        if set_x == set_y {
            1.0
        } else {
            0.0
        }
    }

    fn minimum_overlap(&self, query_size: i64, _candidate_size: i64, _alphaa: f64) -> i64 {
        query_size
    }
}

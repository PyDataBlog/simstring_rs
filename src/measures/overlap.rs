use std::cmp::min;
use std::collections::HashSet;

use super::SimilarityMeasure;

pub struct Overlap;

impl Default for Overlap {
    fn default() -> Self {
        Self::new()
    }
}

impl Overlap {
    pub fn new() -> Self {
        Overlap
    }
}

impl SimilarityMeasure for Overlap {
    fn minimum_feature_size(&self, _query_size: i64, _alpha: f64) -> i64 {
        1
    }

    fn maximum_feature_size(&self, _query_size: i64, _alpha: f64) -> i64 {
        todo!()
    }

    fn similarity_score(&self, x: &[i64], y: &[i64]) -> f64 {
        let set_x: HashSet<_> = x.iter().collect();
        let set_y: HashSet<_> = x.iter().collect();

        let intersection_count = set_x.intersection(&set_y).count() as f64;
        let denominator = min(x.len(), y.len()) as f64;
        intersection_count / denominator
    }

    fn minimum_overlap(&self, query_size: i64, candidate_size: i64, alpha: f64) -> i64 {
        (alpha * min(query_size, candidate_size) as f64).ceil() as i64
    }
}

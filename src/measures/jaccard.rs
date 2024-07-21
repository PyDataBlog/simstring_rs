use std::collections::HashSet;

use super::SimilarityMeasure;

pub struct Jaccard;

impl Default for Jaccard {
    fn default() -> Self {
        Self::new()
    }
}

impl Jaccard {
    pub fn new() -> Self {
        Jaccard
    }
}

impl SimilarityMeasure for Jaccard {
    fn minimum_feature_size(&self, query_size: i64, alpha: f64) -> i64 {
        (alpha * query_size as f64).ceil() as i64
    }

    fn maximum_feature_size(&self, query_size: i64, alpha: f64) -> i64 {
        (query_size as f64 / alpha).floor() as i64
    }

    fn similarity_score(&self, x: &[i64], y: &[i64]) -> f64 {
        let set_x: HashSet<_> = x.iter().collect();
        let set_y: HashSet<_> = y.iter().collect();

        let intersection_count = set_x.intersection(&set_y).count() as f64;
        let union_intersection_count = set_x.union(&set_y).count() as f64;
        intersection_count / union_intersection_count
    }

    fn minimum_overlap(&self, query_size: i64, candidate_size: i64, alpha: f64) -> i64 {
        ((alpha * (query_size + candidate_size) as f64) / (1. + alpha)).ceil() as i64
    }
}

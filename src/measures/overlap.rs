use crate::database::SimStringDB;
use crate::measures::SimilarityMeasure;
use std::cmp::min;
use std::collections::HashSet;

pub struct Overlap;

impl Overlap {
    pub fn new() -> Self {
        Overlap
    }
}

impl Default for Overlap {
    fn default() -> Self {
        Self::new()
    }
}

impl SimilarityMeasure for Overlap {
    fn minimum_feature_size(&self, _query_size: i64, _alpha: f64) -> i64 {
        1
    }

    fn maximum_feature_size<TMeasure: SimilarityMeasure>(
        &self,
        db: &impl SimStringDB<TMeasure>,
        _query_size: i64,
        _alpha: f64,
    ) -> i64 {
        let max_size = db.get_max_feature_size();
        min(i64::MAX, max_size as i64)
    }

    fn similarity_score(&self, x: &[(String, i32)], y: &[(String, i32)]) -> f64 {
        let set_x: HashSet<_> = x.iter().collect();
        let set_y: HashSet<_> = y.iter().collect();

        let intersection_count = set_x.intersection(&set_y).count() as f64;
        let denominator = min(x.len(), y.len()) as f64;
        intersection_count / denominator
    }

    fn minimum_overlap(&self, query_size: i64, candidate_size: i64, alpha: f64) -> i64 {
        (alpha * min(query_size, candidate_size) as f64).ceil() as i64
    }
}

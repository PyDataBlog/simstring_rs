use super::SimilarityMeasure;
use std::collections::HashSet;

pub struct Cosine;

impl Default for Cosine {
    fn default() -> Self {
        Self::new()
    }
}

impl Cosine {
    pub fn new() -> Self {
        Cosine
    }
}

impl SimilarityMeasure for Cosine {
    fn minimum_feature_size(&self, query_size: i64, alpha: f64) -> i64 {
        (alpha * alpha * query_size as f64).ceil() as i64
    }

    fn maximum_feature_size(&self, query_size: i64, alpha: f64) -> i64 {
        (query_size as f64 / (alpha * alpha)).floor() as i64
    }

    fn similarity_score(&self, x: &Vec<i64>, y: &Vec<i64>) -> f64 {
        let set_x: HashSet<_> = x.iter().collect();
        let set_y: HashSet<_> = y.iter().collect();
        let intersection_count = set_x.intersection(&set_y).count() as f64;
        let denominator = ((set_x.len() * set_y.len()) as f64).sqrt();

        intersection_count / denominator
    }

    fn minimum_overlap(&self, query_size: i64, candidate_size: i64, alpha: f64) -> i64 {
        (alpha * (query_size as f64 * candidate_size as f64).sqrt()).ceil() as i64
    }
}

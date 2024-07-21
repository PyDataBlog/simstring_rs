use super::SimilarityMeasure;
use std::collections::HashSet;

pub struct Dice;

impl Default for Dice {
    fn default() -> Self {
        Self::new()
    }
}

impl Dice {
    pub fn new() -> Self {
        Dice
    }
}

impl SimilarityMeasure for Dice {
    fn minimum_feature_size(&self, query_size: i64, alpha: f64) -> i64 {
        ((alpha / (2. - alpha)) * query_size as f64).ceil() as i64
    }

    fn maximum_feature_size(&self, query_size: i64, alpha: f64) -> i64 {
        (((2. - alpha) / alpha) * query_size as f64).floor() as i64
    }

    fn similarity_score(&self, x: &[i64], y: &[i64]) -> f64 {
        let set_x: HashSet<_> = x.iter().collect();
        let set_y: HashSet<_> = y.iter().collect();

        let intersection_count = set_x.intersection(&set_y).count() as f64;

        let denominator = (set_x.len() + set_y.len()) as f64;

        2. * (intersection_count / denominator)
    }

    fn minimum_overlap(&self, query_size: i64, candidate_size: i64, alpha: f64) -> i64 {
        (0.5 * alpha * query_size as f64 * candidate_size as f64).ceil() as i64
    }
}

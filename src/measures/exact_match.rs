use super::Measure;
use crate::database::Database;

#[derive(Default, Clone, Copy)]
pub struct ExactMatch;

impl Measure for ExactMatch {
    fn min_feature_size(&self, query_size: usize, _alpha: f64) -> usize {
        query_size
    }

    fn max_feature_size(&self, query_size: usize, _alpha: f64, _db: &dyn Database) -> usize {
        query_size
    }

    fn minimum_common_feature_count(
        &self,
        query_size: usize,
        _y_size: usize,
        _alpha: f64,
    ) -> usize {
        query_size
    }

    fn similarity_from_counts(&self, x_len: usize, y_len: usize, common: usize) -> f64 {
        if x_len == y_len && common == x_len {
            1.0
        } else {
            0.0
        }
    }
}

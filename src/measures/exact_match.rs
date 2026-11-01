use super::Measure;
use crate::database::Database;

/// Exact membership check rather than a graded similarity: `1.0` when the two
/// feature sets are of equal size and fully shared (i.e. identical), `0.0`
/// otherwise.
///
/// Search still uses threshold-aware bounds — min/max feature size both pin
/// to the query's size and any candidate must share *all* of the query's
/// features — so recall behavior is identical to set equality after the
/// extractor's qualification rules.
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

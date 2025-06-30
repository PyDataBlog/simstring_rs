use super::Measure;
use crate::database::Database;
use ahash::AHashSet;
use lasso::Spur;

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

    fn similarity(&self, x: &[Spur], y: &[Spur]) -> f64 {
        if x.len() != y.len() {
            return 0.0;
        }
        let x_set: AHashSet<_> = x.iter().collect();
        let y_set: AHashSet<_> = y.iter().collect();

        if x_set == y_set {
            1.0
        } else {
            0.0
        }
    }
}

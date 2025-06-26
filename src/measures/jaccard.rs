use super::Measure;
use crate::database::Database;
use ahash::AHashSet;
use lasso::Spur;

#[derive(Default, Clone, Copy)]
pub struct Jaccard;

impl Measure for Jaccard {
    fn min_feature_size(&self, query_size: usize, alpha: f64) -> usize {
        (alpha * query_size as f64).ceil() as usize
    }

    fn max_feature_size(&self, query_size: usize, alpha: f64, _db: &dyn Database) -> usize {
        (query_size as f64 / alpha).floor() as usize
    }

    fn minimum_common_feature_count(&self, query_size: usize, y_size: usize, alpha: f64) -> usize {
        if alpha == -1.0 {
            return 0;
        }
        ((alpha * (query_size as f64 + y_size as f64)) / (1.0 + alpha)).ceil() as usize
    }

    fn similarity(&self, x: &[Spur], y: &[Spur]) -> f64 {
        let x_set: AHashSet<_> = x.iter().collect();
        let y_set: AHashSet<_> = y.iter().collect();

        if x_set.is_empty() && y_set.is_empty() {
            return 1.0;
        }
        if x_set.is_empty() || y_set.is_empty() {
            return 0.0;
        }

        let intersection_size = x_set.intersection(&y_set).count() as f64;
        let union_size = x_set.union(&y_set).count() as f64;

        if union_size == 0.0 {
            0.0
        } else {
            intersection_size / union_size
        }
    }
}

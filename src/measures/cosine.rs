use super::Measure;
use crate::database::Database;
use ahash::AHashSet;

#[derive(Default, Clone, Copy)]
pub struct Cosine;

impl Measure for Cosine {
    fn min_feature_size(&self, query_size: usize, alpha: f64) -> usize {
        (alpha * alpha * query_size as f64).ceil() as usize
    }

    fn max_feature_size(&self, query_size: usize, alpha: f64, db: &dyn Database) -> usize {
        // Avoid division by zero; use the max length in the DB as a sensible upper bound.
        if alpha == 0.0 {
            return db.max_feature_len();
        }
        let calculated_max = (query_size as f64 / (alpha * alpha)).floor() as usize;
        // Cap the max size to what's actually in the DB as an optimization.
        std::cmp::min(calculated_max, db.max_feature_len())
    }

    fn minimum_common_feature_count(&self, query_size: usize, y_size: usize, alpha: f64) -> usize {
        (alpha * (query_size as f64 * y_size as f64).sqrt()).ceil() as usize
    }

    fn similarity(&self, x: &[String], y: &[String]) -> f64 {
        let x_set: AHashSet<_> = x.iter().collect();
        let y_set: AHashSet<_> = y.iter().collect();

        if x_set.is_empty() || y_set.is_empty() {
            return 0.0;
        }

        let intersection_size = x_set.intersection(&y_set).count();
        let denominator = (x_set.len() as f64 * y_set.len() as f64).sqrt();

        if denominator == 0.0 {
            0.0
        } else {
            intersection_size as f64 / denominator
        }
    }
}

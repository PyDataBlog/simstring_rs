use super::Measure;
use crate::database::Database;
use ahash::AHashSet;
use lasso::Spur;
use std::cmp;

#[derive(Default, Clone, Copy)]
pub struct Overlap;

impl Measure for Overlap {
    fn min_feature_size(&self, _query_size: usize, _alpha: f64) -> usize {
        1
    }

    fn max_feature_size(&self, _query_size: usize, _alpha: f64, db: &dyn Database) -> usize {
        db.max_feature_len()
    }

    fn minimum_common_feature_count(&self, query_size: usize, y_size: usize, alpha: f64) -> usize {
        (alpha * cmp::min(query_size, y_size) as f64).ceil() as usize
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
        let denominator = cmp::min(x_set.len(), y_set.len()) as f64;

        if denominator == 0.0 {
            0.0
        } else {
            intersection_size / denominator
        }
    }
}

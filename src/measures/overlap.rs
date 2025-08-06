use super::{compute_intersection_size, Measure};
use crate::database::Database;
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
        if x.is_empty() && y.is_empty() {
            return 1.0;
        }
        if x.is_empty() || y.is_empty() {
            return 0.0;
        }

        let intersection_size = compute_intersection_size(x, y);
        let denominator = cmp::min(x.len(), y.len()) as f64;

        if denominator == 0.0 {
            0.0
        } else {
            intersection_size as f64 / denominator
        }
    }
}

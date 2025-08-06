use super::{compute_intersection_size, Measure};
use crate::database::Database;
use lasso::Spur;

#[derive(Default, Clone, Copy)]
pub struct Dice;

impl Measure for Dice {
    fn min_feature_size(&self, query_size: usize, alpha: f64) -> usize {
        if alpha > 2.0 {
            return 0;
        }
        ((alpha / (2.0 - alpha)) * query_size as f64).ceil() as usize
    }

    fn max_feature_size(&self, query_size: usize, alpha: f64, db: &dyn Database) -> usize {
        if alpha == 0.0 {
            return db.max_feature_len();
        }
        let calculated_max = (((2.0 - alpha) / alpha) * query_size as f64).floor() as usize;
        std::cmp::min(calculated_max, db.max_feature_len())
    }

    fn minimum_common_feature_count(&self, query_size: usize, y_size: usize, alpha: f64) -> usize {
        (0.5 * alpha * (query_size as f64 + y_size as f64)).ceil() as usize
    }

    fn similarity(&self, x: &[Spur], y: &[Spur]) -> f64 {
        if x.is_empty() && y.is_empty() {
            return 1.0;
        }
        if x.is_empty() || y.is_empty() {
            return 0.0;
        }

        let intersection_size = compute_intersection_size(x, y);
        let denominator = (x.len() + y.len()) as f64;

        if denominator == 0.0 {
            0.0
        } else {
            2.0 * intersection_size as f64 / denominator
        }
    }
}

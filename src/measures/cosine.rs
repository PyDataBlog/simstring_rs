use super::Measure;
use crate::database::Database;

use lasso::Spur;

#[derive(Default, Clone, Copy)]
pub struct Cosine;

impl Measure for Cosine {
    fn min_feature_size(&self, query_size: usize, alpha: f64) -> usize {
        (alpha * alpha * query_size as f64).ceil() as usize
    }

    fn max_feature_size(&self, query_size: usize, alpha: f64, db: &dyn Database) -> usize {
        if alpha == 0.0 {
            return db.max_feature_len();
        }
        let calculated_max = (query_size as f64 / (alpha * alpha)).floor() as usize;
        std::cmp::min(calculated_max, db.max_feature_len())
    }

    fn minimum_common_feature_count(&self, query_size: usize, y_size: usize, alpha: f64) -> usize {
        (alpha * (query_size as f64 * y_size as f64).sqrt()).ceil() as usize
    }

    fn similarity(&self, x: &[Spur], y: &[Spur]) -> f64 {
        if x.is_empty() || y.is_empty() {
            return 0.0;
        }

        let mut intersection_size = 0;
        let mut i = 0;
        let mut j = 0;

        while i < x.len() && j < y.len() {
            if x[i] == y[j] {
                intersection_size += 1;
                i += 1;
                j += 1;
            } else if x[i] < y[j] {
                i += 1;
            } else {
                j += 1;
            }
        }

        let denominator = (x.len() as f64 * y.len() as f64).sqrt();

        if denominator == 0.0 {
            0.0
        } else {
            intersection_size as f64 / denominator
        }
    }
}

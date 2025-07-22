use super::Measure;
use crate::database::Database;

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
        if x.is_empty() && y.is_empty() {
            return 1.0;
        }
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

        let union_size = (x.len() + y.len() - intersection_size) as f64;

        if union_size == 0.0 {
            0.0
        } else {
            intersection_size as f64 / union_size
        }
    }
}

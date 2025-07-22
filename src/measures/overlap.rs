use super::Measure;
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

        let denominator = cmp::min(x.len(), y.len()) as f64;

        if denominator == 0.0 {
            0.0
        } else {
            intersection_size as f64 / denominator
        }
    }
}

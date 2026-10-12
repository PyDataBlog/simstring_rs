use super::Measure;
use crate::database::Database;

#[derive(Default, Clone, Copy)]
pub struct Jaccard;

impl Measure for Jaccard {
    fn min_feature_size(&self, query_size: usize, alpha: f64) -> usize {
        (alpha.next_down() * query_size as f64).next_down().ceil() as usize
    }

    fn max_feature_size(&self, query_size: usize, alpha: f64, _db: &dyn Database) -> usize {
        (query_size as f64 / alpha.next_down()).next_up().floor() as usize
    }

    fn minimum_common_feature_count(&self, query_size: usize, y_size: usize, alpha: f64) -> usize {
        if alpha == -1.0 {
            return 0;
        }
        let alpha = alpha.next_down();
        ((alpha * (query_size as f64 + y_size as f64)) / (1.0 + alpha))
            .next_down()
            .ceil() as usize
    }

    fn similarity_from_counts(&self, x_len: usize, y_len: usize, common: usize) -> f64 {
        if x_len == 0 && y_len == 0 {
            return 1.0;
        }
        if x_len == 0 || y_len == 0 {
            return 0.0;
        }

        let union_size = (x_len + y_len - common) as f64;

        if union_size == 0.0 {
            0.0
        } else {
            common as f64 / union_size
        }
    }
}

use super::Measure;
use crate::database::Database;

#[derive(Default, Clone, Copy)]
pub struct Cosine;

impl Measure for Cosine {
    fn min_feature_size(&self, query_size: usize, alpha: f64) -> usize {
        (alpha.next_down() * alpha.next_down() * query_size as f64)
            .next_down()
            .ceil() as usize
    }

    fn max_feature_size(&self, query_size: usize, alpha: f64, db: &dyn Database) -> usize {
        if alpha == 0.0 {
            return db.max_feature_len();
        }
        let calculated_max = (query_size as f64 / (alpha.next_down() * alpha.next_down()))
            .next_up()
            .floor() as usize;
        std::cmp::min(calculated_max, db.max_feature_len())
    }

    fn minimum_common_feature_count(&self, query_size: usize, y_size: usize, alpha: f64) -> usize {
        (alpha.next_down() * (query_size as f64 * y_size as f64).sqrt())
            .next_down()
            .ceil() as usize
    }

    fn similarity_from_counts(&self, x_len: usize, y_len: usize, common: usize) -> f64 {
        if x_len == 0 || y_len == 0 {
            return 0.0;
        }

        let denominator = (x_len as f64 * y_len as f64).sqrt();

        if denominator == 0.0 || !denominator.is_finite() {
            0.0
        } else {
            common as f64 / denominator
        }
    }
}

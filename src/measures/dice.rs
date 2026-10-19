use super::Measure;
use crate::database::Database;

/// Dice coefficient of the two feature sets: `2 * common / (x_len + y_len)`.
///
/// The symmetric, harmonic-style counterpart of [`Overlap`](super::Overlap);
/// also the measure used in the original SimString paper's default
/// configuration. Two empty sets are defined as perfectly similar (`1.0`);
/// exactly one empty side scores `0.0`.
#[derive(Default, Clone, Copy)]
pub struct Dice;

impl Measure for Dice {
    fn min_feature_size(&self, query_size: usize, alpha: f64) -> usize {
        if alpha > 2.0 {
            return 0;
        }
        let alpha = alpha.next_down();
        ((alpha / (2.0 - alpha)) * query_size as f64)
            .next_down()
            .ceil() as usize
    }

    fn max_feature_size(&self, query_size: usize, alpha: f64, db: &dyn Database) -> usize {
        if alpha == 0.0 {
            return db.max_feature_len();
        }
        let alpha = alpha.next_down();
        let calculated_max = (((2.0 - alpha) / alpha) * query_size as f64)
            .next_up()
            .floor() as usize;
        std::cmp::min(calculated_max, db.max_feature_len())
    }

    fn minimum_common_feature_count(&self, query_size: usize, y_size: usize, alpha: f64) -> usize {
        (0.5 * alpha.next_down() * (query_size as f64 + y_size as f64))
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

        let denominator = (x_len + y_len) as f64;

        if denominator == 0.0 {
            0.0
        } else {
            2.0 * common as f64 / denominator
        }
    }
}

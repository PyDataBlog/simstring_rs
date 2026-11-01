use super::Measure;
use crate::database::Database;
use std::cmp;

/// Overlap coefficient of the two feature sets:
/// `common / min(x_len, y_len)`.
///
/// The most permissive of the built-in measures: a string whose feature set
/// fully contains the query's feature set scores `1.0`, regardless of how
/// many extra features it has. Ideal for *containment-style* retrieval — the
/// query `"apple"` scores `1.0` against `"apple juice"`, because every bigram
/// of `"apple"` (`$a1`, `ap1`, `pp1`, `pl1`, `le1`, `e$1`) occurs there.
///
/// Two empty sets are `1.0`; exactly one empty side scores `0.0`.
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
        (alpha.next_down() * cmp::min(query_size, y_size) as f64)
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

        let denominator = cmp::min(x_len, y_len) as f64;

        if denominator == 0.0 {
            0.0
        } else {
            common as f64 / denominator
        }
    }
}

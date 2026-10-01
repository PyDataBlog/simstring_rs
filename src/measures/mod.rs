mod cosine;
mod dice;
mod exact_match;
mod jaccard;
mod overlap;

use crate::database::Database;
use lasso::Spur;

/// Must be Send + Sync to be used in parallel search.
pub trait Measure: Send + Sync {
    // Bounds round outwards in the built-in measures. Exact score
    // filtering follows the join, so floating-point rounding cannot discard a
    // boundary match (e.g. Dice 0.8 when the maximum size computes as 2.999...).
    fn min_feature_size(&self, query_size: usize, alpha: f64) -> usize;
    fn max_feature_size(&self, query_size: usize, alpha: f64, db: &dyn Database) -> usize;
    fn minimum_common_feature_count(&self, query_size: usize, y_size: usize, alpha: f64) -> usize;
    /// Scores an exact intersection of two unique feature sets.
    fn similarity_from_counts(&self, x_len: usize, y_len: usize, common: usize) -> f64;

    fn similarity(&self, x: &[Spur], y: &[Spur]) -> f64 {
        self.similarity_from_counts(x.len(), y.len(), compute_intersection_size(x, y))
    }
}

impl<M: Measure + ?Sized> Measure for &M {
    fn min_feature_size(&self, query_size: usize, alpha: f64) -> usize {
        (**self).min_feature_size(query_size, alpha)
    }
    fn max_feature_size(&self, query_size: usize, alpha: f64, db: &dyn Database) -> usize {
        (**self).max_feature_size(query_size, alpha, db)
    }
    fn minimum_common_feature_count(&self, query_size: usize, y_size: usize, alpha: f64) -> usize {
        (**self).minimum_common_feature_count(query_size, y_size, alpha)
    }
    fn similarity_from_counts(&self, x_len: usize, y_len: usize, common: usize) -> f64 {
        (**self).similarity_from_counts(x_len, y_len, common)
    }
}

// Helper function which computes the number of intersections between to vec of Spur
pub(crate) fn compute_intersection_size(x: &[Spur], y: &[Spur]) -> usize {
    let mut intersection_size = 0;
    let mut i = 0;
    let mut j = 0;

    while i < x.len() && j < y.len() {
        match x[i].cmp(&y[j]) {
            std::cmp::Ordering::Equal => {
                intersection_size += 1;
                i += 1;
                j += 1;
            }
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
        }
    }

    intersection_size
}
pub use cosine::Cosine;
pub use dice::Dice;
pub use exact_match::ExactMatch;
pub use jaccard::Jaccard;
pub use overlap::Overlap;

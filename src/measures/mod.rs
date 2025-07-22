mod cosine;
mod dice;
mod exact_match;
mod jaccard;
mod overlap;

use crate::database::Database;
use lasso::Spur;

/// Must be Send + Sync to be used in parallel search.
pub trait Measure: Send + Sync {
    fn min_feature_size(&self, query_size: usize, alpha: f64) -> usize;
    fn max_feature_size(&self, query_size: usize, alpha: f64, db: &dyn Database) -> usize;
    fn minimum_common_feature_count(&self, query_size: usize, y_size: usize, alpha: f64) -> usize;
    fn similarity(&self, x: &[Spur], y: &[Spur]) -> f64;
}

// FIX: All measures use the same intersection logic so this function can clean up measures module:
//fn compute_intersection_size(x: &[Spur], y: &[Spur]) -> usize {
//    let mut intersection_size = 0;
//    let mut i = 0;
//    let mut j = 0;
//
//    while i < x.len() && j < y.len() {
//        match x[i].cmp(&y[j]) {
//            std::cmp::Ordering::Equal => {
//                intersection_size += 1;
//                i += 1;
//                j += 1;
//            }
//            std::cmp::Ordering::Less => i += 1,
//            std::cmp::Ordering::Greater => j += 1,
//        }
//    }
//
//    intersection_size
//}
pub use cosine::Cosine;
pub use dice::Dice;
pub use exact_match::ExactMatch;
pub use jaccard::Jaccard;
pub use overlap::Overlap;

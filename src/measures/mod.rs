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
    // TODO: Current similarity allocates back to sets for easy set operations. Look at avoiding
    // this for all measures
    fn similarity(&self, x: &[Spur], y: &[Spur]) -> f64;
}

pub use cosine::Cosine;
pub use dice::Dice;
pub use exact_match::ExactMatch;
pub use jaccard::Jaccard;
pub use overlap::Overlap;

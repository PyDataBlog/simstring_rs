mod cosine;

use crate::database::Database;

/// Must be Send + Sync to be used in parallel search.
pub trait Measure: Send + Sync {
    fn min_feature_size(&self, query_size: usize, alpha: f64) -> usize;

    fn max_feature_size(&self, query_size: usize, alpha: f64, db: &dyn Database) -> usize;

    fn minimum_common_feature_count(&self, query_size: usize, y_size: usize, alpha: f64) -> usize;

    fn similarity(&self, x: &[String], y: &[String]) -> f64;
}

pub use cosine::Cosine;

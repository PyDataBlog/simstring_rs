mod hashdb;

use crate::extractors::FeatureExtractor;
use lasso::{Rodeo, Spur};

pub type StringId = usize;

pub trait Database: Send + Sync {
    fn insert(&mut self, text: String);
    fn clear(&mut self);
    /// Returns sorted, unique IDs. Each ID belongs to exactly one feature-size bucket.
    fn lookup_strings(&self, size: usize, feature: Spur) -> Option<&[StringId]>;
    fn get_string(&self, id: StringId) -> Option<&str>;
    fn get_features(&self, id: StringId) -> Option<&[Spur]>;
    fn feature_extractor(&self) -> &dyn FeatureExtractor;
    fn max_feature_len(&self) -> usize;
    fn interner(&self) -> &Rodeo;
    /// Sorted, unique populated feature sizes.
    fn feature_sizes(&self) -> &[usize];
    fn total_strings(&self) -> usize;
}

pub use hashdb::HashDb;

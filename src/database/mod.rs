mod hashdb;

use crate::extractors::FeatureExtractor;
use ahash::AHashSet;

/// A unique identifier for a string stored in the database.
pub type StringId = usize;

/// Defines the contract for a SimString database.
/// Must be Send + Sync to be usable by the parallel searcher.
pub trait Database: Send + Sync {
    /// Adds a string to the database, indexing its features.
    fn insert(&mut self, text: String);

    /// Looks up candidate string IDs by the size of their feature set and a specific feature.
    fn lookup_strings(&self, size: usize, feature: &str) -> Option<&AHashSet<StringId>>;

    /// Retrieves the original string for a given StringId.
    fn get_string(&self, id: StringId) -> Option<&str>;

    fn get_features(&self, id: StringId) -> Option<&Vec<String>>;

    /// Provides access to the feature extractor used by the database.
    fn feature_extractor(&self) -> &dyn FeatureExtractor;

    /// Returns the maximum feature vector length across all strings in the database.
    fn max_feature_len(&self) -> usize;
}

pub use hashdb::HashDb;

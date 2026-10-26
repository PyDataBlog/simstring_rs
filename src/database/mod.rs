mod hashdb;

use crate::extractors::FeatureExtractor;
use lasso::{Rodeo, Spur};

pub type StringId = usize;

pub trait Database: Send + Sync {
    fn insert(&mut self, text: String);
    fn clear(&mut self);
    /// Returns sorted, unique IDs. Each ID belongs to exactly one feature-size bucket.
    fn lookup_strings(&self, size: usize, feature: Spur) -> Option<&[StringId]>;
    /// Calls `visit(size, ids)` for each non-empty posting list of `feature` whose size is
    /// in `min..=max`, in ascending size order. Override when the backing store can
    /// enumerate a feature's sizes directly: most (size, feature) pairs have no postings.
    fn visit_postings<'a>(
        &'a self,
        feature: Spur,
        min: usize,
        max: usize,
        visit: &mut dyn FnMut(usize, &'a [StringId]),
    ) {
        let sizes = self.feature_sizes();
        let start = sizes.partition_point(|&size| size < min);
        for &size in sizes[start..].iter().take_while(|&&size| size <= max) {
            if let Some(ids) = self
                .lookup_strings(size, feature)
                .filter(|ids| !ids.is_empty())
            {
                visit(size, ids);
            }
        }
    }
    fn get_string(&self, id: StringId) -> Option<&str>;
    fn get_features(&self, id: StringId) -> Option<&[Spur]>;
    fn feature_extractor(&self) -> &dyn FeatureExtractor;
    fn max_feature_len(&self) -> usize;
    fn interner(&self) -> &Rodeo;
    /// Sorted, unique populated feature sizes.
    fn feature_sizes(&self) -> &[usize];
    fn total_strings(&self) -> usize;
}

pub use hashdb::{HashDb, PersistenceError};

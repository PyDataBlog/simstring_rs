mod hashdb;

use crate::extractors::FeatureExtractor;
use rustc_hash::FxHashSet;
use lasso::{Rodeo, Spur};
use std::sync::{Arc, Mutex};

pub type StringId = usize;

pub trait Database: Send + Sync {
    fn insert(&mut self, text: String);
    fn clear(&mut self);
    fn lookup_strings(&self, size: usize, feature: Spur) -> Option<&FxHashSet<StringId>>;
    fn get_string(&self, id: StringId) -> Option<&str>;
    fn get_features(&self, id: StringId) -> Option<&Vec<Spur>>;
    fn feature_extractor(&self) -> &dyn FeatureExtractor;
    fn max_feature_len(&self) -> usize;
    fn interner(&self) -> Arc<Mutex<Rodeo>>;
}

pub use hashdb::HashDb;

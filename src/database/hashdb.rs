use crate::database::{Database, StringId};
use crate::extractors::FeatureExtractor;
use lasso::{Rodeo, Spur};
use rustc_hash::FxHashMap;
use std::fmt;
use std::sync::Arc;

pub struct HashDb {
    feature_extractor: Arc<dyn FeatureExtractor>,
    pub strings: Vec<String>,
    string_features: Vec<Vec<Spur>>,
    feature_map: FxHashMap<usize, FxHashMap<Spur, Vec<StringId>>>,
    feature_sizes: Vec<usize>,
    interner: Rodeo,
}

impl fmt::Debug for HashDb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total_unique_features: usize = self
            .feature_map
            .values()
            .map(|size_map| size_map.len())
            .sum();
        let interner = &self.interner;

        f.debug_struct("HashDb")
            .field("num_strings", &self.strings.len())
            .field("num_feature_size_buckets", &self.feature_map.len())
            .field("total_unique_features_interned", &interner.len())
            .field("total_unique_features_indexed", &total_unique_features)
            .finish()
    }
}

impl HashDb {
    pub fn new(feature_extractor: Arc<dyn FeatureExtractor>) -> Self {
        Self {
            feature_extractor,
            strings: Vec::new(),
            string_features: Vec::new(),
            feature_map: FxHashMap::default(),
            feature_sizes: Vec::new(),
            interner: Rodeo::default(),
        }
    }

    pub fn insert(&mut self, text: String) {
        Database::insert(self, text);
    }

    pub fn clear(&mut self) {
        Database::clear(self);
    }
}

impl Database for HashDb {
    fn insert(&mut self, text: String) {
        let features = self.feature_extractor.features(&text, &mut self.interner);
        let size = features.len();
        let string_id = self.strings.len();

        self.strings.push(text);
        if let Err(index) = self.feature_sizes.binary_search(&size) {
            self.feature_sizes.insert(index, size);
        }
        let size_map = self.feature_map.entry(size).or_default();
        for &feature in &features {
            size_map.entry(feature).or_default().push(string_id);
        }
        self.string_features.push(features);
    }

    fn clear(&mut self) {
        self.strings.clear();
        self.string_features.clear();
        self.feature_map.clear();
        // clear the interner to release memory
        self.interner.clear();
        self.feature_sizes.clear();
    }

    fn lookup_strings(&self, size: usize, feature: Spur) -> Option<&[StringId]> {
        self.feature_map
            .get(&size)?
            .get(&feature)
            .map(Vec::as_slice)
    }

    fn get_string(&self, id: StringId) -> Option<&str> {
        self.strings.get(id).map(AsRef::as_ref)
    }

    fn get_features(&self, id: StringId) -> Option<&[Spur]> {
        self.string_features.get(id).map(Vec::as_slice)
    }

    fn feature_extractor(&self) -> &dyn FeatureExtractor {
        &*self.feature_extractor
    }

    fn max_feature_len(&self) -> usize {
        self.feature_sizes.last().copied().unwrap_or(0)
    }

    fn interner(&self) -> &Rodeo {
        &self.interner
    }

    fn feature_sizes(&self) -> &[usize] {
        &self.feature_sizes
    }

    fn total_strings(&self) -> usize {
        self.strings.len()
    }
}

use crate::database::{Database, StringId};
use crate::extractors::FeatureExtractor;
use lasso::{Rodeo, Spur};
use rustc_hash::{FxHashMap, FxHashSet};
use std::fmt;
use std::sync::{Arc, Mutex};

pub struct HashDb {
    feature_extractor: Arc<dyn FeatureExtractor>,
    pub strings: Vec<String>,
    string_features: Vec<Vec<Spur>>,
    feature_map: FxHashMap<usize, FxHashMap<Spur, FxHashSet<StringId>>>,
    interner: Arc<Mutex<Rodeo>>,
}

impl fmt::Debug for HashDb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total_unique_features: usize = self
            .feature_map
            .values()
            .map(|size_map| size_map.len())
            .sum();
        let interner = self.interner.lock().unwrap();

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
            interner: Arc::new(Mutex::new(Rodeo::default())),
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
        let mut interner = self.interner.lock().unwrap();
        let mut features = self.feature_extractor.features(&text, &mut interner);
        // FIX: Hmm append_feature_counts does counts. Should this sorting be moved there and make
        // dedup redundant?
        features.sort_unstable();
        features.dedup();
        let size = features.len();
        let string_id = self.strings.len();

        self.strings.push(text);
        self.string_features.push(features.clone());

        let size_map = self.feature_map.entry(size).or_default();
        for feature in features {
            size_map.entry(feature).or_default().insert(string_id);
        }
    }

    fn clear(&mut self) {
        self.strings.clear();
        self.string_features.clear();
        self.feature_map.clear();
        // clear the interner to release memory
        self.interner.lock().unwrap().clear();
    }

    fn lookup_strings(&self, size: usize, feature: Spur) -> Option<&FxHashSet<StringId>> {
        self.feature_map.get(&size)?.get(&feature)
    }

    fn get_string(&self, id: StringId) -> Option<&str> {
        self.strings.get(id).map(AsRef::as_ref)
    }

    fn get_features(&self, id: StringId) -> Option<&Vec<Spur>> {
        self.string_features.get(id)
    }

    fn feature_extractor(&self) -> &dyn FeatureExtractor {
        &*self.feature_extractor
    }

    fn max_feature_len(&self) -> usize {
        self.feature_map.keys().max().copied().unwrap_or(0)
    }

    fn interner(&self) -> Arc<Mutex<Rodeo>> {
        Arc::clone(&self.interner)
    }
}

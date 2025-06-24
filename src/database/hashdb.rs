use crate::database::{Database, StringId};
use crate::extractors::FeatureExtractor;
use ahash::{AHashMap, AHashSet};
use std::fmt;
use std::sync::Arc;

pub struct HashDb {
    /// The feature extractor, shared with an Arc to avoid cloning.
    feature_extractor: Arc<dyn FeatureExtractor>,
    /// The master list of all strings. The `StringId` is the index into this Vec.
    strings: Vec<String>,
    /// A cache of features for each string, indexed by StringId.
    /// This avoids re-calculating features during search.
    string_features: Vec<Vec<String>>,
    /// The main index:
    /// Map: feature_set_size -> feature -> set_of_string_ids
    feature_map: AHashMap<usize, AHashMap<String, AHashSet<StringId>>>,
}

impl HashDb {
    pub fn new(feature_extractor: Arc<dyn FeatureExtractor>) -> Self {
        Self {
            feature_extractor,
            strings: Vec::new(),
            string_features: Vec::new(),
            feature_map: AHashMap::default(),
        }
    }
}

impl fmt::Debug for HashDb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total_unique_features: usize = self
            .feature_map
            .values()
            .map(|size_map| size_map.len())
            .sum();

        f.debug_struct("HashDb")
            .field("num_strings", &self.strings.len())
            .field("num_feature_size_buckets", &self.feature_map.len())
            .field("total_unique_features", &total_unique_features)
            .finish()
    }
}

impl Database for HashDb {
    fn insert(&mut self, text: String) {
        let features = self.feature_extractor.features(&text);
        let size = features.len();
        // The StringId is simply the current number of strings in the DB.
        let string_id = self.strings.len();

        self.strings.push(text);
        self.string_features.push(features.clone());

        let size_map = self.feature_map.entry(size).or_default();
        for feature in features {
            size_map.entry(feature).or_default().insert(string_id);
        }
    }

    fn lookup_strings(&self, size: usize, feature: &str) -> Option<&AHashSet<StringId>> {
        self.feature_map.get(&size)?.get(feature)
    }

    fn get_string(&self, id: StringId) -> Option<&str> {
        self.strings.get(id).map(AsRef::as_ref)
    }

    fn get_features(&self, id: StringId) -> Option<&Vec<String>> {
        self.string_features.get(id)
    }

    fn feature_extractor(&self) -> &dyn FeatureExtractor {
        &*self.feature_extractor
    }

    fn max_feature_len(&self) -> usize {
        self.feature_map.keys().max().copied().unwrap_or(0)
    }
}

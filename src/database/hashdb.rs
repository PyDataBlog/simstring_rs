use crate::{FeatureExtractor, SimStringDB};
use std::collections::{HashMap, HashSet};

pub struct HashDB<TExtractor>
where
    TExtractor: FeatureExtractor,
{
    pub feature_extractor: TExtractor,
    pub string_collection: Vec<String>,
    pub string_size_map: HashMap<usize, HashSet<String>>,
    pub string_feature_map: HashMap<usize, HashMap<(String, i32), HashSet<String>>>,
    pub lookup_cache: HashMap<(usize, (String, i32)), HashSet<String>>,
}

impl<TExtractor> HashDB<TExtractor>
where
    TExtractor: FeatureExtractor,
{
    pub fn new(feature_extractor: TExtractor) -> Self {
        HashDB {
            feature_extractor,
            string_collection: Vec::new(),
            string_size_map: HashMap::new(),
            string_feature_map: HashMap::new(),
            lookup_cache: HashMap::new(),
        }
    }

    pub fn lookup_feature_set_by_size_feature(
        &mut self,
        size: usize,
        feature: &(String, i32),
    ) -> &HashSet<String> {
        let cache_key = (size, feature.clone());

        self.lookup_cache
            .entry(cache_key.clone())
            .or_insert_with(|| {
                // If not in cache, retrieve from string_feature_map or return an empty set
                self.string_feature_map
                    .get(&size)
                    .and_then(|feature_map| feature_map.get(feature))
                    .cloned()
                    .unwrap_or_else(HashSet::new)
            })
    }
}

impl<TExtractor> SimStringDB for HashDB<TExtractor>
where
    TExtractor: FeatureExtractor,
{
    fn insert(&mut self, s: String) {
        // Add the string to the collection
        self.string_collection.push(s.clone());

        // Extract features from the string
        let features = self.feature_extractor.extract(&s);

        // Determine the size (number of features)
        let size = features.len();

        // Update string_size_map
        self.string_size_map
            .entry(size)
            .or_default()
            .insert(s.clone());

        // Update string_feature_map
        let feature_map = self.string_feature_map.entry(size).or_default();

        for (feature, count) in features {
            let key = (feature.clone(), count);

            feature_map.entry(key).or_default().insert(s.clone());
        }
    }

    fn describe_collection(&self) -> (usize, f64, usize) {
        let total_collection = self.string_collection.len();

        let total_sizes: usize = self
            .string_size_map
            .iter()
            .map(|(size, strings)| size * strings.len())
            .sum();
        let total_strings: usize = self
            .string_size_map
            .values()
            .map(|strings| strings.len())
            .sum();
        let avg_size_ngrams = if total_strings == 0 {
            0.0
        } else {
            total_sizes as f64 / total_strings as f64
        };

        let total_ngrams: usize = self
            .string_feature_map
            .values()
            .map(|feature_map| feature_map.len())
            .sum();

        (total_collection, avg_size_ngrams, total_ngrams)
    }
}

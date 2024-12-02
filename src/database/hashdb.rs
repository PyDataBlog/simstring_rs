use crate::FeatureExtractor;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

pub struct HashDB<T1, T2, T3, T4, T5>
where
    T1: FeatureExtractor,
    T2: Eq + Hash + Clone,
    T3: HashMap<usize, HashSet<T2>>,
    T4: HashMap<usize, HashMap<(T2, i64), HashSet<T2>>>,
    T5: HashMap<usize, HashMap<(T2, i64), HashSet<T2>>>,
{
    feature_extractor: T1,
    string_collection: Vec<T2>,
    string_size_map: T3,
    string_feature_map: T4,
    lookup_cache: T5,
}

impl<T1, T2, T3, T4, T5> SimStringDB for HashDB<T1, T2, T3, T4, T5>
where
    T1: FeatureExtractor,
    T2: Eq + Hash + Clone,
    T3: HashMap<usize, HashSet<T2>>,
    T4: HashMap<usize, HashMap<(T2, i64), HashSet<T2>>>,
    T5: HashMap<usize, HashMap<(T2, i64), HashSet<T2>>>,
{
    fn describe_collection(&self) -> (usize, f64, usize) {
        let total_collection = self.string_collection.len();
        let n: Vec<usize> = self.string_size_map.keys().cloned().collect();
        let avg_size_ngrams = if n.is_empty() {
            0.0
        } else {
            n.iter().sum::<usize>() as f64 / n.len() as f64
        };
        let total_ngrams = self.string_feature_map.values().map(|map| map.len()).sum();

        (total_collection, avg_size_ngrams, total_ngrams)
    }
}

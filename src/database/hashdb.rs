use crate::search::SearchResult;
use crate::{FeatureExtractor, SimStringDB, SimilarityMeasure};
use std::collections::{HashMap, HashSet};

pub struct HashDB<TExtractor, TMeasure>
where
    TExtractor: FeatureExtractor,
    TMeasure: SimilarityMeasure,
{
    pub feature_extractor: TExtractor,
    pub measure: TMeasure,
    pub string_collection: Vec<String>,
    pub string_size_map: HashMap<i64, HashSet<String>>,
    pub string_feature_map: HashMap<i64, HashMap<(String, i32), HashSet<String>>>,
    pub lookup_cache: HashMap<(i64, (String, i32)), HashSet<String>>,
}

impl<TExtractor, TMeasure> HashDB<TExtractor, TMeasure>
where
    TExtractor: FeatureExtractor,
    TMeasure: SimilarityMeasure,
{
    pub fn new(feature_extractor: TExtractor, measure: TMeasure) -> Self {
        HashDB {
            feature_extractor,
            measure,
            string_collection: Vec::new(),
            string_size_map: HashMap::new(),
            string_feature_map: HashMap::new(),
            lookup_cache: HashMap::new(),
        }
    }

    pub fn insert(&mut self, s: String) {
        self.string_collection.push(s.clone());
        let features = self.feature_extractor.extract(&s);
        let feature_size = features.len() as i64;

        self.string_size_map
            .entry(feature_size)
            .or_default()
            .insert(s.clone());

        let feature_map = self.string_feature_map.entry(feature_size).or_default();

        for feature in &features {
            feature_map
                .entry(feature.clone())
                .or_default()
                .insert(s.clone());
        }
    }

    pub fn search(&mut self, query: &str, alpha: f64) -> Vec<SearchResult<String>> {
        // 1. Extract features from the query string
        let features = self.feature_extractor.extract(query);
        let query_size = features.len() as i64;

        // 2. Determine minimum and maximum feature sizes
        let min_size = self.measure.minimum_feature_size(query_size, alpha);
        let max_size = self.measure.maximum_feature_size(self, query_size, alpha);

        let mut results = Vec::new();

        // 3. Iterate over candidate sizes
        for candidate_size in min_size..=max_size {
            let tau = self
                .measure
                .minimum_overlap(query_size, candidate_size, alpha);

            // 4. Perform overlap join
            let candidates = self.overlap_join(&features, tau, candidate_size);
            results.extend(candidates);
        }

        // 5. Rank and return the results
        self.rank_search_results(query, results)
    }

    fn overlap_join(
        &mut self,
        features: &[(String, i32)],
        tau: i64,
        candidate_size: i64,
    ) -> Vec<String> {
        let query_len = features.len() as i64;

        // Sort features based on the frequency in the database
        let mut sorted_features = features.to_vec();
        sorted_features.sort_by_key(|feature| {
            self.lookup_feature_set_by_size_feature(candidate_size, feature)
                .len()
        });

        let mut candidate_match_counts: HashMap<String, i64> = HashMap::new();

        let feature_slice_index = query_len - tau + 1;
        let focus_features = if feature_slice_index <= 0 {
            &sorted_features[..]
        } else {
            &sorted_features[..(feature_slice_index as usize)]
        };

        // First phase: count feature occurrences
        for feature in focus_features {
            let candidates = self.lookup_feature_set_by_size_feature(candidate_size, feature);
            for candidate in candidates {
                *candidate_match_counts.entry(candidate.clone()).or_insert(0) += 1;
            }
        }

        let mut results = Vec::new();

        // Second phase: verify candidates
        for (candidate, mut match_count) in candidate_match_counts {
            let mut idx = feature_slice_index.max(0) as usize;

            while idx < sorted_features.len() {
                let feature = &sorted_features[idx];
                idx += 1;

                if self
                    .lookup_feature_set_by_size_feature(candidate_size, feature)
                    .contains(&candidate)
                {
                    match_count += 1;
                }

                if match_count >= tau {
                    results.push(candidate.clone());
                    break;
                }

                let remaining = (sorted_features.len() as i64) - (idx as i64);
                if match_count + remaining < tau {
                    break;
                }
            }
        }

        results
    }

    pub fn lookup_feature_set_by_size_feature(
        &mut self,
        size: i64,
        feature: &(String, i32),
    ) -> &HashSet<String> {
        let key = (size, feature.clone());
        if !self.lookup_cache.contains_key(&key) {
            let result = self
                .string_feature_map
                .get(&size)
                .and_then(|feature_map| feature_map.get(feature))
                .cloned()
                .unwrap_or_default();
            self.lookup_cache.insert(key.clone(), result);
        }
        self.lookup_cache.get(&key).unwrap()
    }

    fn rank_search_results(
        &self,
        query: &str,
        candidates: Vec<String>,
    ) -> Vec<SearchResult<String>> {
        let query_features = self.feature_extractor.extract(query);

        let mut results = candidates
            .into_iter()
            .map(|candidate| {
                let candidate_features = self.feature_extractor.extract(&candidate);
                let score = self
                    .measure
                    .similarity_score(&query_features, &candidate_features);
                SearchResult {
                    value: candidate,
                    score,
                }
            })
            .collect::<Vec<_>>();

        // Sort results by similarity score in descending order
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    pub fn describe_collection(&self) -> (usize, f64, usize) {
        // Total number of strings in the collection
        let total_collection = self.string_collection.len();

        // Average size of n-gram features
        let n = self.string_size_map.keys().collect::<Vec<&i64>>();
        let sum_n: i64 = n.iter().copied().sum();
        let avg_size_ngrams = if !n.is_empty() {
            sum_n as f64 / n.len() as f64
        } else {
            0.0
        };

        // Total number of n-gram features
        let total_ngrams: usize = self
            .string_feature_map
            .values()
            .map(|feature_map| feature_map.len())
            .sum();

        (total_collection, avg_size_ngrams, total_ngrams)
    }

    pub fn get_max_feature_size(&self) -> i64 {
        *self.string_size_map.keys().max().unwrap_or(&0)
    }
}

impl<TExtractor, TMeasure> SimStringDB<TMeasure> for HashDB<TExtractor, TMeasure>
where
    TExtractor: FeatureExtractor,
    TMeasure: SimilarityMeasure,
{
    fn insert(&mut self, s: String) {
        self.insert(s);
    }

    fn search(&mut self, query: &str, alpha: f64) -> Vec<SearchResult<String>> {
        self.search(query, alpha)
    }

    fn describe_collection(&self) -> (usize, f64, usize) {
        self.describe_collection()
    }

    fn get_max_feature_size(&self) -> i64 {
        self.get_max_feature_size()
    }
}

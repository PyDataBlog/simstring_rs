mod character_ngrams;
mod word_ngrams;

use ahash::AHashMap;

/// Takes a list of features and makes each one unique by appending its occurrence count.
/// For example, `["ab", "bc", "ab"]` becomes `["ab1", "bc1", "ab2"]`.
fn append_feature_counts(non_unique_list: Vec<String>) -> Vec<String> {
    let mut counter: AHashMap<String, usize> = AHashMap::default();
    let mut unique_list = Vec::with_capacity(non_unique_list.len());
    for val in non_unique_list {
        let count = counter.entry(val.clone()).or_insert(0);
        *count += 1;
        unique_list.push(format!("{}{}", val, *count));
    }
    unique_list
}

pub trait FeatureExtractor: Send + Sync {
    fn features(&self, text: &str) -> Vec<String>;
}

pub use character_ngrams::CharacterNgrams;
pub use word_ngrams::WordNgrams;

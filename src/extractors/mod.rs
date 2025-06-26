mod character_ngrams;
mod word_ngrams;

use ahash::AHashMap;
use lasso::{Rodeo, Spur};

/// Takes a list of features and makes each one unique by appending its occurrence count,
/// then interns the result.
fn append_feature_counts(interner: &mut Rodeo, features: Vec<String>) -> Vec<Spur> {
    let mut counter: AHashMap<String, usize> = AHashMap::default();
    let mut unique_features = Vec::with_capacity(features.len());
    for val in features {
        let count = counter.entry(val.clone()).or_insert(0);
        *count += 1;
        let unique_string = format!("{}{}", val, *count);
        unique_features.push(interner.get_or_intern(unique_string));
    }
    unique_features
}

pub trait FeatureExtractor: Send + Sync {
    /// Extracts features from text, interning them and returning their IDs.
    fn features(&self, text: &str, interner: &mut Rodeo) -> Vec<Spur>;
}

pub use character_ngrams::CharacterNgrams;
pub use word_ngrams::WordNgrams;

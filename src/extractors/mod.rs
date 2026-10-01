mod character_ngrams;
mod word_ngrams;

use lasso::{Rodeo, Spur};
use rustc_hash::FxHashMap;

/// Visits raw custom features with their occurrence counts appended.
pub(crate) fn visit_counted_features(features: &[String], visitor: &mut dyn FnMut(&str)) {
    let mut counter: FxHashMap<&str, usize> = FxHashMap::default();
    let mut buffer = String::new();
    let mut number = itoa::Buffer::new();
    for val in features {
        let count = counter.entry(val).or_default();
        *count += 1;
        buffer.clear();
        buffer.push_str(val);
        buffer.push_str(number.format(*count));
        visitor(&buffer);
    }
}

pub trait FeatureExtractor: Send + Sync {
    /// Visits occurrence-qualified features. Each feature must be unique within a text.
    /// The borrowed string is valid only for the duration of the callback.
    fn visit_features(&self, text: &str, visitor: &mut dyn FnMut(&str));

    /// Extracts features for insertion, sorted by interned ID.
    fn features(&self, text: &str, interner: &mut Rodeo) -> Vec<Spur> {
        let mut features = Vec::new();
        self.visit_features(text, &mut |feature| {
            features.push(interner.get_or_intern(feature));
        });
        features.sort_unstable();
        features.dedup();
        features
    }
}

pub use character_ngrams::CharacterNgrams;
pub use word_ngrams::WordNgrams;

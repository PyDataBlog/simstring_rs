mod character_ngrams;
mod word_ngrams;

use lasso::{Rodeo, Spur};

pub trait FeatureExtractor: Send + Sync {
    /// Extracts features from text, interning them and returning their IDs.
    fn features(&self, text: &str, interner: &mut Rodeo) -> Vec<Spur>;
}

pub use character_ngrams::CharacterNgrams;
pub use word_ngrams::WordNgrams;

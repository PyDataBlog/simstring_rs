mod character_ngrams;
mod word_ngrams;

pub trait FeatureExtractor {
    fn extract(&self, s: &str) -> Vec<(String, i32)>;
}

pub use character_ngrams::CharacterNGrams;
pub use word_ngrams::WordNGrams;

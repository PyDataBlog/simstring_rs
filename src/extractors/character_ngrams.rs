use crate::FeatureExtractor;
use lasso::{Rodeo, Spur};

#[derive(Clone)]
pub struct CharacterNgrams {
    n: usize,
    endmarker: String,
}

impl CharacterNgrams {
    pub fn new(n: usize, endmarker: &str) -> Self {
        Self {
            n,
            endmarker: endmarker.to_string(),
        }
    }
}

impl Default for CharacterNgrams {
    fn default() -> Self {
        Self::new(2, "$")
    }
}

impl FeatureExtractor for CharacterNgrams {
    fn features(&self, text: &str, interner: &mut Rodeo) -> Vec<Spur> {
        if self.n == 0 {
            return vec![];
        }

        let mut ngrams = Vec::new();
        let padding = self.endmarker.repeat(self.n.saturating_sub(1));

        // Create an iterator that includes padding
        let padded_text_iter = padding.chars().chain(text.chars()).chain(padding.chars());

        // Use a buffer to collect characters for each n-gram
        let mut buffer: Vec<char> = Vec::with_capacity(self.n);

        for ch in padded_text_iter {
            buffer.push(ch);
            if buffer.len() == self.n {
                ngrams.push(buffer.iter().collect::<String>());
                buffer.remove(0);
            }
        }

        super::append_feature_counts(interner, ngrams)
    }
}

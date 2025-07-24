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

        // Pre-calculate capacity to avoid reallocations
        let text_len = text.chars().count();
        let padding_len = self.n.saturating_sub(1);
        let total_len = text_len + 2 * padding_len;

        if total_len < self.n {
            return vec![];
        }

        let expected_ngrams = total_len - self.n + 1;
        let mut ngrams = Vec::with_capacity(expected_ngrams);

        let padding = self.endmarker.repeat(padding_len);

        // collect chars once, then slice
        let mut all_chars = Vec::with_capacity(total_len);
        all_chars.extend(padding.chars());
        all_chars.extend(text.chars());
        all_chars.extend(padding.chars());

        // Generate n-grams using efficient windowing
        for window in all_chars.windows(self.n) {
            // Pre-allocate string with known capacity
            let mut ngram = String::with_capacity(self.n * 4); // Assume max 4 bytes per char
            for &ch in window {
                ngram.push(ch);
            }
            ngrams.push(ngram);
        }

        super::append_feature_counts(interner, ngrams)
    }
}

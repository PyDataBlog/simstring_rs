use crate::FeatureExtractor;
use lasso::{Rodeo, Spur};
use rustc_hash::FxHashMap;
use std::fmt::Write;

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
        let padding = self.endmarker.repeat(padding_len);

        // Collect chars once, then slice
        let mut all_chars = Vec::with_capacity(total_len);
        all_chars.extend(padding.chars());
        all_chars.extend(text.chars());
        all_chars.extend(padding.chars());

        // Inline counting + interning in one pass (no intermediate Vec<String>)
        let mut counter: FxHashMap<String, usize> = FxHashMap::default();
        let mut result = Vec::with_capacity(expected_ngrams);
        let mut ngram_buffer = String::with_capacity(self.n * 4);
        let mut counted_buffer = String::with_capacity(self.n * 4 + 8);

        for window in all_chars.windows(self.n) {
            // Build n-gram in reusable buffer
            ngram_buffer.clear();
            for &ch in window {
                ngram_buffer.push(ch);
            }

            // Count occurrence
            let count = counter.entry(ngram_buffer.clone()).or_insert(0);
            *count += 1;

            // Build counted string and intern
            counted_buffer.clear();
            counted_buffer.push_str(&ngram_buffer);
            write!(&mut counted_buffer, "{count}").unwrap();
            result.push(interner.get_or_intern(&counted_buffer));
        }

        result.sort_unstable();
        result
    }
}

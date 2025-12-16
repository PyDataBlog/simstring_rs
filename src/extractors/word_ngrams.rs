use crate::FeatureExtractor;
use lasso::{Rodeo, Spur};
use rustc_hash::FxHashMap;
use std::fmt::Write;

#[derive(Clone)]
pub struct WordNgrams {
    n: usize,
    splitter: String,
    padder: String,
}

impl WordNgrams {
    pub fn new(n: usize, splitter: &str, padder: &str) -> Self {
        Self {
            n,
            splitter: splitter.to_string(),
            padder: padder.to_string(),
        }
    }
}

impl Default for WordNgrams {
    fn default() -> Self {
        Self::new(2, " ", " ")
    }
}

impl FeatureExtractor for WordNgrams {
    fn features(&self, text: &str, interner: &mut Rodeo) -> Vec<Spur> {
        if self.n == 0 {
            return vec![];
        }

        let tokens: Vec<&str> = text
            .split(&self.splitter)
            .filter(|s| !s.is_empty())
            .collect();

        // Padded tokens iterator
        let padded_tokens: Vec<&str> = std::iter::once(self.padder.as_str())
            .chain(tokens)
            .chain(std::iter::once(self.padder.as_str()))
            .collect();

        if padded_tokens.len() < self.n {
            return vec![];
        }

        // Inline counting + interning in one pass
        let mut counter: FxHashMap<String, usize> = FxHashMap::default();
        let expected_ngrams = padded_tokens.len() - self.n + 1;
        let mut result = Vec::with_capacity(expected_ngrams);
        let mut counted_buffer = String::with_capacity(64);

        for window in padded_tokens.windows(self.n) {
            let ngram = window.join(" ");

            // Count occurrence
            let count = counter.entry(ngram.clone()).or_insert(0);
            *count += 1;

            // Build counted string and intern
            counted_buffer.clear();
            counted_buffer.push_str(&ngram);
            write!(&mut counted_buffer, "{count}").unwrap();
            result.push(interner.get_or_intern(&counted_buffer));
        }

        result.sort_unstable();
        result
    }
}

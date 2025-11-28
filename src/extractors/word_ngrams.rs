use crate::FeatureExtractor;
use lasso::{Rodeo, Spur};
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

        let tokens: Vec<&str> = text.split(&self.splitter).collect();
        let mut padded_tokens = Vec::with_capacity(tokens.len() + 2);
        padded_tokens.push(self.padder.as_str());
        padded_tokens.extend(tokens.iter().copied());
        padded_tokens.push(self.padder.as_str());

        if padded_tokens.len() < self.n {
            return vec![];
        }

        let mut ngrams = Vec::with_capacity(padded_tokens.len().saturating_sub(self.n) + 1);

        for window in padded_tokens.windows(self.n) {
            ngrams.push(encode_word_window(window));
        }

        super::append_feature_counts(interner, ngrams)
    }
}

fn encode_word_window(window: &[&str]) -> String {
    let estimated_capacity: usize = window
        .iter()
        .map(|token| token.len() + digits(token.len()) + 1)
        .sum::<usize>()
        .saturating_sub(1);

    let mut encoded = String::with_capacity(estimated_capacity);

    for (index, token) in window.iter().enumerate() {
        if index > 0 {
            encoded.push('|');
        }
        let _ = write!(&mut encoded, "{}:", token.len());
        encoded.push_str(token);
    }

    encoded
}

fn digits(mut value: usize) -> usize {
    let mut digits = 1;
    while value >= 10 {
        value /= 10;
        digits += 1;
    }
    digits
}

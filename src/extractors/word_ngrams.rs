use crate::FeatureExtractor;
use lasso::{Rodeo, Spur};

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
        let mut padded_tokens: Vec<&str> = Vec::with_capacity(tokens.len() + 2);

        padded_tokens.push(&self.padder);
        padded_tokens.extend_from_slice(&tokens);
        padded_tokens.push(&self.padder);

        let ngrams: Vec<String> = padded_tokens
            .windows(self.n)
            .map(|window| window.join(" "))
            .collect();

        super::append_feature_counts(interner, ngrams)
    }
}

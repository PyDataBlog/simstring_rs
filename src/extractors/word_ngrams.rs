use crate::FeatureExtractor;
use lasso::{Rodeo, Spur};

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

        let tokens = text.split(&self.splitter).filter(|s| !s.is_empty());

        // an iterator that includes padding
        let padded_tokens_iter = std::iter::once(self.padder.as_str())
            .chain(tokens)
            .chain(std::iter::once(self.padder.as_str()));

        // Use a buffer to collect tokens for each n-gram
        let mut buffer: Vec<&str> = Vec::with_capacity(self.n);
        let mut ngrams = Vec::new();

        for token in padded_tokens_iter {
            buffer.push(token);
            if buffer.len() == self.n {
                ngrams.push(buffer.join(" "));
                buffer.remove(0);
            }
        }

        super::append_feature_counts(interner, ngrams)
    }
}

use crate::FeatureExtractor;
use rustc_hash::FxHashMap;

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
    fn visit_features(&self, text: &str, visitor: &mut dyn FnMut(&str)) {
        if self.n == 0 {
            return;
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
            return;
        }

        // Inline counting + interning in one pass
        let mut counter: FxHashMap<String, usize> = FxHashMap::default();
        let mut counted_buffer = String::with_capacity(64);
        let mut ngram = String::with_capacity(64);
        let mut number = itoa::Buffer::new();

        for window in padded_tokens.windows(self.n) {
            ngram.clear();
            for (i, token) in window.iter().enumerate() {
                if i != 0 {
                    ngram.push(' ');
                }
                ngram.push_str(token);
            }

            // Count occurrence
            let count = if let Some(count) = counter.get_mut(ngram.as_str()) {
                *count += 1;
                *count
            } else {
                counter.insert(ngram.clone(), 1);
                1
            };

            // Build counted string and intern
            counted_buffer.clear();
            counted_buffer.push_str(&ngram);
            counted_buffer.push_str(number.format(count));
            visitor(&counted_buffer);
        }
    }
}

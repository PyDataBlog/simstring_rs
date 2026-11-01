use crate::FeatureExtractor;
use crate::extractors::ExtractorConfig;
use rustc_hash::FxHashMap;

/// Word *n*-gram extractor over a token sequence.
///
/// Tokens are obtained by splitting the text on `splitter`, with empty tokens
/// dropped, and one `padder` token is added at each end of the sequence. Every
/// window of `n` consecutive padded tokens is joined with a single space
/// character to form one feature, occurrence-qualified with a 1-based counter
/// suffix like all built-in extractors (see the
/// [module docs](crate::extractors#occurrence-qualified-features)).
///
/// Note the join is always a single space, independently of the `splitter`:
/// with `splitter = ","` the tokens of `"a, b"` are still joined as `"a b"`.
/// Choose a `padder` that cannot collide with real tokens so padded windows
/// stay recognizable.
///
/// Longer `n` values need correspondingly longer shared sentences before a
/// match can reach a given `alpha`; `n == 0` emits nothing.
///
/// [`Default::default`] yields `n == 2`, splitting on single spaces and
/// padding with `" "`.
///
/// # Example
///
/// ```
/// use simstring_rust::extractors::WordNgrams;
/// use simstring_rust::FeatureExtractor;
///
/// let mut features = Vec::new();
/// WordNgrams::new(2, " ", "_").visit_features("green apples", &mut |feature| {
///     features.push(feature.to_string());
/// });
/// assert_eq!(vec!["_ green1", "green apples1", "apples _1"], features);
/// ```
#[derive(Clone)]
pub struct WordNgrams {
    n: usize,
    splitter: String,
    padder: String,
}

impl WordNgrams {
    /// Configures the extractor: window length `n`, the `splitter` string
    /// passed to [`str::split`] (multi-character split patterns are valid),
    /// and the `padder` token placed at both ends of every sequence. See the
    /// type docs for the exact windowing scheme.
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
    fn persistence_config(&self) -> Option<ExtractorConfig> {
        Some(ExtractorConfig::Word {
            n: self.n,
            splitter: self.splitter.clone(),
            padder: self.padder.clone(),
        })
    }

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

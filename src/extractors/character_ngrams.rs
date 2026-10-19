use crate::FeatureExtractor;
use rustc_hash::FxHashMap;

/// Character *n*-gram extractor, the default feature scheme for SimString.
///
/// The text is padded with `n - 1` copies of `endmarker` on both sides, and
/// every window of `n` characters becomes one feature, occurrence-qualified:
/// repeated grams get a 1-based occurrence counter suffix (see the module docs
/// for the full rationale).
///
/// n-grams are taken over Unicode characters, not bytes: "héllo" with
/// `n == 2` yields `$h1`, `hé1`, `él1`, `ll1`, `lo1`, `o$1` (padded with the
/// endmarker). Setting `n == 0` produces no features at all, so a database
/// built that way can never match anything.
///
/// # Default
///
/// [`Default`] selects the classic SimString setup: `n == 2`, endmarker `$`.
///
/// # Example
///
/// ```
/// use simstring_rust::{extractors::CharacterNgrams, FeatureExtractor};
///
/// let mut features = Vec::new();
/// CharacterNgrams::new(2, "$").visit_features("hello", &mut |feature| {
///     features.push(feature.to_string());
/// });
/// assert_eq!(
///     vec!["$h1", "he1", "el1", "ll1", "lo1", "o$1"],
///     features
/// );
///
/// // Runs of identical grams are kept distinct by the counter suffix.
/// let mut ran = Vec::new();
/// CharacterNgrams::new(2, "$").visit_features("aaa", &mut |feature| {
///     ran.push(feature.to_string());
/// });
/// assert_eq!(vec!["$a1", "aa1", "aa2", "a$1"], ran);
/// ```
#[derive(Clone)]
pub struct CharacterNgrams {
    n: usize,
    padding: String,
}

impl CharacterNgrams {
    /// Configures the extractor: window length `n` (in characters) and
    /// `endmarker`, repeated `n - 1` times on each side of the text before
    /// windows are taken. For `n == 1` no padding is added. A larger `n`
    /// captures more context, but the query must then share longer runs of
    /// characters before a match becomes possible; `n == 0` emits nothing.
    pub fn new(n: usize, endmarker: &str) -> Self {
        Self {
            n,
            padding: endmarker.repeat(n.saturating_sub(1)),
        }
    }
}

impl Default for CharacterNgrams {
    fn default() -> Self {
        Self::new(2, "$")
    }
}

impl FeatureExtractor for CharacterNgrams {
    fn feature_count_hint(&self, text: &str) -> usize {
        // Exact for ASCII text; an upper bound otherwise, as bytes >= characters.
        if self.n == 0 {
            0
        } else {
            text.len() + self.n - 1
        }
    }

    fn visit_features(&self, text: &str, visitor: &mut dyn FnMut(&str)) {
        if self.n == 0 {
            return;
        }
        let mut padded = String::with_capacity(text.len() + 2 * self.padding.len());
        padded.push_str(&self.padding);
        padded.push_str(text);
        padded.push_str(&self.padding);

        // Borrow n-grams directly from the padded text, avoiding an allocation per gram.
        // Sizing the counter for every gram up front avoids rehashing as it fills.
        let grams = padded.len().saturating_add(1).saturating_sub(self.n);
        let mut counter: FxHashMap<&str, usize> =
            FxHashMap::with_capacity_and_hasher(grams, Default::default());
        let mut counted = String::new();
        let mut number = itoa::Buffer::new();
        let mut emit = |start, end| {
            let gram = &padded[start..end];
            let count = counter.entry(gram).or_default();
            *count += 1;
            counted.clear();
            counted.push_str(gram);
            counted.push_str(number.format(*count));
            visitor(&counted);
        };

        if padded.is_ascii() {
            // ASCII byte boundaries are also character boundaries.
            for start in 0..padded.len().saturating_add(1).saturating_sub(self.n) {
                emit(start, start + self.n);
            }
        } else {
            let boundaries: Vec<usize> = padded
                .char_indices()
                .map(|(i, _)| i)
                .chain(std::iter::once(padded.len()))
                .collect();
            for start in 0..boundaries.len().saturating_sub(self.n) {
                emit(boundaries[start], boundaries[start + self.n]);
            }
        }
    }
}

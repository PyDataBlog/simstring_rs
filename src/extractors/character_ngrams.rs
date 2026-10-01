use crate::FeatureExtractor;
use rustc_hash::FxHashMap;

#[derive(Clone)]
pub struct CharacterNgrams {
    n: usize,
    padding: String,
}

impl CharacterNgrams {
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
    fn visit_features(&self, text: &str, visitor: &mut dyn FnMut(&str)) {
        if self.n == 0 {
            return;
        }
        let mut padded = String::with_capacity(text.len() + 2 * self.padding.len());
        padded.push_str(&self.padding);
        padded.push_str(text);
        padded.push_str(&self.padding);

        // Borrow n-grams directly from the padded text, avoiding an allocation per gram.
        let mut counter: FxHashMap<&str, usize> = FxHashMap::default();
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

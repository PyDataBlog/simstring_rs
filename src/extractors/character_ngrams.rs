use crate::FeatureExtractor;
use lasso::{Rodeo, Spur};

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
        let padding = self.endmarker.repeat(self.n.saturating_sub(1));
        let padded_text = format!("{}{}{}", padding, text, padding);

        let ngrams: Vec<String> = padded_text
            .chars()
            .collect::<Vec<char>>()
            .windows(self.n)
            .map(|window| window.iter().collect())
            .collect();

        super::append_feature_counts(interner, ngrams)
    }
}

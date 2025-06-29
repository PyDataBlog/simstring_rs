use super::FeatureExtractor;
use std::collections::HashMap;

pub struct CharacterNGrams {
    pub n: usize,
    pub padder: String,
}

impl FeatureExtractor for CharacterNGrams {
    fn extract(&self, s: &str) -> Vec<(String, i32)> {
        let n = if self.n - 1 == 0 { 1 } else { self.n - 1 };
        let padded_str = pad_string(s, &self.padder, n);
        let ngrams = init_char_ngrams(&padded_str, self.n);
        count_ngrams(ngrams)
    }
}

fn pad_string(s: &str, padder: &str, n: usize) -> String {
    let pad = padder.repeat(n);
    format!("{pad}{s}{pad}")
}

fn init_char_ngrams(s: &str, n: usize) -> Vec<String> {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    let mut ngrams = Vec::new();

    for i in 0..=len - n {
        let ngram: String = chars[i..i + n].iter().collect();
        ngrams.push(ngram);
    }

    ngrams
}

fn count_ngrams(ngrams: Vec<String>) -> Vec<(String, i32)> {
    let mut counter = HashMap::new();
    let mut result = Vec::with_capacity(ngrams.len());

    for ngram in ngrams {
        let count = counter.entry(ngram.clone()).or_insert(0);
        *count += 1;
        result.push((ngram, *count));
    }

    result
}

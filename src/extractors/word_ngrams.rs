use super::FeatureExtractor;
use std::collections::HashMap;

pub struct WordNGrams {
    pub n: usize,
    pub splitter: String,
    pub padder: String,
}

impl FeatureExtractor for WordNGrams {
    fn extract(&self, s: &str) -> Vec<(String, i32)> {
        let mut words: Vec<String> = s.split(&self.splitter).map(|w| w.to_string()).collect();
        pad_vector(&mut words, &self.padder);

        let ngrams = init_word_ngrams(&words, self.n);
        count_word_ngrams(ngrams)
    }
}

fn pad_vector(vec: &mut Vec<String>, padder: &str) {
    vec.insert(0, padder.to_string());
    vec.push(padder.to_string());
}

fn init_word_ngrams(words: &[String], n: usize) -> Vec<String> {
    let mut ngrams = Vec::new();
    let len = words.len();
    if len < n {
        return ngrams;
    }
    for i in 0..=len - n {
        let ngram_words = &words[i..i + n];
        let ngram = ngram_words.join(" ");
        ngrams.push(ngram);
    }
    ngrams
}

fn count_word_ngrams(ngrams: Vec<String>) -> Vec<(String, i32)> {
    let mut counter = HashMap::new();
    let mut result = Vec::with_capacity(ngrams.len());

    for ngram in ngrams {
        let count = counter.entry(ngram.clone()).or_insert(0);
        *count += 1;
        result.push((ngram, *count));
    }

    result
}

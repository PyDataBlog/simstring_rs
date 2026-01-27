use lasso::Rodeo;
use simstring_rust::{CharacterNgrams, FeatureExtractor, WordNgrams};

#[test]
fn test_character_ngrams_basic() {
    let mut interner = Rodeo::default();
    let extractor = CharacterNgrams::new(2, "$");
    let features = extractor.features("test", &mut interner);
    let resolved_features: Vec<String> = features
        .iter()
        .map(|s| interner.resolve(s).to_string())
        .collect();
    let expected = vec!["$t1", "te1", "es1", "st1", "t$1"];
    assert_eq!(resolved_features, expected);
}

#[test]
fn test_character_ngrams_with_repetition() {
    let mut interner = Rodeo::default();
    let extractor = CharacterNgrams::new(2, "$");
    let features = extractor.features("abab", &mut interner);
    let resolved_features: Vec<String> = features
        .iter()
        .map(|s| interner.resolve(s).to_string())
        .collect();
    let expected = vec!["$a1", "ab1", "ba1", "ab2", "b$1"];
    assert_eq!(resolved_features, expected);
}

#[test]
fn test_character_ngrams_different_n_and_marker() {
    let mut interner = Rodeo::default();
    let extractor = CharacterNgrams::new(3, "#");
    let features = extractor.features("rust", &mut interner);
    let resolved_features: Vec<String> = features
        .iter()
        .map(|s| interner.resolve(s).to_string())
        .collect();
    let expected = vec!["##r1", "#ru1", "rus1", "ust1", "st#1", "t##1"];
    assert_eq!(resolved_features, expected);
}

#[test]
fn test_character_ngrams_edge_cases() {
    let mut interner = Rodeo::default();
    let extractor_n0 = CharacterNgrams::new(0, "$");
    assert!(extractor_n0.features("test", &mut interner).is_empty());

    let extractor_n2 = CharacterNgrams::new(2, "$");
    let features_empty = extractor_n2.features("", &mut interner);
    let resolved_empty: Vec<String> = features_empty
        .iter()
        .map(|s| interner.resolve(s).to_string())
        .collect();
    assert_eq!(resolved_empty, vec!["$$1"]);

    let extractor_n3 = CharacterNgrams::new(3, "$");
    let features_short = extractor_n3.features("hi", &mut interner);
    let resolved_short: Vec<String> = features_short
        .iter()
        .map(|s| interner.resolve(s).to_string())
        .collect();
    let expected = vec!["$$h1", "$hi1", "hi$1", "i$$1"];
    assert_eq!(resolved_short, expected);
}

#[test]
fn test_uniquify_logic_with_complex_repetition() {
    let mut interner = Rodeo::default();
    let extractor = CharacterNgrams::new(2, "$");
    let features = extractor.features("aaaa", &mut interner);
    let resolved_features: Vec<String> = features
        .iter()
        .map(|s| interner.resolve(s).to_string())
        .collect();
    let expected = vec!["$a1", "aa1", "aa2", "aa3", "a$1"];
    assert_eq!(resolved_features, expected);
}

#[test]
fn test_character_trigrams_prepress() {
    let mut interner = Rodeo::default();
    let extractor = CharacterNgrams::new(3, "$");
    let features = extractor.features("prepress", &mut interner);
    let resolved_features: Vec<String> = features
        .iter()
        .map(|s| interner.resolve(s).to_string())
        .collect();
    let expected = vec![
        "$$p1", "$pr1", "pre1", "rep1", "epr1", "pre2", "res1", "ess1", "ss$1", "s$$1",
    ];
    assert_eq!(resolved_features, expected);
}

#[cfg(test)]
mod word_ngrams_tests {
    use super::*;

    #[test]
    fn test_word_ngram_default_behavior() {
        let mut interner = Rodeo::default();
        let extractor = WordNgrams::default(); // n=2, splitter=" ", padder=" "
        let features = extractor.features("a b", &mut interner);
        let resolved: Vec<String> = features
            .iter()
            .map(|s| interner.resolve(s).to_string())
            .collect();
        // Corrected expectation: The padder " " and the joiner " " create two spaces.
        let expected = vec!["  a1", "a b1", "b  1"];
        assert_eq!(resolved, expected);
    }

    #[test]
    fn test_word_ngram_single_word() {
        let mut interner = Rodeo::default();
        let extractor = WordNgrams::new(2, " ", "-");
        let features = extractor.features("word", &mut interner);
        let resolved: Vec<String> = features
            .iter()
            .map(|s| interner.resolve(s).to_string())
            .collect();
        let expected = vec!["- word1", "word -1"];
        assert_eq!(resolved, expected);
    }

    #[test]
    fn test_word_ngram_simple_sentence() {
        let mut interner = Rodeo::default();
        let extractor = WordNgrams::new(3, " ", "<PAD>");
        let features = extractor.features("this is a simple test", &mut interner);
        let resolved: Vec<String> = features
            .iter()
            .map(|s| interner.resolve(s).to_string())
            .collect();
        let expected = vec![
            "<PAD> this is1",
            "this is a1",
            "is a simple1",
            "a simple test1",
            "simple test <PAD>1",
        ];
        assert_eq!(resolved, expected);
    }

    #[test]
    fn test_word_ngram_really_really() {
        let mut interner = Rodeo::default();
        let extractor = WordNgrams::new(2, " ", "$");
        let s = "You are a really really really cool dude 😄🍕";
        let features = extractor.features(s, &mut interner);
        let resolved: Vec<String> = features
            .iter()
            .map(|s| interner.resolve(s).to_string())
            .collect();
        let expected = vec![
            "$ You1",
            "You are1",
            "are a1",
            "a really1",
            "really really1",
            "really really2",
            "really cool1",
            "cool dude1",
            "dude 😄🍕1",
            "😄🍕 $1",
        ];
        assert_eq!(resolved, expected);
    }

    #[test]
    fn test_word_ngram_edge_cases() {
        let mut interner = Rodeo::default();
        let extractor = WordNgrams::new(2, " ", "$");
        let features_empty = extractor.features("", &mut interner);
        let resolved_empty: Vec<String> = features_empty
            .iter()
            .map(|s| interner.resolve(s).to_string())
            .collect();
        assert_eq!(resolved_empty, vec!["$ $1"]);

        let features_spaces = extractor.features("   ", &mut interner);
        let resolved_spaces: Vec<String> = features_spaces
            .iter()
            .map(|s| interner.resolve(s).to_string())
            .collect();
        assert_eq!(resolved_spaces, vec!["$ $1"]);
    }

    #[test]
    fn test_word_ngram_parameterized_cases() {
        let mut interner = Rodeo::default();

        // Case 1: n=2, input="abcd", splitter=" ", padder=" "
        let extractor_case1 = WordNgrams::new(2, " ", " ");
        let features_case1 = extractor_case1.features("abcd", &mut interner);
        let resolved1: Vec<String> = features_case1
            .iter()
            .map(|s| interner.resolve(s).to_string())
            .collect();
        // Corrected expectation
        let expected_case1 = vec!["  abcd1", "abcd  1"];
        assert_eq!(resolved1, expected_case1, "Failed on: n=2, input='abcd'");

        // Case 2: n=2, input="hello world", splitter=" ", padder=" "
        let features_case2 = extractor_case1.features("hello world", &mut interner);
        let resolved2: Vec<String> = features_case2
            .iter()
            .map(|s| interner.resolve(s).to_string())
            .collect();
        // Corrected expectation
        let expected_case2 = vec!["  hello1", "hello world1", "world  1"];
        assert_eq!(
            resolved2, expected_case2,
            "Failed on: n=2, input='hello world'"
        );

        // Case 3: n=3, input="hello world", splitter=" ", padder=" "
        let extractor_case3 = WordNgrams::new(3, " ", " ");
        let features_case3 = extractor_case3.features("hello world", &mut interner);
        let resolved3: Vec<String> = features_case3
            .iter()
            .map(|s| interner.resolve(s).to_string())
            .collect();
        // Corrected expectation
        let expected_case3 = vec!["  hello world1", "hello world  1"];
        assert_eq!(
            resolved3, expected_case3,
            "Failed on: n=3, input='hello world'"
        );
    }
}

#[test]
fn test_character_ngrams_input_shorter_than_n() {
    // The condition `total_len < self.n` is only reachable if `text_len + 2*(n-1) < n`.
    // This simplifies to `text_len + n < 2`.
    // This is only possible if n=1 and text_len=0.
    // For any n >= 2, the padding ensures total_len >= n.

    let extractor = CharacterNgrams::new(1, "$");
    let mut interner = Rodeo::default();

    // "" -> len 0. n=1. padding=0. total_len=0. 0 < 1.
    let features = extractor.features("", &mut interner);
    assert!(
        features.is_empty(),
        "Features should be empty when input length is shorter than n (and n=1)"
    );
}

#[test]
fn test_word_ngrams_n_zero() {
    let extractor = WordNgrams::new(0, " ", "#");
    let mut interner = Rodeo::default();

    let features = extractor.features("hello world", &mut interner);
    assert!(features.is_empty(), "Features should be empty when n=0");
}

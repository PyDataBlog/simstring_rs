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
    assert_eq!(
        resolved_features, expected,
        "Character 2-grams of 'test' should produce expected features with counts"
    );
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
    assert_eq!(
        resolved_features, expected,
        "Character 2-grams of 'abab' should handle repeated n-grams with incrementing counts"
    );
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
    assert_eq!(
        resolved_features, expected,
        "Character 3-grams of 'rust' with '#' marker should produce expected features"
    );
}

#[test]
fn test_character_ngrams_edge_cases() {
    let mut interner = Rodeo::default();
    let extractor_n0 = CharacterNgrams::new(0, "$");
    assert!(
        extractor_n0.features("test", &mut interner).is_empty(),
        "Character n-grams with n=0 should return empty features"
    );

    let extractor_n2 = CharacterNgrams::new(2, "$");
    let features_empty = extractor_n2.features("", &mut interner);
    let resolved_empty: Vec<String> = features_empty
        .iter()
        .map(|s| interner.resolve(s).to_string())
        .collect();
    assert_eq!(
        resolved_empty,
        vec!["$$1"],
        "Character 2-grams of empty string should produce only marker pairs"
    );

    let extractor_n3 = CharacterNgrams::new(3, "$");
    let features_short = extractor_n3.features("hi", &mut interner);
    let resolved_short: Vec<String> = features_short
        .iter()
        .map(|s| interner.resolve(s).to_string())
        .collect();
    let expected = vec!["$$h1", "$hi1", "hi$1", "i$$1"];
    assert_eq!(
        resolved_short, expected,
        "Character 3-grams of 'hi' should pad correctly with markers"
    );
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
    assert_eq!(
        resolved_features, expected,
        "Character 2-grams of 'aaaa' should correctly count multiple occurrences of 'aa'"
    );
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
    assert_eq!(
        resolved_features, expected,
        "Character 3-grams of 'prepress' should handle repeated trigrams ('pre' appears twice)"
    );
}

#[cfg(test)]
mod word_ngrams_tests {
    use super::*;

    #[test]
    fn test_word_ngram_default_behavior() {
        let mut interner = Rodeo::default();
        let extractor = WordNgrams::default();
        let features = extractor.features("a b", &mut interner);
        let resolved: Vec<String> = features
            .iter()
            .map(|s| interner.resolve(s).to_string())
            .collect();
        let expected = vec!["  a1", "a b1", "b  1"];
        assert_eq!(
            resolved, expected,
            "Word 2-grams with default settings (space splitter/padder) should produce expected features"
        );
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
        assert_eq!(
            resolved, expected,
            "Word 2-grams of single word 'word' with '-' padding should include padded bigrams"
        );
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
        assert_eq!(
            resolved, expected,
            "Word 3-grams of sentence should produce sliding window of trigrams with <PAD> padding"
        );
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
        assert_eq!(
            resolved, expected,
            "Word 2-grams should handle repeated words and Unicode characters correctly"
        );
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
        assert_eq!(
            resolved_empty,
            vec!["$ $1"],
            "Word 2-grams of empty string should produce only padded bigram"
        );

        let features_spaces = extractor.features("   ", &mut interner);
        let resolved_spaces: Vec<String> = features_spaces
            .iter()
            .map(|s| interner.resolve(s).to_string())
            .collect();
        assert_eq!(
            resolved_spaces,
            vec!["$ $1"],
            "Word 2-grams of whitespace-only string should produce only padded bigram"
        );
    }

    #[test]
    fn test_word_ngram_parameterized_cases() {
        let mut interner = Rodeo::default();

        let extractor_case1 = WordNgrams::new(2, " ", " ");
        let features_case1 = extractor_case1.features("abcd", &mut interner);
        let resolved1: Vec<String> = features_case1
            .iter()
            .map(|s| interner.resolve(s).to_string())
            .collect();
        let expected_case1 = vec!["  abcd1", "abcd  1"];
        assert_eq!(
            resolved1, expected_case1,
            "Word 2-grams of single word 'abcd' with space padding (n=2)"
        );

        let features_case2 = extractor_case1.features("hello world", &mut interner);
        let resolved2: Vec<String> = features_case2
            .iter()
            .map(|s| interner.resolve(s).to_string())
            .collect();
        let expected_case2 = vec!["  hello1", "hello world1", "world  1"];
        assert_eq!(
            resolved2, expected_case2,
            "Word 2-grams of 'hello world' with space padding (n=2)"
        );

        let extractor_case3 = WordNgrams::new(3, " ", " ");
        let features_case3 = extractor_case3.features("hello world", &mut interner);
        let resolved3: Vec<String> = features_case3
            .iter()
            .map(|s| interner.resolve(s).to_string())
            .collect();
        let expected_case3 = vec!["  hello world1", "hello world  1"];
        assert_eq!(
            resolved3, expected_case3,
            "Word 3-grams of 'hello world' with space padding (n=3)"
        );
    }
}

#[test]
fn test_character_ngrams_input_shorter_than_n() {
    let extractor = CharacterNgrams::new(1, "$");
    let mut interner = Rodeo::default();

    let features = extractor.features("", &mut interner);
    assert!(
        features.is_empty(),
        "Character 1-grams of empty string should return empty (input shorter than n with no padding)"
    );
}

#[test]
fn test_word_ngrams_n_zero() {
    let extractor = WordNgrams::new(0, " ", "#");
    let mut interner = Rodeo::default();

    let features = extractor.features("hello world", &mut interner);
    assert!(
        features.is_empty(),
        "Word n-grams with n=0 should return empty features"
    );
}

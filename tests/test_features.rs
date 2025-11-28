use lasso::{Rodeo, Spur};
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

fn sorted_features(interner: &Rodeo, features: &[Spur]) -> Vec<String> {
    let mut resolved: Vec<String> = features
        .iter()
        .map(|spur| interner.resolve(spur).to_string())
        .collect();
    resolved.sort();
    resolved
}

fn sorted_strings(items: Vec<&str>) -> Vec<String> {
    let mut owned: Vec<String> = items.into_iter().map(String::from).collect();
    owned.sort();
    owned
}

#[cfg(test)]
mod word_ngrams_tests {
    use super::*;

    #[test]
    fn test_word_ngram_default_behavior() {
        let mut interner = Rodeo::default();
        let extractor = WordNgrams::default(); // n=2, splitter=" ", padder=" "
        let features = extractor.features("a b", &mut interner);
        let resolved = sorted_features(&interner, &features);
        let expected = sorted_strings(vec!["1: |1:a1", "1:a|1:b1", "1:b|1: 1"]);
        assert_eq!(resolved, expected);
    }

    #[test]
    fn test_word_ngram_single_word() {
        let mut interner = Rodeo::default();
        let extractor = WordNgrams::new(2, " ", "-");
        let features = extractor.features("word", &mut interner);
        let resolved = sorted_features(&interner, &features);
        let expected = sorted_strings(vec!["1:-|4:word1", "4:word|1:-1"]);
        assert_eq!(resolved, expected);
    }

    #[test]
    fn test_word_ngram_simple_sentence() {
        let mut interner = Rodeo::default();
        let extractor = WordNgrams::new(3, " ", "<PAD>");
        let features = extractor.features("this is a simple test", &mut interner);
        let resolved = sorted_features(&interner, &features);
        let expected = sorted_strings(vec![
            "5:<PAD>|4:this|2:is1",
            "4:this|2:is|1:a1",
            "2:is|1:a|6:simple1",
            "1:a|6:simple|4:test1",
            "6:simple|4:test|5:<PAD>1",
        ]);
        assert_eq!(resolved, expected);
    }

    #[test]
    fn test_word_ngram_really_really() {
        let mut interner = Rodeo::default();
        let extractor = WordNgrams::new(2, " ", "$");
        let s = "You are a really really really cool dude 😄🍕";
        let features = extractor.features(s, &mut interner);
        let resolved = sorted_features(&interner, &features);
        let expected = sorted_strings(vec![
            "1:$|3:You1",
            "3:You|3:are1",
            "3:are|1:a1",
            "1:a|6:really1",
            "6:really|6:really1",
            "6:really|6:really2",
            "6:really|4:cool1",
            "4:cool|4:dude1",
            "4:dude|8:😄🍕1",
            "8:😄🍕|1:$1",
        ]);
        assert_eq!(resolved, expected);
    }

    #[test]
    fn test_word_ngram_edge_cases() {
        let mut interner = Rodeo::default();
        let extractor = WordNgrams::new(2, " ", "$");

        let features_empty = extractor.features("", &mut interner);
        let resolved_empty = sorted_features(&interner, &features_empty);
        assert_eq!(resolved_empty, sorted_strings(vec!["1:$|0:1", "0:|1:$1"]));

        let features_spaces = extractor.features("   ", &mut interner);
        let resolved_spaces = sorted_features(&interner, &features_spaces);
        assert_eq!(
            resolved_spaces,
            sorted_strings(vec!["1:$|0:1", "0:|0:1", "0:|0:2", "0:|0:3", "0:|1:$1"])
        );

        let features_double_space = extractor.features("a  b", &mut interner);
        let resolved_double_space = sorted_features(&interner, &features_double_space);
        assert_eq!(
            resolved_double_space,
            sorted_strings(vec!["1:$|1:a1", "1:a|0:1", "0:|1:b1", "1:b|1:$1"])
        );
    }

    #[test]
    fn test_word_ngram_parameterized_cases() {
        let mut interner = Rodeo::default();

        // Case 1: n=2, input="abcd", splitter=" ", padder=" "
        let extractor_case1 = WordNgrams::new(2, " ", " ");
        let features_case1 = extractor_case1.features("abcd", &mut interner);
        let resolved1 = sorted_features(&interner, &features_case1);
        let expected_case1 = sorted_strings(vec!["1: |4:abcd1", "4:abcd|1: 1"]);
        assert_eq!(resolved1, expected_case1, "Failed on: n=2, input='abcd'");

        // Case 2: n=2, input="hello world", splitter=" ", padder=" "
        let features_case2 = extractor_case1.features("hello world", &mut interner);
        let resolved2 = sorted_features(&interner, &features_case2);
        let expected_case2 =
            sorted_strings(vec!["1: |5:hello1", "5:hello|5:world1", "5:world|1: 1"]);
        assert_eq!(
            resolved2, expected_case2,
            "Failed on: n=2, input='hello world'"
        );

        // Case 3: n=3, input="hello world", splitter=" ", padder=" "
        let extractor_case3 = WordNgrams::new(3, " ", " ");
        let features_case3 = extractor_case3.features("hello world", &mut interner);
        let resolved3 = sorted_features(&interner, &features_case3);
        let expected_case3 = sorted_strings(vec!["1: |5:hello|5:world1", "5:hello|5:world|1: 1"]);
        assert_eq!(
            resolved3, expected_case3,
            "Failed on: n=3, input='hello world'"
        );
    }
}

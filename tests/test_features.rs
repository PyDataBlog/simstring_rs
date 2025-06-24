use simstring_rust::{CharacterNgrams, FeatureExtractor, WordNgrams};

#[test]
fn test_character_ngrams_basic() {
    let extractor = CharacterNgrams::new(2, "$");
    let features = extractor.features("test");
    let expected = vec!["$t1", "te1", "es1", "st1", "t$1"];
    assert_eq!(features, expected);
}

#[test]
fn test_character_ngrams_with_repetition() {
    let extractor = CharacterNgrams::new(2, "$");
    let features = extractor.features("abab");
    let expected = vec!["$a1", "ab1", "ba1", "ab2", "b$1"];
    assert_eq!(features, expected);
}

#[test]
fn test_character_ngrams_different_n_and_marker() {
    let extractor = CharacterNgrams::new(3, "#");
    let features = extractor.features("rust");
    let expected = vec!["##r1", "#ru1", "rus1", "ust1", "st#1", "t##1"];
    assert_eq!(features, expected);
}

#[test]
fn test_character_ngrams_edge_cases() {
    // Test with n=0, which should return an empty vec.
    let extractor_n0 = CharacterNgrams::new(0, "$");
    assert!(extractor_n0.features("test").is_empty());

    // Test with an empty string.
    // With n=2, padded string is "$$", so the only 2-gram is "$$".
    let extractor_n2 = CharacterNgrams::new(2, "$");
    let features_empty = extractor_n2.features("");
    assert_eq!(features_empty, vec!["$$1"]); // Corrected assertion

    // Test with a string shorter than n.
    // With n=3, padded string is "$$hi$$".
    let extractor_n3 = CharacterNgrams::new(3, "$");
    let features_short = extractor_n3.features("hi");
    let expected = vec!["$$h1", "$hi1", "hi$1", "i$$1"];
    assert_eq!(features_short, expected);
}

#[test]
fn test_uniquify_logic_with_complex_repetition() {
    let extractor = CharacterNgrams::new(2, "$");
    let features = extractor.features("aaaa");
    let expected = vec!["$a1", "aa1", "aa2", "aa3", "a$1"];
    assert_eq!(features, expected);
}

#[test]
fn test_character_trigrams_prepress() {
    let extractor = CharacterNgrams::new(3, "$");
    let features = extractor.features("prepress");
    let expected = vec![
        "$$p1", "$pr1", "pre1", "rep1", "epr1", "pre2", "res1", "ess1", "ss$1", "s$$1",
    ];
    assert_eq!(features, expected);
}

#[cfg(test)]
mod word_ngrams_tests {
    use super::*;

    #[test]
    fn test_word_ngram_default_behavior() {
        let extractor = WordNgrams::default(); // n=2, splitter=" ", padder=" "
        let features = extractor.features("a b");
        // Padded: [" ", "a", "b", " "]
        // N-grams: ["  a", "a b", "b  "]
        let expected = vec!["  a1", "a b1", "b  1"];
        assert_eq!(features, expected);
    }

    #[test]
    fn test_word_ngram_single_word() {
        let extractor = WordNgrams::new(2, " ", "-");
        let features = extractor.features("word");
        // Padded: ["-", "word", "-"]
        let expected = vec!["- word1", "word -1"];
        assert_eq!(features, expected);
    }

    #[test]
    fn test_word_ngram_simple_sentence() {
        let extractor = WordNgrams::new(3, " ", "<PAD>");
        let features = extractor.features("this is a simple test");
        // Padded: ["<PAD>", "this", "is", "a", "simple", "test", "<PAD>"]
        let expected = vec![
            "<PAD> this is1",
            "this is a1",
            "is a simple1",
            "a simple test1",
            "simple test <PAD>1",
        ];
        assert_eq!(features, expected);
    }

    #[test]
    fn test_word_ngram_really_really() {
        let extractor = WordNgrams::new(2, " ", "$");
        let s = "You are a really really really cool dude 😄🍕";
        let features = extractor.features(s);
        // Padded: ["$", "You", ..., "😄🍕", "$"]
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
        assert_eq!(features, expected);
    }

    #[test]
    fn test_word_ngram_edge_cases() {
        let extractor = WordNgrams::new(2, " ", "$");
        let features_empty = extractor.features("");
        // Padded: ["$","$"] -> n-gram: "$ $"
        assert_eq!(features_empty, vec!["$ $1"]);

        let features_spaces = extractor.features("   ");
        assert_eq!(features_spaces, vec!["$ $1"]);
    }

    #[test]
    fn test_word_ngram_parameterized_cases() {
        // Case 1: n=2, input="abcd", splitter=" ", padder=" "
        let extractor_case1 = WordNgrams::new(2, " ", " ");
        let features_case1 = extractor_case1.features("abcd");
        let expected_case1 = vec!["  abcd1", "abcd  1"];
        assert_eq!(
            features_case1, expected_case1,
            "Failed on: n=2, input='abcd'"
        );

        // Case 2: n=2, input="hello world", splitter=" ", padder=" "
        let extractor_case2 = WordNgrams::new(2, " ", " ");
        let features_case2 = extractor_case2.features("hello world");
        let expected_case2 = vec!["  hello1", "hello world1", "world  1"];
        assert_eq!(
            features_case2, expected_case2,
            "Failed on: n=2, input='hello world'"
        );

        // Case 3: n=3, input="hello world", splitter=" ", padder=" "
        let extractor_case3 = WordNgrams::new(3, " ", " ");
        let features_case3 = extractor_case3.features("hello world");
        let expected_case3 = vec!["  hello world1", "hello world  1"];
        assert_eq!(
            features_case3, expected_case3,
            "Failed on: n=3, input='hello world'"
        );
    }
}

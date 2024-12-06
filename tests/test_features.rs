use simstring_rust::extractors::{CharacterNGrams, FeatureExtractor, WordNGrams};

mod character_ngrams_tests {
    use super::*;

    #[test]
    fn test_char_ngram_prepress() {
        let extractor = CharacterNGrams {
            n: 3,
            padder: " ".to_string(),
        };
        let s = "prepress";
        let features = extractor.extract(s);
        assert_eq!(features[5], ("pre".to_string(), 2));
    }

    #[test]
    fn test_char_ngram_unicode() {
        let extractor = CharacterNGrams {
            n: 2,
            padder: " ".to_string(),
        };
        let s = "∀∃😄🍕";
        let features = extractor.extract(s);
        assert_eq!(features[3], ("😄🍕".to_string(), 1));
    }

    #[test]
    fn test_char_ngram_hello_world() {
        let extractor = CharacterNGrams {
            n: 2,
            padder: "#".to_string(),
        };
        let s = "hello world";
        let features = extractor.extract(s);
        assert_eq!(features[0], ("#h".to_string(), 1));
        assert_eq!(features.len(), s.len() + 1); // Including padding
    }

    #[test]
    fn test_char_ngram_empty_string() {
        let extractor = CharacterNGrams {
            n: 2,
            padder: "*".to_string(),
        };
        let s = "";
        let features = extractor.extract(s);
        assert_eq!(features.len(), 1);
        assert_eq!(features[0], ("**".to_string(), 1));
    }
}

mod word_ngrams_tests {
    use super::*;

    #[test]
    fn test_word_ngram_really_really() {
        let extractor = WordNGrams {
            n: 2,
            splitter: " ".to_string(),
            padder: " ".to_string(),
        };
        let s = "You are a really really really cool dude 😄🍕";
        let features = extractor.extract(s);
        assert_eq!(features[5], ("really really".to_string(), 2));
        assert_eq!(features[8], ("dude 😄🍕".to_string(), 1));
    }

    #[test]
    fn test_word_ngram_simple_sentence() {
        let extractor = WordNGrams {
            n: 3,
            splitter: " ".to_string(),
            padder: "<PAD>".to_string(),
        };
        let s = "this is a simple test";
        let features = extractor.extract(s);
        assert_eq!(features[0], ("<PAD> this is".to_string(), 1));
        assert_eq!(features.len(), 5); // Number of words + 2 padding - n + 1
    }

    #[test]
    fn test_word_ngram_single_word() {
        let extractor = WordNGrams {
            n: 2,
            splitter: " ".to_string(),
            padder: "-".to_string(),
        };
        let s = "word";
        let features = extractor.extract(s);
        assert_eq!(features.len(), 2);
        assert_eq!(features[0], ("- word".to_string(), 1));
        assert_eq!(features[1], ("word -".to_string(), 1));
    }
}

use simstring_rust::database::HashDB;
use simstring_rust::extractors::CharacterNGrams;
use simstring_rust::measures::{Cosine, Dice, ExactMatch, Jaccard, Overlap};

mod test_search {
    use super::*;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn test_dice_search() {
        let feature_extractor = CharacterNGrams {
            n: 2,
            padder: " ".to_string(),
        };
        let measure = Dice::new();
        let mut db = HashDB::new(feature_extractor, measure);

        db.insert("foo".to_string());
        db.insert("bar".to_string());
        db.insert("fooo".to_string());

        let results = db.search("foo", 0.8);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].value, "foo");
        assert!(approx_eq(results[0].score, 1.0));
        assert_eq!(results[1].value, "fooo");
        assert!(approx_eq(results[1].score, 0.8888888888888888));
    }

    #[test]
    fn test_jaccard_search() {
        let feature_extractor = CharacterNGrams {
            n: 2,
            padder: " ".to_string(),
        };
        let measure = Jaccard::new();
        let mut db = HashDB::new(feature_extractor, measure);

        db.insert("foo".to_string());
        db.insert("bar".to_string());
        db.insert("fooo".to_string());

        let results = db.search("foo", 0.8);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].value, "foo");
        assert!(approx_eq(results[0].score, 1.0));
        assert_eq!(results[1].value, "fooo");
        assert!(approx_eq(results[1].score, 0.8));
    }

    #[test]
    fn test_cosine_search() {
        let feature_extractor = CharacterNGrams {
            n: 2,
            padder: " ".to_string(),
        };
        let measure = Cosine::new();
        let mut db = HashDB::new(feature_extractor, measure);

        db.insert("foo".to_string());
        db.insert("bar".to_string());
        db.insert("fooo".to_string());

        let results = db.search("foo", 0.8);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].value, "foo");
        assert!(approx_eq(results[0].score, 1.0));
        assert_eq!(results[1].value, "fooo");
        assert!(approx_eq(results[1].score, 0.8944271909999159));
    }

    #[test]
    fn test_overlap_search() {
        let feature_extractor = CharacterNGrams {
            n: 2,
            padder: " ".to_string(),
        };
        let measure = Overlap::new();
        let mut db = HashDB::new(feature_extractor, measure);

        db.insert("foo".to_string());
        db.insert("bar".to_string());
        db.insert("fooo".to_string());

        let results = db.search("foo", 0.8);
        assert_eq!(results.len(), 2);

        let mut found_foo = false;
        let mut found_fooo = false;

        for result in &results {
            match result.value.as_str() {
                "foo" => {
                    found_foo = true;
                    assert!(approx_eq(result.score, 1.0));
                }
                "fooo" => {
                    found_fooo = true;
                    assert!(approx_eq(result.score, 1.0));
                }
                _ => panic!("Unexpected result: {}", result.value),
            }
        }

        assert!(found_foo && found_fooo, "Should find both 'foo' and 'fooo'");
    }

    #[test]
    fn test_exact_match_search() {
        let feature_extractor = CharacterNGrams {
            n: 2,
            padder: " ".to_string(),
        };
        let measure = ExactMatch::new();
        let mut db = HashDB::new(feature_extractor, measure);

        db.insert("foo".to_string());
        db.insert("bar".to_string());
        db.insert("fooo".to_string());

        let thresholds = [0.1, 0.5, 0.9, 1.0];
        for threshold in thresholds.iter() {
            let results = db.search("foo", *threshold);
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].value, "foo");
            assert!(approx_eq(results[0].score, 1.0));
        }
    }

    #[test]
    fn test_micro_deep_dive_search() {
        let feature_extractor = CharacterNGrams {
            n: 2,
            padder: " ".to_string(),
        };
        let measure = Cosine::new();
        let mut db = HashDB::new(feature_extractor, measure);

        let strings = ["a", "ab", "abc", "abcd", "abcde"];
        for s in strings.iter() {
            db.insert(s.to_string());
        }

        // Test exact matches with threshold 1.0
        for query in strings.iter() {
            let results = db.search(query, 1.0);
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].value, *query);
            assert!(approx_eq(results[0].score, 1.0));
        }

        // Test "ab" with threshold 0.5
        let results_ab = db.search("ab", 0.5);
        assert_eq!(
            results_ab.len(),
            3,
            "Should find exactly 3 matches for 'ab'"
        );
        assert_eq!(results_ab[0].value, "ab");
        assert!(approx_eq(results_ab[0].score, 1.0));
        assert_eq!(results_ab[1].value, "abc");
        assert!(approx_eq(results_ab[1].score, 0.5773502691896258));
        assert_eq!(results_ab[2].value, "abcd");
        assert!(approx_eq(results_ab[2].score, 0.5163977794943222));

        // Test "abc" with threshold 0.6
        let results_abc = db.search("abc", 0.6);
        assert_eq!(
            results_abc.len(),
            3,
            "Should find exactly 3 matches for 'abc'"
        );
        assert_eq!(results_abc[0].value, "abc");
        assert!(approx_eq(results_abc[0].score, 1.0));
        assert_eq!(results_abc[1].value, "abcd");
        assert!(approx_eq(results_abc[1].score, 0.6708203932499369));
        assert_eq!(results_abc[2].value, "abcde");
        assert!(approx_eq(results_abc[2].score, 0.6123724356957946));

        // Test all strings with high threshold (0.9)
        for query in strings.iter() {
            let results = db.search(query, 0.9);
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].value, *query);
            assert!(approx_eq(results[0].score, 1.0));
        }
    }

    #[test]
    fn test_empty_db_search() {
        let feature_extractor = CharacterNGrams {
            n: 2,
            padder: " ".to_string(),
        };
        let measure = Cosine::new();
        let mut db = HashDB::new(feature_extractor, measure);

        let results = db.search("foo", 0.8);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_threshold_edge_cases() {
        let feature_extractor = CharacterNGrams {
            n: 2,
            padder: " ".to_string(),
        };
        let measure = Cosine::new();
        let mut db = HashDB::new(feature_extractor, measure);

        db.insert("foo".to_string());

        // // Test threshold = 0.0 * FIX: Account non zero alphas
        // let results_0 = db.search("bar", 0.0);
        // assert!(results_0.len() <= 1);

        // Test threshold = 1.0 with non-matching string
        let results_1 = db.search("bar", 1.0);
        assert_eq!(results_1.len(), 0);

        // Test threshold = 1.0 with exact match
        let results_exact = db.search("foo", 1.0);
        assert_eq!(results_exact.len(), 1);
        assert!(approx_eq(results_exact[0].score, 1.0));

        // Test with intermediate threshold
        let results_mid = db.search("foo", 0.5);
        assert_eq!(results_mid.len(), 1);
        assert_eq!(results_mid[0].value, "foo");
    }

    #[test]
    fn test_search_with_different_paddings() {
        // Test with space padding
        let fe_space = CharacterNGrams {
            n: 2,
            padder: " ".to_string(),
        };
        let mut db_space = HashDB::new(fe_space, Cosine::new());
        db_space.insert("test".to_string());
        let results_space = db_space.search("test", 0.8);
        assert_eq!(results_space.len(), 1);
        assert!(approx_eq(results_space[0].score, 1.0));

        // Test with different padding
        let fe_hash = CharacterNGrams {
            n: 2,
            padder: "#".to_string(),
        };
        let mut db_hash = HashDB::new(fe_hash, Cosine::new());
        db_hash.insert("test".to_string());
        let results_hash = db_hash.search("test", 0.8);
        assert_eq!(results_hash.len(), 1);
        assert!(approx_eq(results_hash[0].score, 1.0));
    }

    #[test]
    fn test_search_with_different_ngram_sizes() {
        // Test with bigrams
        let fe_2 = CharacterNGrams {
            n: 2,
            padder: " ".to_string(),
        };
        let mut db_2 = HashDB::new(fe_2, Cosine::new());
        db_2.insert("test".to_string());
        let results_2 = db_2.search("test", 0.8);
        assert_eq!(results_2.len(), 1);

        // Test with trigrams
        let fe_3 = CharacterNGrams {
            n: 3,
            padder: " ".to_string(),
        };
        let mut db_3 = HashDB::new(fe_3, Cosine::new());
        db_3.insert("test".to_string());
        let results_3 = db_3.search("test", 0.8);
        assert_eq!(results_3.len(), 1);
    }
}

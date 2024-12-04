use simstring_rs::database::{HashDB, SimStringDB};
use simstring_rs::extractors::{CharacterNGrams, FeatureExtractor, WordNGrams};
use std::collections::{HashMap, HashSet};

mod hashdb_tests {
    use super::*;

    #[test]
    fn test_insert_single_string() {
        let feature_extractor = CharacterNGrams {
            n: 2,
            padder: " ".to_string(),
        };
        let mut db = HashDB::new(feature_extractor);

        let s = "hello";
        db.insert(s.to_string());

        // Check if the string is in string_collection
        assert!(db.string_collection.contains(&s.to_string()));

        // Check if the size is correct
        let features = db.feature_extractor.extract(s);
        let size = features.len();
        assert!(db.string_size_map.contains_key(&size));
        assert!(db.string_size_map.get(&size).unwrap().contains(s));

        // Check if the features are correctly stored
        let feature_map = db.string_feature_map.get(&size).unwrap();
        for (feature, count) in features {
            let key = (feature.clone(), count);
            assert!(feature_map.contains_key(&key));
            assert!(feature_map.get(&key).unwrap().contains(s));
        }
    }

    #[test]
    fn test_describe_collection() {
        let feature_extractor = CharacterNGrams {
            n: 3,
            padder: " ".to_string(),
        };
        let mut db = HashDB::new(feature_extractor);

        db.insert("hello".to_string());
        db.insert("world".to_string());

        let (total_collection, avg_size_ngrams, total_ngrams) = db.describe_collection();

        assert_eq!(total_collection, 2);

        // Calculate expected average size
        let features_hello = db.feature_extractor.extract("hello");
        let features_world = db.feature_extractor.extract("world");
        let total_sizes = features_hello.len() + features_world.len();
        let expected_avg_size = total_sizes as f64 / 2.0;

        assert_eq!(avg_size_ngrams, expected_avg_size);

        // Calculate expected total n-grams
        let total_ngrams_expected: usize = db
            .string_feature_map
            .values()
            .map(|feature_map| feature_map.len())
            .sum();

        assert_eq!(total_ngrams, total_ngrams_expected);
    }

    #[test]
    fn test_lookup_feature_set_by_size_feature() {
        let feature_extractor = CharacterNGrams {
            n: 2,
            padder: "#".to_string(),
        };
        let mut db = HashDB::new(feature_extractor);

        db.insert("hello".to_string());
        db.insert("help".to_string());
        db.insert("hero".to_string());

        // Extract features for "hello" to get a feature and size
        let features = db.feature_extractor.extract("hello");
        let size = features.len(); // size for "hello"

        // Choose a feature that is common to some strings
        let feature = ("he".to_string(), 1);

        let result_set = db.lookup_feature_set_by_size_feature(size, &feature);

        // Expected string is only "hello" because "hero" has a different size
        let expected_strings: HashSet<String> = ["hello".to_string()].iter().cloned().collect();

        assert_eq!(result_set, &expected_strings);
    }

    #[test]
    fn test_insert_multiple_strings_same_size() {
        let feature_extractor = CharacterNGrams {
            n: 3,
            padder: " ".to_string(),
        };
        let mut db = HashDB::new(feature_extractor);

        db.insert("cat".to_string());
        db.insert("dog".to_string());
        db.insert("bat".to_string());

        // All strings should have the same size
        let size = db.feature_extractor.extract("cat").len();
        assert!(db.string_size_map.contains_key(&size));

        let strings_with_size = db.string_size_map.get(&size).unwrap();
        let expected_strings: HashSet<String> =
            ["cat".to_string(), "dog".to_string(), "bat".to_string()]
                .iter()
                .cloned()
                .collect();

        assert_eq!(strings_with_size, &expected_strings);
    }

    #[test]
    fn test_insert_strings_different_sizes() {
        let feature_extractor = WordNGrams {
            n: 2,
            splitter: " ".to_string(),
            padder: "<PAD>".to_string(),
        };
        let mut db = HashDB::new(feature_extractor);

        db.insert("hello world".to_string()); // Size depends on word n-grams
        db.insert("a quick brown fox".to_string());
        db.insert("jumps over the lazy dog".to_string());

        // Check that sizes are correctly stored
        let sizes: Vec<usize> = db.string_size_map.keys().cloned().collect();
        assert!(sizes.len() >= 2); // At least two different sizes

        for size in sizes {
            let strings_with_size = db.string_size_map.get(&size).unwrap();
            for s in strings_with_size {
                let features = db.feature_extractor.extract(s);
                assert_eq!(features.len(), size);
            }
        }
    }

    #[test]
    fn test_describe_collection_empty_db() {
        let feature_extractor = CharacterNGrams {
            n: 2,
            padder: "*".to_string(),
        };
        let db = HashDB::new(feature_extractor);

        let (total_collection, avg_size_ngrams, total_ngrams) = db.describe_collection();

        assert_eq!(total_collection, 0);
        assert_eq!(avg_size_ngrams, 0.0);
        assert_eq!(total_ngrams, 0);
    }

    #[test]
    fn test_lookup_feature_set_with_cache() {
        let feature_extractor = CharacterNGrams {
            n: 2,
            padder: "-".to_string(),
        };
        let mut db = HashDB::new(feature_extractor);

        db.insert("test".to_string());
        db.insert("text".to_string());

        let size = db.feature_extractor.extract("test").len();
        let feature = ("te".to_string(), 1);

        // First lookup (cache miss)
        let _ = db.lookup_feature_set_by_size_feature(size, &feature);

        // Cache should now contain the result
        assert!(db.lookup_cache.contains_key(&(size, feature.clone())));

        // Second lookup (cache hit)
        let result_set = db.lookup_feature_set_by_size_feature(size, &feature);

        // Expected strings are "test" and "text"
        let expected_strings: HashSet<String> = ["test".to_string(), "text".to_string()]
            .iter()
            .cloned()
            .collect();

        assert_eq!(result_set, &expected_strings);
    }

    // New tests adapted from the Julia test suite

    #[test]
    fn test_single_updates_character_ngrams() {
        let feature_extractor = CharacterNGrams {
            n: 3,
            padder: " ".to_string(),
        };
        let mut db = HashDB::new(feature_extractor);

        db.insert("foo".to_string());
        db.insert("bar".to_string());
        db.insert("fooo".to_string());

        // Test string_collection
        assert_eq!(
            db.string_collection,
            vec!["foo".to_string(), "bar".to_string(), "fooo".to_string()]
        );

        // Test string_size_map
        let mut expected_size_map = HashMap::new();
        expected_size_map.insert(
            5,
            ["foo".to_string(), "bar".to_string()]
                .iter()
                .cloned()
                .collect(),
        );
        expected_size_map.insert(6, ["fooo".to_string()].iter().cloned().collect());
        assert_eq!(db.string_size_map, expected_size_map);

        // Test keys of string_feature_map
        let expected_keys: Vec<usize> = vec![5, 6];
        let mut actual_keys: Vec<usize> = db.string_feature_map.keys().cloned().collect();
        actual_keys.sort();
        assert_eq!(actual_keys, expected_keys);

        // Test values of string_feature_map[5]
        let feature_map_5 = db.string_feature_map.get(&5).unwrap();
        let values_5: Vec<&HashSet<String>> = feature_map_5.values().collect();

        // Since both "foo" and "bar" have size 5, and each feature maps to one of them,
        // we can check that the sets in values_5 contain either "foo" or "bar"
        let mut expected_sets = vec![
            HashSet::from(["foo".to_string()]),
            HashSet::from(["bar".to_string()]),
        ];
        for value_set in values_5 {
            assert!(expected_sets.contains(value_set));
        }

        // Test values of string_feature_map[6]
        let feature_map_6 = db.string_feature_map.get(&6).unwrap();
        let values_6: Vec<&HashSet<String>> = feature_map_6.values().collect();

        // All values should be HashSet containing "fooo"
        for value_set in values_6 {
            assert_eq!(value_set, &HashSet::from(["fooo".to_string()]));
        }
    }

    #[test]
    fn test_single_update_word_ngrams() {
        let feature_extractor = WordNGrams {
            n: 2,
            splitter: " ".to_string(),
            padder: " ".to_string(),
        };
        let mut db = HashDB::new(feature_extractor);

        let s = "You are a really really really cool dude.";
        db.insert(s.to_string());

        // Test string_collection
        assert_eq!(db.string_collection, vec![s.to_string()]);

        // Test string_size_map
        let features = db.feature_extractor.extract(s);
        let size = features.len();
        assert_eq!(
            db.string_size_map.get(&size).unwrap(),
            &HashSet::from([s.to_string()])
        );

        // Test keys of string_feature_map
        let keys: Vec<usize> = db.string_feature_map.keys().cloned().collect();
        assert_eq!(keys, vec![size]);

        // Test values of string_feature_map[size]
        let feature_map = db.string_feature_map.get(&size).unwrap();
        let values: Vec<&HashSet<String>> = feature_map.values().collect();

        // All values should be HashSet containing the string
        for value_set in values {
            assert_eq!(value_set, &HashSet::from([s.to_string()]));
        }
    }

    #[test]
    fn test_describe_collection_after_search() {
        let feature_extractor = CharacterNGrams {
            n: 2,
            padder: " ".to_string(),
        };
        let mut db = HashDB::new(feature_extractor);

        db.insert("foo".to_string());
        db.insert("bar".to_string());
        db.insert("fooo".to_string());

        // Simulate a search (even though search function is not implemented here)
        // For the purpose of the test, we'll just call describe_collection
        let (total_collection, avg_size_ngrams, total_ngrams) = db.describe_collection();

        assert_eq!(total_collection, 3);

        // Manually calculate average size of n-grams
        let size_foo = db.feature_extractor.extract("foo").len();
        let size_bar = db.feature_extractor.extract("bar").len();
        let size_fooo = db.feature_extractor.extract("fooo").len();
        let expected_avg_size = (size_foo + size_bar + size_fooo) as f64 / 3.0;
        assert_eq!(avg_size_ngrams, expected_avg_size);

        // Manually calculate total n-grams
        let total_ngrams_expected: usize = db
            .string_feature_map
            .values()
            .map(|feature_map| feature_map.len())
            .sum();
        assert_eq!(total_ngrams, total_ngrams_expected);
    }
}


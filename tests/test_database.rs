use simstring_rust::database::{HashDB, SimStringDB};
use simstring_rust::extractors::{CharacterNGrams, FeatureExtractor, WordNGrams};
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

        assert!(db.string_collection.contains(&s.to_string()));

        let features = db.feature_extractor.extract(s);
        let size = features.len();
        assert!(db.string_size_map.contains_key(&size));
        assert!(db.string_size_map.get(&size).unwrap().contains(s));

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

        let features_hello = db.feature_extractor.extract("hello");
        let features_world = db.feature_extractor.extract("world");
        let total_sizes = features_hello.len() + features_world.len();
        let expected_avg_size = total_sizes as f64 / 2.0;

        assert_eq!(avg_size_ngrams, expected_avg_size);

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

        let features = db.feature_extractor.extract("hello");
        let size = features.len();

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

        db.insert("hello world".to_string());
        db.insert("a quick brown fox".to_string());
        db.insert("jumps over the lazy dog".to_string());

        let sizes: Vec<usize> = db.string_size_map.keys().cloned().collect();
        assert!(sizes.len() >= 2);

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

        let _ = db.lookup_feature_set_by_size_feature(size, &feature);

        assert!(db.lookup_cache.contains_key(&(size, feature.clone())));

        let result_set = db.lookup_feature_set_by_size_feature(size, &feature);

        let expected_strings: HashSet<String> = ["test".to_string(), "text".to_string()]
            .iter()
            .cloned()
            .collect();

        assert_eq!(result_set, &expected_strings);
    }

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

        assert_eq!(
            db.string_collection,
            vec!["foo".to_string(), "bar".to_string(), "fooo".to_string()]
        );

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

        let expected_keys: Vec<usize> = vec![5, 6];
        let mut actual_keys: Vec<usize> = db.string_feature_map.keys().cloned().collect();
        actual_keys.sort();
        assert_eq!(actual_keys, expected_keys);

        let feature_map_5 = db.string_feature_map.get(&5).unwrap();
        let values_5: Vec<&HashSet<String>> = feature_map_5.values().collect();

        let vec = vec![
            HashSet::from(["foo".to_string()]),
            HashSet::from(["bar".to_string()]),
        ];
        let expected_sets = vec;
        for value_set in values_5 {
            assert!(expected_sets.contains(value_set));
        }

        let feature_map_6 = db.string_feature_map.get(&6).unwrap();
        let values_6: Vec<&HashSet<String>> = feature_map_6.values().collect();

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

        assert_eq!(db.string_collection, vec![s.to_string()]);

        let features = db.feature_extractor.extract(s);
        let size = features.len();
        assert_eq!(
            db.string_size_map.get(&size).unwrap(),
            &HashSet::from([s.to_string()])
        );

        let keys: Vec<usize> = db.string_feature_map.keys().cloned().collect();
        assert_eq!(keys, vec![size]);

        let feature_map = db.string_feature_map.get(&size).unwrap();
        let values: Vec<&HashSet<String>> = feature_map.values().collect();

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

        let (total_collection, avg_size_ngrams, total_ngrams) = db.describe_collection();

        assert_eq!(total_collection, 3);

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

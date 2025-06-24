use simstring_rust::{CharacterNgrams, Database, HashDb};
use std::sync::Arc;

#[test]
fn test_insert_and_lookup_single_string() {
    // This test verifies that after an insert, the string, its features,
    // and its inverted index entries are all correctly stored and retrievable.
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
    let mut db = HashDb::new(feature_extractor);

    db.insert("hello".to_string());
    let string_id = 0;

    // 1. Verify the string is stored and retrievable by its ID.
    assert_eq!(db.get_string(string_id), Some("hello"));

    // 2. Verify the features are correctly cached for the ID.
    let features = db.get_features(string_id).unwrap();
    let expected_features: Vec<String> = vec!["$h1", "he1", "el1", "ll1", "lo1", "o$1"]
        .into_iter()
        .map(String::from)
        .collect();
    assert_eq!(features, &expected_features);

    // 3. Verify the inverted index (feature_map) was populated correctly.
    let feature_size = features.len();
    for feature in features {
        let ids = db.lookup_strings(feature_size, feature).unwrap();
        assert!(
            ids.contains(&string_id),
            "Index should contain ID for feature '{}'",
            feature
        );
        assert_eq!(ids.len(), 1);
    }

    // 4. Verify a non-existent feature returns nothing for this size.
    assert!(db.lookup_strings(feature_size, "xx1").is_none());
}

#[test]
fn test_lookup_separates_by_size() {
    // Verifies that lookups correctly use feature set size as a primary key.
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "#"));
    let mut db = HashDb::new(feature_extractor);

    db.insert("hello".to_string()); // id 0
    db.insert("hellos".to_string()); // id 1, has a different feature count

    let features_0 = db.get_features(0).unwrap();
    let features_1 = db.get_features(1).unwrap();
    let size_0 = features_0.len();
    let size_1 = features_1.len();

    assert_ne!(
        size_0, size_1,
        "Strings should have different feature sizes"
    );

    // The feature "#h1" is common to both, but should be in different size buckets.
    let common_feature = "#h1";

    // Lookup for size of "hello" should only find id 0.
    let ids_for_size_0 = db.lookup_strings(size_0, common_feature).unwrap();
    assert!(ids_for_size_0.contains(&0));
    assert!(!ids_for_size_0.contains(&1));

    // Lookup for size of "hellos" should only find id 1.
    let ids_for_size_1 = db.lookup_strings(size_1, common_feature).unwrap();
    assert!(ids_for_size_1.contains(&1));
    assert!(!ids_for_size_1.contains(&0));
}

#[test]
fn test_insert_multiple_strings_same_size() {
    // Verifies that multiple strings with the same feature size are indexed correctly.
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
    let mut db = HashDb::new(feature_extractor);

    db.insert("cat".to_string()); // id 0
    db.insert("bat".to_string()); // id 1
    db.insert("dog".to_string()); // id 2

    let feature_size = db.get_features(0).unwrap().len();
    assert_eq!(db.get_features(1).unwrap().len(), feature_size);
    assert_eq!(db.get_features(2).unwrap().len(), feature_size);

    // "at1" is common to "cat" and "bat".
    let ids = db.lookup_strings(feature_size, "at1").unwrap();
    assert!(ids.contains(&0));
    assert!(ids.contains(&1));
    assert!(!ids.contains(&2));
    assert_eq!(ids.len(), 2);

    // "og1" is unique to "dog".
    let ids_dog = db.lookup_strings(feature_size, "og1").unwrap();
    assert!(ids_dog.contains(&2));
    assert_eq!(ids_dog.len(), 1);
}

#[test]
fn test_initial_db_state() {
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
    let db = HashDb::new(feature_extractor);

    assert_eq!(
        db.max_feature_len(),
        0,
        "max_feature_len should be 0 for empty DB"
    );
    assert_eq!(
        db.get_string(0),
        None,
        "get_string should return None for empty DB"
    );
    assert_eq!(
        db.get_features(0),
        None,
        "get_features should return None for empty DB"
    );
    assert!(
        db.lookup_strings(5, "any").is_none(),
        "lookup_strings should return None for empty DB"
    );
}

#[test]
fn test_string_collection_retrieval() {
    // This test verifies that all inserted strings are stored and can be
    // retrieved by their sequential IDs.
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
    let mut db = HashDb::new(feature_extractor);

    let corpus = vec!["foo", "bar", "fooo", "test"];
    for s in &corpus {
        db.insert(s.to_string());
    }

    // Verify that each string can be retrieved by its expected ID (0, 1, 2, ...).
    for (id, &expected_string) in corpus.iter().enumerate() {
        assert_eq!(
            db.get_string(id),
            Some(expected_string),
            "String at ID {} should be '{}'",
            id,
            expected_string
        );
    }

    // Verify that an ID just beyond the last valid one returns None.
    assert_eq!(
        db.get_string(corpus.len()),
        None,
        "An out-of-bounds ID should return None"
    );
}

#[test]
fn test_hashdb_debug_output() {
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
    let mut db = HashDb::new(feature_extractor);
    db.insert("test".to_string()); // 5 features
    db.insert("apple".to_string()); // 6 features

    let debug_output = format!("{:?}", db);

    assert!(debug_output.contains("HashDb"));
    assert!(debug_output.contains("num_strings: 2"));
    assert!(debug_output.contains("num_feature_size_buckets: 2"));
    assert!(debug_output.contains("total_unique_features: 11")); // 5 + 6 features
}

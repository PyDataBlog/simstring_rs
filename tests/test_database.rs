use simstring_rust::{CharacterNgrams, Database, HashDb};
use std::sync::Arc;

#[test]
fn test_insert_and_lookup_single_string() {
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
    let mut db = HashDb::new(feature_extractor);
    db.insert("hello".to_string());
    let string_id = 0;

    assert_eq!(
        db.get_string(string_id),
        Some("hello"),
        "Database should return 'hello' for string_id 0"
    );

    let features_spurs = db.get_features(string_id).unwrap();

    let interner_arc = db.interner();
    let interner = interner_arc.lock().unwrap();

    let features_strings: Vec<String> = features_spurs
        .iter()
        .map(|s| interner.resolve(s).to_string())
        .collect();
    let expected_features: Vec<String> = vec!["$h1", "he1", "el1", "ll1", "lo1", "o$1"]
        .into_iter()
        .map(String::from)
        .collect();
    assert_eq!(
        features_strings, expected_features,
        "Extracted features should match expected 2-grams with counts for 'hello'"
    );

    let feature_size = features_spurs.len();
    for feature_spur in features_spurs {
        let ids = db.lookup_strings(feature_size, *feature_spur).unwrap();
        assert!(
            ids.contains(&string_id),
            "Each feature should map back to string_id 0"
        );
    }

    let non_existent_spur = interner.get("xx1");
    if let Some(spur) = non_existent_spur {
        assert!(
            db.lookup_strings(feature_size, spur).is_none(),
            "Non-existent feature 'xx1' should not return any string IDs"
        );
    }
}

#[test]
fn test_lookup_separates_by_size() {
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "#"));
    let mut db = HashDb::new(feature_extractor);
    db.insert("hello".to_string());
    db.insert("hellos".to_string());

    let features_0 = db.get_features(0).unwrap();
    let features_1 = db.get_features(1).unwrap();
    let size_0 = features_0.len();
    let size_1 = features_1.len();
    assert_ne!(
        size_0, size_1,
        "Feature count for 'hello' and 'hellos' should be different"
    );

    let interner_arc = db.interner();
    let interner = interner_arc.lock().unwrap();
    let common_feature_spur = interner.get("#h1").unwrap();

    let ids_for_size_0 = db.lookup_strings(size_0, common_feature_spur).unwrap();
    assert!(
        ids_for_size_0.contains(&0),
        "Common feature '#h1' should be found in size bucket for 'hello' (id=0)"
    );
    assert!(
        !ids_for_size_0.contains(&1),
        "Size bucket for 'hello' should not contain 'hellos' (id=1)"
    );

    let ids_for_size_1 = db.lookup_strings(size_1, common_feature_spur).unwrap();
    assert!(
        ids_for_size_1.contains(&1),
        "Common feature '#h1' should be found in size bucket for 'hellos' (id=1)"
    );
    assert!(
        !ids_for_size_1.contains(&0),
        "Size bucket for 'hellos' should not contain 'hello' (id=0)"
    );
}

#[test]
fn test_insert_multiple_strings_same_size() {
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
    let mut db = HashDb::new(feature_extractor);
    db.insert("cat".to_string());
    db.insert("bat".to_string());
    db.insert("dog".to_string());

    let feature_size = db.get_features(0).unwrap().len();

    let interner_arc = db.interner();
    let interner = interner_arc.lock().unwrap();

    let at1_spur = interner.get("at1").unwrap();
    let ids = db.lookup_strings(feature_size, at1_spur).unwrap();
    assert!(
        ids.contains(&0),
        "Feature 'at1' should be found in 'cat' (id=0)"
    );
    assert!(
        ids.contains(&1),
        "Feature 'at1' should be found in 'bat' (id=1)"
    );
    assert!(
        !ids.contains(&2),
        "Feature 'at1' should not be found in 'dog' (id=2)"
    );

    let og1_spur = interner.get("og1").unwrap();
    let ids_dog = db.lookup_strings(feature_size, og1_spur).unwrap();
    assert!(
        ids_dog.contains(&2),
        "Feature 'og1' should be found in 'dog' (id=2)"
    );
    assert_eq!(
        ids_dog.len(),
        1,
        "Feature 'og1' should only be found in one string ('dog')"
    );
}

#[test]
fn test_initial_db_state() {
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
    let db = HashDb::new(feature_extractor);
    assert_eq!(
        db.max_feature_len(),
        0,
        "Empty database should have max_feature_len of 0"
    );
    assert_eq!(
        db.get_string(0),
        None,
        "Empty database should return None for any string_id"
    );
    assert_eq!(
        db.get_features(0),
        None,
        "Empty database should return None for any feature lookup"
    );
}

#[test]
fn test_string_collection_retrieval() {
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
    let mut db = HashDb::new(feature_extractor);
    let corpus = vec!["foo", "bar", "fooo", "test"];
    for s in &corpus {
        db.insert(s.to_string());
    }
    for (id, &expected_string) in corpus.iter().enumerate() {
        assert_eq!(
            db.get_string(id),
            Some(expected_string),
            "Database should return correct string for id {id}"
        );
    }
    assert_eq!(
        db.get_string(corpus.len()),
        None,
        "Database should return None for out-of-bounds string_id"
    );
}

#[test]
fn test_hashdb_debug_output() {
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
    let mut db = HashDb::new(feature_extractor);
    db.insert("test".to_string());
    db.insert("apple".to_string());
    let debug_output = format!("{db:?}");
    assert!(
        debug_output.contains("num_strings: 2"),
        "Debug output should show 2 strings inserted"
    );
    assert!(
        debug_output.contains("num_feature_size_buckets: 2"),
        "Debug output should show 2 feature size buckets (different lengths)"
    );
    assert!(
        debug_output.contains("total_unique_features_interned: 11"),
        "Debug output should show 11 total unique features interned"
    );
}

#[test]
fn test_db_clear() {
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
    let mut db = HashDb::new(feature_extractor);
    db.insert("test".to_string());

    assert_eq!(
        db.get_string(0),
        Some("test"),
        "Database should contain 'test' before clear"
    );
    assert_eq!(
        db.max_feature_len(),
        5,
        "Database should have max_feature_len of 5 before clear"
    );

    db.clear();

    assert_eq!(
        db.get_string(0),
        None,
        "Database should return None after clear"
    );
    assert_eq!(
        db.max_feature_len(),
        0,
        "Database should have max_feature_len of 0 after clear"
    );
    assert_eq!(
        db.interner().lock().unwrap().len(),
        0,
        "Interner should be empty after clear"
    );
}

#[test]
fn test_total_strings() {
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
    let mut db = HashDb::new(feature_extractor);
    db.insert("test1".to_string());
    db.insert("test2".to_string());

    assert_eq!(
        db.total_strings(),
        2,
        "Database should report 2 total strings after inserting 2 strings"
    );
}

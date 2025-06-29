use simstring_rust::{CharacterNgrams, Database, HashDb};
use std::sync::Arc;

#[test]
fn test_insert_and_lookup_single_string() {
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
    let mut db = HashDb::new(feature_extractor);
    db.insert("hello".to_string());
    let string_id = 0;

    assert_eq!(db.get_string(string_id), Some("hello"));

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
    assert_eq!(features_strings, expected_features);

    let feature_size = features_spurs.len();
    for feature_spur in features_spurs {
        let ids = db.lookup_strings(feature_size, *feature_spur).unwrap();
        assert!(ids.contains(&string_id));
    }

    let non_existent_spur = interner.get("xx1");
    if let Some(spur) = non_existent_spur {
        assert!(db.lookup_strings(feature_size, spur).is_none());
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
    assert_ne!(size_0, size_1);

    // Fix: Bind the Arc to extend its lifetime
    let interner_arc = db.interner();
    let interner = interner_arc.lock().unwrap();
    let common_feature_spur = interner.get("#h1").unwrap();

    let ids_for_size_0 = db.lookup_strings(size_0, common_feature_spur).unwrap();
    assert!(ids_for_size_0.contains(&0));
    assert!(!ids_for_size_0.contains(&1));

    let ids_for_size_1 = db.lookup_strings(size_1, common_feature_spur).unwrap();
    assert!(ids_for_size_1.contains(&1));
    assert!(!ids_for_size_1.contains(&0));
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
    assert!(ids.contains(&0));
    assert!(ids.contains(&1));
    assert!(!ids.contains(&2));

    let og1_spur = interner.get("og1").unwrap();
    let ids_dog = db.lookup_strings(feature_size, og1_spur).unwrap();
    assert!(ids_dog.contains(&2));
    assert_eq!(ids_dog.len(), 1);
}

#[test]
fn test_initial_db_state() {
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
    let db = HashDb::new(feature_extractor);
    assert_eq!(db.max_feature_len(), 0);
    assert_eq!(db.get_string(0), None);
    assert_eq!(db.get_features(0), None);
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
        assert_eq!(db.get_string(id), Some(expected_string));
    }
    assert_eq!(db.get_string(corpus.len()), None);
}

#[test]
fn test_hashdb_debug_output() {
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
    let mut db = HashDb::new(feature_extractor);
    db.insert("test".to_string());
    db.insert("apple".to_string());
    let debug_output = format!("{db:?}");
    assert!(debug_output.contains("num_strings: 2"));
    assert!(debug_output.contains("num_feature_size_buckets: 2"));
    assert!(debug_output.contains("total_unique_features_interned: 11"));
}

#[test]
fn test_db_clear() {
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
    let mut db = HashDb::new(feature_extractor);
    db.insert("test".to_string());

    // Ensure DB is not empty
    assert_eq!(db.get_string(0), Some("test"));
    assert_eq!(db.max_feature_len(), 5);

    db.clear();

    // Ensure DB is now empty
    assert_eq!(db.get_string(0), None);
    assert_eq!(db.max_feature_len(), 0);
    assert_eq!(db.interner().lock().unwrap().len(), 0);
}

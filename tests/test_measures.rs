use simstring_rust::{CharacterNgrams, Cosine, Database, HashDb, Measure};
use std::sync::Arc;

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn test_cosine_similarity_score() {
    let cosine = Cosine;
    let x: Vec<String> = ["a", "b", "c"].iter().map(|&s| s.to_string()).collect();
    let y: Vec<String> = ["a", "b", "d", "e"]
        .iter()
        .map(|&s| s.to_string())
        .collect();

    let score = cosine.similarity(&x, &y);
    assert!(
        approx_eq(score, 0.5773502691896258),
        "Expected Cosine similarity to be ~0.577, got {}",
        score
    );

    let z: Vec<String> = ["a", "b", "c"].iter().map(|&s| s.to_string()).collect();
    let score_exact = cosine.similarity(&x, &z);
    assert!(
        approx_eq(score_exact, 1.0),
        "Expected Cosine similarity for exact match to be 1.0, got {}",
        score_exact
    );
}

#[test]
fn test_cosine_min_feature_size() {
    let cosine = Cosine;
    let query_size = 5;

    assert_eq!(
        cosine.min_feature_size(query_size, 1.0),
        5,
        "Expected min feature size for alpha=1.0 to be 5"
    );

    assert_eq!(
        cosine.min_feature_size(query_size, 0.5),
        2,
        "Expected min feature size for alpha=0.5 to be 2"
    );
}

#[test]
fn test_cosine_max_feature_size() {
    let cosine = Cosine;
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
    let mut db = HashDb::new(feature_extractor);

    db.insert("123456789".to_string());
    assert_eq!(
        db.max_feature_len(),
        10,
        "DB max feature length should be 10"
    );

    let query_size = 5;

    assert_eq!(
        cosine.max_feature_size(query_size, 1.0, &db),
        5,
        "Expected max feature size for alpha=1.0 to be 5"
    );

    assert_eq!(
        cosine.max_feature_size(query_size, 0.5, &db),
        10,
        "Expected max feature size for alpha=0.5 to be 10 (capped by db)"
    );
}

#[test]
fn test_cosine_minimum_common_feature_count() {
    let cosine = Cosine;
    let query_size = 5;

    assert_eq!(
        cosine.minimum_common_feature_count(query_size, 5, 1.0),
        5,
        "Expected min overlap for sizes (5,5) and alpha=1.0 to be 5"
    );

    assert_eq!(
        cosine.minimum_common_feature_count(query_size, 20, 1.0),
        10,
        "Expected min overlap for sizes (5,20) and alpha=1.0 to be 10"
    );

    assert_eq!(
        cosine.minimum_common_feature_count(query_size, 5, 0.5),
        3,
        "Expected min overlap for sizes (5,5) and alpha=0.5 to be 3"
    );
}

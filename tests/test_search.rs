use simstring_rust::{
    CharacterNgrams, Cosine, Dice, ExactMatch, HashDb, Jaccard, Overlap, SearchError, Searcher,
};
use std::sync::Arc;

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn test_cosine_search_basic() {
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
    let measure = Cosine;
    let mut db = HashDb::new(feature_extractor);

    db.insert("foo".to_string());
    db.insert("bar".to_string());
    db.insert("fooo".to_string());

    let searcher = Searcher::new(&db, measure);
    let results = searcher.ranked_search("foo", 0.8).unwrap();

    assert_eq!(results.len(), 2, "Expected 2 results");
    assert_eq!(results[0].0, "foo", "First result should be 'foo'");
    assert!(
        approx_eq(results[0].1, 1.0),
        "Score for 'foo' should be 1.0, got {}",
        results[0].1
    );
    assert_eq!(results[1].0, "fooo", "Second result should be 'fooo'");
    assert!(
        approx_eq(results[1].1, 0.8944271909999159),
        "Score for 'fooo' should be ~0.8944, got {}",
        results[1].1
    );
}

#[test]
fn test_dice_search_basic() {
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
    let measure = Dice;
    let mut db = HashDb::new(feature_extractor);

    db.insert("foo".to_string());
    db.insert("bar".to_string());
    db.insert("fooo".to_string());

    let searcher = Searcher::new(&db, measure);
    let results = searcher.ranked_search("foo", 0.8).unwrap();

    assert_eq!(results.len(), 2, "Expected 2 results");
    assert_eq!(results[0].0, "foo", "First result should be 'foo'");
    assert!(
        approx_eq(results[0].1, 1.0),
        "Score for 'foo' should be 1.0, got {}",
        results[0].1
    );
    assert_eq!(results[1].0, "fooo", "Second result should be 'fooo'");
    assert!(
        approx_eq(results[1].1, 0.8888888888888888),
        "Score for 'fooo' should be ~0.8889, got {}",
        results[1].1
    );
}

#[test]
fn test_jaccard_search_basic() {
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
    let measure = Jaccard;
    let mut db = HashDb::new(feature_extractor);

    db.insert("foo".to_string());
    db.insert("bar".to_string());
    db.insert("fooo".to_string());

    let searcher = Searcher::new(&db, measure);
    let results = searcher.ranked_search("foo", 0.8).unwrap();

    assert_eq!(results.len(), 2, "Expected 2 results");
    assert_eq!(results[0].0, "foo", "First result should be 'foo'");
    assert!(
        approx_eq(results[0].1, 1.0),
        "Score for 'foo' should be 1.0, got {}",
        results[0].1
    );
    assert_eq!(results[1].0, "fooo", "Second result should be 'fooo'");
    assert!(
        approx_eq(results[1].1, 0.8),
        "Score for 'fooo' should be 0.8, got {}",
        results[1].1
    );
}

#[test]
fn test_overlap_search_basic() {
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
    let measure = Overlap;
    let mut db = HashDb::new(feature_extractor);

    db.insert("foo".to_string());
    db.insert("bar".to_string());
    db.insert("fooo".to_string());

    let searcher = Searcher::new(&db, measure);
    let results = searcher.ranked_search("foo", 0.8).unwrap();

    assert_eq!(results.len(), 2, "Expected 2 results");
    assert_eq!(results[0].0, "foo");
    assert!(approx_eq(results[0].1, 1.0));
    assert_eq!(results[1].0, "fooo");
    assert!(approx_eq(results[1].1, 1.0));
}

#[test]
fn test_exact_match_search_basic() {
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
    let measure = ExactMatch;
    let mut db = HashDb::new(feature_extractor);

    db.insert("foo".to_string());
    db.insert("bar".to_string());
    db.insert("fooo".to_string());

    let searcher = Searcher::new(&db, measure);

    let thresholds = [0.1, 0.5, 0.9, 1.0];
    for &threshold in &thresholds {
        let results = searcher.ranked_search("foo", threshold).unwrap();
        assert_eq!(
            results.len(),
            1,
            "Expected 1 result for threshold {}",
            threshold
        );
        assert_eq!(results[0].0, "foo", "Result should be 'foo'");
        assert!(
            approx_eq(results[0].1, 1.0),
            "Score for 'foo' should be 1.0, got {}",
            results[0].1
        );
    }

    let results_none = searcher.ranked_search("baz", 0.5).unwrap();
    assert!(results_none.is_empty(), "Expected no results for 'baz'");
}

#[test]
fn test_search_with_different_endmarkers_cosine() {
    let measure = Cosine;

    let fe_dollar = Arc::new(CharacterNgrams::new(2, "$"));
    let mut db_dollar = HashDb::new(fe_dollar);
    db_dollar.insert("test".to_string());
    let searcher_dollar = Searcher::new(&db_dollar, measure);
    let results_dollar = searcher_dollar.ranked_search("test", 0.8).unwrap();
    assert_eq!(results_dollar.len(), 1, "Using '$' endmarker");
    if !results_dollar.is_empty() {
        assert!(approx_eq(results_dollar[0].1, 1.0));
    }

    let fe_hash = Arc::new(CharacterNgrams::new(2, "#"));
    let mut db_hash = HashDb::new(fe_hash);
    db_hash.insert("test".to_string());
    let searcher_hash = Searcher::new(&db_hash, measure);
    let results_hash = searcher_hash.ranked_search("test", 0.8).unwrap();
    assert_eq!(results_hash.len(), 1, "Using '#' endmarker");
    if !results_hash.is_empty() {
        assert!(approx_eq(results_hash[0].1, 1.0));
    }
}

#[test]
fn test_search_with_different_ngram_sizes_cosine() {
    let measure = Cosine;
    let endmarker = "$";

    let fe_2 = Arc::new(CharacterNgrams::new(2, endmarker));
    let mut db_2 = HashDb::new(fe_2);
    db_2.insert("test".to_string());
    let searcher_2 = Searcher::new(&db_2, measure);
    let results_2 = searcher_2.ranked_search("test", 0.8).unwrap();
    assert_eq!(results_2.len(), 1, "Using n=2 ngrams");
    if !results_2.is_empty() {
        assert!(approx_eq(results_2[0].1, 1.0));
    }

    let fe_3 = Arc::new(CharacterNgrams::new(3, endmarker));
    let mut db_3 = HashDb::new(fe_3);
    db_3.insert("test".to_string());
    let searcher_3 = Searcher::new(&db_3, measure);
    let results_3 = searcher_3.ranked_search("test", 0.8).unwrap();
    assert_eq!(results_3.len(), 1, "Using n=3 ngrams");
    if !results_3.is_empty() {
        assert!(approx_eq(results_3[0].1, 1.0));
    }
}

#[test]
fn test_cosine_with_repeated_ngrams_in_query() {
    let query = "aaaa";
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
    let measure = Cosine;
    let mut db = HashDb::new(feature_extractor);

    db.insert("aaab".to_string());
    db.insert("bbbb".to_string());

    let searcher = Searcher::new(&db, measure);
    let results = searcher.ranked_search(query, 0.5).unwrap();

    assert_eq!(results.len(), 1, "Expected 1 result for 'aaaa' vs 'aaab'");
    assert_eq!(results[0].0, "aaab");
    assert!(
        approx_eq(results[0].1, 0.6),
        "Score for 'aaab' should be 0.6, got {}",
        results[0].1
    );
}

#[test]
fn test_unranked_search() {
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
    let measure = Cosine;
    let mut db = HashDb::new(feature_extractor);

    // Non-alphabetical order to test the sorting
    db.insert("fooo".to_string());
    db.insert("bar".to_string());
    db.insert("foo".to_string());

    let searcher = Searcher::new(&db, measure);

    let results = searcher.search("foo", 0.8).unwrap();
    // The unranked search should still find the same candidates as ranked_search,
    // but without scores and sorted alphabetically.
    assert_eq!(results, vec!["foo", "fooo"]);

    let results_exact = searcher.search("bar", 1.0).unwrap();
    assert_eq!(results_exact, vec!["bar"]);

    let results_none = searcher.search("xyz", 0.9).unwrap();
    assert!(results_none.is_empty());

    let result_err = searcher.search("foo", -0.5);
    assert!(result_err.is_err());
    assert_eq!(result_err.unwrap_err(), SearchError::InvalidThreshold(-0.5));
}

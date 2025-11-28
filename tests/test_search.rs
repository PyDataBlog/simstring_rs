use lasso::{Rodeo, Spur};
use rustc_hash::FxHashSet;
use simstring_rust::database::StringId;
use simstring_rust::{
    CharacterNgrams, Cosine, Database, Dice, ExactMatch, FeatureExtractor, HashDb, Jaccard,
    Overlap, SearchError, Searcher,
};
use std::sync::{Arc, Mutex};

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

    assert_eq!(
        results.len(),
        2,
        "Cosine search for 'foo' with alpha=0.8 should return 2 results"
    );
    assert_eq!(
        results[0].0, "foo",
        "First result should be exact match 'foo'"
    );
    assert!(
        approx_eq(results[0].1, 1.0),
        "Cosine score for exact match 'foo' should be 1.0, got {}",
        results[0].1
    );
    assert_eq!(results[1].0, "fooo", "Second result should be 'fooo'");
    assert!(
        approx_eq(results[1].1, 0.8944271909999159),
        "Cosine score for 'fooo' should be ~0.8944, got {}",
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

    assert_eq!(
        results.len(),
        2,
        "Dice search for 'foo' with alpha=0.8 should return 2 results"
    );
    assert_eq!(
        results[0].0, "foo",
        "First result should be exact match 'foo'"
    );
    assert!(
        approx_eq(results[0].1, 1.0),
        "Dice score for exact match 'foo' should be 1.0, got {}",
        results[0].1
    );
    assert_eq!(results[1].0, "fooo", "Second result should be 'fooo'");
    assert!(
        approx_eq(results[1].1, 0.8888888888888888),
        "Dice score for 'fooo' should be ~0.8889, got {}",
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

    assert_eq!(
        results.len(),
        2,
        "Jaccard search for 'foo' with alpha=0.8 should return 2 results"
    );
    assert_eq!(
        results[0].0, "foo",
        "First result should be exact match 'foo'"
    );
    assert!(
        approx_eq(results[0].1, 1.0),
        "Jaccard score for exact match 'foo' should be 1.0, got {}",
        results[0].1
    );
    assert_eq!(results[1].0, "fooo", "Second result should be 'fooo'");
    assert!(
        approx_eq(results[1].1, 0.8),
        "Jaccard score for 'fooo' should be 0.8, got {}",
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

    assert_eq!(
        results.len(),
        2,
        "Overlap search for 'foo' with alpha=0.8 should return 2 results"
    );
    assert_eq!(
        results[0].0, "foo",
        "First result should be exact match 'foo'"
    );
    assert!(
        approx_eq(results[0].1, 1.0),
        "Overlap score for exact match 'foo' should be 1.0, got {}",
        results[0].1
    );
    assert_eq!(results[1].0, "fooo", "Second result should be 'fooo'");
    assert!(
        approx_eq(results[1].1, 1.0),
        "Overlap score for 'fooo' should be 1.0 (all query features present), got {}",
        results[1].1
    );
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
            "ExactMatch search for 'foo' with alpha={threshold} should return exactly 1 result"
        );
        assert_eq!(
            results[0].0, "foo",
            "ExactMatch result should be 'foo' for threshold {threshold}"
        );
        assert!(
            approx_eq(results[0].1, 1.0),
            "ExactMatch score should be 1.0 for threshold {threshold}, got {}",
            results[0].1
        );
    }

    let results_none = searcher.ranked_search("baz", 0.5).unwrap();
    assert!(
        results_none.is_empty(),
        "ExactMatch search for 'baz' (not in database) should return no results"
    );
}

#[test]
fn test_search_with_different_endmarkers_cosine() {
    let measure = Cosine;

    let fe_dollar = Arc::new(CharacterNgrams::new(2, "$"));
    let mut db_dollar = HashDb::new(fe_dollar);
    db_dollar.insert("test".to_string());
    let searcher_dollar = Searcher::new(&db_dollar, measure);
    let results_dollar = searcher_dollar.ranked_search("test", 0.8).unwrap();
    assert_eq!(
        results_dollar.len(),
        1,
        "Search with '$' endmarker should find 'test'"
    );
    if !results_dollar.is_empty() {
        assert!(
            approx_eq(results_dollar[0].1, 1.0),
            "Exact match score with '$' endmarker should be 1.0"
        );
    }

    let fe_hash = Arc::new(CharacterNgrams::new(2, "#"));
    let mut db_hash = HashDb::new(fe_hash);
    db_hash.insert("test".to_string());
    let searcher_hash = Searcher::new(&db_hash, measure);
    let results_hash = searcher_hash.ranked_search("test", 0.8).unwrap();
    assert_eq!(
        results_hash.len(),
        1,
        "Search with '#' endmarker should find 'test'"
    );
    if !results_hash.is_empty() {
        assert!(
            approx_eq(results_hash[0].1, 1.0),
            "Exact match score with '#' endmarker should be 1.0"
        );
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
    assert_eq!(results_2.len(), 1, "Search with 2-grams should find 'test'");
    if !results_2.is_empty() {
        assert!(
            approx_eq(results_2[0].1, 1.0),
            "Exact match score with 2-grams should be 1.0"
        );
    }

    let fe_3 = Arc::new(CharacterNgrams::new(3, endmarker));
    let mut db_3 = HashDb::new(fe_3);
    db_3.insert("test".to_string());
    let searcher_3 = Searcher::new(&db_3, measure);
    let results_3 = searcher_3.ranked_search("test", 0.8).unwrap();
    assert_eq!(results_3.len(), 1, "Search with 3-grams should find 'test'");
    if !results_3.is_empty() {
        assert!(
            approx_eq(results_3[0].1, 1.0),
            "Exact match score with 3-grams should be 1.0"
        );
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

    assert_eq!(
        results.len(),
        1,
        "Cosine search for 'aaaa' with alpha=0.5 should find 'aaab' (repeated n-grams)"
    );
    assert_eq!(results[0].0, "aaab", "Result should be 'aaab'");
    assert!(
        approx_eq(results[0].1, 0.6),
        "Cosine score for 'aaab' with repeated n-grams should be 0.6, got {}",
        results[0].1
    );
}

#[test]
fn test_unranked_search() {
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
    let measure = Cosine;
    let mut db = HashDb::new(feature_extractor);

    db.insert("fooo".to_string());
    db.insert("bar".to_string());
    db.insert("foo".to_string());

    let searcher = Searcher::new(&db, measure);

    let results = searcher.search("foo", 0.8).unwrap();
    assert_eq!(
        results,
        vec!["foo", "fooo"],
        "Unranked search should return results in alphabetical order without scores"
    );

    let results_exact = searcher.search("bar", 1.0).unwrap();
    assert_eq!(
        results_exact,
        vec!["bar"],
        "Unranked search with alpha=1.0 should return exact match only"
    );

    let results_none = searcher.search("xyz", 0.9).unwrap();
    assert!(
        results_none.is_empty(),
        "Unranked search for non-existent string should return empty results"
    );

    let result_err = searcher.search("foo", -0.5);
    assert!(
        result_err.is_err(),
        "Search with negative alpha should return error"
    );
    assert_eq!(
        result_err.unwrap_err(),
        SearchError::InvalidThreshold(-0.5),
        "Error should be InvalidThreshold with alpha=-0.5"
    );
}

struct MockDatabase {
    real_db: HashDb,
}

impl MockDatabase {
    fn new() -> Self {
        let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
        let mut real_db = HashDb::new(feature_extractor);
        real_db.insert("foo".to_string());
        Self { real_db }
    }
}

impl Database for MockDatabase {
    fn insert(&mut self, text: String) {
        self.real_db.insert(text);
    }

    fn clear(&mut self) {
        self.real_db.clear();
    }

    fn lookup_strings(&self, size: usize, feature: Spur) -> Option<&FxHashSet<StringId>> {
        self.real_db.lookup_strings(size, feature)
    }

    fn get_string(&self, _id: StringId) -> Option<&str> {
        None
    }

    fn get_features(&self, _id: StringId) -> Option<&Vec<Spur>> {
        None
    }

    fn feature_extractor(&self) -> &dyn FeatureExtractor {
        self.real_db.feature_extractor()
    }

    fn max_feature_len(&self) -> usize {
        self.real_db.max_feature_len()
    }

    fn interner(&self) -> Arc<Mutex<Rodeo>> {
        self.real_db.interner()
    }

    fn total_strings(&self) -> usize {
        self.real_db.total_strings()
    }
}

#[test]
fn test_empty_string_insertion_and_search() {
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
    let measure = Cosine;
    let mut db = HashDb::new(feature_extractor);

    db.insert("".to_string());
    db.insert("foo".to_string());

    let searcher = Searcher::new(&db, measure);

    let results = searcher.search("", 1.0).unwrap();
    assert!(
        results.contains(&""),
        "Search for empty string should find empty string in database"
    );

    let results_foo = searcher.search("foo", 1.0).unwrap();
    assert_eq!(
        results_foo,
        vec!["foo"],
        "Search for 'foo' with alpha=1.0 should return only 'foo', not empty string"
    );
}

#[test]
fn test_unicode_handling() {
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
    let measure = Cosine;
    let mut db = HashDb::new(feature_extractor);

    let emoji_str = "🦀🚀";
    let cjk_str = "你好世界";
    let mixed_str = "hello world 🌍";

    db.insert(emoji_str.to_string());
    db.insert(cjk_str.to_string());
    db.insert(mixed_str.to_string());

    let searcher = Searcher::new(&db, measure);

    let results_emoji = searcher.search("🦀🚀", 1.0).unwrap();
    assert_eq!(
        results_emoji,
        vec![emoji_str],
        "Search should handle emoji characters correctly and return exact match"
    );

    let results_cjk = searcher.search("你好世界", 1.0).unwrap();
    assert_eq!(
        results_cjk,
        vec![cjk_str],
        "Search should handle CJK characters correctly and return exact match"
    );

    let results_mixed = searcher.search("hello 🌍", 0.5).unwrap();
    assert!(
        results_mixed.contains(&mixed_str),
        "Partial search should find mixed ASCII/Unicode string"
    );
}

#[test]
fn test_ngram_size_larger_than_string() {
    let feature_extractor = Arc::new(CharacterNgrams::new(5, "$"));
    let measure = Cosine;
    let mut db = HashDb::new(feature_extractor);

    db.insert("hi".to_string());

    let searcher = Searcher::new(&db, measure);

    let results = searcher.search("hi", 1.0).unwrap();
    assert_eq!(
        results,
        vec!["hi"],
        "Search should work even when n-gram size (5) is larger than string length (2)"
    );
}

#[test]
fn test_threshold_boundaries() {
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
    let measure = Cosine;
    let mut db = HashDb::new(feature_extractor);

    db.insert("foo".to_string());
    let searcher = Searcher::new(&db, measure);

    let results_1 = searcher.search("foo", 1.0).unwrap();
    assert_eq!(
        results_1,
        vec!["foo"],
        "Threshold 1.0 should return exact match only"
    );

    let err_0 = searcher.search("foo", 0.0);
    assert!(
        matches!(err_0, Err(SearchError::InvalidThreshold(0.0))),
        "Threshold 0.0 (inclusive lower bound) should be invalid"
    );

    let results_small = searcher.search("foo", 0.0001).unwrap();
    assert_eq!(
        results_small,
        vec!["foo"],
        "Threshold slightly above 0.0 should be valid and return results"
    );

    let err_large = searcher.search("foo", 1.1);
    assert!(
        matches!(err_large, Err(SearchError::InvalidThreshold(1.1))),
        "Threshold > 1.0 should be invalid"
    );
}

#[test]
fn test_no_results() {
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
    let measure = Cosine;
    let mut db = HashDb::new(feature_extractor);

    db.insert("foo".to_string());
    let searcher = Searcher::new(&db, measure);

    let results = searcher.search("bar", 0.9).unwrap();
    assert!(
        results.is_empty(),
        "Search for string with no similar matches should return empty results"
    );
}

#[test]
fn test_search_tau_equals_one_optimization() {
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
    let measure = Overlap;
    let mut db = HashDb::new(feature_extractor);

    db.insert("foo".to_string());
    db.insert("far".to_string());

    let searcher = Searcher::new(&db, measure);

    let results = searcher.search("far", 0.1).unwrap();
    assert!(
        results.contains(&"foo"),
        "Overlap search with low alpha should find 'foo' (tau=1 optimization triggers)"
    );
    assert!(
        results.contains(&"far"),
        "Overlap search should find exact match 'far'"
    );
}

#[test]
fn test_ranked_search_none_handling() {
    let db = MockDatabase::new();
    let measure = Overlap;
    let searcher = Searcher::new(&db, measure);

    let results = searcher.ranked_search("foo", 0.1).unwrap();
    assert!(
        results.is_empty(),
        "Ranked search should filter out candidates when get_string/get_features return None"
    );
}

#[test]
fn test_search_candidates_empty_query() {
    let feature_extractor = Arc::new(CharacterNgrams::new(100, ""));
    let measure = Overlap;
    let mut db = HashDb::new(feature_extractor);
    db.insert("foo".to_string());

    let searcher = Searcher::new(&db, measure);

    let results = searcher.search("foo", 0.5).unwrap();
    assert!(
        results.is_empty(),
        "Search with empty query features (n-gram size too large) should return empty results"
    );
}

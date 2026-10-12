use lasso::Rodeo;
use simstring_rust::{
    CharacterNgrams, Cosine, Database, Dice, ExactMatch, FeatureExtractor, HashDb, Jaccard,
    Measure, Overlap, Searcher, WordNgrams,
};
use std::sync::Arc;

fn compare<M: Measure + Copy>(extractor: Arc<dyn FeatureExtractor>, measure: M) {
    let mut corpus = vec![
        "".to_owned(),
        "aaaa".into(),
        "aaaa".into(),
        "東京🍕".into(),
        "café".into(),
        "a b a b".into(),
        "a b c".into(),
    ];
    for length in 1..=6 {
        for bits in 0..(1 << length) {
            corpus.push(
                (0..length)
                    .map(|i| if bits & (1 << i) == 0 { 'a' } else { 'b' })
                    .collect(),
            );
        }
    }
    let mut db = HashDb::new(Arc::clone(&extractor));
    let mut reference = Rodeo::default();
    let candidate_features: Vec<_> = corpus
        .iter()
        .map(|text| {
            db.insert(text.clone());
            extractor.features(text, &mut reference)
        })
        .collect();
    let interned = db.interner().len();
    let searcher = Searcher::new(&db, measure);
    // Use an independent interner and brute-force feature comparisons as the oracle.
    for query in corpus.iter().step_by(7).map(String::as_str).chain([
        "never seen 🦀",
        "aa🦀aa",
        "東京",
        "café 🍕",
        "a b unknown",
        "aaaaaaaaaaaaaa",
    ]) {
        let query_features = extractor.features(query, &mut reference);
        let mut thresholds = vec![f64::from_bits(1), 0.01, 0.3, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
        thresholds.extend(
            candidate_features
                .iter()
                .step_by(11)
                .map(|features| measure.similarity(&query_features, features))
                .filter(|&score| score > 0.0),
        );
        for alpha in thresholds {
            let mut expected: Vec<_> = corpus
                .iter()
                .zip(&candidate_features)
                .filter_map(|(text, features)| {
                    // Keep the established no-features search behavior.
                    if query_features.is_empty() || features.is_empty() {
                        return None;
                    }
                    let score = measure.similarity(&query_features, features);
                    (score >= alpha).then_some((text.as_str(), score))
                })
                .collect();
            expected.sort_unstable_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(b.0)));
            assert_eq!(
                searcher.ranked_search(query, alpha).unwrap(),
                expected,
                "query={query:?}, alpha={alpha}"
            );
            let mut expected_text: Vec<_> = expected.iter().map(|&(text, _)| text).collect();
            expected_text.sort_unstable();
            assert_eq!(searcher.search(query, alpha).unwrap(), expected_text);
        }
    }
    assert_eq!(
        db.interner().len(),
        interned,
        "queries must not intern features"
    );
}

#[test]
fn parallel_size_buckets_preserve_duplicates() {
    let mut db = HashDb::new(Arc::new(CharacterNgrams::default()));
    let corpus = [
        "a", "aa", "aaa", "aaaa", "aaaaa", "aaaaaa", "aaaaaaa", "aaaaaaaa",
    ];
    for text in corpus.iter().cycle().take(100_000) {
        db.insert((*text).into());
    }
    let searcher = Searcher::new(&db, Cosine);
    let mut expected = Vec::new();
    let mut interner = Rodeo::default();
    let extractor = CharacterNgrams::default();
    let query = extractor.features("aaaa", &mut interner);
    for text in corpus {
        let score = Cosine.similarity(&query, &extractor.features(text, &mut interner));
        if score >= 0.5 {
            expected.extend(std::iter::repeat_n((text, score), 12_500));
        }
    }
    expected.sort_unstable_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    assert_eq!(searcher.ranked_search("aaaa", 0.5).unwrap(), expected);
}

#[test]
fn all_measures_match_brute_force() {
    let extractors: Vec<Arc<dyn FeatureExtractor>> = vec![
        Arc::new(CharacterNgrams::new(2, "$")),
        Arc::new(CharacterNgrams::new(3, "🍕!")),
        Arc::new(CharacterNgrams::new(2, "")),
        Arc::new(WordNgrams::new(2, " ", "$")),
    ];
    for extractor in extractors {
        compare(Arc::clone(&extractor), Cosine);
        compare(Arc::clone(&extractor), Dice);
        compare(Arc::clone(&extractor), Jaccard);
        compare(Arc::clone(&extractor), Overlap);
        compare(extractor, ExactMatch);
    }
}

#[test]
fn concurrent_queries_and_reinsertion() {
    let mut db = HashDb::new(Arc::new(CharacterNgrams::default()));
    db.insert("old entry".into());
    db.clear();
    db.insert("new entry".into());
    let searcher = Searcher::new(&db, Cosine);
    std::thread::scope(|scope| {
        for _ in 0..8 {
            scope.spawn(|| {
                for _ in 0..50 {
                    assert_eq!(searcher.search("new entry", 1.0).unwrap(), ["new entry"]);
                    assert!(searcher.search("unknown 🦀", 1.0).unwrap().is_empty());
                }
            });
        }
    });
    for alpha in [0.0, -0.1, 1.1, f64::NAN, f64::INFINITY] {
        assert!(searcher.search("new entry", alpha).is_err());
    }
}

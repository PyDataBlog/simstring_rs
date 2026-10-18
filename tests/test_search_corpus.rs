//! Locks search behaviour on realistic data: every result of `search` and `ranked_search`
//! must equal a brute-force scan over a slice of the benchmark corpus.

use lasso::{Rodeo, Spur};
use simstring_rust::{
    CharacterNgrams, Cosine, Dice, ExactMatch, FeatureExtractor, HashDb, Jaccard, Measure, Overlap,
    Searcher,
};
use std::sync::Arc;

/// Intersection of two sorted, unique feature sets, independent of the library.
fn common(x: &[Spur], y: &[Spur]) -> usize {
    let (mut i, mut j, mut n) = (0, 0, 0);
    while i < x.len() && j < y.len() {
        match x[i].cmp(&y[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => (i, j, n) = (i + 1, j + 1, n + 1),
        }
    }
    n
}

fn check<M: Measure + Copy>(corpus: &[String], queries: &[String], n: usize, measure: M) {
    let extractor = CharacterNgrams::new(n, " ");
    let mut db = HashDb::new(Arc::new(extractor.clone()));
    let mut reference = Rodeo::default();
    let features: Vec<_> = corpus
        .iter()
        .map(|text| {
            db.insert(text.clone());
            extractor.features(text, &mut reference)
        })
        .collect();
    let searcher = Searcher::new(&db, measure);
    for query in queries {
        let query_features = extractor.features(query, &mut reference);
        let scores: Vec<_> = features
            .iter()
            .map(|f| {
                let common = common(&query_features, f);
                measure.similarity_from_counts(query_features.len(), f.len(), common)
            })
            .collect();
        for alpha in [0.5, 0.6, 0.7, 0.8, 0.9, 1.0] {
            let mut expected: Vec<_> = corpus
                .iter()
                .zip(&scores)
                .filter(|&(_, &score)| score >= alpha)
                .map(|(text, &score)| (text.as_str(), score))
                .collect();
            expected.sort_unstable_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(b.0)));
            let ranked = searcher.ranked_search(query, alpha).unwrap();
            // Scores must match bit for bit, not approximately.
            let bits = |v: &[(&str, f64)]| -> Vec<_> {
                v.iter()
                    .map(|&(t, s)| (t.to_owned(), s.to_bits()))
                    .collect()
            };
            assert_eq!(
                bits(&ranked),
                bits(&expected),
                "n={n} q={query:?} a={alpha}"
            );
            let mut texts: Vec<_> = expected.into_iter().map(|(t, _)| t).collect();
            texts.sort_unstable();
            assert_eq!(searcher.search(query, alpha).unwrap(), texts);
        }
    }
}

#[test]
fn benchmark_corpus_matches_brute_force() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/benches/data/company_names.txt"
    );
    let corpus: Vec<String> = std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .take(1000)
        .map(str::to_owned)
        .collect();
    // Indexed queries plus a few that do not occur verbatim.
    let mut queries: Vec<String> = corpus.iter().step_by(34).cloned().collect();
    queries.extend(["acme holdings", "zzzz", "International"].map(String::from));
    for n in [2, 3, 4] {
        check(&corpus, &queries, n, Cosine);
        check(&corpus, &queries, n, Dice);
        check(&corpus, &queries, n, Jaccard);
        check(&corpus, &queries, n, Overlap);
        check(&corpus, &queries, n, ExactMatch);
    }
}

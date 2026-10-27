use serde::Serialize;
use serde_json::{Value, json};
use simstring_rust::{CharacterNgrams, Cosine, FeatureExtractor, HashDb, Searcher, WordNgrams};
use std::{env, fs, hint::black_box, sync::Arc, time::Instant};

const SAMPLES: usize = 7;
const SEARCH_REPEATS: usize = 50;

#[derive(Serialize)]
struct ResultRow {
    case: String,
    samples_ms: Vec<f64>,
    check: Value,
}

fn measure<T>(mut run: impl FnMut() -> T, repeats: usize) -> Vec<f64> {
    let mut samples = Vec::new();
    for iteration in 0..SAMPLES + 2 {
        let start = Instant::now();
        let value = run();
        let elapsed = start.elapsed().as_secs_f64() * 1000. / repeats as f64;
        drop(black_box(value));
        if iteration >= 2 {
            samples.push(elapsed);
        }
    }
    samples
}

fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    let corpus: Vec<_> = fs::read_to_string(&args[0])
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();
    let queries: Vec<_> = corpus
        .iter()
        .take(100)
        .map(String::as_str)
        .chain(["unknown 東京🦀", ""])
        .collect();
    let selected = &args[1..];
    let mut results = Vec::new();
    for kind in ["character", "word"] {
        for n in [2, 3, 4] {
            let extractor: Arc<dyn FeatureExtractor> = match kind {
                "character" => Arc::new(CharacterNgrams::new(n, " ")),
                _ => Arc::new(WordNgrams::new(n, " ", " ")),
            };
            let build = || {
                let mut db = HashDb::new(Arc::clone(&extractor));
                for text in &corpus {
                    db.insert(text.clone());
                }
                db
            };
            let db = build();
            assert_eq!(db.strings, corpus);
            let case = format!("{kind}/{n}/insert");
            if selected.is_empty() || selected.contains(&case) {
                results.push(ResultRow {
                    case,
                    samples_ms: measure(build, 1),
                    check: json!(corpus.len()),
                });
            }
            let searcher = Searcher::new(&db, Cosine);
            for (label, alpha) in [("0.6", 0.6), ("0.9", 0.9)] {
                let case = format!("{kind}/{n}/search/{label}");
                if !selected.is_empty() && !selected.contains(&case) {
                    continue;
                }
                let check: Vec<_> = queries
                    .iter()
                    .map(|query| searcher.search(query, alpha).unwrap())
                    .collect();
                let samples_ms = measure(
                    || {
                        for _ in 0..SEARCH_REPEATS {
                            for query in &queries {
                                black_box(searcher.search(query, alpha).unwrap());
                            }
                        }
                    },
                    SEARCH_REPEATS,
                );
                results.push(ResultRow {
                    case,
                    samples_ms,
                    check: json!(check),
                });
            }
        }
    }
    println!("{}", serde_json::to_string(&results).unwrap());
}

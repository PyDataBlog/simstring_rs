//! `cargo bench --bench persistence -- [company|unigrams] [--json]`
use serde::Serialize;
use simstring_rust::{CharacterNgrams, HashDb};
use std::{fs, hint::black_box, path::Path, sync::Arc, time::Instant};

const SAMPLES: usize = 7;

#[derive(Serialize)]
struct BenchmarkResult {
    language: &'static str,
    backend: &'static str,
    benchmark: &'static str,
    parameters: Parameters,
    stats: Stats,
}

#[derive(Serialize)]
struct Parameters {
    dataset: &'static str,
    ngram_size: usize,
}

#[derive(Serialize)]
struct Stats {
    build_ms: f64,
    save_ms: f64,
    load_ms: f64,
    size_mib: f64,
    iterations: usize,
}

fn median(samples: usize, mut run: impl FnMut() -> f64) -> f64 {
    run();
    let mut times: Vec<_> = (0..samples).map(|_| run()).collect();
    times.sort_by(f64::total_cmp);
    times[times.len() / 2]
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let only = args.iter().find(|arg| !arg.starts_with('-'));
    let json_output = args.iter().any(|arg| arg == "--json");
    let mut results = Vec::new();
    for (name, file) in [
        ("company", "company_names.txt"),
        ("unigrams", "eng_google_books_unigram.txt"),
    ] {
        if only.is_some_and(|value| value.as_str() != name) {
            continue;
        }
        let corpus = fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("benches/data")
                .join(file),
        )
        .unwrap();
        let directory = tempfile::tempdir().unwrap();
        for n in [2, 3, 4] {
            let build = || {
                let mut db = HashDb::new(Arc::new(CharacterNgrams::new(n, " ")));
                for text in corpus.lines() {
                    db.insert(text.to_owned());
                }
                db
            };
            let build_ms = median(SAMPLES, || {
                let start = Instant::now();
                let db = build();
                let elapsed = start.elapsed().as_secs_f64() * 1000.;
                drop(black_box(db));
                elapsed
            });
            let db = build();
            let save_ms = median(SAMPLES, || {
                let start = Instant::now();
                db.save_to_file(directory.path(), "db.bin").unwrap();
                start.elapsed().as_secs_f64() * 1000.
            });
            let bytes = fs::metadata(directory.path().join("db.bin")).unwrap().len();
            let load_ms = median(SAMPLES, || {
                let start = Instant::now();
                let loaded = HashDb::load_from_file(directory.path(), "db.bin").unwrap();
                let elapsed = start.elapsed().as_secs_f64() * 1000.;
                assert_eq!(db.strings, loaded.strings);
                drop(black_box(loaded));
                elapsed
            });
            results.push(BenchmarkResult {
                language: "rust",
                backend: "simstring-rust (native)",
                benchmark: "persistence",
                parameters: Parameters {
                    dataset: name,
                    ngram_size: n,
                },
                stats: Stats {
                    build_ms,
                    save_ms,
                    load_ms,
                    size_mib: bytes as f64 / 1048576.,
                    iterations: SAMPLES,
                },
            });
        }
    }
    if json_output {
        println!("{}", serde_json::to_string_pretty(&results).unwrap());
    } else {
        for result in results {
            let Parameters {
                dataset,
                ngram_size,
            } = result.parameters;
            let Stats {
                build_ms,
                save_ms,
                load_ms,
                size_mib,
                ..
            } = result.stats;
            println!(
                "{dataset:<8} n={ngram_size} build {build_ms:8.2} ms  save {save_ms:8.2} ms  load {load_ms:8.2} ms  size {size_mib:.2} MiB"
            );
        }
    }
}

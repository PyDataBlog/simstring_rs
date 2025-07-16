use rayon::ThreadPoolBuilder;
use serde::Serialize;
use simstring_rust::{CharacterNgrams, Cosine, HashDb, Searcher};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::{Arc, Once};
use std::time::{Duration, Instant};

static INIT_BENCH_RAYON: Once = Once::new();

#[derive(Serialize)]
struct Stats {
    mean: f64,
    stddev: f64,
    iterations: usize,
}

#[derive(Serialize)]
struct BenchmarkResult {
    language: String,
    backend: String,
    benchmark: String,
    parameters: serde_json::Value,
    stats: Stats,
}

fn setup_benchmark_environment() {
    INIT_BENCH_RAYON.call_once(|| {
        let num_threads = num_cpus::get_physical();
        if let Err(e) = ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build_global()
        {
            eprintln!("Failed to initialize global Rayon thread pool for benchmarks: {e:?}.");
        }
    });
}

fn bench_insert(results: &mut Vec<BenchmarkResult>) {
    setup_benchmark_environment();
    let companies = load_companies();

    for ngram_size in [2, 3, 4].iter() {
        let mut measurements = Vec::new();
        let start_time = Instant::now();
        let mut iteration = 0;

        while start_time.elapsed() < Duration::from_secs(20) && iteration < 100 {
            let setup_start = Instant::now();
            let fe = Arc::new(CharacterNgrams::new(*ngram_size, " "));
            let mut db = HashDb::new(fe);
            let setup_duration = setup_start.elapsed();

            let start = Instant::now();
            for company in &companies {
                db.insert(company.clone());
            }
            let duration = start.elapsed() - setup_duration;
            measurements.push(duration.as_secs_f64() * 1000.0);
            iteration += 1;
        }

        let mean = measurements.iter().sum::<f64>() / measurements.len() as f64;
        let stddev = if measurements.len() > 1 {
            let variance = measurements
                .iter()
                .map(|value| {
                    let diff = mean - value;
                    diff * diff
                })
                .sum::<f64>()
                / (measurements.len() - 1) as f64;
            variance.sqrt()
        } else {
            0.0
        };

        results.push(BenchmarkResult {
            language: "rust".to_string(),
            backend: "simstring-rust (native)".to_string(),
            benchmark: "insert".to_string(),
            parameters: serde_json::json!({ "ngram_size": ngram_size }),
            stats: Stats {
                mean,
                stddev,
                iterations: measurements.len(),
            },
        });
    }
}

fn bench_search(results: &mut Vec<BenchmarkResult>) {
    setup_benchmark_environment();
    let companies = load_companies();
    let search_terms: Vec<String> = companies.iter().take(100).cloned().collect();

    for ngram_size in [2, 3, 4].iter() {
        let fe = Arc::new(CharacterNgrams::new(*ngram_size, " "));
        let mut db = HashDb::new(fe);
        for company in &companies {
            db.insert(company.clone());
        }

        let measure = Cosine;
        let searcher = Searcher::new(&db, measure);

        for threshold in [0.6, 0.7, 0.8, 0.9].iter() {
            let mut measurements = Vec::new();
            let start_time = Instant::now();
            let mut iteration = 0;

            while start_time.elapsed() < Duration::from_secs(20) && iteration < 100 {
                let start = Instant::now();
                for term in &search_terms {
                    let _ = searcher.ranked_search(term, *threshold).unwrap();
                }
                let duration = start.elapsed();
                measurements.push(duration.as_secs_f64() * 1000.0);
                iteration += 1;
            }

            let mean = measurements.iter().sum::<f64>() / measurements.len() as f64;
            let stddev = if measurements.len() > 1 {
                let variance = measurements
                    .iter()
                    .map(|value| {
                        let diff = mean - value;
                        diff * diff
                    })
                    .sum::<f64>()
                    / (measurements.len() - 1) as f64;
                variance.sqrt()
            } else {
                0.0
            };

            results.push(BenchmarkResult {
                language: "rust".to_string(),
                backend: "simstring-rust (native)".to_string(),
                benchmark: "search".to_string(),
                parameters: serde_json::json!({ "ngram_size": ngram_size, "threshold": threshold }),
                stats: Stats {
                    mean,
                    stddev,
                    iterations: measurements.len(),
                },
            });
        }
    }
}

fn load_companies() -> Vec<String> {
    let file_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("benches")
        .join("data")
        .join("company_names.txt");

    let file = File::open(&file_path)
        .unwrap_or_else(|e| panic!("Failed to open company names file at {file_path:?}: {e}"));

    let reader = BufReader::new(file);
    reader.lines().map_while(Result::ok).collect()
}

fn main() {
    let mut results = Vec::new();
    bench_insert(&mut results);
    bench_search(&mut results);
    println!("{}", serde_json::to_string_pretty(&results).unwrap());
}

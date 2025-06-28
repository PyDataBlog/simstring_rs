use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use simstring_rust::{CharacterNgrams, Cosine, HashDb, Searcher};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::Arc;

use rayon::ThreadPoolBuilder;
use std::sync::Once;

static INIT_BENCH_RAYON: Once = Once::new();

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

pub fn bench_insert(c: &mut Criterion) {
    setup_benchmark_environment();
    let companies = load_companies();

    let mut group = c.benchmark_group("db_insert");
    group.measurement_time(std::time::Duration::from_secs(50));

    for ngram_size in [2, 3, 4].iter() {
        group.bench_with_input(
            BenchmarkId::new("ngram_size", ngram_size),
            ngram_size,
            |b, &size| {
                b.iter_with_setup(
                    || {
                        let fe = Arc::new(CharacterNgrams::new(size, " "));
                        (HashDb::new(fe), companies.clone())
                    },
                    |(mut db, companies_batch)| {
                        for company in companies_batch {
                            db.insert(company);
                        }
                    },
                );
            },
        );
    }
    group.finish();
}

pub fn bench_search(c: &mut Criterion) {
    setup_benchmark_environment();
    let companies = load_companies();
    let search_terms: Vec<String> = companies
        .iter()
        .take(100.min(companies.len()))
        .cloned()
        .collect();
    if search_terms.is_empty() {
        println!("Warning: No search terms available for benchmark. Company list might be too small or empty.");
        return;
    }

    let mut group = c.benchmark_group("db_search");
    group.measurement_time(std::time::Duration::from_secs(20));

    for ngram_size in [2, 3, 4].iter() {
        let fe = Arc::new(CharacterNgrams::new(*ngram_size, " "));
        let mut db = HashDb::new(fe);
        for company in &companies {
            db.insert(company.clone());
        }

        let measure = Cosine;
        let searcher = Searcher::new(&db, measure);

        for threshold in [0.6, 0.7, 0.8, 0.9].iter() {
            let bench_id_str = format!("ngram_{ngram_size}_threshold_{threshold}");
            group.bench_with_input(
                BenchmarkId::new("params", bench_id_str),
                threshold,
                |b, &thresh| {
                    b.iter(|| {
                        for term in &search_terms {
                            let dummy = searcher.ranked_search(term, thresh).unwrap();
                            std::hint::black_box(dummy);
                        }
                    })
                },
            );
        }
    }
    group.finish();
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

criterion_group!(benches, bench_insert, bench_search);
criterion_main!(benches);

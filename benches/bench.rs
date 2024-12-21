use criterion::{black_box, criterion_group, criterion_main, Criterion};
use simstring_rust::database::HashDB;
use simstring_rust::extractors::CharacterNGrams;
use simstring_rust::measures::Cosine;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn bench_insert(c: &mut Criterion) {
    let companies = load_companies();

    let mut group = c.benchmark_group("db_insert");
    group.measurement_time(std::time::Duration::from_secs(20));

    for ngram_size in [2, 3, 4] {
        group.bench_function(format!("ngram_{}", ngram_size), |b| {
            b.iter_with_setup(
                || (create_db(ngram_size), companies.clone()),
                |(mut db, companies)| {
                    for company in companies {
                        db.insert(company);
                    }
                },
            )
        });
    }
    group.finish();
}

pub fn bench_search(c: &mut Criterion) {
    let companies = load_companies();
    let search_terms: Vec<String> = companies.iter().take(100).cloned().collect();

    let mut group = c.benchmark_group("db_search");
    group.measurement_time(std::time::Duration::from_secs(20));

    for ngram_size in [2, 3, 4] {
        let mut db = create_db(ngram_size);
        // Populate database
        for company in &companies {
            db.insert(company.clone());
        }

        for threshold in [0.6, 0.7, 0.8] {
            group.bench_function(
                format!("ngram_{}_threshold_{}", ngram_size, threshold),
                |b| {
                    b.iter(|| {
                        for term in &search_terms {
                            db.search(term, threshold);
                        }
                    })
                },
            );
        }
    }
    group.finish();
}

fn create_db(ngram_size: usize) -> HashDB<CharacterNGrams, Cosine> {
    HashDB::new(
        CharacterNGrams {
            n: ngram_size,
            padder: " ".to_string(),
        },
        Cosine::new(),
    )
}

fn load_companies() -> Vec<String> {
    let current_dir = env::current_dir().unwrap();
    let file_path = current_dir
        .join("benches")
        .join("data")
        .join("company_names.txt");

    let file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);
    reader.lines().map_while(Result::ok).collect()
}

criterion_group!(benches, bench_insert, bench_search);
criterion_main!(benches);

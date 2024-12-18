use criterion::{black_box, criterion_group, criterion_main, Criterion};
use simstring_rust::database::HashDB;
use simstring_rust::extractors::CharacterNGrams;
use simstring_rust::measures::Cosine;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn bench_insert(c: &mut Criterion) {
    let current_dir = env::current_dir().unwrap();
    let file_path = current_dir
        .join("benches")
        .join("data")
        .join("company_names.txt");

    let file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);
    let companies: Vec<String> = reader.lines().map_while(Result::ok).collect();

    let mut group = c.benchmark_group("db_insert");
    group.measurement_time(std::time::Duration::from_secs(20));

    // Test for n-gram sizes 2, 3, and 4
    for ngram_size in [2, 3, 4] {
        group.bench_function(format!("ngram_{}", ngram_size), |b| {
            b.iter_with_setup(
                || (create_db(ngram_size), companies.clone()),
                |(mut db, companies)| {
                    for company in companies {
                        black_box(&mut db).insert(company);
                    }
                },
            )
        });
    }
    group.finish();
}

fn create_db(ngram_size: usize) -> HashDB<CharacterNGrams, Cosine> {
    let measure = Cosine;
    let feature_extractor = CharacterNGrams {
        n: ngram_size,
        padder: " ".to_string(),
    };

    HashDB::new(feature_extractor, measure)
}

criterion_group!(benches, bench_insert);
criterion_main!(benches);

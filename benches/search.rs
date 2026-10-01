use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use rayon::prelude::*;
use simstring_rust::{
    CharacterNgrams, Cosine, Dice, ExactMatch, HashDb, Jaccard, Measure, Overlap, Searcher,
};
use std::{hint::black_box, sync::Arc, time::Duration};

fn measure<M: Measure>(c: &mut Criterion, name: &str, measure: M) {
    let companies: Vec<_> = include_str!("data/company_names.txt").lines().collect();
    let mut group = c.benchmark_group(name);
    group
        .sample_size(20)
        .warm_up_time(Duration::from_millis(100))
        .measurement_time(Duration::from_millis(300));
    for size in [100, companies.len(), 100_000] {
        let mut db = HashDb::new(Arc::new(CharacterNgrams::new(3, " ")));
        for company in companies.iter().cycle().take(size) {
            db.insert((*company).to_owned());
        }
        let searcher = Searcher::new(&db, &measure);
        let near = format!("{}🦀", companies[0]);
        for (kind, query) in [
            ("hit", companies[0]),
            ("near", near.as_str()),
            ("miss", "🦀🦀🦀"),
        ] {
            for alpha in [0.6, 0.9] {
                group.bench_with_input(
                    BenchmarkId::new(format!("ranked/{kind}/{alpha}"), size),
                    &query,
                    |b, query| {
                        b.iter(|| {
                            black_box(searcher.ranked_search(black_box(query), alpha).unwrap())
                        })
                    },
                );
                group.bench_with_input(
                    BenchmarkId::new(format!("unranked/{kind}/{alpha}"), size),
                    &query,
                    |b, query| {
                        b.iter(|| black_box(searcher.search(black_box(query), alpha).unwrap()))
                    },
                );
            }
        }
        let queries = &companies[..32];
        group.bench_function(BenchmarkId::new("concurrent/32", size), |b| {
            b.iter(|| {
                queries.par_iter().for_each(|query| {
                    black_box(searcher.ranked_search(black_box(query), 0.8).unwrap());
                });
            })
        });
    }
    group.finish();
}

fn search(c: &mut Criterion) {
    measure(c, "cosine", Cosine);
    measure(c, "dice", Dice);
    measure(c, "jaccard", Jaccard);
    measure(c, "overlap", Overlap);
    measure(c, "exact", ExactMatch);
}

criterion_group!(benches, search);
criterion_main!(benches);

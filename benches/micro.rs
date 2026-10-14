//! Microbenchmarks used to gate performance changes.
//!
//! `cargo bench --bench micro -- [--save FILE] [--baseline FILE]`
//!
//! Values are the fastest sample in the unit given by the row name: these loops are
//! deterministic, so interference only ever adds time and the minimum is the most stable
//! estimate. `--baseline` prints the change against an earlier `--save`.

use serde_json::{Map, Value};
use simstring_rust::{CharacterNgrams, Cosine, HashDb, Searcher};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

fn fastest(samples: Vec<f64>) -> f64 {
    samples.into_iter().fold(f64::INFINITY, f64::min)
}

fn end_to_end(rows: &mut Vec<(String, f64)>) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("benches/data/company_names.txt");
    let companies: Vec<String> = std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();
    let queries = &companies[..100];
    for n in [2, 3, 4] {
        let mut db = HashDb::new(Arc::new(CharacterNgrams::new(n, " ")));
        for company in &companies {
            db.insert(company.clone());
        }
        let searcher = Searcher::new(&db, Cosine);
        for alpha in [0.6, 0.7, 0.8, 0.9] {
            let run = || {
                let start = Instant::now();
                for query in queries {
                    std::hint::black_box(searcher.search(query, alpha).unwrap());
                }
                start.elapsed().as_secs_f64() * 1e3
            };
            (0..3).for_each(|_| _ = run());
            let ms = fastest((0..30).map(|_| run()).collect());
            rows.push((format!("e2e/n={n}/alpha={alpha} (ms)"), ms));
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let flag = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1).cloned())
    };
    let mut rows = Vec::new();
    end_to_end(&mut rows);

    let baseline: Map<String, Value> = flag("--baseline")
        .and_then(|file| std::fs::read_to_string(file).ok())
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default();
    for (name, value) in &rows {
        match baseline.get(name).and_then(Value::as_f64) {
            Some(old) => {
                let change = 100.0 * (value - old) / old;
                println!("{name:<44} {value:>9.3}  (was {old:>9.3}, {change:+6.1}%)");
            }
            None => println!("{name:<44} {value:>9.3}"),
        }
    }
    if let Some(file) = flag("--save") {
        if let Some(dir) = Path::new(&file).parent() {
            std::fs::create_dir_all(dir).unwrap();
        }
        let map: Map<String, Value> = rows.into_iter().map(|(k, v)| (k, v.into())).collect();
        std::fs::write(file, serde_json::to_string_pretty(&map).unwrap()).unwrap();
    }
}

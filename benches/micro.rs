//! Microbenchmarks used to gate performance changes.
//!
//! `cargo bench --bench micro -- [--save FILE] [--baseline FILE]`
//!
//! End-to-end rows use the backend selected by `SIMSTRING_RS_BACKEND`; kernel rows run
//! every supported backend interleaved in this process. Values are the fastest sample in
//! the unit given by the row name: these loops are deterministic, so interference only
//! ever adds time and the minimum is the most stable estimate. `--baseline` prints the
//! change against an earlier `--save`.

use serde_json::{Map, Value};
use simstring_rust::execution::{Backend, Kernels, kernels};
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
    let backend = kernels().backend().name();
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
            rows.push((format!("e2e/{backend}/n={n}/alpha={alpha} (ms)"), ms));
        }
    }
}

fn kernel(rows: &mut Vec<(String, f64)>) {
    let backends: Vec<_> = Backend::ALL
        .into_iter()
        .filter(|b| b.is_supported())
        // SAFETY: filtered by runtime detection.
        .map(|b| unsafe { Kernels::new(b) })
        .collect();
    let mut seed = 0x9E37_79B9_7F4A_7C15_u64;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed % 6000) as u32
    };
    for len in [8, 16, 32, 64, 128, 512] {
        // Posting-like lists from a 6000-id universe, probed with sorted ids as in a join.
        let lists: Vec<Vec<_>> = (0..256)
            .map(|_| {
                let mut list: Vec<_> = (0..len).map(|_| next()).collect();
                list.sort_unstable();
                list.dedup();
                list
            })
            .collect();
        let mut probes: Vec<_> = (0..64).map(|_| next()).collect();
        probes.sort_unstable();
        let probes_per_run = (lists.len() * probes.len() * 50) as f64;
        let mut samples = vec![Vec::new(); backends.len()];
        for _ in 0..15 {
            // Interleave backends so drift affects them equally.
            for (k, sample) in backends.iter().zip(&mut samples) {
                let start = Instant::now();
                let mut hits = 0;
                for _ in 0..50 {
                    for list in &lists {
                        hits += probes.iter().filter(|&&p| k.contains(list, p)).count();
                    }
                }
                std::hint::black_box(hits);
                sample.push(start.elapsed().as_nanos() as f64 / probes_per_run);
            }
        }
        for (k, sample) in backends.iter().zip(samples) {
            let name = k.backend().name();
            rows.push((
                format!("kernel/{name}/len={len} (ns/probe)"),
                fastest(sample),
            ));
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
    kernel(&mut rows);

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

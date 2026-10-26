//! `cargo bench --bench persistence -- [company|unigrams]`
use simstring_rust::{CharacterNgrams, HashDb};
use std::{fs, hint::black_box, path::Path, sync::Arc, time::Instant};

fn median(samples: usize, mut run: impl FnMut() -> f64) -> f64 {
    run();
    let mut times: Vec<_> = (0..samples).map(|_| run()).collect();
    times.sort_by(f64::total_cmp);
    times[times.len() / 2]
}

fn main() {
    let only = std::env::args().skip(1).find(|arg| !arg.starts_with('-'));
    for (name, file) in [
        ("company", "company_names.txt"),
        ("unigrams", "eng_google_books_unigram.txt"),
    ] {
        if only.as_deref().is_some_and(|value| value != name) {
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
            let index_ms = median(7, || {
                let start = Instant::now();
                let db = build();
                let elapsed = start.elapsed().as_secs_f64() * 1000.;
                drop(black_box(db));
                elapsed
            });
            let db = build();
            let save_ms = median(7, || {
                let start = Instant::now();
                db.save_to_file(directory.path(), "db.bin").unwrap();
                start.elapsed().as_secs_f64() * 1000.
            });
            let bytes = fs::metadata(directory.path().join("db.bin")).unwrap().len();
            let load_ms = median(7, || {
                let start = Instant::now();
                let loaded = HashDb::load_from_file(directory.path(), "db.bin").unwrap();
                let elapsed = start.elapsed().as_secs_f64() * 1000.;
                assert_eq!(db.strings, loaded.strings);
                drop(black_box(loaded));
                elapsed
            });
            println!(
                "{name:<8} n={n} index {index_ms:8.2} ms  save {save_ms:8.2} ms  load {load_ms:8.2} ms  size {:.2} MiB",
                bytes as f64 / 1048576.
            );
        }
    }
}

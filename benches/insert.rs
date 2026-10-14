//! Insert benchmark: time to build a `HashDb`, split into feature extraction and indexing.
//!
//! `cargo bench --bench insert -- [company|unigrams]`
//!
//! Times are the fastest sample in milliseconds. `extract` runs feature extraction alone,
//! so `index` (`insert - extract`) is the cost of storing the features. A counting global
//! allocator reports the allocations, reallocations and reallocated bytes of one build.

use lasso::Rodeo;
use simstring_rust::{CharacterNgrams, FeatureExtractor, HashDb};
use std::alloc::{GlobalAlloc, Layout, System};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use std::time::Instant;

struct Counting;
static ALLOCS: AtomicUsize = AtomicUsize::new(0);
static REALLOCS: AtomicUsize = AtomicUsize::new(0);
static MOVED: AtomicUsize = AtomicUsize::new(0);
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Relaxed);
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        REALLOCS.fetch_add(1, Relaxed);
        MOVED.fetch_add(layout.size(), Relaxed);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}
#[global_allocator]
static GLOBAL: Counting = Counting;

fn load(file: &str) -> Vec<String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("benches/data")
        .join(file);
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect()
}

fn fastest(samples: usize, mut f: impl FnMut() -> f64) -> f64 {
    f();
    (0..samples).map(|_| f()).fold(f64::INFINITY, f64::min)
}

fn main() {
    let only: Option<String> = std::env::args().skip(1).find(|a| !a.starts_with('-'));
    for (name, file, samples) in [
        ("company", "company_names.txt", 30),
        ("unigrams", "eng_google_books_unigram.txt", 5),
    ] {
        if only.as_deref().is_some_and(|o| o != name) {
            continue;
        }
        let corpus = load(file);
        for n in [2, 3, 4] {
            let extractor = CharacterNgrams::new(n, " ");
            let extract = fastest(samples, || {
                let mut rodeo = Rodeo::default();
                let start = Instant::now();
                for text in &corpus {
                    std::hint::black_box(extractor.features(text, &mut rodeo));
                }
                start.elapsed().as_secs_f64() * 1e3
            });
            let insert = fastest(samples, || {
                let start = Instant::now();
                let mut db = HashDb::new(Arc::new(extractor.clone()));
                for text in &corpus {
                    db.insert(text.clone());
                }
                let ms = start.elapsed().as_secs_f64() * 1e3;
                drop(std::hint::black_box(db));
                ms
            });
            let counts = || {
                (
                    ALLOCS.load(Relaxed),
                    REALLOCS.load(Relaxed),
                    MOVED.load(Relaxed),
                )
            };
            let (a0, r0, m0) = counts();
            let mut db = HashDb::new(Arc::new(extractor.clone()));
            for text in &corpus {
                db.insert(text.clone());
            }
            let (a1, r1, m1) = counts();
            drop(db);
            println!(
                "{name:<8} n={n}  insert {insert:9.3} ms   extract {extract:9.3} ms   index {:9.3} ms   allocs {:>9} reallocs {:>9} realloc-bytes {:>11}",
                insert - extract,
                a1 - a0,
                r1 - r0,
                m1 - m0
            );
        }
    }
}

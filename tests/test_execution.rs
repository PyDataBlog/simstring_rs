//! `SIMSTRING_RS_BACKEND` is read once per process, so each case runs in a child process.

use simstring_rust::{CharacterNgrams, Cosine, HashDb, Searcher};
use std::process::{Command, Output};
use std::sync::Arc;

#[test]
#[ignore = "run by `backend_override` in a child process"]
fn child_search() {
    let mut db = HashDb::new(Arc::new(CharacterNgrams::default()));
    db.insert("hello world".into());
    let searcher = Searcher::new(&db, Cosine);
    assert_eq!(
        searcher.search("hello world", 0.9).unwrap(),
        ["hello world"]
    );
}

fn run_child(backend: &str) -> Output {
    Command::new(std::env::current_exe().unwrap())
        .args(["child_search", "--exact", "--ignored", "--nocapture"])
        .env("SIMSTRING_RS_BACKEND", backend)
        .output()
        .unwrap()
}

fn supported(backend: &str) -> bool {
    match backend {
        "" | "auto" | "native" => true,
        #[cfg(target_arch = "x86_64")]
        "avx2" => std::is_x86_feature_detected!("avx2"),
        #[cfg(target_arch = "x86_64")]
        "avx512" => std::is_x86_feature_detected!("avx512f"),
        _ => false,
    }
}

#[test]
fn backend_override() {
    for backend in ["", "auto", "native", "avx2", "avx512", "bogus"] {
        let output = run_child(backend);
        let stderr = String::from_utf8_lossy(&output.stderr);
        if supported(backend) {
            assert!(output.status.success(), "{backend:?}: {stderr}");
        } else {
            assert!(!output.status.success(), "{backend:?} must be rejected");
            let reason = if backend == "bogus" {
                "invalid"
            } else {
                "not supported"
            };
            assert!(stderr.contains(reason), "{backend:?}: {stderr}");
        }
    }
}

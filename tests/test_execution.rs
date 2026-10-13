//! Execution backend selection and differential tests of every supported backend.

use simstring_rust::execution::{Backend, Kernels, contains_native, kernels, resolve};
use simstring_rust::{CharacterNgrams, Cosine, HashDb, Searcher};
use std::process::{Command, Output};
use std::sync::Arc;

fn supported_kernels() -> Vec<Kernels> {
    Backend::ALL
        .into_iter()
        .filter(|b| b.is_supported())
        // SAFETY: filtered by runtime detection.
        .map(|b| unsafe { Kernels::new(b) })
        .collect()
}

/// Deterministic LCG; avoids a random-number dependency.
fn lcg(seed: &mut u64) -> u64 {
    *seed = seed
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    *seed >> 33
}

#[test]
fn resolve_prefers_widest_supported_backend() {
    let only = |set: &'static [Backend]| move |b| set.contains(&b);
    use Backend::*;
    assert_eq!(resolve("", only(&[Native, Avx2, Avx512])), Ok(Avx512));
    assert_eq!(resolve("auto", only(&[Native, Avx2])), Ok(Avx2));
    assert_eq!(resolve("AUTO", only(&[Native])), Ok(Native));
    assert_eq!(resolve("native", only(&[Native, Avx2, Avx512])), Ok(Native));
    assert_eq!(resolve("Avx2", only(&[Native, Avx2, Avx512])), Ok(Avx2));
    assert_eq!(resolve("avx512", only(&[Native, Avx2, Avx512])), Ok(Avx512));
    assert!(resolve("avx512", only(&[Native, Avx2])).is_err());
    assert!(resolve("avx2", only(&[Native])).is_err());
    assert!(resolve("sse", only(&[Native, Avx2, Avx512])).is_err());
}

#[test]
fn selected_backend_matches_environment() {
    let selected = kernels().backend();
    assert!(selected.is_supported());
    // CI sets this to prove the expected SIMD path really runs.
    if let Ok(expected) = std::env::var("SIMSTRING_RS_EXPECT_BACKEND") {
        assert_eq!(selected.name(), expected);
    }
}

#[test]
fn contains_matches_native() {
    let mut seed = 1;
    for kernels in supported_kernels() {
        for len in (0..=40).chain([63, 64, 65, 1000]) {
            // The middle base straddles the i32 sign boundary.
            for base in [0, (1 << 31) - 20, u32::MAX - 4 * len - 8] {
                let mut ids: Vec<u32> = (0..len).map(|i| base + 2 * i + 1).collect();
                if len > 0 && lcg(&mut seed).is_multiple_of(2) {
                    ids.retain(|_| !lcg(&mut seed).is_multiple_of(4));
                }
                for offset in 0..ids.len().min(17) + 1 {
                    let slice = &ids[offset..];
                    for probe in (base..base + 2 * len + 3).chain([0, u32::MAX]) {
                        assert_eq!(
                            kernels.contains(slice, probe),
                            contains_native(slice, probe),
                            "{:?} len={} probe={probe}",
                            kernels.backend(),
                            slice.len()
                        );
                    }
                }
            }
        }
    }
}

// `SIMSTRING_RS_BACKEND` is read once per process, so override cases run in a child process.

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
#[cfg(target_arch = "x86_64")]
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

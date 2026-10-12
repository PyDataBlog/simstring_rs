//! Runtime selection of the integer-matching kernels used by search and similarity.
//!
//! The fastest backend supported by the running CPU is chosen once per process:
//! AVX-512F, then AVX2, then the portable native code. Non-x86_64 targets always use
//! native. `SIMSTRING_RS_BACKEND` (`auto`, `native`, `avx2` or `avx512`; unset or
//! empty means `auto`) is read on first use; an unknown or unsupported value panics.

use crate::database::StringId;
use lasso::Spur;
use std::sync::OnceLock;

const ENV_VAR: &str = "SIMSTRING_RS_BACKEND";

type ContainsFn = unsafe fn(&[StringId], StringId) -> bool;
type IntersectFn = unsafe fn(&[Spur], &[Spur]) -> usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Backend {
    Native,
    Avx2,
    Avx512,
}

impl Backend {
    const ALL: [Self; 3] = [Self::Native, Self::Avx2, Self::Avx512];

    fn name(self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::Avx2 => "avx2",
            Self::Avx512 => "avx512",
        }
    }

    fn is_supported(self) -> bool {
        match self {
            Self::Native => true,
            #[cfg(target_arch = "x86_64")]
            Self::Avx2 => std::is_x86_feature_detected!("avx2"),
            #[cfg(target_arch = "x86_64")]
            Self::Avx512 => std::is_x86_feature_detected!("avx512f"),
            #[cfg(not(target_arch = "x86_64"))]
            _ => false,
        }
    }
}

/// Resolves a requested backend name against the available CPU features.
fn resolve(requested: &str, supported: impl Fn(Backend) -> bool) -> Result<Backend, String> {
    let requested = requested.to_ascii_lowercase();
    if requested.is_empty() || requested == "auto" {
        let best = Backend::ALL.into_iter().rev().find(|&b| supported(b));
        return Ok(best.unwrap_or(Backend::Native));
    }
    match Backend::ALL.into_iter().find(|b| b.name() == requested) {
        Some(backend) if supported(backend) => Ok(backend),
        Some(_) => Err(format!(
            "{ENV_VAR}={requested} is not supported by this CPU or target"
        )),
        None => Err(format!(
            "invalid {ENV_VAR}={requested:?}; expected auto, native, avx2 or avx512"
        )),
    }
}

/// The selected kernels. Only constructible for backends the CPU supports.
pub(crate) struct Kernels {
    #[cfg(test)]
    backend: Backend,
    contains: ContainsFn,
    intersect: IntersectFn,
}

impl Kernels {
    /// # Safety
    /// The running CPU must support `backend`.
    unsafe fn new(backend: Backend) -> Self {
        let (contains, intersect): (ContainsFn, IntersectFn) = match backend {
            Backend::Native => (contains_native, intersect_native),
            #[cfg(target_arch = "x86_64")]
            Backend::Avx2 => (x86::contains_avx2, x86::intersect_avx2),
            #[cfg(target_arch = "x86_64")]
            Backend::Avx512 => (x86::contains_avx512, x86::intersect_avx512),
            #[cfg(not(target_arch = "x86_64"))]
            _ => unreachable!("SIMD backends are never supported on this target"),
        };
        Self {
            #[cfg(test)]
            backend,
            contains,
            intersect,
        }
    }

    /// Whether the sorted slice `ids` contains `id`.
    #[inline]
    pub(crate) fn contains(&self, ids: &[StringId], id: StringId) -> bool {
        // SAFETY: `Kernels::new` requires CPU support for the selected backend.
        unsafe { (self.contains)(ids, id) }
    }

    /// Size of the multiset intersection of two sorted slices.
    #[inline]
    pub(crate) fn intersect(&self, x: &[Spur], y: &[Spur]) -> usize {
        // SAFETY: as for `contains`.
        unsafe { (self.intersect)(x, y) }
    }
}

/// Process-wide kernels, selected on first use.
///
/// # Panics
/// If `SIMSTRING_RS_BACKEND` is invalid or names a backend this CPU does not support.
pub(crate) fn kernels() -> &'static Kernels {
    static KERNELS: OnceLock<Kernels> = OnceLock::new();
    KERNELS.get_or_init(|| {
        let requested = std::env::var_os(ENV_VAR).unwrap_or_default();
        let requested = requested.to_str().unwrap_or("<non-UTF-8>");
        let backend = resolve(requested, Backend::is_supported).unwrap_or_else(|e| panic!("{e}"));
        // SAFETY: `resolve` only returns supported backends.
        unsafe { Kernels::new(backend) }
    })
}

fn contains_native(ids: &[StringId], id: StringId) -> bool {
    ids.binary_search(&id).is_ok()
}

fn intersect_native(x: &[Spur], y: &[Spur]) -> usize {
    let mut intersection_size = 0;
    let mut i = 0;
    let mut j = 0;

    while i < x.len() && j < y.len() {
        match x[i].cmp(&y[j]) {
            std::cmp::Ordering::Equal => {
                intersection_size += 1;
                i += 1;
                j += 1;
            }
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
        }
    }

    intersection_size
}

#[cfg(target_arch = "x86_64")]
mod x86 {
    use super::{StringId, contains_native, intersect_native};
    use lasso::Spur;
    use std::arch::x86_64::*;

    /// Narrows a sorted slice by binary search to an in-bounds block of `W` elements
    /// that contains `id` if the slice does. `None` if the slice is shorter than `W`.
    #[inline(always)]
    fn block_for<const W: usize>(ids: &[StringId], id: StringId) -> Option<&[StringId; W]> {
        let (mut lo, mut hi) = (0, ids.len());
        while hi - lo > W {
            let mid = lo + (hi - lo) / 2;
            if ids[mid] <= id {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let start = lo.min(ids.len().checked_sub(W)?);
        ids[start..start + W].try_into().ok()
    }

    /// Sorted-merge intersection matching one key of the shorter input against `W` keys
    /// of the longer one. `eq_mask` returns a bit mask of the lanes equal to the key.
    /// Consuming only the first equal lane preserves multiplicity for duplicate keys.
    #[inline(always)]
    fn intersect_blocks<const W: usize>(
        x: &[Spur],
        y: &[Spur],
        eq_mask: impl Fn(&[u32; W], u32) -> u32,
    ) -> usize {
        let (x, y) = if x.len() <= y.len() { (x, y) } else { (y, x) };
        // `Spur` orders by this public value.
        let key = |spur: Spur| spur.into_inner().get();
        let (mut i, mut j, mut count) = (0, 0, 0);
        while i < x.len() && j + W <= y.len() {
            if y[j + W - 1] < x[i] {
                j += W;
                continue;
            }
            let block: [Spur; W] = y[j..j + W].try_into().unwrap();
            let mask = eq_mask(&block.map(key), key(x[i]));
            if mask != 0 {
                count += 1;
                j += mask.trailing_zeros() as usize + 1;
            }
            i += 1;
        }
        count + intersect_native(&x[i..], &y[j..])
    }

    /// # Safety
    /// The CPU must support AVX2.
    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn contains_avx2(ids: &[StringId], id: StringId) -> bool {
        let Some(block) = block_for::<8>(ids, id) else {
            return contains_native(ids, id);
        };
        let ptr = block.as_ptr().cast::<__m256i>();
        let needle = _mm256_set1_epi64x(id as i64);
        // SAFETY: `block` holds 8 u64 values, i.e. two unaligned 256-bit loads.
        let eq = unsafe {
            _mm256_or_si256(
                _mm256_cmpeq_epi64(_mm256_loadu_si256(ptr), needle),
                _mm256_cmpeq_epi64(_mm256_loadu_si256(ptr.add(1)), needle),
            )
        };
        _mm256_testz_si256(eq, eq) == 0
    }

    /// # Safety
    /// The CPU must support AVX-512F.
    #[target_feature(enable = "avx512f")]
    pub(super) unsafe fn contains_avx512(ids: &[StringId], id: StringId) -> bool {
        let Some(block) = block_for::<16>(ids, id) else {
            return contains_native(ids, id);
        };
        let ptr = block.as_ptr();
        let needle = _mm512_set1_epi64(id as i64);
        // SAFETY: `block` holds 16 u64 values, i.e. two unaligned 512-bit loads.
        unsafe {
            (_mm512_cmpeq_epi64_mask(_mm512_loadu_si512(ptr.cast()), needle)
                | _mm512_cmpeq_epi64_mask(_mm512_loadu_si512(ptr.add(8).cast()), needle))
                != 0
        }
    }

    /// # Safety
    /// The CPU must support AVX2.
    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn intersect_avx2(x: &[Spur], y: &[Spur]) -> usize {
        intersect_blocks::<8>(x, y, |block, key| {
            // SAFETY: AVX2 is enabled here; `block` holds 8 u32 values.
            unsafe { mask_avx2(block, key) }
        })
    }

    /// # Safety
    /// The CPU must support AVX-512F.
    #[target_feature(enable = "avx512f")]
    pub(super) unsafe fn intersect_avx512(x: &[Spur], y: &[Spur]) -> usize {
        intersect_blocks::<16>(x, y, |block, key| {
            // SAFETY: AVX-512F is enabled here; `block` holds 16 u32 values.
            unsafe { mask_avx512(block, key) }
        })
    }

    #[inline]
    #[target_feature(enable = "avx2")]
    unsafe fn mask_avx2(block: &[u32; 8], key: u32) -> u32 {
        // SAFETY: `block` is exactly 256 bits.
        let lanes = unsafe { _mm256_loadu_si256(block.as_ptr().cast()) };
        let eq = _mm256_cmpeq_epi32(lanes, _mm256_set1_epi32(key as i32));
        _mm256_movemask_ps(_mm256_castsi256_ps(eq)) as u32
    }

    #[inline]
    #[target_feature(enable = "avx512f")]
    unsafe fn mask_avx512(block: &[u32; 16], key: u32) -> u32 {
        // SAFETY: `block` is exactly 512 bits.
        let lanes = unsafe { _mm512_loadu_si512(block.as_ptr().cast()) };
        u32::from(_mm512_cmpeq_epi32_mask(
            lanes,
            _mm512_set1_epi32(key as i32),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lasso::Key;

    fn supported() -> Vec<Kernels> {
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
        let selected = kernels().backend;
        assert!(selected.is_supported());
        // CI sets this to prove the expected SIMD path really runs (e.g. under emulation).
        if let Ok(expected) = std::env::var("SIMSTRING_RS_EXPECT_BACKEND") {
            assert_eq!(selected.name(), expected);
        }
    }

    #[test]
    fn spur_order_matches_its_public_value() {
        let keys: Vec<Spur> = [0, 1, 2, 0xFFFF, u32::MAX as usize - 2]
            .into_iter()
            .map(|n| Spur::try_from_usize(n).unwrap())
            .collect();
        for pair in keys.windows(2) {
            assert!(pair[0] < pair[1]);
            assert!(pair[0].into_inner().get() < pair[1].into_inner().get());
        }
    }

    #[test]
    fn contains_matches_native() {
        let mut seed = 1;
        for kernels in supported() {
            for len in (0..=40).chain([63, 64, 65, 1000]) {
                for base in [0, u32::MAX as usize - 20, usize::MAX - 4 * len - 8] {
                    let mut ids: Vec<_> = (0..len).map(|i| base + 2 * i + 1).collect();
                    if len > 0 && lcg(&mut seed).is_multiple_of(2) {
                        ids.retain(|_| !lcg(&mut seed).is_multiple_of(4));
                    }
                    for offset in 0..ids.len().min(17) + 1 {
                        let slice = &ids[offset..];
                        for probe in (base..base + 2 * len + 3).chain([0, usize::MAX]) {
                            assert_eq!(
                                kernels.contains(slice, probe),
                                contains_native(slice, probe),
                                "{:?} len={} probe={probe}",
                                kernels.backend,
                                slice.len()
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn intersect_matches_native() {
        let spur = |n: u64| Spur::try_from_usize(n as usize).unwrap();
        let mut seed = 7;
        for kernels in supported() {
            for case in 0..3000 {
                let mut set = |len: u64, range: u64, high: u64| {
                    let mut v: Vec<_> = (0..len)
                        .map(|_| spur(high + lcg(&mut seed) % range))
                        .collect();
                    v.sort_unstable();
                    v
                };
                let range = 1 + case % 97;
                let high = if case % 5 == 0 {
                    u32::MAX as u64 - 200
                } else {
                    0
                };
                let x = set(case % 41, range, high);
                let y = set((case * 7) % 67, range, high);
                let mut unique = y.clone();
                unique.dedup();
                for (a, b) in [(&x, &y), (&y, &x), (&unique, &unique), (&x, &unique)] {
                    for offset in 0..b.len().min(17) {
                        assert_eq!(
                            kernels.intersect(a, &b[offset..]),
                            intersect_native(a, &b[offset..]),
                            "{:?} a={a:?} b={:?}",
                            kernels.backend,
                            &b[offset..]
                        );
                    }
                }
            }
        }
    }
}

//! Runtime selection of the posting-list membership kernel used by search.
//!
//! The fastest backend supported by the running CPU is chosen once per process:
//! AVX-512F, then AVX2, then the portable native code. Non-x86_64 targets always use
//! native. `SIMSTRING_RS_BACKEND` (`auto`, `native`, `avx2` or `avx512`; unset or
//! empty means `auto`) is read on first use; an unknown or unsupported value panics.

use crate::database::StringId;
use std::sync::OnceLock;

const ENV_VAR: &str = "SIMSTRING_RS_BACKEND";

type ContainsFn = unsafe fn(&[StringId], StringId) -> bool;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    Native,
    Avx2,
    Avx512,
}

impl Backend {
    pub const ALL: [Self; 3] = [Self::Native, Self::Avx2, Self::Avx512];

    pub fn name(self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::Avx2 => "avx2",
            Self::Avx512 => "avx512",
        }
    }

    pub fn is_supported(self) -> bool {
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
pub fn resolve(requested: &str, supported: impl Fn(Backend) -> bool) -> Result<Backend, String> {
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
pub struct Kernels {
    backend: Backend,
    contains: ContainsFn,
}

impl Kernels {
    /// # Safety
    /// The running CPU must support `backend`.
    pub unsafe fn new(backend: Backend) -> Self {
        let contains: ContainsFn = match backend {
            Backend::Native => contains_native,
            #[cfg(target_arch = "x86_64")]
            Backend::Avx2 => x86::contains_avx2,
            #[cfg(target_arch = "x86_64")]
            Backend::Avx512 => x86::contains_avx512,
            #[cfg(not(target_arch = "x86_64"))]
            _ => unreachable!("SIMD backends are never supported on this target"),
        };
        Self { backend, contains }
    }

    pub fn backend(&self) -> Backend {
        self.backend
    }

    /// Whether the sorted slice `ids` contains `id`.
    #[inline]
    pub fn contains(&self, ids: &[StringId], id: StringId) -> bool {
        // SAFETY: `Kernels::new` requires CPU support for the selected backend.
        unsafe { (self.contains)(ids, id) }
    }
}

/// Process-wide kernels, selected on first use.
///
/// # Panics
/// If `SIMSTRING_RS_BACKEND` is invalid or names a backend this CPU does not support.
pub fn kernels() -> &'static Kernels {
    static KERNELS: OnceLock<Kernels> = OnceLock::new();
    KERNELS.get_or_init(|| {
        let requested = std::env::var_os(ENV_VAR).unwrap_or_default();
        let requested = requested.to_str().unwrap_or("<non-UTF-8>");
        let backend = resolve(requested, Backend::is_supported).unwrap_or_else(|e| panic!("{e}"));
        // SAFETY: `resolve` only returns supported backends.
        unsafe { Kernels::new(backend) }
    })
}

pub fn contains_native(ids: &[StringId], id: StringId) -> bool {
    ids.binary_search(&id).is_ok()
}

#[cfg(target_arch = "x86_64")]
mod x86 {
    use super::{StringId, contains_native};
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
}

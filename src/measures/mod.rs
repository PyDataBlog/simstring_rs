//! Built-in similarity measures over feature sets.
//!
//! A [`Measure`] converts the sizes of two feature sets and the size of their
//! intersection into a similarity score in `0.0..=1.0`, and additionally
//! supplies the pruning bounds that let the search join skip feature-size
//! buckets that can no longer reach `alpha`. Every measure is [`Send`]
//! + [`Sync`], so an individual search can scan large indexes in parallel.
//!
//! Built-in sets and their formulas (`x`, `y` = the two feature-set sizes,
//! `c` = the size of their intersection):
//!
//! | Type          | Similarity                 | 1.0 exactly when            |
//! |---------------|----------------------------|-----------------------------|
//! | [`Cosine`]    | `c / sqrt(x * y)`          | feature sets are equal      |
//! | [`Dice`]      | `2 * c / (x + y)`          | feature sets are equal      |
//! | [`Jaccard`]   | `c / (x + y - c)`          | feature sets are equal      |
//! | [`Overlap`]   | `c / min(x, y)`            | smaller set fully contained |
//! | [`ExactMatch`]| `1.0` or `0.0`             | feature sets are equal      |
//!
//! Similarity with an empty query is measure-specific: [`Dice`], [`Jaccard`],
//! and [`Overlap`] score two empty sets as `1.0` (`x == y == c == 0`), whereas
//! [`Cosine`] scores any empty side as `0.0`. In practice search already
//! returns no results for a query without features, so these conventions only
//! surface when calling [`Measure::similarity`] directly.

mod cosine;
mod dice;
mod exact_match;
mod jaccard;
mod overlap;

use crate::database::Database;
use lasso::Spur;

/// Scores a pair of feature sets and decides which feature-set sizes can
/// still reach the requested threshold during a search.
///
/// Implementors must satisfy one invariant for the pruning bounds: the
/// partnered `min_feature_size`, `max_feature_size`, and
/// `minimum_common_feature_count` must *under-promise* — any string that could
/// be at least `alpha` similar to the query must be inside the reported
/// bounds. The built-in measures compute the mathematically tight bounds but
/// round them outwards (`f64::next_down` / `f64::next_up`) so that floating
/// point rounding can never exclude a boundary match. Exact score filtering
/// (`similarity_from_counts >= alpha`) then removes the small over-approximation.
pub trait Measure: Send + Sync {
    /// Smallest feature-set size that can be at least `alpha` similar to a
    /// query of `query_size` features. Must not exclude any true candidate:
    /// return `0` if unsure, that's a safe (slower) under-promise.
    fn min_feature_size(&self, query_size: usize, alpha: f64) -> usize;
    /// Largest feature-set size that can be at least `alpha` similar to a
    /// query of `query_size` features. Returning
    /// [`db.max_feature_len()`](Database::max_feature_len) is always safe.
    fn max_feature_size(&self, query_size: usize, alpha: f64, db: &dyn Database) -> usize;
    /// Smallest intersection size `c` between a query of `query_size` features
    /// and a string of `y_size` features that can still reach the threshold.
    fn minimum_common_feature_count(&self, query_size: usize, y_size: usize, alpha: f64) -> usize;
    /// Scores an exact intersection of two unique feature sets; see the
    /// [module](self) for each built-in's formula. Implementations must agree
    /// with the pruning bounds above.
    fn similarity_from_counts(&self, x_len: usize, y_len: usize, common: usize) -> f64;

    /// Convenience: similarity between two explicit (unique, sorted) feature
    /// sets. Search never calls this — it counts overlaps during the join —
    /// but it is handy for evaluation pipelines.
    fn similarity(&self, x: &[Spur], y: &[Spur]) -> f64 {
        self.similarity_from_counts(x.len(), y.len(), compute_intersection_size(x, y))
    }
}

impl<M: Measure + ?Sized> Measure for &M {
    fn min_feature_size(&self, query_size: usize, alpha: f64) -> usize {
        (**self).min_feature_size(query_size, alpha)
    }
    fn max_feature_size(&self, query_size: usize, alpha: f64, db: &dyn Database) -> usize {
        (**self).max_feature_size(query_size, alpha, db)
    }
    fn minimum_common_feature_count(&self, query_size: usize, y_size: usize, alpha: f64) -> usize {
        (**self).minimum_common_feature_count(query_size, y_size, alpha)
    }
    fn similarity_from_counts(&self, x_len: usize, y_len: usize, common: usize) -> f64 {
        (**self).similarity_from_counts(x_len, y_len, common)
    }
}

// Size of the intersection of two sorted feature slices. Search never calls this (it
// counts overlaps in the join), and feature sets are short, so a scalar merge suffices.
pub(crate) fn compute_intersection_size(x: &[Spur], y: &[Spur]) -> usize {
    let (mut i, mut j, mut count) = (0, 0, 0);
    while i < x.len() && j < y.len() {
        match x[i].cmp(&y[j]) {
            std::cmp::Ordering::Equal => {
                count += 1;
                i += 1;
                j += 1;
            }
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
        }
    }
    count
}
pub use cosine::Cosine;
pub use dice::Dice;
pub use exact_match::ExactMatch;
pub use jaccard::Jaccard;
pub use overlap::Overlap;

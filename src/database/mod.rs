//! Inverted-index string databases.
//!
//! [`Database`] is the storage contract the search engine is written against:
//! it stores strings, the feature sets they were tokenized into, and an
//! inverted index mapping every `(feature, feature-set size)` pair to the
//! matching string IDs. A query shares each candidate string's exact feature
//! set size, so the search engine can skip sizes that must miss.
//!
//! [`HashDb`] is the built-in implementation. Implement [`Database`] (plus the
//! feature extractor of your choice) to plug in a different storage engine,
//! from an LSM-backed store to a read-only memory-mapped corpus.

mod hashdb;

use crate::extractors::FeatureExtractor;
use lasso::{Rodeo, Spur};

/// Identifier of an indexed string: `i` equals the position of the string in
/// the database's storage. Unique until the database is cleared — after
/// [`clear`](Database::clear), IDs refer to the new contents.
pub type StringId = usize;

/// Storage of strings, their feature sets, and the inverted index that search
/// reads. Implementations must be safe to query from multiple threads
/// concurrently (`Send + Sync`); each ID belongs to exactly one feature-size
/// bucket, and stored feature sets are sorted and unique in the database's own
/// [`interner`](Database::interner) keyspace.
pub trait Database: Send + Sync {
    /// Indexes `text`, allocating a fresh [`StringId`]. Inserting the same
    /// string twice keeps both entries: each gets its own ID and both are
    /// returned by searches.
    fn insert(&mut self, text: String);
    /// Removes every string and releases interner memory. Subsequent IDs count
    /// from zero again.
    fn clear(&mut self);
    /// Returns sorted, unique IDs. Each ID belongs to exactly one feature-size bucket.
    fn lookup_strings(&self, size: usize, feature: Spur) -> Option<&[StringId]>;
    /// Calls `visit(size, ids)` for each non-empty posting list of `feature` whose size is
    /// in `min..=max`, in ascending size order. Override when the backing store can
    /// enumerate a feature's sizes directly: most (size, feature) pairs have no postings.
    fn visit_postings<'a>(
        &'a self,
        feature: Spur,
        min: usize,
        max: usize,
        visit: &mut dyn FnMut(usize, &'a [StringId]),
    ) {
        let sizes = self.feature_sizes();
        let start = sizes.partition_point(|&size| size < min);
        for &size in sizes[start..].iter().take_while(|&&size| size <= max) {
            if let Some(ids) = self
                .lookup_strings(size, feature)
                .filter(|ids| !ids.is_empty())
            {
                visit(size, ids);
            }
        }
    }
    /// Returns the indexed string `id`, if the ID is still valid.
    fn get_string(&self, id: StringId) -> Option<&str>;
    /// Returns the sorted, unique feature IDs computed at insertion time for
    /// `id`. Feature IDs are [`Spur`]s resolved through [`interner`].
    ///
    /// [`interner`]: Database::interner
    fn get_features(&self, id: StringId) -> Option<&[Spur]>;
    /// Returns the extractor this database was constructed with. The search
    /// engine applies it to queries so query and corpus features come from the
    /// same scheme.
    fn feature_extractor(&self) -> &dyn FeatureExtractor;
    /// Largest populated feature-set size, or `0` for an empty database.
    /// Bounds the candidate size range a query can match in.
    fn max_feature_len(&self) -> usize;
    /// The interner shared by all feature IDs in this database.
    fn interner(&self) -> &Rodeo;
    /// Sorted, unique populated feature sizes.
    fn feature_sizes(&self) -> &[usize];
    /// Number of strings currently index-accessible through this database.
    fn total_strings(&self) -> usize;
}

/// In-memory [`Database`] holding an inverted index in dense vectors: one
/// posting list per `(feature, feature-set size)` pair, indexed directly by
/// interned feature key without hashing. Inserts are linear in the string's
/// feature count; lookups are O(1) per pair.
///
/// Index a corpus with [`new`](HashDb::new) and
/// [`insert`](HashDb::insert); then hand it to a
/// [`Searcher`](crate::Searcher) for queries.
pub use hashdb::{HashDb, PersistenceError};

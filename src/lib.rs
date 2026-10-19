//! A native Rust implementation of the SimString approximate string matching
//! algorithm (Okazaki & Tsujii, 2010), with bindings for Python.
//!
//! The library indexes a corpus of strings using an inverted index over
//! *feature sets* — typically character or word *n*-grams — and answers queries
//! of the form "which strings are at least `alpha` similar to my query?"
//! rather than exact matches only. The guarantee from the CPMerge paper holds:
//! retrieval is exact, meaning every string whose similarity to the query
//! meets the threshold is returned, and no false results are added.
//!
//! # Concepts
//!
//! Three components cooperate, mirroring the original SimString design:
//!
//! 1. A [`FeatureExtractor`] turns any string into an unordered but
//!    occurrence-qualified feature set (for example the bigrams of `"hello"`
//!    with an end marker: `"$h1"`, `"he1"`, `"el1"`, `"ll1"`, `"lo1"`,
//!    `"o$1"`).
//! 2. A [`Database`] stores strings and their interned feature sets in an
//!    inverted index grouped by *feature-set size*. Duplicate insertions are
//!    kept: two equal strings get distinct IDs and both are returned.
//! 3. A [`Searcher`] prunes candidate feature sizes, joins posting lists, and
//!    scores survivors with a [`Measure`] to decide which strings satisfy the
//!    threshold alpha.
//!
//! Five standard measures are built in: [`Cosine`], [`Dice`], [`Jaccard`],
//! [`Overlap`], and [`ExactMatch`]. Any number of custom feature extractors and
//! similarity measures can be plugged in by implementing the two
//! [`FeatureExtractor`] and [`Measure`] traits.
//!
//! # Quick start
//!
//! Index a few strings and run a ranked lookup:
//!
//! ```
//! use std::sync::Arc;
//!
//! use simstring_rust::database::HashDb;
//! use simstring_rust::extractors::CharacterNgrams;
//! use simstring_rust::measures::Cosine;
//! use simstring_rust::Searcher;
//!
//! // 1. Choose a feature extractor and build an empty database.
//! let mut db = HashDb::new(Arc::new(CharacterNgrams::new(2, "$")));
//!
//! // 2. Index some strings (duplicates stay distinct entries).
//! db.insert("hello".to_string());
//! db.insert("help".to_string());
//! db.insert("world".to_string());
//!
//! // 3. Search: returns strings whose similarity >= alpha.
//! let searcher = Searcher::new(&db, Cosine);
//! let results = searcher.ranked_search("hell", 0.5)?;
//! assert_eq!(results.first().map(|(text, _)| *text), Some("hello"));
//! assert!(results.len() == 2 && results[1].0 == "help");
//! # println!("{results:?}");
//! # Ok::<(), simstring_rust::SearchError>(())
//! ```
//!
//! [`search`](Searcher::search) returns the same strings without scores, sorted
//! ascending; [`ranked_search`](Searcher::ranked_search) returns
//! `(string, score)` pairs sorted from the most to the least similar.
//!
//! Alpha must satisfy `0.0 < alpha <= 1.0` or both methods return
//! [`SearchError::InvalidThreshold`].
//!
//! # Python bindings
//!
//! The same functionality ships as the `simstring-rust` package on PyPI
//! (imported as `simstring_rust`), with submodules `database`, `extractors`,
//! `measures`, `searcher`, and `errors`. See the project documentation for the
//! Python guide.
//!
//! # Project status
//!
//! Unicode is fully supported (extractors operate on characters, not bytes).
//! Building a database directly from a file is planned but not yet implemented.

#![warn(missing_docs)]

pub mod database;
pub mod extractors;
pub mod measures;
pub mod python;
pub mod search;

pub use database::{Database, HashDb};
pub use extractors::{CharacterNgrams, FeatureExtractor, WordNgrams};
pub use measures::{Cosine, Dice, ExactMatch, Jaccard, Measure, Overlap};
pub use search::{SearchError, Searcher};

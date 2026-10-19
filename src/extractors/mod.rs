//! Feature extraction: generating the feature sets that get indexed and
//! compared. This module defines the [`FeatureExtractor`] trait and the two
//! built-in extractors, [`CharacterNgrams`] and [`WordNgrams`].
//!
//! # Occurrence-qualified features
//!
//! A feature set must not contain the same feature twice. So that repeated
//! n-grams are kept distinguishable, the built-in extractors qualify each
//! occurrence with a 1-based occurrence counter, appended directly to the
//! feature string: the bigrams of `"aaa"` are `"$a1"`, `"aa1"`, `"aa2"`,
//! `"a$1"`. Custom extractors must apply the same convention to their own
//! features, since search matches features by exact string, and unqualified
//! duplicates would collapse into one.
//!
//! # Example: the built-in character bigrams of a word
//!
//! ```
//! use simstring_rust::{extractors::CharacterNgrams, FeatureExtractor};
//!
//! let mut features = Vec::new();
//! CharacterNgrams::new(2, "$").visit_features("hello", &mut |feature| {
//!     features.push(feature.to_string());
//! });
//! assert_eq!(
//!     vec!["$h1", "he1", "el1", "ll1", "lo1", "o$1"],
//!     features
//! );
//! ```
//!
//! # Implementing the trait
//!
//! [`CharacterNgrams`] and [`WordNgrams`] cover the common cases; see their
//! docs for the exact feature scheme. One small [`FeatureExtractor`] gives
//! full control over similarity geometry, e.g. lowercased word features.
//! Implement [`visit_features`](FeatureExtractor::visit_features) to produce
//! occurrence-qualified features: indexing runs the same extractors through
//! the default [`features`](FeatureExtractor::features) (which interns and
//! dedupes them), and search calls `visit_features` per query:
//!
//! ```
//! use std::collections::HashMap;
//! use std::sync::Arc;
//!
//! use simstring_rust::database::HashDb;
//! use simstring_rust::measures::Overlap;
//! use simstring_rust::{FeatureExtractor, Searcher};
//!
//! // Lowercased words, qualified by how often they appear, e.g. "hello1".
//! struct LowerWords;
//!
//! impl FeatureExtractor for LowerWords {
//!     fn visit_features(&self, text: &str, visitor: &mut dyn FnMut(&str)) {
//!         let mut counts: HashMap<String, usize> = HashMap::new();
//!         for word in text.split_whitespace() {
//!             let lowered = word.to_lowercase();
//!             let counter = counts.entry(lowered.clone()).or_insert(0);
//!             *counter += 1;
//!             // The feature string is only borrowed for the callback call.
//!             visitor(&format!("{lowered}{counter}"));
//!         }
//!     }
//! }
//!
//! let mut db = HashDb::new(Arc::new(LowerWords));
//! db.insert("Hello world".to_owned());
//! db.insert("Well hello again WORLD, hello".to_owned());
//!
//! // The feature "hello1" is present in both strings: Overlap finds them.
//! let scorer = Searcher::new(&db, Overlap);
//! assert_eq!(
//!     scorer.search("hello", 0.99)?,
//!     ["Hello world", "Well hello again WORLD, hello"]
//! );
//! # Ok::<(), simstring_rust::SearchError>(())
//! ```

mod character_ngrams;
mod word_ngrams;

use lasso::{Rodeo, Spur};
use rustc_hash::FxHashMap;

/// Visits raw custom features with their occurrence counts appended.
pub(crate) fn visit_counted_features(features: &[String], visitor: &mut dyn FnMut(&str)) {
    let mut counter: FxHashMap<&str, usize> = FxHashMap::default();
    let mut buffer = String::new();
    let mut number = itoa::Buffer::new();
    for val in features {
        let count = counter.entry(val).or_default();
        *count += 1;
        buffer.clear();
        buffer.push_str(val);
        buffer.push_str(number.format(*count));
        visitor(&buffer);
    }
}

/// Feature generator used by both indexing and querying.
///
/// Implement this trait to plug custom features into the engine. The contract
/// each implementation follows:
///
/// - [`visit_features`](Self::visit_features) emits *occurrence-qualified*
///   features: every emitted feature is unique within `text`, with repeats
///   distinguished by a 1-based suffix — the built-ins append the counter
///   directly (`"aa1"`, `"aa2"`, …). The same scheme must also appear when the
///   database stores features, which happens through the default
///   [`features`](Self::features); as long as `visit_features` is
///   deterministic per text, both call sites stay consistent.
/// - Feature strings are borrowed only for the callback call: the extractor
///   may reuse its buffer between visits (the built-ins do, for efficiency).
/// - Feature sets are small: emitting a feature count in the hundreds per
///   string is normal, but avoid quadratic behavior per call.
pub trait FeatureExtractor: Send + Sync {
    /// Visits occurrence-qualified features. Each feature must be unique within a text.
    /// The borrowed string is valid only for the duration of the callback.
    fn visit_features(&self, text: &str, visitor: &mut dyn FnMut(&str));

    /// Expected number of features for `text`, used to pre-size the feature vector.
    /// An estimate is fine; the default of 0 disables pre-sizing.
    fn feature_count_hint(&self, _text: &str) -> usize {
        0
    }

    /// Extracts features for insertion, sorted by interned ID.
    fn features(&self, text: &str, interner: &mut Rodeo) -> Vec<Spur> {
        let mut features = Vec::with_capacity(self.feature_count_hint(text));
        self.visit_features(text, &mut |feature| {
            features.push(interner.get_or_intern(feature));
        });
        features.sort_unstable();
        features.dedup();
        features
    }
}

pub use character_ngrams::CharacterNgrams;
pub use word_ngrams::WordNgrams;

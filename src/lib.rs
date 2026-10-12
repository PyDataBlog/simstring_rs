pub mod database;
#[doc(hidden)] // Internal; public only for integration tests.
pub mod execution;
pub mod extractors;
pub mod measures;
pub mod python;
pub mod search;

pub use database::{Database, HashDb};
pub use extractors::{CharacterNgrams, FeatureExtractor, WordNgrams};
pub use measures::{Cosine, Dice, ExactMatch, Jaccard, Measure, Overlap};
pub use search::{SearchError, Searcher};

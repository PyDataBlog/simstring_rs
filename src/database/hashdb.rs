use crate::database::{Database, StringId};
use crate::extractors::FeatureExtractor;
use lasso::{Rodeo, Spur};
use rustc_hash::FxHashMap;
use std::fmt;
use std::sync::Arc;

pub struct HashDb {
    feature_extractor: Arc<dyn FeatureExtractor>,
    pub strings: Vec<String>,
    string_features: Vec<Vec<Spur>>,
    /// Posting lists keyed by `posting_key(size, feature)`.
    postings: FxHashMap<u64, Vec<StringId>>,
    feature_sizes: Vec<usize>,
    interner: Rodeo,
}

/// Packs a (feature-set size, feature) pair into one key: size in the high 32 bits, the
/// interner key in the low 32. One flat map means one hash probe per lookup instead of a
/// size lookup followed by a feature lookup. `None` for sizes no inserted string can have.
fn posting_key(size: usize, feature: Spur) -> Option<u64> {
    let size = u32::try_from(size).ok()?;
    Some(u64::from(size) << 32 | u64::from(feature.into_inner().get()))
}

impl fmt::Debug for HashDb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HashDb")
            .field("num_strings", &self.strings.len())
            .field("num_feature_size_buckets", &self.feature_sizes.len())
            .field("total_unique_features_interned", &self.interner.len())
            .field("total_unique_features_indexed", &self.postings.len())
            .finish()
    }
}

impl HashDb {
    pub fn new(feature_extractor: Arc<dyn FeatureExtractor>) -> Self {
        Self {
            feature_extractor,
            strings: Vec::new(),
            string_features: Vec::new(),
            postings: FxHashMap::default(),
            feature_sizes: Vec::new(),
            interner: Rodeo::default(),
        }
    }

    pub fn insert(&mut self, text: String) {
        Database::insert(self, text);
    }

    pub fn clear(&mut self) {
        Database::clear(self);
    }
}

impl Database for HashDb {
    fn insert(&mut self, text: String) {
        let features = self.feature_extractor.features(&text, &mut self.interner);
        let size = features.len();
        let string_id =
            StringId::try_from(self.strings.len()).expect("HashDb holds at most u32::MAX strings");

        self.strings.push(text);
        if let Err(index) = self.feature_sizes.binary_search(&size) {
            self.feature_sizes.insert(index, size);
        }
        for &feature in &features {
            let key = posting_key(size, feature).expect("a string has at most u32::MAX features");
            self.postings.entry(key).or_default().push(string_id);
        }
        self.string_features.push(features);
    }

    fn clear(&mut self) {
        self.strings.clear();
        self.string_features.clear();
        self.postings.clear();
        // clear the interner to release memory
        self.interner.clear();
        self.feature_sizes.clear();
    }

    fn lookup_strings(&self, size: usize, feature: Spur) -> Option<&[StringId]> {
        self.postings
            .get(&posting_key(size, feature)?)
            .map(Vec::as_slice)
    }

    fn get_string(&self, id: StringId) -> Option<&str> {
        self.strings.get(id as usize).map(AsRef::as_ref)
    }

    fn get_features(&self, id: StringId) -> Option<&[Spur]> {
        self.string_features.get(id as usize).map(Vec::as_slice)
    }

    fn feature_extractor(&self) -> &dyn FeatureExtractor {
        &*self.feature_extractor
    }

    fn max_feature_len(&self) -> usize {
        self.feature_sizes.last().copied().unwrap_or(0)
    }

    fn interner(&self) -> &Rodeo {
        &self.interner
    }

    fn feature_sizes(&self) -> &[usize] {
        &self.feature_sizes
    }

    fn total_strings(&self) -> usize {
        self.strings.len()
    }
}

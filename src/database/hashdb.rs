use crate::database::{Database, StringId};
use crate::extractors::FeatureExtractor;
use lasso::{Key, Rodeo, Spur};
use std::fmt;
use std::sync::Arc;

/// One feature's posting lists. `lists[i]` holds the strings with `sizes[i]` features;
/// `sizes` is sorted and kept apart so searching it touches few cache lines.
#[derive(Default)]
struct FeaturePostings {
    sizes: Vec<usize>,
    lists: Vec<Vec<StringId>>,
}

pub struct HashDb {
    feature_extractor: Arc<dyn FeatureExtractor>,
    pub strings: Vec<String>,
    string_features: Vec<Vec<Spur>>,
    /// Indexed by `Spur::into_usize()`: interner keys are dense, so no hashing is needed,
    /// and `visit_postings` walks only the sizes a feature actually occurs in.
    postings: Vec<FeaturePostings>,
    feature_sizes: Vec<usize>,
    interner: Rodeo,
}

impl fmt::Debug for HashDb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indexed: usize = self.postings.iter().map(|p| p.sizes.len()).sum();
        f.debug_struct("HashDb")
            .field("num_strings", &self.strings.len())
            .field("num_feature_size_buckets", &self.feature_sizes.len())
            .field("total_unique_features_interned", &self.interner.len())
            .field("total_unique_features_indexed", &indexed)
            .finish()
    }
}

impl HashDb {
    pub fn new(feature_extractor: Arc<dyn FeatureExtractor>) -> Self {
        Self {
            feature_extractor,
            strings: Vec::new(),
            string_features: Vec::new(),
            postings: Vec::new(),
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
        let string_id = self.strings.len();

        self.strings.push(text);
        if let Err(index) = self.feature_sizes.binary_search(&size) {
            self.feature_sizes.insert(index, size);
        }
        for &feature in &features {
            let index = feature.into_usize();
            if index >= self.postings.len() {
                self.postings
                    .resize_with(index + 1, FeaturePostings::default);
            }
            let postings = &mut self.postings[index];
            match postings.sizes.binary_search(&size) {
                Ok(i) => postings.lists[i].push(string_id),
                Err(i) => {
                    postings.sizes.insert(i, size);
                    postings.lists.insert(i, vec![string_id]);
                }
            }
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
        let postings = self.postings.get(feature.into_usize())?;
        let i = postings.sizes.binary_search(&size).ok()?;
        Some(&postings.lists[i])
    }

    fn visit_postings<'a>(
        &'a self,
        feature: Spur,
        min: usize,
        max: usize,
        visit: &mut dyn FnMut(usize, &'a [StringId]),
    ) {
        let Some(postings) = self.postings.get(feature.into_usize()) else {
            return;
        };
        let start = postings.sizes.partition_point(|&size| size < min);
        let end = postings.sizes.partition_point(|&size| size <= max);
        for (&size, ids) in postings.sizes[start..end.max(start)]
            .iter()
            .zip(&postings.lists[start..])
        {
            visit(size, ids);
        }
    }

    fn get_string(&self, id: StringId) -> Option<&str> {
        self.strings.get(id).map(AsRef::as_ref)
    }

    fn get_features(&self, id: StringId) -> Option<&[Spur]> {
        self.string_features.get(id).map(Vec::as_slice)
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

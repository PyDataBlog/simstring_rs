use crate::Database;
use crate::database::StringId;
use crate::measures::Measure;
use lasso::Spur;
use rayon::prelude::*;
use rustc_hash::FxHashSet;
use std::cell::RefCell;
use thiserror::Error;

/// Per-thread join state, reused so a query never allocates O(total strings).
#[derive(Default)]
struct Scratch {
    /// Prefix-list occurrences per string ID. All zero between joins.
    counts: Vec<u32>,
    /// IDs with a non-zero count in the current join.
    touched: Vec<StringId>,
}

thread_local! {
    static SCRATCH: RefCell<Scratch> = RefCell::default();
}

#[derive(Error, Debug, PartialEq)]
pub enum SearchError {
    #[error(
        "Search threshold alpha must be between 0.0 (exclusive) and 1.0 (inclusive), but was {0}"
    )]
    InvalidThreshold(f64),
}

pub struct Searcher<'db, M: Measure> {
    db: &'db dyn Database,
    measure: M,
}

impl<'db, M: Measure> Searcher<'db, M> {
    pub fn new(db: &'db dyn Database, measure: M) -> Self {
        // Select the execution backend now so an invalid override fails before searching.
        crate::execution::kernels();
        Self { db, measure }
    }

    pub fn search(&self, query_string: &str, alpha: f64) -> Result<Vec<&str>, SearchError> {
        let candidates = self.search_candidates(query_string, alpha)?;
        let mut results: Vec<_> = candidates
            .into_iter()
            .filter_map(|(id, _)| self.db.get_string(id))
            .collect();
        results.sort_unstable();
        Ok(results)
    }

    pub fn ranked_search(
        &self,
        query_string: &str,
        alpha: f64,
    ) -> Result<Vec<(&str, f64)>, SearchError> {
        let candidates = self.search_candidates(query_string, alpha)?;
        let mut results: Vec<_> = candidates
            .into_iter()
            .filter_map(|(id, score)| {
                let text = self.db.get_string(id)?;
                Some((text, score))
            })
            .collect();
        results.sort_unstable_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(b.0)));
        Ok(results)
    }

    fn search_candidates(
        &self,
        text: &str,
        alpha: f64,
    ) -> Result<Vec<(StringId, f64)>, SearchError> {
        if !(alpha > 0.0 && alpha <= 1.0) {
            return Err(SearchError::InvalidThreshold(alpha));
        }
        let interner = self.db.interner();
        let mut features = Vec::new();
        // Unknown features contribute to the denominator, but never to an overlap.
        let mut unknown = FxHashSet::default();
        self.db
            .feature_extractor()
            .visit_features(text, &mut |feature| {
                if let Some(id) = interner.get(feature) {
                    features.push(id);
                } else {
                    unknown.insert(feature.to_owned());
                }
            });
        features.sort_unstable();
        features.dedup();
        Ok(self.search_for_ids(&features, features.len() + unknown.len(), alpha))
    }

    fn search_for_ids(
        &self,
        features: &[Spur],
        query_size: usize,
        alpha: f64,
    ) -> Vec<(StringId, f64)> {
        if features.is_empty() {
            return Vec::new();
        }
        let min = self.measure.min_feature_size(query_size, alpha);
        let max = self.measure.max_feature_size(query_size, alpha, self.db);
        let sizes = self.db.feature_sizes();
        let start = sizes.partition_point(|&size| size < min);
        let end = sizes.partition_point(|&size| size <= max);
        if start >= end {
            return Vec::new();
        }
        let sizes = &sizes[start..end];
        let join = |&size: &usize| {
            let tau = self
                .measure
                .minimum_common_feature_count(query_size, size, alpha)
                .max(1);
            if tau > features.len() {
                return Vec::new();
            }
            self.overlap_join(features, query_size, tau, size, alpha)
        };
        // Small joins cost less than scheduling a Rayon task. Parallelize only large
        // databases; independent callers can always search concurrently.
        if self.db.total_strings() >= 100_000 && sizes.len() > 4 {
            sizes.par_iter().flat_map_iter(join).collect()
        } else {
            sizes.iter().flat_map(join).collect()
        }
    }

    fn overlap_join(
        &self,
        features: &[Spur],
        query_size: usize,
        tau: usize,
        size: usize,
        alpha: f64,
    ) -> Vec<(StringId, f64)> {
        if tau == 0 || tau > features.len() {
            return Vec::new();
        }
        let mut postings: Vec<_> = features
            .iter()
            .filter_map(|&feature| self.db.lookup_strings(size, feature))
            .filter(|ids| !ids.is_empty())
            .collect();
        if postings.len() < tau {
            return Vec::new();
        }
        postings.sort_unstable_by_key(|ids| ids.len());
        // Every match must occur in this prefix: only tau - 1 lists remain outside it.
        // Missing query features cannot contribute and need not enlarge the prefix.
        let prefix_len = postings.len() - tau + 1;
        let (prefix, rest) = postings.split_at(prefix_len);
        let kernels = crate::execution::kernels();
        // Only library code runs while the scratch is borrowed; the measure is called
        // afterwards, so a re-entrant or panicking measure cannot see a dirty scratch.
        let survivors: Vec<(StringId, usize)> = SCRATCH.with_borrow_mut(|scratch| {
            let Scratch { counts, touched } = scratch;
            // Lists are sorted, so their last IDs bound every counter used below.
            let max_id = prefix.iter().map(|ids| ids[ids.len() - 1]).max();
            let needed = max_id.map_or(0, |id| id as usize + 1);
            if counts.len() < needed {
                counts.resize(needed, 0);
            }
            // ScanCount: count prefix occurrences without materialising and sorting
            // the concatenated lists. `touched` records each ID once.
            for ids in prefix {
                for &id in *ids {
                    let slot = &mut counts[id as usize];
                    if *slot == 0 {
                        touched.push(id);
                    }
                    *slot += 1;
                }
            }
            touched.sort_unstable();
            let survivors = touched
                .iter()
                .filter_map(|&id| {
                    // Taking the count also restores the all-zero invariant.
                    let mut count = std::mem::take(&mut counts[id as usize]) as usize;
                    for (index, ids) in rest.iter().enumerate() {
                        count += usize::from(kernels.contains(ids, id));
                        // Even if every remaining list matched, tau is out of reach.
                        if count + rest.len() - index - 1 < tau {
                            return None;
                        }
                    }
                    Some((id, count))
                })
                .collect();
            touched.clear();
            survivors
        });
        survivors
            .into_iter()
            .filter_map(|(id, count)| {
                let score = self.measure.similarity_from_counts(query_size, size, count);
                (score >= alpha).then_some((id, score))
            })
            .collect()
    }
}

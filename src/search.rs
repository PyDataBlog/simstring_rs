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
        let sizes = &sizes[sizes.partition_point(|&size| size < min)..];
        let sizes = &sizes[..sizes.partition_point(|&size| size <= max)];
        let (Some(&lo), Some(&hi)) = (sizes.first(), sizes.last()) else {
            return Vec::new();
        };
        // Gather each feature's posting lists once. Looking up every (size, feature)
        // pair instead costs a probe per pair, and most pairs miss.
        let mut hits: Vec<(usize, &[StringId])> = Vec::new();
        for &feature in features {
            self.db.visit_postings(feature, lo, hi, &mut |size, ids| {
                // Tolerate implementations that ignore the documented contract.
                if (lo..=hi).contains(&size) && !ids.is_empty() {
                    hits.push((size, ids));
                }
            });
        }
        // Group by size with a counting sort: sizes span a small range, and a comparison
        // sort of all hits costs more than the lookups it replaces.
        let mut starts = vec![0; hi - lo + 2];
        for &(size, _) in &hits {
            starts[size - lo + 1] += 1;
        }
        for i in 1..starts.len() {
            starts[i] += starts[i - 1];
        }
        let mut grouped: Vec<&[StringId]> = vec![&[]; hits.len()];
        let mut next = starts.clone();
        for &(size, ids) in &hits {
            grouped[next[size - lo]] = ids;
            next[size - lo] += 1;
        }
        // Shortest lists first: each join scans a prefix of its group. Sizes with fewer
        // than tau lists cannot match, so skip them before paying for the sort.
        let mut joins = Vec::new();
        for &size in sizes {
            let range = starts[size - lo]..starts[size - lo + 1];
            let tau = self
                .measure
                .minimum_common_feature_count(query_size, size, alpha)
                .max(1);
            if range.len() >= tau {
                grouped[range.clone()].sort_unstable_by_key(|ids| ids.len());
                joins.push((size, range, tau));
            }
        }
        let join = |(size, range, tau): &(usize, std::ops::Range<usize>, usize)| {
            self.overlap_join(&grouped[range.clone()], query_size, *tau, *size, alpha)
        };
        // Small joins cost less than scheduling a Rayon task. Parallelize only large
        // databases; independent callers can always search concurrently.
        if self.db.total_strings() >= 100_000 && joins.len() > 4 {
            joins.par_iter().flat_map_iter(join).collect()
        } else {
            joins.iter().flat_map(join).collect()
        }
    }

    /// `postings` holds one feature-set size's non-empty lists, shortest first.
    fn overlap_join(
        &self,
        postings: &[&[StringId]],
        query_size: usize,
        tau: usize,
        size: usize,
        alpha: f64,
    ) -> Vec<(StringId, f64)> {
        if tau == 0 || postings.len() < tau {
            return Vec::new();
        }
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
            let needed = max_id.map_or(0, |id| id + 1);
            if counts.len() < needed {
                counts.resize(needed, 0);
            }
            // ScanCount: count prefix occurrences without materialising and sorting
            // the concatenated lists. `touched` records each ID once.
            for ids in prefix {
                for &id in *ids {
                    if counts[id] == 0 {
                        touched.push(id);
                    }
                    counts[id] += 1;
                }
            }
            touched.sort_unstable();
            let survivors = touched
                .iter()
                .filter_map(|&id| {
                    // Taking the count also restores the all-zero invariant.
                    let mut count = std::mem::take(&mut counts[id]) as usize;
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

# Migrating to 0.4.0 / 0.5.0

Version `0.4.0` reorganized the Rust traits under a read-only query engine.
The usual `HashDb` and `Searcher` call sites — and all Python signatures —
keep working; only custom Rust trait implementations need updating.

## `FeatureExtractor`

Implement `visit_features(&self, text: &str, visitor: &mut dyn FnMut(&str))`,
emitting **occurrence-qualified** features (every feature unique within one
text; the built-ins append 1-based counters directly). The default
`features` method interns, sorts, and deduplicates for insertion; search
reads `visit_features` directly per query. See the
[custom extractor guide](guides/custom-extractors.md).

## `Database`

- `interner()` returns `&Rodeo` rather than a locked `Arc`.
- `lookup_strings()` returns sorted, unique `Option<&[StringId]>` postings —
  one slice per `(feature size, feature)` pair.
- `get_features()` returns `Option<&[Spur]>`.
- Implement `feature_sizes()` as a sorted, unique slice of the populated
  sizes. Each stored ID belongs to exactly one size bucket, and stored
  feature sets must be sorted and unique under the same interner.
- Optionally override `visit_postings()` to enumerate a feature's postings
  across a size range directly; the default calls `lookup_strings()` once per
  size.

## `Measure`

Implement `similarity_from_counts(x_len, y_len, common) -> f64` — search
counts overlaps in the join and only calls the trait to score that exact
intersection, so this method must agree with the pruning bounds:
`min_feature_size`, `max_feature_size`, and `minimum_common_feature_count`
should be *conservative* (never excluding a true candidate), computed with
outward rounding like the built-ins. The default `similarity(x, y)` trait
method intersects two sorted feature sets and delegates to
`similarity_from_counts`.

## Unchanged behavior details to rely on

- Unknown query features still contribute to similarity denominators.
- Duplicate insertions remain distinct results.
- Queries with no features return no results.
- Both search methods apply exact score filtering at threshold boundaries
  (`score >= alpha`), so floating point rounding cannot drop a borderline
  match.
- `interner()` keys are dense `Spur`s, backing `HashDb`'s vector-based
  posting index; implementations indexing by `Spur::into_usize()` can match
  its [performance](guides/performance.md) profile.

## 0.5.0 highlights

For completeness (no code changes required): overlap-join optimization that
drops hashing in favor of a per-feature index, Python 3.15 support, and
Wheel/CI polish. See the
[changelog](https://github.com/PyDataBlog/simstring_rs/blob/main/CHANGELOG.md)
for the exact list.

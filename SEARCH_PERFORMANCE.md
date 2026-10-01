# Search engine optimization

## Implementation

- Queries borrow the database's interner immutably. Unknown features are counted
  separately, preserving similarity denominators without growing the index.
- Sorted posting vectors replace per-feature hash sets. Increasing insertion IDs
  keep postings sorted without sorting on insertion.
- Only populated feature-size buckets are considered. The smallest necessary
  posting lists generate candidates; sorted candidate runs retain their overlap
  counts (`slice::chunk_by`). Binary searches of the remaining lists complete the
  exact count, with early rejection of impossible candidates.
- Ranked scores reuse those counts instead of intersecting features again.
  Both search methods apply the same final threshold check.
- Small databases use sequential joins. Databases of at least 100,000 rows with
  more than four eligible size buckets use Rayon. These are workload-dependent
  heuristics, not a universal optimal cutoff; callers can also parallelize queries.
- Character n-grams borrow slices of padded text, with an ASCII fast path and
  Unicode character-boundary fallback. Padding is precomputed and occurrence
  numbers use `itoa`. Word extraction reuses string buffers.
- Python searches release the GIL through PyO3 `Python::detach`. The database
  remains immutably borrowed until the results have been converted.
- Floating-point pruning bounds round outward using `next_down`/`next_up`;
  the final score check prevents false positives. This fixes boundary matches
  previously lost by rounding, including Dice at 0.8.

Uses Rust edition 2024, Rust 1.88+, PyO3 0.29.3 and Rayon 1.12.0 in the lockfile.
Release builds remain portable and use the existing compiler profile.

## Local measurements

Measured on Apple M1 Max, macOS 27.0, Rust 1.99.0 and CPython 3.14.8, comparing
the parent branch with this optimization. Numbers are local workload results,
not cross-platform guarantees. Existing JSON benchmarks use 5,797 company names,
100 query terms, Cosine similarity, and 100 measured batches. Times below are
**milliseconds per 100 ranked searches** (mean, rounded).

| N-gram | Threshold | Rust before | Rust after | Python before | Python after |
|---|---|---|---|---|---|
| 2 | 0.6 | 10.767 | 5.801 | 11.937 | 6.474 |
| 2 | 0.7 | 7.090 | 3.298 | 7.815 | 3.550 |
| 2 | 0.8 | 4.760 | 1.691 | 5.111 | 1.679 |
| 2 | 0.9 | 3.309 | 0.608 | 3.646 | 0.636 |
| 3 | 0.6 | 8.456 | 3.193 | 9.467 | 3.493 |
| 3 | 0.7 | 5.833 | 2.057 | 6.422 | 2.109 |
| 3 | 0.8 | 4.213 | 1.163 | 4.904 | 1.210 |
| 3 | 0.9 | 3.479 | 0.517 | 3.800 | 0.609 |
| 4 | 0.6 | 7.269 | 2.564 | 8.736 | 2.823 |
| 4 | 0.7 | 5.665 | 1.727 | 6.207 | 1.846 |
| 4 | 0.8 | 4.068 | 1.046 | 4.788 | 1.096 |
| 4 | 0.9 | 3.702 | 0.530 | 4.141 | 0.576 |

These runs show about **1.9–7.0× native** and **1.8–7.2× Python** speedups.
They precede the final unknown-feature fast rejection, which was subsequently
measured separately: the Criterion no-known-feature query at 100,000 rows takes
about 0.47 µs across all five measures.

Native insertion fell from 18.58/21.06/22.80 ms to 12.44/14.04/15.37 ms for
n-grams 2/3/4. The old insertion timer incorrectly subtracted setup time a second
time; the new timer measures only insertion, so this comparison slightly
understates the improvement.

Python threaded throughput (bigrams, threshold 0.6, 1,000 queries, median of five
post-warmup runs, including executor scheduling):

| Workers | Before queries/s | After queries/s |
|---|---|---|
| 1 | 6,861 | 14,016 |
| 4 | 6,716 | 38,011 |

All runs returned the same total match count. Process peak RSS after inserting
100,000 trigram-indexed rows (cycling the corpus) was **101.75 MiB before** and
**91.08 MiB after**, including the Python interpreter and input strings. This is
process memory, not an isolated index-allocation measurement.

## Reproducing measurements

After merging current `main`, the shared JSON benchmark uses **unranked** search
to match the other language implementations and C++ comparisons. The historical
table above records ranked searches from before that merge; use the Criterion
ranked cases for new ranked measurements. Current `main` also adds Python
`CustomExtractor` and direct extractor `apply` methods; these are preserved,
with custom callbacks adapted to the read-only feature visitor. Python callbacks
reattach to the interpreter only while collecting their raw features.

```sh
cargo bench --bench bench
cargo bench --bench search -- --noplot
# Run against the release wheel installed in your Python environment:
python benches/bench_rust.py
python benches/bench_threads.py
python benches/memory.py
```

The Criterion suite covers all five measures, ranked/unranked hits, near matches,
misses, and concurrent queries at 100, 5,797 and 100,000 rows. The largest index
cycles the source corpus and therefore intentionally includes duplicate strings.
Criterion results are stored under `target/criterion`; use `--save-baseline NAME`
and `--baseline NAME` for future comparisons. The new suite was run on the
optimized code; the before/after table uses the pre-existing JSON benchmark.

## Rust API migration

Normal `HashDb::new`, `insert`, `clear`, `Searcher::new`, `search`, and
`ranked_search` calls keep their signatures. Python method signatures also stay
the same; wheels now target ABI3 Python 3.10+, matching the package metadata.

Custom Rust implementations need these changes:

1. **FeatureExtractor**: implement
   `visit_features(&self, text: &str, visitor: &mut dyn FnMut(&str))`.
   Emit occurrence-qualified features through the callback. The default
   `features` method interns, sorts and deduplicates them for insertion.
   Search uses the same callback without interning.
2. **Database**: `interner()` returns `&Rodeo`; remove `.lock().unwrap()` from
   read sites. `lookup_strings` returns `Option<&[StringId]>`, sorted and unique.
   `get_features` returns `Option<&[Spur]>`. Implement `feature_sizes()` as a
   sorted, unique slice of populated sizes. Every ID must belong to exactly one
   size bucket; stored features must be sorted and unique, using the same interner.
3. **Measure**: implement
   `similarity_from_counts(x_len, y_len, common) -> f64`. `similarity` now has a
   default implementation which intersects sorted feature sets and calls it.
   Search uses the count-based method directly, so it must agree with your metric
   and pruning bounds. Borrowed measures (`&M`) are supported.

Duplicate inserted strings retain distinct IDs and can appear repeatedly in
results. Searches with no extracted features retain their existing empty-result
behavior. Unranked search now applies exact score filtering too, keeping it
consistent with ranked search at floating-point boundaries.

## Verification

Rust tests include an independent-interner brute-force search oracle for all
measures, exact-score thresholds, subnormal thresholds, unknown features, repeated
n-grams, Unicode, duplicates, clear/reinsert, concurrent readers, and a 100,000-row
parallel-join case. Character extraction is compared against a character-window
reference, including empty and multicharacter markers.

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo test --test test_python_integration -- --ignored
cargo +1.88.0 check --all-targets --locked
```

All passed locally after merging current `main`, including 72 Rust tests,
22 Python tests (including concurrent custom extractors), and the minimum
Rust version check. The Python
integration test builds and installs a release wheel before testing it.

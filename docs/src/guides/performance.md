# Performance and concurrency

## What search costs

The heavy work is the overlap join: for each viable feature-set size, the
engine gathers posting lists and counts shared features (ScanCount-style),
skipping pairs that cannot possibly satisfy `alpha`. Publish-worthy numbers
are in [BENCHMARKS.md](https://github.com/PyDataBlog/simstring_rs/blob/main/BENCHMARKS.md);
a typical x86_64 machine sees 0.3–4.2 ms for 100 queries over the ~5.8k-entry
name corpus (`benches/data/company_names.txt`) with character bigrams,
depending on `alpha` and `n`.

## Throughput knobs

- **Measure and alpha.** Higher alpha and stricter measures (`Jaccard` over
  `Overlap`) shrink candidate sets faster. `Overlap` explores the most
  candidates: its bounds admit everything.
- **Window size.** Bigger `n` (character or word) shortens features lists and
  costs more index probes; smaller `n` raises candidate counts for very
  short queries.
- **Insert once, query many.** `HashDb::insert` maintains the inverted index
  incrementally (fine for streaming), but bulk building and *then* searching
  lets the precomputed posting indexes work in their friendliest state.
- **Same database, many searchers.** `Searcher::new(&db, measure)` doesn't
  mutate anything; searches from multiple threads proceed concurrently.
  Interleave insert/clear with searches and the borrow checker will stop you.

## Threading behavior

- Rust: each thread's join state is lazily allocated scratch (reused across
  queries); two searchers in two threads never share it.
  Very large searches (roughly ≥ 100k strings and ≥ 4 size groups in the
  corpus) fan out with Rayon internally; smaller ones run on the calling
  thread only.
- Python: `Searcher.search` / `ranked_search` **release the GIL** while the
  Rust join runs, so `ThreadPoolExecutor` workers genuinely run in parallel
  on the same database. The executor snippet:

```python
from concurrent.futures import ThreadPoolExecutor
from simstring_rust.database import HashDb
from simstring_rust.extractors import CharacterNgrams
from simstring_rust.measures import Cosine
from simstring_rust.searcher import Searcher

db = HashDb(CharacterNgrams(2, "$"))
for name in ["apple", "apply", "banana"]:
    db.insert(name)
searcher = Searcher(db, Cosine())

queries = ["appl", "banan", "apples"]
with ThreadPoolExecutor(max_workers=4) as pool:
    futures = [pool.submit(searcher.search, q, 0.7) for q in queries]
    for f in futures:
        print(f.result())
```

`test_database.rs`, `test_search.rs`, and
`test_search_reference.rs` (which includes a
`concurrent_queries_and_reinsertion` test) all exercise these guarantees in
CI.

## Memory

Memory is bounded by the corpus strings plus one interned feature ID per
feature occurrence and one posting entry per distinct `(feature, size)` pair.
`clear()` releases interner memory too. If you run many indexes with
overlapping vocabularies, keep one database — even across languages — rather
than several partial ones to reuse the shared feature space.

## Related reading

- [Custom extractors](custom-extractors.md) for feature-price considerations.
- `benches/bench.rs`, `benches/micro.rs`, `benches/insert.rs` for
  benchmark methodologies; `benches/run_benches.py` cross-compares against
  Python, Julia, Ruby, and C++ implementations of SimString.

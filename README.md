# simstring_rust

[![Build Status](https://github.com/PyDataBlog/simstring_rs/actions/workflows/CI.yml/badge.svg)](https://github.com/PyDataBlog/simstring_rs/actions)
[![Project docs](https://img.shields.io/badge/docs-user%20guide-blue.svg)](https://pydatablog.github.io/simstring_rs/)
[![docs.rs](https://docs.rs/simstring_rust/badge.svg)](https://docs.rs/simstring_rust)
[![Crates.io](https://img.shields.io/crates/v/simstring_rust.svg)](https://crates.io/crates/simstring_rust)
[![PyPI version](https://badge.fury.io/py/simstring-rust.svg)](https://badge.fury.io/py/simstring-rust)
[![Python versions](https://img.shields.io/badge/python-3.10%E2%80%933.15-blue.svg)](https://github.com/PyDataBlog/simstring_rs/blob/main/pyproject.toml)
[![Rust](https://img.shields.io/badge/rust-1.89%2B-blue.svg?maxAge=3600)](https://github.com/PyDataBlog/simstring_rs)
[![Codecov](https://img.shields.io/codecov/c/github/PyDataBlog/simstring_rs?token=XJM8O8TD4U)](https://codecov.io/gh/PyDataBlog/simstring_rs)

A native Rust implementation of the [SimString](https://www.chokkan.org/software/simstring/)
algorithm (CPMerge): fast, exact threshold-based approximate string matching.
Give it a corpus of strings and a query, and it returns **every** string at
least `alpha`-similar to the query — no missing matches, no fabricated ones.
Built for natural language processing tasks that need fuzzy lookup over very
large corpora, in Rust or via Python bindings.

The same engine ships as **Rust crate** [`simstring_rust`](https://docs.rs/simstring_rust)
and **Python package** [`simstring-rust`](https://pypi.org/project/simstring-rust/).

## Features

- ✅ Exact threshold-based retrieval for large corpora (CPMerge / ScanCount join)
- ✅ Character n-gram and word n-gram feature extractors, plus custom extractors
- ✅ Unicode support (extractors operate on characters, not bytes)
- ✅ Thread-safe concurrent searches, in Rust and from Python
- ✅ Five standard similarity measures: Dice, Jaccard, Cosine, Overlap, ExactMatch
- [ ] Building databases directly from text files
  ([issue #9](https://github.com/PyDataBlog/simstring_rs/issues/9))
- [ ] MeCab-based tokenizer support ([issue #44](https://github.com/PyDataBlog/simstring_rs/issues/44))

## Installation

Requires Rust 1.89 or newer (edition 2024). Python bindings support Python
3.10–3.15 on Linux, macOS, and Windows.

### Rust

```toml
[dependencies]
simstring_rust = "0.5"
```

### Python

```bash
pip install simstring-rust
```

## Quick start (Rust)

```rust
use std::sync::Arc;

use simstring_rust::database::HashDb;
use simstring_rust::extractors::CharacterNgrams;
use simstring_rust::measures::Cosine;
use simstring_rust::Searcher;

fn main() {
    // 1. Choose a feature extractor and create an empty database.
    let mut db = HashDb::new(Arc::new(CharacterNgrams::new(2, "$")));

    // 2. Index some strings (duplicates stay distinct entries).
    db.insert("hello".to_string());
    db.insert("help".to_string());
    db.insert("world".to_string());

    // 3. Search: strings whose similarity >= alpha, best first.
    let searcher = Searcher::new(&db, Cosine);
    let results = searcher.ranked_search("hell", 0.5).unwrap();
    for (text, score) in results {
        println!("- {text} ({score:.4})");
    }
    // - hello (0.7303)
    // - help (0.6000)
}
```

The equivalent runnable examples live in [`examples/basic.rs`](https://github.com/PyDataBlog/simstring_rs/blob/main/examples/basic.rs)
(`cargo run --example basic`).

For the latest features, you can track the development branch:

```toml
[dependencies]
simstring_rust = { git = "https://github.com/PyDataBlog/simstring_rs.git", branch = "main" }
```

Note: the git branch may include experimental features and breakages. Use with caution, and pin a released version for stability.

## Quick start (Python)

```python
from simstring_rust.database import HashDb
from simstring_rust.extractors import CharacterNgrams
from simstring_rust.measures import Cosine
from simstring_rust.searcher import Searcher

db = HashDb(CharacterNgrams(2, "$"))
for name in ["apple", "apply", "apples"]:
    db.insert(name)

searcher = Searcher(db, Cosine())

# Ranked results: (text, score) pairs, best first.
print(searcher.ranked_search("appl", 0.7))
# [('apple', 0.7302967433402214), ('apply', 0.7302967433402214)]

# Unranked results: plain strings, sorted ascending.
print(searcher.search("appl", 0.7))
# ['apple', 'apply']
```

A fuller tour — including custom extractors and error handling — is in
[`examples/python_basic.py`](https://github.com/PyDataBlog/simstring_rs/blob/main/examples/python_basic.py).
Every exported class and method also documents itself: try `help(simstring_rust)`.

## Core concepts

Matching works over *feature sets* produced by a **feature extractor**:

| Extractor | Configuration | Features of `"green apples"` |
|---|---|---|
| `CharacterNgrams(n, endmarker)` | windows of `n` characters over the end-marked text | with `(2, "$")`: `$g1`, `gr1`, `re1`, `ee1`, `en1`, `n 1`, ` a1`, `ap1`, … |
| `WordNgrams(n, splitter, padder)` | windows of `n` padded tokens | with `(2, " ", "_")`: `_ green1`, `green apples1`, `apples _1` |
| `CustomExtractor(callable)` | any object with `apply(text) -> Iterable[str]` | whatever the callable yields, suffix-qualified automatically |

Repeated features are qualified with a 1-based occurrence counter (`aa1`,
`aa2`, …), so every feature is unique within one text. Extractors operate on
Unicode characters (not bytes) — `"héllo"` behaves exactly as its six
characters suggest.

The database stores each string's feature set **grouped by feature-set size**,
which lets search skip whole size buckets that cannot reach the threshold. A
**similarity measure** scores the intersection of two feature sets:

| Measure | Formula (x, y = set sizes, c = intersection) | Best for |
|---|---|---|
| `Cosine`    | `c / sqrt(x * y)`         | balanced generic search |
| `Dice`      | `2 * c / (x + y)`         | the classic SimString default |
| `Jaccard`   | `c / (x + y - c)`         | strict, high-precision retrieval |
| `Overlap`   | `c / min(x, y)`           | substring/containment search |
| `ExactMatch`| `1.0` iff sets are equal  | membership probes |

`alpha` is the minimum similarity a result must have and must satisfy
`0.0 < alpha <= 1.0` — outside that range, `search`/`ranked_search` raise
`SearchError` (Rust: `SearchError::InvalidThreshold`). Higher values return
fewer, closer matches; the exact retrieval guarantee means the answer set is
always complete for the chosen threshold.

Behavior worth knowing:

- **Duplicates stay distinct.** Inserting a string twice yields two database
  entries, both returned by searches.
- **Unknown features count against the query.** Query features that never
  appeared in the corpus contribute to similarity denominators but can never
  add to an overlap, lowering scores.
- **Empty queries match nothing.** A query that extracts no features returns
  an empty result.
- **Searches are concurrent-safe.** Rust `Searcher` uses no mutable state; the
  Python bindings release the GIL during search and can be driven from
  `ThreadPoolExecutor` workers.

## Documentation

- **Project guide** (this documentation site): <https://pydatablog.github.io/simstring_rs/>
- **Rust API reference**: <https://docs.rs/simstring_rust>
- **Python API reference**: <https://pydatablog.github.io/simstring_rs/api/python.html>
- **Changelog**: [CHANGELOG.md](https://github.com/PyDataBlog/simstring_rs/blob/main/CHANGELOG.md)
- **Benchmarks**: [BENCHMARKS.md](https://github.com/PyDataBlog/simstring_rs/blob/main/BENCHMARKS.md)

## Performance

Most search time goes to gathering postings and counting overlaps; the join
prunes candidate feature sizes and skips pairs that cannot satisfy the
threshold. Measured on one x86_64 machine (`cargo bench --bench micro`, 100
queries over `benches/data/company_names.txt`, Cosine), before and after the
ScanCount and per-feature posting index changes:

| n-gram | alpha | before | now |
|---|---|---|---|
| 2 | 0.6 | 5.22 ms | 4.14 ms |
| 2 | 0.9 | 0.55 ms | 0.47 ms |
| 3 | 0.6 | 2.93 ms | 1.66 ms |
| 3 | 0.9 | 0.47 ms | 0.34 ms |
| 4 | 0.6 | 2.28 ms | 1.07 ms |
| 4 | 0.9 | 0.46 ms | 0.32 ms |

Cross-language comparisons (Rust, Python, Julia, Ruby, C++) live in
[BENCHMARKS.md](https://github.com/PyDataBlog/simstring_rs/blob/main/BENCHMARKS.md),
produced by [`benches/run_benches.py`](https://github.com/PyDataBlog/simstring_rs/blob/main/benches/run_benches.py).

## Migrating from 0.3 / 0.4

`0.4.0` reorganized the Rust traits under a read-only query engine; Python
signatures are unchanged. See
[the migration guide](https://pydatablog.github.io/simstring_rs/migration.html)
for the exact `FeatureExtractor`, `Database`, and `Measure` changes.

## Contributing

Bug reports, feature requests, and pull requests are welcome — start with
[CONTRIBUTING.md](https://github.com/PyDataBlog/simstring_rs/blob/main/CONTRIBUTING.md)
for the local setup, test commands, and release process.

## License

This project is licensed under the MIT License.

## Acknowledgements

Inspired by the [SimString](https://www.chokkan.org/software/simstring/)
project by Naoki Okazaki and Yasuhito Asanuma, and the original CPMerge
algorithm paper [SimString: A Fast and Flexible Method for Approximate String
Matching](https://www.aclweb.org/anthology/D10-1044/) (Okazaki & Tsujii, 2010).

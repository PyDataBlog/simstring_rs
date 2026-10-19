# Getting started (Python)

The `simstring-rust` package on PyPI wraps the Rust engine with PyO3. Wheels
are available for Python 3.10–3.15 on Linux, macOS, and Windows (abi3).

## Install

```bash
pip install simstring-rust
```

## Index and query

```python
from simstring_rust.database import HashDb
from simstring_rust.extractors import CharacterNgrams
from simstring_rust.measures import Cosine
from simstring_rust.searcher import Searcher

db = HashDb(CharacterNgrams(2, "$"))
for name in ["apple", "apply", "apples"]:
    db.insert(name)

searcher = Searcher(db, Cosine())

# Ranked: (text, score) pairs, best first.
print(searcher.ranked_search("appl", 0.7))
# [('apple', 0.7302967433402214), ('apply', 0.7302967433402214)]

# Unranked: plain strings, sorted ascending.
print(searcher.search("appl", 0.7))
# ['apple', 'apply']
```

Invalid thresholds (`0.0` or above `1.0`) raise
`simstring_rust.errors.SearchError`, which derives from `ValueError`.

## Modules

| Module | Contents |
|---|---|
| `simstring_rust.database` | `HashDb` |
| `simstring_rust.extractors` | `CharacterNgrams`, `WordNgrams`, `CustomExtractor` |
| `simstring_rust.measures` | `Cosine`, `Dice`, `ExactMatch`, `Jaccard`, `Overlap` |
| `simstring_rust.searcher` | `Searcher` |
| `simstring_rust.errors` | `SearchError` |

Every class, method, and module documents itself — `help(simstring_rust)` and
`help(simstring_rust.searcher.Searcher)` show the same material as the
[Python API reference](../api/python.md). A runnable tour, including custom
extractors, lives in
[`examples/python_basic.py`](https://github.com/PyDataBlog/simstring_rs/blob/main/examples/python_basic.py).

## Notes

- The extractor is chosen once at `HashDb` construction and is used for every
  inserted string *and* every query.
- Inserting the same string twice is fine: the database keeps two entries and
  both are returned by matching searches.
- Searches hold no Python state and hand work off to a thread pool; the GIL is
  released during search, so `ThreadPoolExecutor` workers scale for large
  databases.
- Type stubs are planned — see
  [issue #102](https://github.com/PyDataBlog/simstring_rs/issues/102).

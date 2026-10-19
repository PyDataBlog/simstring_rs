# Python API reference

Import names come from the `simstring-rust` package (the underlying Rust
crate is `simstring_rust`; on PyPI, with a dash). Docstrings for every item
ship in the wheel, so `help(...)` mirrors this page.

## Module layout

| Module | Contents |
|---|---|
| `simstring_rust.database` | `HashDb` |
| `simstring_rust.extractors` | `CharacterNgrams`, `WordNgrams`, `CustomExtractor` |
| `simstring_rust.measures` | `Cosine`, `Dice`, `ExactMatch`, `Jaccard`, `Overlap` |
| `simstring_rust.searcher` | `Searcher` |
| `simstring_rust.errors` | `SearchError` |

## Extractors

**`CharacterNgrams(n, endmarker)`** — character window features over the text
padded with `n - 1` copies of `endmarker` on each side; repeated grams
counter-suffixed. `n = 0` emits nothing.

```python
>>> CharacterNgrams(2, '$').apply('hello')
['$h1', 'he1', 'el1', 'll1', 'lo1', 'o$1']
```

**`WordNgrams(n, splitter, padder)`** — token windows; split on `splitter`
(empty tokens dropped), one `padder` token per side, results joined with a
single space and counter-suffixed.

```python
>>> WordNgrams(2, ' ', '_').apply('green apples')
['_ green1', 'green apples1', 'apples _1']
```

**`CustomExtractor(obj)`** — wraps any `obj` exposing
`apply(text: str) -> Iterable[str]`, adding the occurrence suffixes for you:

```python
>>> class LowerBigrams:
...     def apply(self, text):
...         tokens = text.lower().split()
...         return [f"{a}|{b}" for a, b in zip(tokens, tokens[1:])]
>>> CustomExtractor(LowerBigrams()).apply('Some Custom Text')
['some|custom1', 'custom|text1']
```

## Measures

All are stateless constructors; call e.g. `Cosine()` and use the result as
`Searcher`'s second argument.

| Class | Similarity |
|---|---|
| `Cosine()` | `common / sqrt(x_len * y_len)` |
| `Dice()` | `2 * common / (x_len + y_len)` |
| `ExactMatch()` | `1.0` for identical feature sets, else `0.0` |
| `Jaccard()` | `common / (x_len + y_len - common)` |
| `Overlap()` | `common / min(x_len, y_len)` |

## `simstring_rust.database.HashDb`

- `HashDb(extractor)` — create an empty database; `extractor` must be one of
  the three extractor classes above, else `TypeError`.
- `insert(text: str)` — index one string; duplicates stay distinct entries.
- `clear()` — drop everything, including the interner's memory.
- `strings() -> list[str]` — insertion-order strings; index `i` is string ID `i`.
- `__len__()` — the number of indexed strings.

```python
>>> db = HashDb(CharacterNgrams(2, '$'))
>>> db.insert('apple'); db.insert('apple'); len(db)
2
```

## `simstring_rust.searcher.Searcher`

- `Searcher(db, measure)` — search engine over a `HashDb`; `measure` must be
  one of the five measure classes, else `TypeError`.
- `search(query_string: str, alpha: float) -> list[str]` — matches
  (`score >= alpha`) sorted ascending.
- `ranked_search(query_string: str, alpha: float)` → `list[tuple[str, float]]`
  — `(text, score)` pairs, most-to-least similar, ties ascending.

Both release the GIL during the Rust join and raise `SearchError` when
`alpha` fails `0.0 < alpha <= 1.0`. A feature-less query returns an empty
list.

## `simstring_rust.errors.SearchError`

Exception for invalid thresholds; derives from `ValueError`. Raised with the
offending value in the message:

```python
>>> from simstring_rust.errors import SearchError
>>> try:
...     Searcher(db, Cosine()).search('apple', 0.0)
... except SearchError as e:
...     print(e)
Invalid threshold: 0.0
```

## Notes

- Type annotations for the modules are planned in
  [issue #102](https://github.com/PyDataBlog/simstring_rs/issues/102); today
  the package is imported dynamically (extension module) — no hand-written
  stubs yet.
- Rust-side docs for the same engine:
  [docs.rs/simstring_rust](https://docs.rs/simstring_rust).

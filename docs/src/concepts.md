# Concepts

Three cooperating components produce SimString's exact fuzzy retrieval:

1. A **feature extractor** turns any string into a feature set.
2. A **database** stores strings and an inverted index grouped by feature-set
   size.
3. A **searcher** prunes feature-size buckets, joins the remaining posting
   lists, and scores survivors against a **similarity measure** and threshold
   `alpha`.

## Feature extractors

A feature set must never repeat a feature, so every occurrence gets a 1-based
counter suffix: the character bigrams of `"aaa"` are `$a1`, `aa1`, `aa2`,
`a$1`. Extractors work on Unicode characters, not bytes.

| Extractor | Configure | Features of example |
|---|---|---|
| `CharacterNgrams(n, endmarker)` | windows of `n` characters over the text padded with `n - 1` endmarkers on each side | `(2, "$")` on `"hello"`: `$h1`, `he1`, `el1`, `ll1`, `lo1`, `o$1` |
| `WordNgrams(n, splitter, padder)` | windows of `n` tokens, split on `splitter`, padded one `padder` token per side, joined with a single space | `(2, " ", "_")` on `"green apples"`: `_ green1`, `green apples1`, `apples _1` |
| `CustomExtractor(fn)` (Python) | any object with `apply(text) -> Iterable[str]`; occurrence counters are added automatically | see [custom extractors](guides/custom-extractors.md) |

Window sizes trade recall against similarity pressure: bigger `n` means the
query must share longer runs of text before any candidate reaches a given
`alpha`.

Extractors are chosen at database construction and used for each inserted
string **and** for each query — the two sides of every comparison always see
the same feature scheme.

## Databases

`HashDb` is the in-memory implementation of the `Database` trait. It stores
the inserted strings (`strings[i]` is string ID `i`), their per-string sorted
feature lists, and a posting index: for each interned feature and each
feature-set size, the IDs that carry that feature with that exact size. Two
things this design means:

- **Duplicates are kept.** Re-inserted strings get their own IDs and remain
  individually retrievable.
- **Sizes partition the corpus.** Each ID belongs to exactly one feature-size
  bucket; search visits only the buckets its candidates can possibly match.

`clear()` empties strings, postings, and the interner. There is no on-disk
format yet ([issue #9](https://github.com/PyDataBlog/simstring_rs/issues/9));
build a database per process.

## Search

The engine runs the CPMerge candidate generation with a ScanCount-style
overlap join:

1. Extract the query's features (unknown-to-corpus ones count in
   denominators, never in overlaps).
2. Ask the measure for the compression of viable feature-set sizes and the
   smallest viable intersection for each size.
3. Join each remaining size's posting lists, shortest first, counting
   how many query features each candidate string shares.
4. Score survivors exactly (`similarity_from_counts >= alpha`) so rounding at
   a bound can never discard a borderline match.

Results are deterministic: `search()` returns strings sorted ascending;
`ranked_search()` returns `(text, score)` pairs, most-to-least similar with
ascending text as the tie-break. A query whose feature set is empty matches
nothing, and `alpha` must be in `(0.0, 1.0]`.

## Similarity measures

`x`, `y` are the two feature-set sizes, `c` the size of their intersection.

| Measure | Formula | Notes |
|---|---|---|
| `Cosine` | `c / sqrt(x * y)` | empty side scores `0.0` |
| `Dice` | `2 * c / (x + y)` | two empty sets score `1.0` |
| `Jaccard` | `c / (x + y - c)` intersection over union | two empty sets score `1.0` |
| `Overlap` | `c / min(x, y)` | superset of the smaller set scores `1.0` |
| `ExactMatch` | `1.0` iff both feature sets are identical | graded scoring off |

Rounding matters for the pruning bounds, not the verdict: the join
conservatively widens min/max feature sizes and the minimum common feature
count, then the exact score check decides inclusion. Custom measures must
preserve that invariant; see [`Measure`
docs](https://docs.rs/simstring_rust) for the details.

## Threshold intuition

`alpha` is a fraction of the maximum achievable similarity under the chosen
measure. Example over `{"hello", "help", "world"}` with character bigrams and
`Cosine`, query `"hell"`: `hello` scores 0.7303 and `help` scores 0.6000, so
`alpha = 0.5` returns both, while `alpha = 0.8` returns nothing because no
string reaches 0.8. Practical starting points:

| Look for | Starting point |
|---|---|
| Strict, near-duplicate names | 0.85–1.0 with `Dice` or `Cosine` |
| Moderate fuzzy matching | 0.7–0.85 |
| Loose "related" matches | 0.6–0.7 |
| Containment / substring patterns | `Overlap` with a high alpha (0.9+) |

Higher window sizes (`n`) behave like lower alphas: results get stricter, so
bump `n` before hacking `alpha` towards `1.0` if query traffic is noisy.

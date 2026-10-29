# FAQ

## General

**Is retrieval really exact?**
Yes — subject only to the chosen threshold. CPMerge's size-based pruning plus
outward-rounded measure bounds guarantee every string with
`similarity >= alpha` is returned; no extra strings can pass the final exact
score check without meeting the threshold.

**Where do files and other storage engines fit?**
`HashDb` can save and load versioned snapshots (`save_to_file` /
`load_from_file`). Loading a raw corpus file directly and alternative storage
engines are not built in yet ([issue
#9](https://github.com/PyDataBlog/simstring_rs/issues/9) tracks direct file
loading). The `Database` trait exists so you *can* plug a different backend;
queries only need read access to the trait.

**Is there a CLI?**
Not on `main`; see
[issue #51](https://github.com/PyDataBlog/simstring_rs/issues/51).

**Which languages/algorithms are supported?**
Rust 1.89+ for the crate; Python 3.10–3.15 via abi3 wheels. The engine
implements CPMerge (SimString). MeCab tokenization is a future
addition ([issue #44](https://github.com/PyDataBlog/simstring_rs/issues/44)).

## Expected behaviors

**I inserted `"apple"` twice — will it appear twice in results?**
Yes. Duplicates are distinct entries with distinct IDs.

**Why does my query match nothing at all?**
Three common causes: (1) the query has *no extractable features* under the
database's extractor (e.g. empty query or `n = 0`); (2) the threshold is
simply higher than any string's score; (3) the corpus is empty. Diagnose by
comparing `extractor.apply(query)` (Python) to a `db.strings()` entry's
features, and lower `alpha`.

**`alpha = 0.0` errors out — is that intentional?**
Yes: `SearchError::InvalidThreshold` guards a threshold that would match the
whole corpus (or worse, below it). Use a small positive alpha like `0.1`
instead.

**Does search mutate the database?**
No. Searches only read. Rust's borrows reject inserted-while-searching races;
Python threads achieve parallel queries thanks to GIL release during the
join.

**How do unknown query words behave?**
They count against the query's denominator but contribute no overlap: scores
for queries with vocabulary outside the corpus are lower, sometimes below
threshold. Consider extracting more features of the query, or raising `n`.

## Feature and measure questions

**Why do reappearing n-grams get numbers appended (`aa1`, `aa2`)?**
Feature sets can't repeat features; the suffix keeps repeated features
distinguishable so scores reflect real occurrences. The same qualification
rule must apply to custom extractors (see the
[contract](guides/custom-extractors.md#the-contract)).

**WordNgrams dropped my empty tokens — is that a bug?**
Current behavior splits on `splitter` and discards empty tokens; a
[known issue](https://github.com/PyDataBlog/simstring_rs/issues/41) tracks
whether those tokens should be preserved. CharacterNgrams is unaffected.

**Which measure should I start with?**
`Cosine` for a general balance; `Dice`
for the classic SimString behavior; `Overlap` for containment-style lookups;
`Jaccard` when only strongly similar results matter;
`ExactMatch` for memberships. Details:
[choosing measures](guides/measures.md).

**Does the engine care about Unicode?**
Extractors work in characters (not bytes), so languages and emoji behave as
their characters indicate: `"héllo"` with character bigrams `2` yields six
distinct grams including `hé1`.

## Python

**Are there type stubs?**
Not yet — see
[issue #102](https://github.com/PyDataBlog/simstring_rs/issues/102).

**Why does `help(...)` look blank for some items?**
It shouldn't be: every public item in the current source tree has a docstring.
If a released wheel shows a blank `help` entry, report it with the wheel
version — it's a bug.

**Does search release the GIL?**
Yes. `ThreadPoolExecutor`-driven queries run truly in parallel on large
databases ([performance](guides/performance.md)).

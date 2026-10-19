# Rust API reference

The generated per-item reference lives at
[docs.rs/simstring_rust](https://docs.rs/simstring_rust) and is rebuilt at
every stable release, complete with doctested examples. This page is the map
to that reference.

## Crate root re-exports

```rust
use simstring_rust::database::{Database, HashDb};
use simstring_rust::extractors::{CharacterNgrams, FeatureExtractor, WordNgrams};
use simstring_rust::measures::{Cosine, Dice, ExactMatch, Jaccard, Measure, Overlap};
use simstring_rust::search::{SearchError, Searcher};
```

`StringId` (an alias of `usize`) names the per-string index: string `i` on
insert refers to `HashDb.strings[i]` until `clear()`.

## Module map

| Module | Items |
|---|---|
| [`database`](https://docs.rs/simstring_rust/simstring_rust/database/index.html) | `Database` trait (storage contract), `HashDb` (in-memory inverted index) |
| [`extractors`](https://docs.rs/simstring_rust/simstring_rust/extractors/index.html) | `FeatureExtractor` trait, `CharacterNgrams`, `WordNgrams` |
| [`measures`](https://docs.rs/simstring_rust/simstring_rust/measures/index.html) | `Measure` trait plus `Cosine`, `Dice`, `ExactMatch`, `Jaccard`, `Overlap` |
| [`search`](https://docs.rs/simstring_rust/simstring_rust/search/index.html) | `Searcher` over `&dyn Database` and `SearchError::InvalidThreshold` |
| [`python`](https://docs.rs/simstring_rust/simstring_rust/python/index.html) | PyO3 binding layer for the `simstring-rust` PyPI package |

## Typical wiring

```rust
use std::sync::Arc;

use simstring_rust::database::HashDb;
use simstring_rust::extractors::WordNgrams;
use simstring_rust::measures::Dice;
use simstring_rust::Searcher;

let mut db = HashDb::new(Arc::new(WordNgrams::new(2, " ", "_")));
db.insert("green apples".into());
db.insert("green bananas".into());

let searcher = Searcher::new(&db, Dice);
let ranked = searcher.ranked_search("green apples", 0.8)?;
assert_eq!(ranked[0].0, "green apples"); // 1.0 — the query, indexed verbatim
# Ok::<(), simstring_rust::SearchError>(())
```

Three extension points match the components: implement
[`Database`](https://docs.rs/simstring_rust/simstring_rust/database/trait.Database.html)
for alternative storage, [`FeatureExtractor`](https://docs.rs/simstring_rust/simstring_rust/extractors/trait.FeatureExtractor.html)
for different tokenizers ([guide](../guides/custom-extractors.md)), and
[`Measure`](https://docs.rs/simstring_rust/simstring_rust/measures/trait.Measure.html)
for other scoring rules ([guide](../guides/measures.md)). All three need `Send + Sync`.

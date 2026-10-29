# Getting started (Rust)

`simstring_rust` is a normal Rust crate: add it to your `Cargo.toml`, create a
database, insert strings, and search. Rust 1.89+ (edition 2024) is required.

## Install

```toml
[dependencies]
simstring_rust = "0.6"
```

## Index and query

```rust
use std::sync::Arc;

use simstring_rust::database::HashDb;
use simstring_rust::extractors::CharacterNgrams;
use simstring_rust::measures::Cosine;
use simstring_rust::Searcher;

fn main() {
    // 1. Choose how text is tokenized into features, then create the database.
    let mut db = HashDb::new(Arc::new(CharacterNgrams::new(2, "$")));

    // 2. Insert the corpus. Duplicates stay distinct entries.
    db.insert("hello".to_string());
    db.insert("help".to_string());
    db.insert("world".to_string());

    // 3. Search. `alpha` is the minimum similarity a result must reach.
    let searcher = Searcher::new(&db, Cosine);

    let matches = searcher.search("hell", 0.5).unwrap();
    assert_eq!(matches, ["hello", "help"]); // sorted ascending

    let ranked = searcher.ranked_search("hell", 0.5).unwrap();
    assert_eq!(ranked[0].0, "hello");
}
```

Both search methods return `Result<_, SearchError>`: an
`Err(SearchError::InvalidThreshold(alpha))` whenever `alpha` is outside
`0.0 < alpha <= 1.0`.

## Notes

- `HashDb` is `Send + Sync`; searchers are immutable, so any number of them
  can query one database concurrently. Inserting and searching must not
  overlap, which the borrow checker enforces.
- The extractor given to `HashDb::new` is used for queries too: features come
  from the same scheme and the same interner.
- Full signatures: [docs.rs/simstring_rust](https://docs.rs/simstring_rust).
  Corpus-retrieved references such as dup-free insert and clearing behavior are
  in [concepts](../concepts.md); the runnable setup lives in
  [`examples/basic.rs`](https://github.com/PyDataBlog/simstring_rs/blob/main/examples/basic.rs).

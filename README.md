# simstring_rust

[![Build Status](https://github.com/PyDataBlog/simstring_rs/actions/workflows/CI.yml/badge.svg)](https://github.com/PyDataBlog/simstring_rs/actions)
[![Crates.io](https://img.shields.io/crates/v/simstring_rust.svg)](https://crates.io/crates/simstring_rust)
[![Documentation](https://docs.rs/simstring_rust/badge.svg)](https://docs.rs/simstring_rust)
[![Rust](https://img.shields.io/badge/rust-1.63.0%2B-blue.svg?maxAge=3600)](https://github.com/PyDataBlog/simstring_rs)

A native Rust implementation of the CPMerge algorithm, designed for approximate string matching. This crate is particularly useful for natural language processing tasks that require the retrieval of strings/texts from very large corpora (big amounts of texts). Currently, this crate supports both character and word-based N-grams feature generation, with plans to allow custom user-defined feature generation methods.

## Features

- ✅ Fast algorithm for string matching
- ✅ 100% exact retrieval
- ✅ Support for Unicode
- [ ] Support for building databases directly from text files
- [ ] Mecab-based tokenizer support

## Supported String Similarity Measures

- ✅ Dice coefficient
- ✅ Jaccard coefficient
- ✅ Cosine coefficient
- ✅ Overlap coefficient
- ✅ Exact match

## Installation

Add `simstring_rust` to your `Cargo.toml`:

```toml
[dependencies]
simstring_rust = "0.3.0" # change version accordingly
```

For the latest features, you can add the master branch by specifying the Git repository:

```toml
[dependencies]
simstring_rust = { git = "https://github.com/PyDataBlog/simstring_rs.git", branch = "main" }
```

Note: Using the master branch may include experimental features and potential breakages. Use with caution!

To revert to a stable version, ensure your Cargo.toml specifies a specific version number instead of the Git repository.

## Usage

Here is a basic example of how to use simstring_rs in your Rust project:

```Rust
use simstring_rust::database::HashDb;
use simstring_rust::extractors::CharacterNgrams;
use simstring_rust::measures::Cosine;
use simstring_rust::Searcher;

use std::sync::Arc;

fn main() {
    // 1. Setup the database
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
    let mut db = HashDb::new(feature_extractor);

    // 2. Index some strings
    db.insert("hello".to_string());
    db.insert("help".to_string());
    db.insert("halo".to_string());
    db.insert("world".to_string());

    // 3. Search for strings
    let measure = Cosine;
    let searcher = Searcher::new(&db, measure);
    let query = "hell";
    let alpha = 0.5;

    if let Ok(results) = searcher.ranked_search(query, alpha) {
        println!("Found {} results for query '{}'", results.len(), query);
        for (item, score) in results {
            println!("- Match: '{}', Score: {:.4}", item, score);
        }
    }
}
```

<!-- ## Releasing -->
<!----> INFO: Release notes for maintainer(s)
<!-- This project uses [`cargo-release`](https://github.com/crate-ci/cargo-release) and [`git-cliff`](https://github.com/orhun/git-cliff) to automate the release process. -->
<!---->
<!-- _NB_: Ensure local `main` branch is up-to-date: -->
<!---->
<!-- ```bash -->
<!-- git checkout main -->
<!-- git pull origin main -->
<!-- ``` -->
<!---->
<!-- To create a new release, simply run the following command: -->
<!---->
<!-- ```bash -->
<!-- cargo release <LEVEL> # dry-run is default -->
<!-- cargo release <LEVEL> --execute -->
<!-- ``` -->
<!---->
<!-- Replace `<LEVEL>` with the desired release level (`patch`, `minor`, or `major`). For example, to create a patch release: -->
<!---->
<!-- ```bash -->
<!-- cargo release patch -->
<!-- ``` -->
<!---->
<!-- This command will automatically: -->
<!---->
<!-- 1.  Generate and update the `CHANGELOG.md` file. -->
<!-- 2.  Bump the version in `Cargo.toml`. -->
<!-- 3.  Commit the changes. -->
<!-- 4.  Create a new Git tag. -->
<!-- 5.  Push the commit and tag to GitHub. -->
<!---->
<!-- The CI/CD pipeline will then automatically publish the new version to `crates.io`. -->

## Contributing

Contributions are welcome! Please open an issue or submit a pull request on GitHub.
License

This project is licensed under the MIT License.

## Acknowledgements

Inspired by the [SimString](https://www.chokkan.org/software/simstring/) project.

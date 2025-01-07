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
simstring_rust = "0.1.0" # change version accordingly
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
use simstring_rust::database::HashDB;
use simstring_rust::extractors::CharacterNGrams;
use simstring_rust::measures::Cosine;

fn main() {
    let feature_extractor = CharacterNGrams {
        n: 2,
        padder: " ".to_string(),
    };
    let measure = Cosine::new();
    let mut db = HashDB::new(feature_extractor, measure);

    db.insert("hello".to_string());
    db.insert("help".to_string());
    db.insert("halo".to_string());
    db.insert("world".to_string());

    let threshold = 0.5;
    let results = db.search("hell", threshold);

    if results.is_empty() {
        println!("No results found with threshold {}", threshold);
    } else {
        println!("Results with threshold {}:", threshold);
        for result in results {
            println!("Match: '{}' (score: {})", result.value, result.score);
        }
    }
}
```

## Contributing

Contributions are welcome! Please open an issue or submit a pull request on GitHub.
License

This project is licensed under the MIT License.

## Acknowledgements

Inspired by the [SimString.jl](https://github.com/PyDataBlog/SimString.jl) project.

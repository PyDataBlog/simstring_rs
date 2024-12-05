# simstring_rs

A native Rust implementation of the CPMerge algorithm, designed for approximate string matching. This crate is particularly useful for natural language processing tasks that require the retrieval of strings/texts from very large corpora (big amounts of texts). Currently, this crate supports both character and word-based N-grams feature generation, with plans to allow custom user-defined feature generation methods.

## Features

- ✅ Fast algorithm for string matching
- ✅ 100% exact retrieval
- ✅ Support for Unicode
- [ ] Support for building databases directly from text files
- [ ] Mecab-based tokenizer support
- [ ] Support for persistent databases like MongoDB

## Supported String Similarity Measures

- ✅ Dice coefficient
- ✅ Jaccard coefficient
- ✅ Cosine coefficient
- ✅ Overlap coefficient
- ✅ Exact match

## Installation

Add `simstring_rs` to your `Cargo.toml`:

```toml
[dependencies]
simstring_rs = "0.1.0"
```

For the latest features, you can add the master branch by specifying the Git repository:

```toml
[dependencies]
simstring_rs = { git = "https://github.com/PyDataBlog/simstring_rs.git", branch = "main" }
```

Note: Using the master branch may include experimental features and potential breakages. Use with caution!

To revert to a stable version, ensure your Cargo.toml specifies a specific version number instead of the Git repository.

## Usage

Here is a basic example of how to use simstring_rs in your Rust project:

```Rust

```

## Contributing

Contributions are welcome! Please open an issue or submit a pull request on GitHub.
License

This project is licensed under the MIT License.

## Acknowledgements

Inspired by the [SimString.jl](https://github.com/PyDataBlog/SimString.jl) project.

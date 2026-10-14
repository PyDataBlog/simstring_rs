# simstring_rust

[![Build Status](https://github.com/PyDataBlog/simstring_rs/actions/workflows/CI.yml/badge.svg)](https://github.com/PyDataBlog/simstring_rs/actions)
[![Crates.io](https://img.shields.io/crates/v/simstring_rust.svg)](https://crates.io/crates/simstring_rust)
[![PyPI version](https://badge.fury.io/py/simstring-rust.svg)](https://badge.fury.io/py/simstring-rust)
[![Python versions](https://img.shields.io/badge/python-3.10%E2%80%933.15-blue.svg)](pyproject.toml)
[![Documentation](https://docs.rs/simstring_rust/badge.svg)](https://docs.rs/simstring_rust)
[![Rust](https://img.shields.io/badge/rust-1.88%2B-blue.svg?maxAge=3600)](https://github.com/PyDataBlog/simstring_rs)
[![Codecov](https://img.shields.io/codecov/c/github/PyDataBlog/simstring_rs?token=XJM8O8TD4U)](https://codecov.io/gh/PyDataBlog/simstring_rs)

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

Requires Rust 1.88 or newer (edition 2024). Python bindings require Python 3.10+
and are tested on Python 3.10–3.15.
See [Rust API migration](#rust-api-migration) for the read-only query engine's trait changes.

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

## Releasing

[Release Please](https://github.com/googleapis/release-please) manages stable
releases from `main`. Its Rust strategy updates `Cargo.toml`, `Cargo.lock`,
`CHANGELOG.md`, and `.release-please-manifest.json` in a release PR. Maturin derives
the Python package version from Cargo, so crates.io and PyPI share one version.

### Release process

1. Squash-merge changes into `main` using Conventional Commit PR titles. After CI
   passes, Release Please creates or updates the pending release PR.
2. Review the proposed version and release notes after the release PR passes CI.
   **Merging this PR is the approval to release.**
3. After CI passes, Release Please creates the `vMAJOR.MINOR.PATCH` tag and GitHub
   release. The shared [build/test workflow](.github/workflows/test.yml) runs against
   that exact release commit, producing separately named release wheel artifacts.
4. The crate is verified and published to crates.io, followed by the tested release
   wheels to PyPI. TestPyPI receives the same wheels independently of production
   publishing. Publication is automatic after the release tests pass.

Ordinary merges update the proposed release and run CI. Registry publishing is
driven by Release Please's `release_created` output.

The [configuration](release-please-config.json) and manifest start from
`v0.3.5-beta.1`. The `prerelease` versioning strategy with `prerelease: false`
promotes beta/RC versions to stable versions. With the pending feature changes,
the first release is **`0.4.0`**. Subsequent versions follow Conventional Commits:
`fix` and `perf` bump the patch, `feat` bumps the minor, and breaking changes bump
the minor while the project is below `1.0.0`. Include the
[Rust API migration](#rust-api-migration) when reviewing the first release notes.

### Repository setup

- Install a release GitHub App on this repository with **Contents**, **Issues**,
  and **Pull requests** read/write permissions. Add its App ID and private key as
  repository secrets `RELEASE_APP_ID` and `RELEASE_APP_PRIVATE_KEY`. The App token
  allows CI to run on the release PR that it creates.
- Configure `main` to require the **CI** and **PR title** checks before merging.
  Enable squash merging and use the PR title as the squash commit title so Release
  Please can determine the version from the merged history.
- Configure trusted publishers for `PyDataBlog/simstring_rs`, workflow **`CI.yml`**,
  and environment **`testpypi`** on TestPyPI / **`pypi`** on PyPI. These environments
  use automatic publishing, with no required reviewers.
- Provide `CARGO_REGISTRY_TOKEN` with permission to publish `simstring_rust`.

### Retrying publication

Use **Re-run failed jobs** on the original release workflow run. This reuses Release
Please's outputs and the successful build artifacts. Already-published crate
versions and wheel files are skipped. Check the publishing jobs to confirm that
the packages are available in their registries.

## Rust API migration

Version `0.4.0` introduces breaking changes for custom Rust trait implementations.
The read-only query engine preserves the usual `HashDb` and `Searcher` calls,
as well as Python method signatures. Custom Rust implementations need these updates:

- **FeatureExtractor**: implement
  `visit_features(&self, text: &str, visitor: &mut dyn FnMut(&str))`, emitting
  occurrence-qualified features. The default `features` method interns, sorts,
  and deduplicates them for insertion; search visits them without interning.
- **Database**: `interner()` returns `&Rodeo` rather than a locked `Arc`.
  `lookup_strings()` returns sorted, unique `Option<&[StringId]>` postings, and
  `get_features()` returns `Option<&[Spur]>`. Implement `feature_sizes()` as a
  sorted, unique slice of populated sizes. Each ID belongs to exactly one size
  bucket; stored features must be sorted and unique, using the same interner.
- **Measure**: implement `similarity_from_counts(x_len, y_len, common) -> f64`.
  The default `similarity` intersects sorted feature sets and delegates to it.
  Search uses counts directly, so this method must agree with the pruning bounds.

Unknown query features still contribute to similarity denominators. Duplicate
insertions remain distinct results, and queries with no features return no results.
Both search methods now apply exact score filtering at threshold boundaries.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request on GitHub.

Use [Conventional Commits](https://www.conventionalcommits.org/) for PR titles,
for example `fix(search): handle empty queries` or `feat(bindings): add an extractor`.
Mark breaking changes with `!`, such as `refactor(api)!: change the Database trait`,
and describe the migration in the PR. PR titles are checked automatically and
become the squash commit titles used by Release Please.

License

This project is licensed under the MIT License.

## Benchmarks

The `benches/run_benches.py` harness compares several language bindings (Rust, Python, Julia, Ruby, C++).

- `git`, `autoconf`, `automake`, `libtool`, `make`, `python`, `uv` and a C++ compiler (`g++`) to build the C++ CLI.

The C++ sources are cloned into `benches/.simstring_cpp/` and a local copy of the `simstring` binary is installed
under that directory. If you need to rebuild from scratch, remove `benches/.simstring_cpp/` before re-running the benchmark suite.

## Acknowledgements

Inspired by the [SimString](https://www.chokkan.org/software/simstring/) project.

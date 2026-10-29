# Contributing

Thanks for helping improve `simstring_rust`! This guide covers the local
setup, the commands CI runs, and the release process. For usage questions see
the [project guide](https://pydatablog.github.io/simstring_rs/) or open an
[issue](https://github.com/PyDataBlog/simstring_rs/issues).

Coding agents should start with [AGENTS.md](AGENTS.md) for repository layout,
behavioral contracts, acceptance criteria, and validation steps.

## Prerequisites

- Rust toolchain 1.89+ (edition 2024), via [rustup](https://rustup.rs/)
- Python 3.10–3.15 and [uv](https://docs.astral.sh/uv/) for the bindings
- [maturin](https://www.maturin.rs/) to build the Python extension module
- mdBook 0.5+ if you work on this documentation site (`cargo install mdbook`)

## Local setup

```bash
git clone https://github.com/PyDataBlog/simstring_rs
cd simstring_rs

# Rust side
cargo test

# Python side
uv venv
source .venv/bin/activate   # Windows: .venv\Scripts\activate
uv pip install maturin pytest
maturin develop
pytest tests/python/ -vv
```

## What CI runs

Every PR is checked with (see [`.github/workflows/test.yml`](.github/workflows/test.yml)):

| Command | Purpose |
|---|---|
| `cargo fmt --all -- --check` | formatting |
| `cargo clippy -- -D warnings` | lints (doc comments included via `missing_docs`) |
| `cargo test` | unit, integration, and doc tests |
| `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items` | docs build, broken links, missing docs |
| `maturin build --release --locked` + `pytest tests/python/ -vv` | Python bindings |
| `cargo run --example basic --release`, `cargo run --example persistence --release`, and the Python examples | examples still run |

Please run `cargo fmt --all`, `cargo test`, and the Python suite before
pushing. Benchmarks cost extra CI time; they run separately on the `benchmark`
job and do not gate ordinary changes beyond the summary they produce.

## Docs site

The [guide](https://pydatablog.github.io/simstring_rs/) builds from
`docs/src/*.md` via mdBook (`book.toml` at the repo root):

```bash
mdbook serve    # live reload at http://localhost:3000
mdbook build    # output to ./book (gitignored)
```

The `.github/workflows/docs.yml` workflow builds the site on every PR that
touches `docs/**` or `book.toml`, and deploys it to GitHub Pages on merges to
`main`. Code snippets in the guide should mirror the doctests in `src/` so
`cargo test --doc` keeps them honest.

## Repository layout

```
src/lib.rs                crate root: re-exports and the user-facing overview
src/database/             Database trait + HashDb (inverted index)
src/extractors/           FeatureExtractor trait, CharacterNgrams, WordNgrams
src/measures/             Measure trait + Cosine, Dice, Jaccard, Overlap, ExactMatch
src/search.rs             Searcher (CPMerge/ScanCount join)
src/python/mod.rs         PyO3 bindings for the simstring-rust PyPI package
tests/                    Rust integration tests (tests/python/ are pytest)
benches/                  Criterion benches + cross-language harness
docs/src/                 mdBook sources for the project guide
```

## Commit and PR conventions

PR titles use [Conventional Commits](https://www.conventionalcommits.org/),
e.g. `fix(search): handle empty queries` or `feat(bindings): add an
extractor`. Mark breaking changes with `!` (as in
`refactor(api)!: change the Database trait`) and describe the migration in the
PR. Titles are checked automatically (see
[`.github/workflows/pr-title.yml`](.github/workflows/pr-title.yml)) and become
the squash commit titles that Release Please consumes. Keep the PR body short:
what changed, why, and how you validated it.

## Releasing

[Release Please](https://github.com/googleapis/release-please) manages stable
releases from `main`. Its Rust strategy updates `Cargo.toml`, `Cargo.lock`,
`CHANGELOG.md`, and `.release-please-manifest.json` in a release PR. Maturin
derives the Python package version from Cargo, so crates.io and PyPI share one
version.

### Release process

1. Squash-merge changes into `main` using Conventional Commit PR titles. After
   CI passes, Release Please creates or updates the pending release PR.
2. Review the proposed version and release notes after the release PR passes
   CI. **Merging this PR is the approval to release.**
3. After CI passes, Release Please creates the `vMAJOR.MINOR.PATCH` tag and
   GitHub release. The shared [build/test workflow](.github/workflows/test.yml)
   runs against that exact release commit, producing separately named release
   wheel artifacts.
4. The crate is verified and published to crates.io, followed by the tested
   release wheels to PyPI. TestPyPI receives the same wheels independently of
   production publishing. Publication is automatic after the release tests
   pass.

Ordinary merges update the proposed release and run CI. Registry publishing is
driven by Release Please's `release_created` output.

The [configuration](release-please-config.json) and
[manifest](.release-please-manifest.json) drive release versioning from the
latest release. The `prerelease` versioning strategy with `prerelease: false`
promotes beta/RC versions to stable versions. Versions follow Conventional
Commits: `fix` and `perf` bump the patch, `feat` bumps the minor, and breaking
changes bump the minor while the project is below `1.0.0`. Include migration
notes when a release contains breaking API changes.

### Repository setup (maintainers only)

- Install a release GitHub App on this repository with **Contents**,
  **Issues**, and **Pull requests** read/write permissions. Add its App ID and
  private key as repository secrets `RELEASE_APP_ID` and
  `RELEASE_APP_PRIVATE_KEY`. The App token allows CI to run on the release PR
  that it creates.
- Configure `main` to require the **CI** and **PR title** checks before
  merging. Enable squash merging and use the PR title as the squash commit
  title so Release Please can determine the version from the merged history.
- Configure trusted publishers for `PyDataBlog/simstring_rs`, workflow
  **`CI.yml`**, and environment **`testpypi`** on TestPyPI / **`pypi`** on
  PyPI. These environments use automatic publishing, with no required
  reviewers.
- Provide `CARGO_REGISTRY_TOKEN` with permission to publish `simstring_rust`.

### Retrying publication

Use **Re-run failed jobs** on the original release workflow run. This reuses
Release Please's outputs and the successful build artifacts. Already-published
crate versions and wheel files are skipped. Check the publishing jobs to
confirm that the packages are available in their registries.

## Benchmarks

`benches/run_benches.py` compares several language bindings (Rust, Python,
Julia, Ruby, C++). To run it locally you need `git`, `autoconf`, `automake`,
`libtool`, `make`, `python`, `uv`, and a C++ compiler (`g++`) to build the C++
CLI. The C++ sources are cloned into `benches/.simstring_cpp/`, and a local
copy of the `simstring` binary is installed under that directory — remove
`benches/.simstring_cpp/` to force a rebuild. Rust-specific benches:

```bash
cargo bench --bench bench    # end-to-end corpus search
cargo bench --bench micro    # search micro benchmarks (published numbers)
cargo bench --bench insert   # insertion throughput
```

The CI job summary and benchmark PR comment also report Rust and Python
database build, save, and load medians, snapshot sizes, and sample counts for
both corpora with 2-, 3-, and 4-grams. Run
`cargo bench --bench persistence -- --json` (Rust) or
`python benches/bench_persistence.py` (Python bindings) for the persistence
measurements in machine-readable form.

CI also checks insert and unranked Cosine search against the latest stable
published crate, rebuilding release and PR sources with matching settings for
Rust and Python. It uses company names, character/word 2–4 grams, and
thresholds 0.6/0.9. After three paired runs, cases slower by more than 5% are
rechecked: confirmed >5% warns and confirmed >10% fails. The median and every
recheck pair must exceed the threshold. Correctness mismatches and benchmark
errors fail immediately. Raw samples and build metadata are uploaded as the
`release-regression` artifact, with results in the existing benchmark comment.
Run `.venv/bin/python benches/check_release.py --baseline 0.6.0` locally;
Maturin must be installed, and each run needs a fresh `--output` directory.

## Getting help

- [Issue tracker](https://github.com/PyDataBlog/simstring_rs/issues)
- [Project guide](https://pydatablog.github.io/simstring_rs/)
- [Rust docs](https://docs.rs/simstring_rust)

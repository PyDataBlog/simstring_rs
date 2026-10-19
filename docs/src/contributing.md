# Contributing

Short form here; the full local setup, CI checks, release and benchmarking
procedures live in
[CONTRIBUTING.md](https://github.com/PyDataBlog/simstring_rs/blob/main/CONTRIBUTING.md).

## Fixing or extending docs content

This whole site builds from `docs/src/*.md` with mdBook; `book.toml` sits at
the repository root:

```bash
cargo install mdbook
mdbook serve    # live preview at http://localhost:3000
mdbook build    # output to ./book, gitignored
```

Keep numbers, snippets, and outcome claims in sync with the source of truth:

- Code snippets change → mirror the doctests in `src/` (they run in `cargo
  test --doc`).
- Performance numbers → `cargo bench --bench micro`, table in
  [BENCHMARKS.md](https://github.com/PyDataBlog/simstring_rs/blob/main/BENCHMARKS.md).
- Python API tables → the `#[pymethods]` docstrings in `src/python/mod.rs`
  (guarded by `tests/python/test_docstrings.py`).
- CI runs a docs build on every PR touching `docs/**` or `book.toml`; deploy
  to GitHub Pages happens on `main`.

## Issues and PRs

- Search existing [issues](https://github.com/PyDataBlog/simstring_rs/issues)
  before opening a new one.
- PR titles follow
  [Conventional Commits](https://www.conventionalcommits.org/), e.g.
  `fix(search): handle empty queries` or `feat(bindings): add an extractor`;
  breaking changes get `!` and a migration paragraph.
- Docs-only changes may be merged faster than code changes as CI only
  rebuilds the site.

## Standards

- Every public Rust item carries a doc comment (`#![warn(missing_docs)]`
  plus `-D warnings` in CI enforce it).
- Rust code runs through rustfmt and clippy before merging; Python-binding
  changes should keep `pytest tests/python/` green.

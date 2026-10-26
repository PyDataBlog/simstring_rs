# Agent guidelines

This file is the entry point for coding agents working in this repository. It describes
how to scope changes, preserve existing behavior, and validate work.

For user-facing usage, release details, and API migration notes, use [README.md](README.md).
`Cargo.toml`, `pyproject.toml`, `release-please-config.json`, and
`.release-please-manifest.json` are authoritative for release versioning and toolchain
settings; the workflows in `.github/workflows/` are authoritative for CI checks.

## Keeping this file current

Code, tests, manifests, and workflows are authoritative. If this file disagrees with
them, update this file in the same PR. Update it when public behavior, validation
commands, key paths, or the release/CI process changes. Prefer stable invariants and
links over copied version-specific details. Keep this file focused on workflow and
contracts; detailed API and release documentation belongs in README or rustdoc.

## Project

`simstring_rust` is a native Rust implementation of the SimString/CPMerge approximate
string-matching algorithm. PyO3 exposes the same functionality to Python, and Maturin
builds the Python package. The public Rust and Python APIs must remain aligned.

## Key paths (non-exhaustive)

| Path | Contents |
| --- | --- |
| `src/database/` | `Database` trait, `HashDb`, and snapshot persistence |
| `src/extractors/` | `FeatureExtractor`, `CharacterNgrams`, and `WordNgrams` |
| `src/measures/` | `Measure` plus the Cosine, Dice, Jaccard, Overlap, and Exact Match measures |
| `src/search.rs` | `Searcher`, candidate generation, overlap counting, and `SearchError` |
| `src/python/mod.rs` | PyO3 classes and module layout |
| `tests/test_search_reference.rs` | Brute-force reference checks for search and all measures |
| `tests/test_search_corpus.rs` | Search checks against the benchmark corpus |
| `tests/` | Focused behavior and snapshot tests |
| `tests/python/` | Python binding tests |
| `tests/fixtures/` | Versioned snapshot compatibility fixtures |
| `tests/test_python_integration.rs` | Ignored helper that builds a wheel and runs `tests/python/` |
| `examples/` | Basic and persistence examples for Rust and Python |
| `benches/` | Rust benchmarks, persistence and release-regression checks, corpus data, and the cross-language harness |
| `.github/workflows/` | CI, wheel builds, coverage, release, and PR-title validation |

## Ground rules

- Read the relevant README section and tests before changing behavior.
- Check `git status` first and leave unrelated modifications untouched.
- Keep the diff focused on the issue or PR acceptance criteria. Do not reformat
  unrelated files or upgrade dependencies as a side effect.
- Do not weaken, skip, or delete tests to make a change pass. Add regression coverage
  for every fixed defect.
- Do not hand-edit release versions, `CHANGELOG.md`, or
  `.release-please-manifest.json`; Release Please owns them. Let Cargo update
  `Cargo.lock` when dependencies change, and do not edit `BENCHMARKS.md` because CI
  generates it.
- Treat `tests/fixtures/persistence_v1_*.hex` as compatibility contracts. Do not
  regenerate them to make a test pass; a format change needs a new version and fixtures.
- If the required behavior is ambiguous and cannot be resolved from the README, tests,
  or issue, state the assumption or ask instead of inventing public behavior.
- Never claim a command or test passed unless it was run successfully.

## Feature workflow

### 1. Define acceptance criteria

Translate the request into a short, observable "Done when" checklist, normally two to
five items. Each criterion should name the input or operation and the expected result,
including important errors and edge cases. Record material non-goals when they help keep
the change scoped.

Acceptance criteria describe behavior, not implementation steps. Include compatibility
work only when the change affects it: Rust trait implementors, Python callers, the public
package API, or the documented migration path. A lightweight issue can be clarified
before implementation; users are not expected to write a full specification.

### 2. Add tests with the behavior

Add or update tests that fail without the intended change and map directly to the
acceptance criteria. Extend the reference and corpus tests for search or measure changes.
Add Python tests for behavior exposed through the bindings. Prefer deterministic,
small examples unless the invariant needs a larger corpus.

### 3. Implement the smallest complete change

Keep public interfaces coherent across Rust and Python. Update documentation and examples
when behavior, signatures, or supported usage changes. Avoid speculative abstractions and
unrelated optimization.

### 4. Validate

Run the checks relevant to the change and the standard Rust checks below.
For user-visible Python changes, build a fresh wheel and run the Python suite; plain
`cargo test` does not run it because `tests/test_python_integration.rs` is ignored by
default.

### 5. Report

Summarize what changed, map the result to the acceptance criteria, list the validation
commands and outcomes, and call out checks that were not run or behavior that remains
unverified. Mention any compatibility or migration impact.

## Behavioral contracts

These are existing expectations, not optional suggestions. Update them deliberately and
document any intended change.

- **Search completeness and ordering:** return every indexed string whose score is at
  least the threshold. `ranked_search` sorts by descending score, then ascending text;
  `search` sorts results alphabetically.
- **Exact scores and boundaries:** scores must agree with the measure's
  `similarity_from_counts` calculation, including results exactly at the threshold.
  Floating-point pruning must not discard a boundary match.
- **Thresholds:** valid thresholds are greater than `0.0` and at most `1.0`. Zero,
  negative values, values above `1.0`, NaN, and infinity return
  `SearchError::InvalidThreshold`; Python exposes this as `SearchError`.
- **Duplicates and unknown features:** every insertion remains a distinct result.
  Unknown query features contribute to the similarity denominator but cannot match.
  A query with no extracted features returns no results.
- **Immutable queries:** searching must not insert strings, intern unknown features, or
  otherwise mutate the database.
- **Unicode and empty input:** character n-grams operate on Unicode scalar values.
  Empty strings may be indexed and searched according to extractor behavior.
- **Database invariants:** posting IDs and populated feature sizes are sorted and unique;
  each string ID belongs to exactly one feature-size bucket; stored features are sorted,
  unique, and use the database's interner.
- **Snapshots:** save/load round-trips preserve strings, duplicates, insertion order,
  extractor configuration, and search results. Saving requires an existing directory and
  a single file name, and replaces an existing file atomically. Built-in extractors are
  supported; Rust custom extractors must opt in with `persistence_config`, and Python
  `CustomExtractor` objects are unsupported. Unsupported extractors fail without
  modifying the destination. Snapshot compatibility is versioned; the current format is
  defined by `src/database/hashdb/persistence.rs` and the fixtures in `tests/fixtures/`.
  Do not change it without a new version and compatibility fixtures.
- **Custom extractors:** features are occurrence-qualified (the same feature emitted twice
  becomes two distinct features). Rust implementations provide `visit_features`; Python
  callables provide `apply(text) -> Iterable[str]`.
- **Concurrency:** `Database`, `FeatureExtractor`, and `Measure` implementations are
  `Send + Sync`. Concurrent searches and clear-then-reinsert workflows must not race or
  share mutable scratch state incorrectly.

The strongest regression checks for these contracts are
`tests/test_search_reference.rs` and `tests/test_search_corpus.rs`.

## Validation commands

Run these from the repository root. CI also checks every supported OS and target. If
these local commands differ from `.github/workflows/test.yml`, the workflow wins; update
this file when the workflow changes.

```bash
cargo fmt --all -- --check
cargo clippy -- -D warnings
cargo test
cargo doc --no-deps --document-private-items
cargo run --example basic --release
cargo run --example persistence --release
```

For Python-facing changes, use a clean virtual environment and test the installed wheel:

```bash
python -m pip install uv
uv venv --clear
source .venv/bin/activate        # Windows: .venv\Scripts\activate
uv pip install maturin pytest
rm -rf target/wheels
maturin build --release --locked --interpreter python
uv pip install target/wheels/*.whl
pytest tests/python/ -vv
```

For performance-sensitive changes, compare before and after with the relevant benchmark:

```bash
cargo bench --bench micro
cargo bench --bench persistence -- unigrams
```

Report the relevant benchmark conditions and results. Do not edit `BENCHMARKS.md`; the CI
workflow regenerates it. The complete cross-language harness is
`python benches/run_benches.py` and is not required for routine changes. CI also compares
insert and search performance against the latest stable release; reproduce a release
check with `.venv/bin/python benches/check_release.py --baseline <version>` using a fresh
output directory and an environment with Maturin installed.

## Change-specific guidance

| Change | Also update |
| --- | --- |
| Public Rust API or trait contract | README usage/migration notes, Rust docs, examples, and relevant trait tests |
| Python API or binding behavior | `src/python/mod.rs`, `tests/python/`, `examples/python_basic.py`, and README when public |
| Search, measure, or database behavior | Reference/corpus tests and any affected invariants above |
| Snapshot format or persistence behavior | `tests/test_persistence.rs`, `tests/python/test_persistence.py`, versioned fixtures, README save/load docs, and persistence examples |
| Performance work | Reproducible benchmark comparison, release regression check when relevant, and no regressions in correctness tests |
| Release or CI configuration | Explain the need and preserve the existing Release Please and TestPyPI/PyPI flow |

PR titles follow [Conventional Commits](https://www.conventionalcommits.org/) because CI
validates them and Release Please derives versions from them. Mark breaking changes with
`!` and describe the migration. Do not bump versions or update release metadata manually.

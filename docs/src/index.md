# simstring_rust

A native Rust implementation of the [SimString](https://www.chokkan.org/software/simstring/)
algorithm (Okazaki & Tsujii, 2010), with Python bindings shipped as the
[`simstring-rust`](https://pypi.org/project/simstring-rust/) PyPI package.

SimString answers threshold queries over a corpus: *which strings are at
least `alpha` similar to this query?* Thanks to the CPMerge algorithm, the
answer is **exact** — every string above the threshold is returned, and none
are fabricated — while remaining fast enough for corpora with hundreds of
thousands to millions of entries.

```python
from simstring_rust.database import HashDb
from simstring_rust.extractors import CharacterNgrams
from simstring_rust.measures import Cosine
from simstring_rust.searcher import Searcher

db = HashDb(CharacterNgrams(2, "$"))
db.insert("apple")

Searcher(db, Cosine()).search("appel", 0.8)
# ['apple']
```

The crate is maintained on [GitHub](https://github.com/PyDataBlog/simstring_rs).
Content pages of this guide:

- [Getting started — Rust](getting-started/rust.md)
- [Getting started — Python](getting-started/python.md)
- [Concepts](concepts.md)
- [Choosing measures and thresholds](guides/measures.md)
- [Custom feature extractors](guides/custom-extractors.md)
- [Performance and concurrency](guides/performance.md)
- [Rust API reference](api/rust.md) and [Python API reference](api/python.md)
- [Migration](migration.md) and the [FAQ](faq.md)

Other places worth knowing:

- [Rust API generated docs](https://docs.rs/simstring_rust) (docs.rs, per release)
- [Benchmarks](https://github.com/PyDataBlog/simstring_rs/blob/main/BENCHMARKS.md)
- [Changelog](https://github.com/PyDataBlog/simstring_rs/blob/main/CHANGELOG.md)
- [Contributing](https://github.com/PyDataBlog/simstring_rs/blob/main/CONTRIBUTING.md)

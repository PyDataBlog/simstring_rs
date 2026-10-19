# Writing a custom feature extractor

Extractor choice defines what "similarity" means. The two built-ins cover char
and word n-grams; **custom** extractors cover everything else — lowercased
matching, fields at fixed positions in `AAA|BBB|CCC` records, phonetic keys,
domain tokens, etc.

## The contract

Implement the [`FeatureExtractor` trait](https://docs.rs/simstring_rust/trait.FeatureExtractor.html)
(exactly one required method):

```rust
use std::collections::HashMap;
use std::sync::Arc;

use simstring_rust::database::HashDb;
use simstring_rust::measures::Overlap;
use simstring_rust::{FeatureExtractor, Searcher};

// Lowercased words, qualified by how often they appear, e.g. "hello1".
struct LowerWords;

impl FeatureExtractor for LowerWords {
    fn visit_features(&self, text: &str, visitor: &mut dyn FnMut(&str)) {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for word in text.split_whitespace() {
            let lowered = word.to_lowercase();
            let counter = counts.entry(lowered.clone()).or_insert(0);
            *counter += 1;
            // The feature string is only borrowed for the callback call.
            visitor(&format!("{lowered}{counter}"));
        }
    }
}

let mut db = HashDb::new(Arc::new(LowerWords));
db.insert("Hello world".to_owned());
db.insert("Well hello again WORLD, hello".to_owned());

// The feature "hello1" is present in both strings: Overlap finds them.
let scorer = Searcher::new(&db, Overlap);
assert_eq!(
    scorer.search("hello", 0.99)?,
    ["Hello world", "Well hello again WORLD, hello"]
);
# Ok::<(), simstring_rust::SearchError>(())
```

Rules each implementation must follow:

1. **Occurrence-qualified features.** Every feature must be unique *within one
   text*. Qualify repeated features with a 1-based occurrence suffix — the
   built-ins append the number directly (`aa1`, `aa2`). Without it, two
   repetitions would collapse into one feature and skew the geometry.
2. **Borrowed features are short-lived.** Strings passed to the visitor are
   valid only inside the callback; reuse an internal buffer to avoid one
   allocation per feature.
3. **Deterministic per text.** `visit_features` is called for insertion *and*
   for every query against the same interner, so the same input must always
   emit the same feature list.
4. **Deterministic interning.** `features(text, interner)` (the default
   implementation) interns, sorts, and dedupes whatever `visit_features`
   produced; the sort/dedup relies on the occurrences being unique.
5. **Bound your work.** A feature count in the hundreds per string is normal;
   avoid anything quadratic per call.
6. **`Send + Sync`.** Extractors are shared across parallel search threads.

You may override `feature_count_hint` to pre-size storage (an under-estimate
costs a reallocation; the default `0` is always safe).

## In Python: `CustomExtractor`

The `CustomExtractor` class wraps any Python object with an
`apply(text) -> Iterable[str]` method — adding the occurrence counters
automatically, so the Python-side callable may return bare, repeated features:

```python
from simstring_rust.database import HashDb
from simstring_rust.extractors import CustomExtractor, CharacterNgrams
from simstring_rust.measures import Cosine
from simstring_rust.searcher import Searcher

class LowerBigrams:
    def apply(self, text):
        tokens = text.lower().split()
        return [f"{left}|{right}" for left, right in zip(tokens, tokens[1:])]

extractor = CharacterNgrams(2, "$")           # base corpus features
custom = CustomExtractor(LowerBigrams())      # or drive HashDb with it
```

Pass the `CustomExtractor` into `HashDb(...)` instead of `CharacterNgrams` and
every insert/query will use it. Errors raised inside `apply` propagate to the
call site (a search panicking at the Rust boundary prints the Python-side
traceback and re-raises).

## When extractors change

The feature scheme is baked into the index at insertion time: to switch
extractors you build a new database and re-insert the corpus. The interner is
per-database, so feature IDs never leak across schemes.

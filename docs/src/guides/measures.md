# Choosing a similarity measure and threshold

All five measures compare two *feature* sets — every extractor turns `"apple"`
into a set like {$a1, ap1, pp1, pl1, le1, e$1}, and measures score the
intersection of the query's set against each candidate's set. With
`x`, `y` for the two set sizes and `c` for their intersection:

## The measures

| Measure | Similarity | Character |
|---|---|---|
| `Cosine` | `c / sqrt(x * y)` | balanced; the go-to default |
| `Dice` | `2 * c / (x + y)` | the original SimString default |
| `Jaccard` | `c / (x + y - c)` | strictest; punishes extra features hardest |
| `Overlap` | `c / min(x, y)` | most permissive; full containment scores 1.0 |
| `ExactMatch` | `1.0` iff x == y and c == x | membership probe, no fuzzy scoring |

All scored results satisfy `0.0 <= score <= 1.0`. The built-ins differ on
empty inputs: `Dice`, `Jaccard`, and `Overlap` call two empty sets `1.0`,
while `Cosine` scores any empty side `0.0`. In practice a query without
features matches nothing, so those conventions only surface when calling
`Measure::similarity` directly (Rust).

## Rule-of-thumb triage

- **Typo tolerance over a clean corpus** (search bars, deduplication of
  company names): character `n = 2` or `3` plus `Cosine`, alpha 0.7–0.9.
- **Name matching with many short queries**: character `n = 2` with `Dice`,
  alpha 0.8+; bigrams keep short queries from over-reaching.
- **Subtitle / snippet search where one document may contain the query:
  `Overlap`**. The query `"apple"` reaches a **perfect 1.0** against
  `"apple juice"` because the query bigrams all occur there — no other
  built-in behaves that way.
- **Sentence or label-level matching**: `WordNgrams(2, " ", "_")` with
  `Jaccard` for strong containment-only matches, or `Cosine` when paraphrases
  should still surface.

## Alpha in practice

- The threshold is *exact*: raising alpha from 0.79 to 0.80 removes every
  string below 0.80, so rescored output can drop a little.
- `alpha` must be `0.0 < alpha <= 1.0`; `0.0` raises
  `SearchError::InvalidThreshold` (in Python, `SearchError` derives from
  `ValueError`).
- `ranked_search` gives you a monotonic frontier: as you test smaller alphas
  scores only grow, and you can then walk the list to pick a threshold from
  data instead of guessing.
- If your corpus has wildly differing lengths, prefer measures with
  symmetric handling (`Dice`, `Cosine`) over `Overlap`, since longer strings
  gain an advantage by simply containing everything.

## Custom measures (Rust only)

Implement the `Measure` trait: `min_feature_size`, `max_feature_size`, and
`minimum_common_feature_count` are conservative pruning bounds
(`n >= min`) used to skip impossible buckets — compute tight mathematical
bounds and round them **outwards** (`f64::next_down`/`next_up`, like the
built-ins) — and `similarity_from_counts` is the exact verdict. The
`similarity(x, y)` trait method is a convenience over two explicit feature
slices and is not used by search itself.

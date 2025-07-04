import time

from simstring_rust.database import HashDb
from simstring_rust.errors import SearchError
from simstring_rust.extractors import CharacterNgrams
from simstring_rust.measures import Cosine
from simstring_rust.searcher import Searcher


def main():
    # 1. Setup
    print("--- 1. Setting up components ---")
    # Choose a feature extractor.
    # Options: CharacterNgrams(n, endmarker), WordNgrams(n, splitter, padder)
    extractor = CharacterNgrams(n=2, endmarker="$")
    print("Using extractor: CharacterNgrams(n=2, endmarker='$')")

    # Choose a similarity measure.
    # Options: Cosine(), Dice(), Jaccard(), Overlap(), ExactMatch()
    measure = Cosine()
    print("Using measure: Cosine()")

    # 2. Indexing
    print("\n--- 2. Indexing strings ---")
    db = HashDb(extractor)

    corpus = ["foo", "bar", "fooo", "foooobar", "apple", "apply", "apples"]
    print(f"Corpus: {corpus}")

    start_time = time.time()
    for item in corpus:
        db.insert(item)
    end_time = time.time()
    print(f"Indexing {len(db)} strings took {(end_time - start_time) * 1000:.2f} ms.")

    # 3. Searching
    print("\n--- 3. Performing searches ---")
    searcher = Searcher(db, measure)

    query = "apple"
    alpha = 0.8
    print(f"\nRanked search for '{query}' with alpha >= {alpha}:")
    ranked_results = searcher.ranked_search(query, alpha)

    if not ranked_results:
        print("  No matches found.")
    else:
        for item, score in ranked_results:
            print(f"  - Match: '{item}', Score: {score:.4f}")

    print(f"\nUnranked search for '{query}' with alpha >= {alpha}:")
    unranked_results = searcher.search(query, alpha)
    print(f"  - Matches: {unranked_results}")

    # 4. Error Handling
    print("\n--- 4. Testing error handling ---")
    try:
        invalid_alpha = 1.1
        print(f"Attempting search with invalid alpha = {invalid_alpha}...")
        searcher.search("test", invalid_alpha)
    except SearchError as e:
        print(f"  Successfully caught expected error: {e}")


if __name__ == "__main__":
    main()

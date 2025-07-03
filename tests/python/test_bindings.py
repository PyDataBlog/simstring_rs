import pytest
from simstring_rust.database import HashDb
from simstring_rust.errors import SearchError
from simstring_rust.extractors import CharacterNgrams
from simstring_rust.measures import Cosine
from simstring_rust.searcher import Searcher


class TestSimstringBindings:
    def setup_method(self):
        self.extractor = CharacterNgrams(n=2, endmarker="$")
        self.db = HashDb(self.extractor)
        # Insert in non-alphabetical order to also test sorting logic
        self.db.insert("apply")
        self.db.insert("apple")
        self.db.insert("banana")
        self.searcher = Searcher(self.db, Cosine())

    def test_db_creation_and_insertion(self):
        # This test requires a clean DB
        db = HashDb(self.extractor)
        assert len(db) == 0

        db.insert("apple")
        db.insert("apply")
        assert len(db) == 2

        db.clear()
        assert len(db) == 0

    def test_ranked_search_correctness(self):
        # With a threshold of 0.8, only "apple" should be returned.
        results = self.searcher.ranked_search("apple", 0.8)
        assert len(results) == 1
        assert results[0][0] == "apple"
        assert results[0][1] == pytest.approx(1.0)

        # With a lower threshold, "apply" should also be returned.
        results_lower_thresh = self.searcher.ranked_search("apple", 0.6)
        assert len(results_lower_thresh) == 2
        assert results_lower_thresh[0][0] == "apple"
        assert results_lower_thresh[1][0] == "apply"
        assert results_lower_thresh[1][1] == pytest.approx(4 / 6)

    def test_unranked_search_correctness(self):
        # With a threshold of 0.8, only "apple" should be returned.
        results = self.searcher.search("apple", 0.8)
        assert results == ["apple"]

        # With a lower threshold, both are returned, sorted alphabetically.
        results_lower_thresh = self.searcher.search("apple", 0.6)
        assert results_lower_thresh == ["apple", "apply"]

    def test_search_error_on_invalid_threshold(self):
        with pytest.raises(SearchError, match=r"Invalid threshold: 1\.1"):
            self.searcher.search("test", 1.1)

        with pytest.raises(SearchError, match=r"Invalid threshold: 0(\.0)?"):
            self.searcher.search("test", 0.0)

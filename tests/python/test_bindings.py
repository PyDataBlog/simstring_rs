import pytest
from collections import Counter

from simstring_rust.database import HashDb
from simstring_rust.errors import SearchError
from simstring_rust.extractors import CharacterNgrams, WordNgrams, CustomExtractor
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

    def test_db_strings(self):
        db = HashDb(self.extractor)

        db.insert("apple")
        db.insert("apply")

        db_collection = db.strings()
        assert db_collection == ["apple", "apply"]

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

    def test_character_ngram_apply(self):
        extractor = CharacterNgrams(n=2, endmarker="$")
        features = extractor.apply("apple")

        expected = ["$a1", "ap1", "pp1", "pl1", "le1", "e$1"]
        assert Counter(features) == Counter(expected)

    def test_word_ngram_apply(self):
        extractor = WordNgrams(n=2, splitter=" ", padder="#")
        features = extractor.apply("foo bar baz")

        expected = ["# foo1", "foo bar1", "bar baz1", "baz #1"]
        assert Counter(features) == Counter(expected)

    def test_custom_extractor_apply(self):
        class UnigramExtractor:
            def apply(self, text: str):
                return list(text)

        extractor = CustomExtractor(UnigramExtractor())
        features = extractor.apply("foo")

        expected = ["f1", "o1", "o2"]
        assert Counter(features) == Counter(expected)

    def test_custom_extractor_in_db(self):
        class UnigramExtractor:
            def apply(self, text: str):
                return list(text)

        extractor = CustomExtractor(UnigramExtractor())
        db = HashDb(extractor)
        db.insert("foo")
        db.insert("bar")

        searcher = Searcher(db, Cosine())
        results = searcher.search("foo", 0.8)

        assert results == ["foo"]

    def test_word_ngram_edge_cases(self):
        # Empty string
        extractor = WordNgrams(n=2, splitter=" ", padder="#")
        features = extractor.apply("")
        # With n=2 and 1 padding on each side, we get ["# #"] -> ["# #1"]
        assert features == ["# #1"]

        # String with only separators
        features_sep = extractor.apply("   ")
        assert features_sep == ["# #1"]

        # Different splitter
        extractor_comma = WordNgrams(n=2, splitter=",", padder="#")
        features_comma = extractor_comma.apply("foo,bar")
        expected_comma = ["# foo1", "foo bar1", "bar #1"]
        assert Counter(features_comma) == Counter(expected_comma)

    def test_word_ngrams_in_db(self):
        extractor = WordNgrams(n=2, splitter=" ", padder="#")
        db = HashDb(extractor)
        db.insert("foo bar")
        searcher = Searcher(db, Cosine())
        results = searcher.search("foo bar", 1.0)
        assert results == ["foo bar"]

    def test_invalid_extractor_in_db(self):
        with pytest.raises(TypeError, match="Extractor must be CharacterNgrams, WordNgrams, or CustomExtractor"):
            HashDb("not an extractor")

    def test_ranked_search_error_on_invalid_threshold(self):
        with pytest.raises(SearchError, match=r"Invalid threshold: 1\.1"):
            self.searcher.ranked_search("test", 1.1)

import pytest
from simstring_rust.database import HashDb
from simstring_rust.extractors import CharacterNgrams
from simstring_rust.measures import Dice, Jaccard, Overlap, ExactMatch
from simstring_rust.searcher import Searcher

class TestMeasures:
    def setup_method(self):
        self.extractor = CharacterNgrams(n=2, endmarker="$")
        self.db = HashDb(self.extractor)
        self.db.insert("foo")
        self.db.insert("bar")
        self.db.insert("fooo")

    def test_dice(self):
        searcher = Searcher(self.db, Dice())
        results = searcher.ranked_search("foo", 0.8)
        # "foo" (4 features) vs "foo" (4 features) -> 2*4 / (4+4) = 1.0
        # "foo" vs "fooo" (5 features) -> intersect is 4 ($f, fo, oo, o$) -> 2*4 / (4+5) = 8/9 ~= 0.88
        assert len(results) == 2
        assert results[0][0] == "foo"
        assert results[0][1] == pytest.approx(1.0)
        assert results[1][0] == "fooo"
        assert results[1][1] == pytest.approx(0.88888888)

    def test_jaccard(self):
        searcher = Searcher(self.db, Jaccard())
        results = searcher.ranked_search("foo", 0.8)
        # "foo" vs "foo" -> 1.0
        # "foo" vs "fooo" -> 4 / 5 = 0.8
        assert len(results) == 2
        assert results[0][0] == "foo"
        assert results[0][1] == pytest.approx(1.0)
        assert results[1][0] == "fooo"
        assert results[1][1] == pytest.approx(0.8)

    def test_overlap(self):
        searcher = Searcher(self.db, Overlap())
        results = searcher.ranked_search("foo", 0.8)

        assert len(results) == 2
        assert results[0][0] == "foo"
        assert results[0][1] == pytest.approx(1.0)
        assert results[1][0] == "fooo"
        assert results[1][1] == pytest.approx(1.0)

    def test_exact_match(self):
        searcher = Searcher(self.db, ExactMatch())
        results = searcher.ranked_search("foo", 1.0)
        assert len(results) == 1
        assert results[0][0] == "foo"
        assert results[0][1] == pytest.approx(1.0)

        results_partial = searcher.ranked_search("foo", 0.5)
        assert len(results_partial) == 1
        assert results_partial[0][0] == "foo"

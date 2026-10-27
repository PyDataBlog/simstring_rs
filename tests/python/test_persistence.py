from pathlib import Path

import pytest

from simstring_rust.database import HashDb
from simstring_rust.extractors import CharacterNgrams, CustomExtractor, WordNgrams
from simstring_rust.measures import Cosine, Dice, ExactMatch, Jaccard, Overlap
from simstring_rust.searcher import Searcher


TEXTS = ["", "aaaa", "aaaa", "東京🍕", "foo,bar", "café"]


def assert_searches_equal(expected, actual):
    for measure in [Cosine(), Dice(), Jaccard(), Overlap(), ExactMatch()]:
        a, b = Searcher(expected, measure), Searcher(actual, measure)
        for query in TEXTS + ["unseen 🦀"]:
            for threshold in [0.3, 0.6, 0.9, 1.0]:
                assert a.search(query, threshold) == b.search(query, threshold)
                assert a.ranked_search(query, threshold) == b.ranked_search(query, threshold)


@pytest.mark.parametrize("name,extractor", [
    ("character", CharacterNgrams(3, "🍕!")),
    ("word", WordNgrams(2, ",", "#")),
])
def test_shared_fixtures_roundtrip_and_mutation(tmp_path, name, extractor):
    fixture = Path(__file__).parents[1] / "fixtures" / f"persistence_v1_{name}.hex"
    data = bytes.fromhex(fixture.read_text())
    (tmp_path / "fixture.bin").write_bytes(data)
    restored = HashDb.load_from_file(tmp_path, Path("fixture.bin"))
    expected = HashDb(extractor)
    for text in TEXTS:
        expected.insert(text)
    assert restored.strings() == TEXTS
    assert len(restored) == len(expected)
    assert_searches_equal(expected, restored)
    expected.save_to_file(str(tmp_path), "generated.bin")
    assert (tmp_path / "generated.bin").read_bytes() == data
    restored.save_to_file(tmp_path, "restored.bin")
    assert (tmp_path / "restored.bin").read_bytes() == data
    for db in [expected, restored]:
        db.insert("unseen 🦀")
    assert_searches_equal(expected, restored)
    for db in [expected, restored]:
        db.clear()
        db.insert("café")
    assert_searches_equal(expected, restored)


@pytest.mark.parametrize("extractor", [CharacterNgrams(0, "$"), WordNgrams(0, " ", "#")])
def test_empty_and_zero_feature_databases(tmp_path, extractor):
    db = HashDb(extractor)
    for text in [None, "東京🍕" * 20_000]:
        if text is not None:
            db.insert(text)
        db.save_to_file(tmp_path, "db.bin")
        restored = HashDb.load_from_file(tmp_path, "db.bin")
        assert restored.strings() == db.strings()
        assert Searcher(restored, Cosine()).search("anything", 0.5) == []


def test_io_and_filename_errors(tmp_path):
    db = HashDb(CharacterNgrams(2, "$"))
    with pytest.raises(FileNotFoundError):
        HashDb.load_from_file(tmp_path, "missing.bin")
    with pytest.raises(FileNotFoundError):
        db.save_to_file(tmp_path / "missing", "db.bin")
    for name in ["", ".", "..", "nested/db.bin", "/db.bin"]:
        with pytest.raises(ValueError, match="single filename"):
            db.save_to_file(tmp_path, name)
        with pytest.raises(ValueError, match="single filename"):
            HashDb.load_from_file(tmp_path, name)


def test_corrupt_and_future_snapshots(tmp_path):
    db = HashDb(CharacterNgrams(2, "$"))
    db.insert("hello")
    db.save_to_file(tmp_path, "db.bin")
    path = tmp_path / "db.bin"
    original = path.read_bytes()
    for damaged in [b"", original[:-1], original + b"extra"]:
        path.write_bytes(damaged)
        with pytest.raises(ValueError):
            HashDb.load_from_file(tmp_path, "db.bin")
    damaged = bytearray(original)
    damaged[-1] ^= 1
    path.write_bytes(damaged)
    with pytest.raises(ValueError, match="checksum"):
        HashDb.load_from_file(tmp_path, "db.bin")
    damaged = bytearray(original)
    damaged[8:12] = (99).to_bytes(4, "little")
    path.write_bytes(damaged)
    with pytest.raises(ValueError, match="version: 99"):
        HashDb.load_from_file(tmp_path, "db.bin")


def test_custom_extractor_preserves_destination(tmp_path):
    class Custom:
        def apply(self, text):
            return list(text)

    db = HashDb(CustomExtractor(Custom()))
    path = tmp_path / "db.bin"
    path.write_bytes(b"previous contents")
    with pytest.raises(TypeError, match="does not support snapshots"):
        db.save_to_file(tmp_path, "db.bin")
    assert path.read_bytes() == b"previous contents"
    assert list(tmp_path.iterdir()) == [path]

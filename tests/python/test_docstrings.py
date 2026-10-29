"""Docstring coverage tests for the Python bindings.

These guard the user-facing documentation that ships inside the extension
module: every public class, method, submodule, and the root module must
render non-empty ``__doc__`` strings, so ``help(simstring_rust...)`` is
never blank.
"""

import simstring_rust
import simstring_rust.database
import simstring_rust.errors
import simstring_rust.extractors
import simstring_rust.measures
import simstring_rust.searcher


def assert_doc(object_, name):
    doc = object_.__doc__
    assert doc is not None and doc.strip(), f"{name} is missing its docstring"
    return doc


class TestModuleDocstrings:
    def test_root_module(self):
        doc = assert_doc(simstring_rust, "simstring_rust")
        assert "SimString" in doc
        assert "database" in doc  # submodule overview

    def test_submodules(self):
        modules = [
            ("database", simstring_rust.database, "HashDb"),
            ("extractors", simstring_rust.extractors, "CharacterNgrams"),
            ("measures", simstring_rust.measures, "Cosine"),
            ("searcher", simstring_rust.searcher, "Searcher"),
            ("errors", simstring_rust.errors, "SearchError"),
        ]
        for name, module, needle in modules:
            doc = assert_doc(module, f"simstring_rust.{name}")
            assert needle in doc, f"{name} docs do not mention {needle}"


class TestExtractorDocstrings:
    def test_character_ngrams(self):
        extractor = simstring_rust.extractors.CharacterNgrams
        assert_doc(extractor, "CharacterNgrams")
        assert_doc(extractor.apply, "CharacterNgrams.apply")
        assert "n" in extractor.__doc__
        assert "$h1" in extractor.__doc__  # includes a worked example

    def test_word_ngrams(self):
        extractor = simstring_rust.extractors.WordNgrams
        assert_doc(extractor, "WordNgrams")
        assert_doc(extractor.apply, "WordNgrams.apply")
        assert "splitter" in extractor.__doc__
        assert "padder" in extractor.__doc__

    def test_custom_extractor(self):
        extractor = simstring_rust.extractors.CustomExtractor
        assert_doc(extractor, "CustomExtractor")
        assert_doc(extractor.apply, "CustomExtractor.apply")
        assert "Iterable" in extractor.__doc__


class TestMeasureDocstrings:
    def test_measures(self):
        measures = {
            "Cosine": "sqrt",
            "Dice": "common",
            "ExactMatch": "identical",
            "Jaccard": "common",
            "Overlap": "min",
        }
        for name, needle in measures.items():
            doc = assert_doc(
                getattr(simstring_rust.measures, name), f"measures.{name}"
            )
            assert needle in doc, f"{name} docs do not mention {needle}"


class TestDatabaseDocstrings:
    def test_hash_db(self):
        db = None
        extractor = simstring_rust.extractors.CharacterNgrams(2, "$")
        db = simstring_rust.database.HashDb
        assert_doc(db, "HashDb")
        assert_doc(db.insert, "HashDb.insert")
        assert_doc(db.clear, "HashDb.clear")
        assert_doc(db.save_to_file, "HashDb.save_to_file")
        assert_doc(db.load_from_file, "HashDb.load_from_file")
        assert_doc(db.strings, "HashDb.strings")
        # The instance-dependent dunder: exercised on a real database.
        database = db(extractor)
        assert_doc(type(database).__len__, "HashDb.__len__")

    def test_search_error(self):
        exception = simstring_rust.errors.SearchError
        assert_doc(exception, "SearchError")
        assert "alpha" in exception.__doc__
        # Consistent exception type, documented as a ValueError subclass.
        assert issubclass(exception, ValueError)


class TestSearcherDocstrings:
    def test_searcher(self):
        searcher = simstring_rust.searcher.Searcher
        assert_doc(searcher, "Searcher")
        assert_doc(searcher.search, "Searcher.search")
        assert_doc(searcher.ranked_search, "Searcher.ranked_search")
        assert "alpha" in searcher.search.__doc__
        assert "SearchError" in searcher.search.__doc__

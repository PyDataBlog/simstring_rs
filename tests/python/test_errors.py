import pytest
import multiprocessing
import os
import subprocess
import sys
from simstring_rust.extractors import CustomExtractor
from simstring_rust.database import HashDb

def run_crashing_extractor():  # pragma: no cover
    class CrashingExtractor:
        def apply(self, text):
            raise ValueError("Crash!")

    extractor = CustomExtractor(CrashingExtractor())
    db = HashDb(extractor)
    # This should panic the Rust side because of the unhandled exception in the callback
    db.insert("foo")

def test_custom_extractor_panic():
    # Run the crashing code in a separate process
    p = multiprocessing.Process(target=run_crashing_extractor)
    p.start()
    p.join()

    # Check if the process exited with an error (panic usually causes non-zero exit code)
    assert p.exitcode != 0

def test_custom_extractor_missing_apply():
    class BadExtractor:
        pass

    with pytest.raises(TypeError, match="Custom extractor must provide an apply"):
        CustomExtractor(BadExtractor())


SEARCH_SCRIPT = """
from simstring_rust.database import HashDb
from simstring_rust.extractors import CharacterNgrams
from simstring_rust.measures import Cosine
from simstring_rust.searcher import Searcher
db = HashDb(CharacterNgrams(n=2, endmarker="$"))
db.insert("apple")
assert Searcher(db, Cosine()).search("apple", 1.0) == ["apple"]
"""


@pytest.mark.parametrize("backend", ["auto", "native"])
def test_backend_override_accepted(backend):
    # The backend is selected once per process, so overrides need a fresh interpreter.
    env = {**os.environ, "SIMSTRING_RS_BACKEND": backend}
    result = subprocess.run([sys.executable, "-c", SEARCH_SCRIPT], env=env, capture_output=True, text=True)
    assert result.returncode == 0, result.stderr


def test_invalid_backend_override_rejected():
    env = {**os.environ, "SIMSTRING_RS_BACKEND": "bogus"}
    result = subprocess.run([sys.executable, "-c", SEARCH_SCRIPT], env=env, capture_output=True, text=True)
    assert result.returncode != 0
    assert "invalid SIMSTRING_RS_BACKEND" in result.stderr

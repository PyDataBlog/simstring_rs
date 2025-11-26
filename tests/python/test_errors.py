import pytest
import multiprocessing
import sys
from simstring_rust.extractors import CustomExtractor
from simstring_rust.database import HashDb

def run_crashing_extractor():
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

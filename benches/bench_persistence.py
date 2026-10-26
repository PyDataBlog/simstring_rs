import argparse
import json
from pathlib import Path
from statistics import median
from tempfile import TemporaryDirectory
from time import perf_counter

from simstring_rust.database import HashDb
from simstring_rust.extractors import CharacterNgrams


SAMPLES = 7


def median_ms(run, verify=None):
    measurements = []
    for iteration in range(SAMPLES + 1):
        start = perf_counter()
        result = run()
        elapsed = (perf_counter() - start) * 1000
        if verify is not None:
            verify(result)
        del result
        if iteration:
            measurements.append(elapsed)
    return median(measurements)


def bench_persistence(dataset=None):
    data = Path(__file__).parent / "data"
    results = []
    for name, file in [
        ("company", "company_names.txt"),
        ("unigrams", "eng_google_books_unigram.txt"),
    ]:
        if dataset is not None and dataset != name:
            continue
        corpus = (data / file).read_text(encoding="utf-8").splitlines()
        with TemporaryDirectory() as directory:
            path = Path(directory)
            for ngram_size in [2, 3, 4]:

                def build():
                    db = HashDb(CharacterNgrams(ngram_size, " "))
                    for text in corpus:
                        db.insert(text)
                    return db

                def verify_loaded(loaded):
                    assert loaded.strings() == corpus

                build_ms = median_ms(build)
                db = build()
                save_ms = median_ms(lambda: db.save_to_file(path, "db.bin"))
                size = (path / "db.bin").stat().st_size
                load_ms = median_ms(
                    lambda: HashDb.load_from_file(path, "db.bin"), verify_loaded
                )
                results.append(
                    {
                        "language": "python",
                        "backend": "simstring-rust (python bindings)",
                        "benchmark": "persistence",
                        "parameters": {"dataset": name, "ngram_size": ngram_size},
                        "stats": {
                            "build_ms": build_ms,
                            "save_ms": save_ms,
                            "load_ms": load_ms,
                            "size_mib": size / 1048576,
                            "iterations": SAMPLES,
                        },
                    }
                )
                del db
    return results


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("dataset", nargs="?", choices=["company", "unigrams"])
    args = parser.parse_args()
    print(json.dumps(bench_persistence(args.dataset), indent=2))

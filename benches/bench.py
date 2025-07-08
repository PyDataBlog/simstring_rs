# /// script
# dependencies = [
#   "simstring-fast",
# ]
# ///

import json
import statistics
import time
import typing as t
from pathlib import Path

from simstring.database.dict import DictDatabase
from simstring.feature_extractor.character_ngram import CharacterNgramFeatureExtractor
from simstring.measure.cosine import CosineMeasure
from simstring.searcher import Searcher


def create_database(ngrams_size: int) -> DictDatabase:
    return DictDatabase(CharacterNgramFeatureExtractor(ngrams_size))


def measure_time(func: t.Callable) -> float:
    start = time.perf_counter()
    func()
    end = time.perf_counter()
    return end - start


def load_company_names() -> list[str]:
    current_dir = Path.cwd()
    file_path = current_dir / "benches" / "data" / "company_names.txt"
    with open(file_path) as f:
        return [line.strip() for line in f]


def bench_insert(results: list):
    company_names = load_company_names()
    iterations = 100
    measurement_time = 20

    for ngram_size in [2, 3, 4]:
        measurements = []
        start_time = time.time()
        iteration = 0

        while time.time() - start_time < measurement_time and iteration < iterations:

            def benchmark_iteration():
                db = create_database(ngram_size)
                for name in company_names:
                    db.add(name)

            duration = measure_time(benchmark_iteration)
            measurements.append(duration)
            iteration += 1

        mean_time = statistics.mean(measurements)
        stddev = statistics.stdev(measurements) if len(measurements) > 1 else 0

        results.append(
            {
                "language": "python",
                "backend": "simstring-fast",
                "benchmark": "insert",
                "parameters": {"ngram_size": ngram_size},
                "stats": {
                    "mean": mean_time * 1000,
                    "stddev": stddev * 1000,
                    "iterations": len(measurements),
                },
            }
        )


def bench_search(results: list):
    company_names = load_company_names()
    search_terms = company_names[:100]
    iterations = 100
    measurement_time = 20
    similarity_thresholds = [0.6, 0.7, 0.8, 0.9]

    for ngram_size in [2, 3, 4]:
        db = create_database(ngram_size)
        for name in company_names:
            db.add(name)

        searcher = Searcher(db, CosineMeasure())

        for threshold in similarity_thresholds:
            measurements = []
            start_time = time.time()
            iteration = 0

            while (
                time.time() - start_time < measurement_time and iteration < iterations
            ):

                def benchmark_iteration():
                    for term in search_terms:
                        searcher.ranked_search(term, threshold)

                duration = measure_time(benchmark_iteration)
                measurements.append(duration)
                iteration += 1

            mean_time = statistics.mean(measurements)
            stddev = statistics.stdev(measurements) if len(measurements) > 1 else 0

            results.append(
                {
                    "language": "python",
                    "backend": "simstring-fast",
                    "benchmark": "search",
                    "parameters": {"ngram_size": ngram_size, "threshold": threshold},
                    "stats": {
                        "mean": mean_time * 1000,
                        "stddev": stddev * 1000,
                        "iterations": len(measurements),
                    },
                }
            )


if __name__ == "__main__":
    results = []
    bench_insert(results)
    bench_search(results)
    print(json.dumps(results, indent=2))

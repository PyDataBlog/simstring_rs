# /// script
# dependencies = [
#   "simstring-fast",
# ]
# ///

import statistics
import time
import typing as t
from pathlib import Path

from simstring.database.dict import DictDatabase
from simstring.feature_extractor.character_ngram import CharacterNgramFeatureExtractor


def create_database(ngrams_size: int) -> DictDatabase:
    return DictDatabase(CharacterNgramFeatureExtractor(ngrams_size))


def measure_time(func: t.Callable) -> float:
    start = time.perf_counter()
    func()
    end = time.perf_counter()
    return end - start


def bench_insert():
    current_dir = Path.cwd()
    file_path = current_dir / "benches" / "data" / "company_names.txt"

    with open(file_path) as f:
        company_names = [line.strip() for line in f]

    # Test for n-gram sizes 2, 3, and 4
    iterations = 100
    measurement_time = 20

    print("\nBenchmarking database insertions:")
    print("-" * 40)

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

        print(f"ngram_{ngram_size}:")
        print(f"  Mean: {mean_time * 1000:.2f}ms")
        print(f"  Std Dev: {stddev * 1000:.2f}ms")
        print(f"  Iterations: {len(measurements)}")


if __name__ == "__main__":
    bench_insert()

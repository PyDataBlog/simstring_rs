#!/usr/bin/env python3
"""
Benchmarks for the original C++ SimString CLI.

This script clones/builds https://github.com/chokkan/simstring into
benches/.simstring_cpp (if needed) and benchmarks using the CLI binary
"""

from __future__ import annotations

import json
import os
import statistics
import subprocess
import tempfile
import time
from pathlib import Path
from typing import Callable, Iterable

THIS_DIR = Path(__file__).parent
WORK_DIR = THIS_DIR / ".simstring_cpp"
SRC_DIR = WORK_DIR / "src"
INSTALL_DIR = WORK_DIR / "install"
SIMSTRING_BIN = INSTALL_DIR / "bin" / "simstring"
REPO_URL = "https://github.com/chokkan/simstring.git"

COMPANY_FILE = THIS_DIR / "data" / "company_names.txt"
# Allow overriding via env vars for quicker smoke tests.
MEASUREMENT_TIME = int(os.environ.get("SIMSTRING_BENCH_DURATION", "20"))
MAX_ITERATIONS = int(os.environ.get("SIMSTRING_BENCH_MAX_ITERATIONS", "100"))
NGRAM_SIZES = (2, 3, 4)
THRESHOLDS = (0.6, 0.7, 0.8, 0.9)


DEBUG = bool(os.environ.get("SIMSTRING_BENCH_DEBUG"))


def run_command(cmd: list[str], cwd: Path | None = None) -> None:
    """Run a command quietly unless it fails or debug is enabled."""
    proc = subprocess.run(
        cmd, cwd=cwd, check=False, text=True, capture_output=not DEBUG
    )
    if DEBUG:
        # Stream output immediately in debug mode for easier troubleshooting.
        pass
    if proc.returncode != 0:
        stdout = proc.stdout or ""
        stderr = proc.stderr or ""
        raise RuntimeError(
            f"Command {' '.join(cmd)} failed with code {proc.returncode}\n"
            f"stdout:\n{stdout}\n\nstderr:\n{stderr}"
        )


def ensure_simstring_binary() -> Path:
    """Clone/build the original C++ SimString CLI if necessary."""
    if SIMSTRING_BIN.exists():
        return SIMSTRING_BIN

    WORK_DIR.mkdir(parents=True, exist_ok=True)

    if not SRC_DIR.exists():
        run_command(["git", "clone", "--depth", "1", REPO_URL, str(SRC_DIR)])

    autogen = SRC_DIR / "autogen.sh"
    if autogen.exists() and not (SRC_DIR / "configure").exists():
        run_command(["/bin/sh", str(autogen)], cwd=SRC_DIR)

    # Fresh configure against the local install dir.
    if not (SRC_DIR / "Makefile").exists():
        run_command(
            [
                "./configure",
                f"--prefix={INSTALL_DIR}",
            ],
            cwd=SRC_DIR,
        )

    # Build & install.
    jobs = os.cpu_count() or 2
    run_command(["make", f"-j{jobs}"], cwd=SRC_DIR)
    run_command(["make", "install"], cwd=SRC_DIR)

    return SIMSTRING_BIN


def load_company_names() -> list[str]:
    with COMPANY_FILE.open() as fh:
        return [line.strip() for line in fh if line.strip()]


def measure(func: Callable[[], None]) -> float:
    start = time.perf_counter()
    func()
    return (time.perf_counter() - start) * 1000.0  # milliseconds


def run_simstring(args: Iterable[str], stdin_data: str) -> None:
    subprocess.run(
        [str(SIMSTRING_BIN), *args],
        input=stdin_data,
        text=True,
        capture_output=True,
        check=True,
    )


def build_database(ngram_size: int, db_path: Path, company_blob: str) -> None:
    db_path.parent.mkdir(parents=True, exist_ok=True)
    run_simstring(
        [
            "-b",
            "-d",
            str(db_path),
            "-n",
            str(ngram_size),
            "-m",
            "-q",
        ],
        company_blob,
    )


def ensure_persistent_db(ngram_size: int, company_blob: str) -> Path:
    db_path = WORK_DIR / f"company_names_{ngram_size}.db"
    if db_path.exists():
        return db_path
    build_database(ngram_size, db_path, company_blob)
    return db_path


def summarize_measurements(measurements: list[float]) -> tuple[float, float]:
    mean = statistics.fmean(measurements)
    stddev = statistics.stdev(measurements) if len(measurements) > 1 else 0.0
    return mean, stddev


def bench_insert(results: list[dict], company_blob: str) -> None:
    for ngram_size in NGRAM_SIZES:
        measurements: list[float] = []
        start = time.time()
        iteration = 0

        while time.time() - start < MEASUREMENT_TIME and iteration < MAX_ITERATIONS:
            with tempfile.TemporaryDirectory(dir=WORK_DIR) as tmpdir:
                db_path = Path(tmpdir) / "bench.db"

                def workload() -> None:
                    build_database(ngram_size, db_path, company_blob)

                duration_ms = measure(workload)
                measurements.append(duration_ms)
                iteration += 1

        mean, stddev = summarize_measurements(measurements)
        results.append(
            {
                "language": "c++",
                "backend": "simstring (C++ CLI)",
                "benchmark": "insert",
                "parameters": {"ngram_size": ngram_size},
                "stats": {
                    "mean": mean,
                    "stddev": stddev,
                    "iterations": len(measurements),
                },
            }
        )


def bench_search(results: list[dict], company_blob: str, query_blob: str) -> None:
    for ngram_size in NGRAM_SIZES:
        db_path = ensure_persistent_db(ngram_size, company_blob)

        for threshold in THRESHOLDS:
            measurements: list[float] = []
            start = time.time()
            iteration = 0

            cmd = [
                "-d",
                str(db_path),
                "-t",
                f"{threshold}",
                "-s",
                "cosine",
                "-m",
                "-p",
                "-q",
            ]

            while time.time() - start < MEASUREMENT_TIME and iteration < MAX_ITERATIONS:
                duration_ms = measure(lambda: run_simstring(cmd, query_blob))
                measurements.append(duration_ms)
                iteration += 1

            mean, stddev = summarize_measurements(measurements)
            results.append(
                {
                    "language": "c++",
                    "backend": "simstring (C++ CLI)",
                    "benchmark": "search",
                    "parameters": {"ngram_size": ngram_size, "threshold": threshold},
                    "stats": {
                        "mean": mean,
                        "stddev": stddev,
                        "iterations": len(measurements),
                    },
                }
            )


def main() -> None:
    ensure_simstring_binary()
    companies = load_company_names()
    company_blob = "\n".join(companies) + "\n"
    queries = companies[:100]
    query_blob = "\n".join(queries) + "\n"

    results: list[dict] = []
    bench_insert(results, company_blob)
    bench_search(results, company_blob, query_blob)

    print(json.dumps(results, indent=2))


if __name__ == "__main__":
    main()

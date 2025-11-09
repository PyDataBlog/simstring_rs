#!/usr/bin/env python3
"""
Benchmark harness for the original SimString Python bindings (SWIG).

This script clones/builds https://github.com/chokkan/simstring into
benches/.simstring_cpp (shared with the CLI benchmarks), builds the SWIG
Python module in-place, and benchmarks insert/search workloads using the
same company-name corpus as the other backends.
"""

from __future__ import annotations

import json
import os
import statistics
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from typing import Callable, Iterable

THIS_DIR = Path(__file__).parent
WORK_DIR = THIS_DIR / ".simstring_cpp"
SRC_DIR = WORK_DIR / "src"
INSTALL_DIR = WORK_DIR / "install"
PYTHON_DIR = SRC_DIR / "swig" / "python"
REPO_URL = "https://github.com/chokkan/simstring.git"

COMPANY_FILE = THIS_DIR / "data" / "company_names.txt"
MEASUREMENT_TIME = int(os.environ.get("SIMSTRING_BENCH_DURATION", "20"))
MAX_ITERATIONS = int(os.environ.get("SIMSTRING_BENCH_MAX_ITERATIONS", "100"))
NGRAM_SIZES = (2, 3, 4)
THRESHOLDS = (0.6, 0.7, 0.8, 0.9)

CUSTOM_SETUP_TEMPLATE = """#!/usr/bin/env python
\"\"\"
setup.py replacement for building the SimString SWIG module.
\"\"\"
import os
import os.path
from setuptools import setup, Extension


def get_rootdir():
    return os.path.abspath(os.path.join(os.path.dirname(__file__), "../.."))


def get_includedir():
    return os.path.join(get_rootdir(), "include")


simstring_module = Extension(
    "_simstring",
    sources=["export.cpp", "export_wrap.cpp"],
    include_dirs=[get_includedir()],
    libraries=["iconv"],
    extra_compile_args=["-std=c++11"],
    language="c++",
)

setup(
    name="simstring",
    version="1.1",
    author="Naoaki Okazaki",
    description=\"\"\"SimString Python module\"\"\",
    ext_modules=[simstring_module],
    py_modules=["simstring"],
)
"""


def run_command(cmd: list[str], cwd: Path | None = None) -> None:
    subprocess.run(cmd, cwd=cwd, check=True)


def ensure_repo() -> None:
    if SRC_DIR.exists():
        return
    WORK_DIR.mkdir(parents=True, exist_ok=True)
    run_command(["git", "clone", "--depth", "1", REPO_URL, str(SRC_DIR)])


def ensure_configured() -> None:
    ensure_repo()
    autogen = SRC_DIR / "autogen.sh"
    if autogen.exists():
        run_command(["/bin/sh", str(autogen)], cwd=SRC_DIR)
    run_command(["./configure", f"--prefix={INSTALL_DIR}"], cwd=SRC_DIR)


def ensure_built() -> None:
    ensure_configured()
    jobs = os.cpu_count() or 2
    run_command(["make", f"-j{jobs}"], cwd=SRC_DIR)


def ensure_python_module() -> None:
    ensure_built()
    built = list(PYTHON_DIR.glob("_simstring*.so"))
    if built:
        return

    PYTHON_DIR.mkdir(parents=True, exist_ok=True)
    prepare_script = PYTHON_DIR / "prepare.sh"
    if prepare_script.exists():
        run_command(["/bin/sh", str(prepare_script), "--swig"], cwd=PYTHON_DIR)

    setup_py = PYTHON_DIR / "setup.py"
    setup_py.write_text(CUSTOM_SETUP_TEMPLATE)
    run_command([sys.executable, "setup.py", "build_ext", "--inplace"], cwd=PYTHON_DIR)


def load_company_names() -> list[str]:
    with COMPANY_FILE.open() as fh:
        return [line.strip() for line in fh if line.strip()]


def measure(func: Callable[[], None]) -> float:
    start = time.perf_counter()
    func()
    return (time.perf_counter() - start) * 1000.0


def build_database(simstring_mod, path: Path, names: Iterable[str], ngram_size: int) -> None:
    writer = simstring_mod.writer(str(path), ngram_size, True, False)
    for name in names:
        writer.insert(name)
    writer.close()


def ensure_persistent_db(simstring_mod, names: list[str], ngram_size: int) -> Path:
    db_path = WORK_DIR / f"python_company_names_{ngram_size}.db"
    if not db_path.exists():
        build_database(simstring_mod, db_path, names, ngram_size)
    return db_path


def summarize(measurements: list[float]) -> tuple[float, float]:
    mean = statistics.fmean(measurements)
    stddev = statistics.stdev(measurements) if len(measurements) > 1 else 0.0
    return mean, stddev


def bench_insert(simstring_mod, names: list[str], results: list[dict]) -> None:
    for ngram_size in NGRAM_SIZES:
        measurements: list[float] = []
        start = time.time()
        iteration = 0

        while time.time() - start < MEASUREMENT_TIME and iteration < MAX_ITERATIONS:
            with tempfile.TemporaryDirectory(dir=WORK_DIR) as tmpdir:
                db_path = Path(tmpdir) / "bench.db"

                def workload() -> None:
                    build_database(simstring_mod, db_path, names, ngram_size)

                measurements.append(measure(workload))
                iteration += 1

        mean, stddev = summarize(measurements)
        results.append(
            {
                "language": "python",
                "backend": "simstring (C++ python bindings)",
                "benchmark": "insert",
                "parameters": {"ngram_size": ngram_size},
                "stats": {
                    "mean": mean,
                    "stddev": stddev,
                    "iterations": len(measurements),
                },
            }
        )


def bench_search(simstring_mod, names: list[str], results: list[dict]) -> None:
    search_terms = names[:100]

    for ngram_size in NGRAM_SIZES:
        db_path = ensure_persistent_db(simstring_mod, names, ngram_size)

        for threshold in THRESHOLDS:
            measurements: list[float] = []
            start = time.time()
            iteration = 0

            while time.time() - start < MEASUREMENT_TIME and iteration < MAX_ITERATIONS:

                def workload() -> None:
                    reader = simstring_mod.reader(str(db_path))
                    reader.measure = simstring_mod.cosine
                    reader.threshold = threshold
                    for term in search_terms:
                        reader.retrieve(term)
                    reader.close()

                measurements.append(measure(workload))
                iteration += 1

            mean, stddev = summarize(measurements)
            results.append(
                {
                    "language": "python",
                    "backend": "simstring (C++ python bindings)",
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
    ensure_python_module()
    if str(PYTHON_DIR) not in sys.path:
        sys.path.insert(0, str(PYTHON_DIR))
    import simstring as simstring_mod  # type: ignore

    names = load_company_names()
    results: list[dict] = []
    bench_insert(simstring_mod, names, results)
    bench_search(simstring_mod, names, results)
    print(json.dumps(results, indent=2))


if __name__ == "__main__":
    main()

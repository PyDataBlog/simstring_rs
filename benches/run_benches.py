import json
import subprocess
from pathlib import Path

import pandas as pd


def compare_benchmarks():
    benches_dir = Path(__file__).parent
    results_path = benches_dir / "results.json"
    with open(results_path) as f:
        data = json.load(f)

    df = pd.json_normalize(data)

    df = df.sort_values(["benchmark", "language", "backend"])

    for benchmark, group in df.groupby("benchmark"):
        print(f"### {str(benchmark).capitalize()} Benchmark")
        param_cols = [
            col.replace("parameters.", "")
            for col in df.columns
            if col.startswith("parameters.")
        ]
        display_cols = (
            ["language", "backend"]
            + [f"parameters.{p}" for p in param_cols]
            + ["stats.mean", "stats.stddev", "stats.iterations"]
        )
        display_cols = [col for col in display_cols if col in group.columns]

        group = group[display_cols].dropna(axis=1, how="all")

        group.columns = [
            col.replace("parameters.", "").replace("stats.", "")
            for col in group.columns
        ]

        print(group.to_markdown(index=False))
        print("\n")


def run_benchmarks():
    benches_dir = Path(__file__).parent
    results = []

    # Run Python benchmarks
    for script in ["bench.py", "bench_rust.py"]:
        process = subprocess.run(
            ["python", str(benches_dir / script)],
            capture_output=True,
            text=True,
        )
        if process.stdout:
            results.extend(json.loads(process.stdout))
        else:
            print(f"Error running {script}: {process.stderr}")

    # Run Rust benchmark
    subprocess.run(["cargo", "build", "--release", "--bench", "bench"], check=True)
    process = subprocess.run(
        [str(benches_dir.parent / "target/release/deps/bench-*")],
        capture_output=True,
        text=True,
        shell=True,
    )
    if process.stdout:
        results.extend(json.loads(process.stdout))
    else:
        print(f"Error running rust bench: {process.stderr}")

    # Run Ruby benchmark
    process = subprocess.run(
        ["ruby", str(benches_dir / "bench.rb")],
        capture_output=True,
        text=True,
    )
    if process.stdout:
        results.extend(json.loads(process.stdout))
    else:
        print(f"Error running bench.rb: {process.stderr}")

    # Run Julia benchmark
    process = subprocess.run(
        ["julia", str(benches_dir / "bench.jl")],
        capture_output=True,
        text=True,
    )
    if process.stdout:
        results.extend(json.loads(process.stdout))
    else:
        print(f"Error running bench.jl: {process.stderr}")

    with open(benches_dir / "results.json", "w") as f:
        json.dump(results, f, indent=2)


if __name__ == "__main__":
    run_benchmarks()
    compare_benchmarks()

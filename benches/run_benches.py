import json
import subprocess
from pathlib import Path

from compare_benches import compare_benchmarks


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


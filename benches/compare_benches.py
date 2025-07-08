import json
from pathlib import Path

import pandas as pd


def compare_benchmarks():
    benches_dir = Path(__file__).parent
    results_path = benches_dir / "results.json"
    with open(results_path) as f:
        data = json.load(f)

    df = pd.json_normalize(data)

    df = df.sort_values(["benchmark", "language", "backend"])

    with open(benches_dir.parent / "BENCHMARKS.md", "w") as f:
        for benchmark, group in df.groupby("benchmark"):
            f.write(f"### {str(benchmark).capitalize()} Benchmark\n")
            # Dynamically create table with all parameter columns
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
            # Filter out columns that don't exist in the group
            display_cols = [col for col in display_cols if col in group.columns]

            # Drop parameter columns that are all NaN
            group = group[display_cols].dropna(axis=1, how="all")

            # rename columns for display
            group.columns = [
                col.replace("parameters.", "").replace("stats.", "")
                for col in group.columns
            ]

            markdown_table = group.to_markdown(index=False)
            if markdown_table:
                f.write(markdown_table)
            f.write("\n\n")
            f.write("\n\n")


if __name__ == "__main__":
    compare_benchmarks()
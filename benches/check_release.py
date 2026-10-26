import argparse
import hashlib
import io
import json
import math
import os
from pathlib import Path
import platform
import shutil
from statistics import median
import subprocess
import sys
import tarfile
from urllib.request import Request, urlopen
import venv


ROOT = Path(__file__).resolve().parent.parent
CASES = {f"{kind}/{n}/{operation}" for kind in ["character", "word"] for n in [2, 3, 4]
         for operation in ["insert", "search/0.6", "search/0.9"]}
ENV = {**os.environ, "PYTHONPATH": "", "PYTHONNOUSERSITE": "1", "PYTHONHASHSEED": "0",
       "RAYON_NUM_THREADS": "1", "CARGO_PROFILE_RELEASE_DEBUG": "2",
       "CARGO_PROFILE_RELEASE_OPT_LEVEL": "3", "CARGO_PROFILE_RELEASE_LTO": "false",
       "CARGO_PROFILE_RELEASE_CODEGEN_UNITS": "16"}


def fetch(url):
    with urlopen(Request(url, headers={"User-Agent": "simstring-release-regression"}), timeout=60) as response:
        return response.read()


def run(command, log=None, **kwargs):
    result = subprocess.run(list(map(str, command)), env=kwargs.pop("env", ENV),
                            capture_output=True, text=True, **kwargs)
    if log is not None:
        with log.open("a") as file:
            file.write(f"$ {' '.join(map(str, command))}\n{result.stdout}{result.stderr}\n")
    result.check_returncode()
    return result.stdout


def prepare(work, version):
    info = json.loads(fetch(f"https://crates.io/api/v1/crates/simstring_rust/{version}"))["version"]
    archive = fetch(f"https://crates.io/api/v1/crates/simstring_rust/{version}/download")
    if hashlib.sha256(archive).hexdigest() != info["checksum"]:
        raise ValueError("released crate checksum mismatch")
    with tarfile.open(fileobj=io.BytesIO(archive)) as package:
        package.extractall(work / "package", filter="data")
    sources = {"release": work / "package" / f"simstring_rust-{version}", "pr": work / "source"}
    shutil.copytree(ROOT, sources["pr"], ignore=shutil.ignore_patterns(
        ".git", "target", ".venv", "__pycache__", ".pytest_cache", ".simstring_cpp"))
    shutil.copy2(ROOT / "Cargo.lock", sources["release"] / "Cargo.lock")
    commands = {}
    identities = {}
    for variant, source in sources.items():
        folder = work / variant
        folder.mkdir()
        log = folder / "build.log"
        native = folder / "native"
        native.mkdir()
        manifest = f'''[package]
name = "release-check"
version = "0.0.0"
edition = "2024"
[workspace]
[dependencies]
simstring_rust = {{ path = {json.dumps(str(source))} }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
[[bin]]
name = "release-check"
path = {json.dumps(str(ROOT / "benches/release_check/worker.rs"))}
'''
        (native / "Cargo.toml").write_text(manifest)
        shutil.copy2(ROOT / "Cargo.lock", native / "Cargo.lock")
        run(["cargo", "build", "--release", "--manifest-path", native / "Cargo.toml",
             "--target-dir", native / "target"], log)
        interpreter = folder / "venv/bin/python"
        venv.EnvBuilder(with_pip=True).create(interpreter.parent.parent)
        build_env = {**ENV, "CARGO_TARGET_DIR": str(folder / "python-build")}
        command = [sys.executable, "-m", "maturin", "build", "--release", "--manifest-path",
                   source / "Cargo.toml", "--interpreter", interpreter, "--out", folder / "wheels"]
        if variant == "pr":
            command.append("--locked")
        run(command, log, env=build_env)
        wheels = list((folder / "wheels").glob("*.whl"))
        if len(wheels) != 1:
            raise ValueError(f"expected one {variant} wheel")
        run([interpreter, "-m", "pip", "install", "--no-deps", "--report", folder / "install.json", wheels[0]], log)
        identity = json.loads(run([interpreter, "-c",
            "import importlib.metadata,json,simstring_rust; print(json.dumps({'version':importlib.metadata.version('simstring-rust'),'module':simstring_rust.__file__}))"]))
        if not Path(identity["module"]).is_relative_to(interpreter.parent.parent):
            raise ValueError(f"{variant} imported outside its isolated environment")
        if variant == "release" and identity["version"] != version:
            raise ValueError("wrong released Python version")
        identity["wheel_sha256"] = hashlib.sha256(wheels[0].read_bytes()).hexdigest()
        identities[variant] = identity
        commands[variant] = {"rust": [native / "target/release/release-check"],
                             "python": [interpreter, ROOT / "benches/release_check/worker.py"]}
    return commands, {"baseline": version, "crate_checksum": info["checksum"], "packages": identities,
                      "rustc": run(["rustc", "--version"]).strip(), "python": sys.version,
                      "maturin": run([sys.executable, "-m", "maturin", "--version"]).strip(),
                      "platform": platform.platform(), "candidate": run(["git", "rev-parse", "HEAD"], cwd=ROOT).strip(),
                      "corpus_sha256": hashlib.sha256((ROOT / "benches/data/company_names.txt").read_bytes()).hexdigest(),
                      "cpu_affinity": sorted(os.sched_getaffinity(0)) if hasattr(os, "sched_getaffinity") else None,
                      "build_settings": {key: value for key, value in ENV.items() if key.startswith("CARGO_PROFILE_RELEASE_")},
                      "python_hash_seed": ENV["PYTHONHASHSEED"]}


def validate(rows, expected):
    expected = set(expected)
    if len(rows) != len(expected) or {row["case"] for row in rows} != expected:
        raise ValueError("missing, duplicate, or unexpected benchmark cases")
    for row in rows:
        if len(row["samples_ms"]) != 7 or not all(math.isfinite(value) and value > 0 for value in row["samples_ms"]):
            raise ValueError(f"invalid samples: {row['case']}")


def collect(work, commands, language, selected, phase):
    pairs = []
    for index in range(3):
        pair = {}
        for variant in (["release", "pr"] if index % 2 == 0 else ["pr", "release"]):
            rows = json.loads(run(commands[variant][language] + [ROOT / "benches/data/company_names.txt", *sorted(selected)]))
            validate(rows, selected or CASES)
            pair[variant] = {row["case"]: row for row in rows}
            with (work / "samples.jsonl").open("a") as file:
                file.write(json.dumps({"language": language, "phase": phase, "pair": index,
                                       "variant": variant, "rows": rows}) + "\n")
        for case in pair["release"]:
            if pair["release"][case]["check"] != pair["pr"][case]["check"]:
                raise ValueError(f"correctness mismatch: {language} {case}")
        pairs.append(pair)
    return pairs


def compare(pairs, case):
    values = {variant: median(value for pair in pairs for value in pair[variant][case]["samples_ms"])
              for variant in ["release", "pr"]}
    ratios = [median(pair["pr"][case]["samples_ms"]) / median(pair["release"][case]["samples_ms"]) for pair in pairs]
    return {"release_ms": values["release"], "pr_ms": values["pr"],
            "ratio": values["pr"] / values["release"], "pair_ratios": ratios}


def verdict(result):
    if result["ratio"] > 1.10 and min(result["pair_ratios"]) > 1.10:
        return "fail"
    if result["ratio"] > 1.05 and min(result["pair_ratios"]) > 1.05:
        return "warning"
    return "pass"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--baseline", help="stable crate version; defaults to latest published")
    parser.add_argument("--output", type=Path, default=ROOT / "target/release-regression")
    args = parser.parse_args()
    work = args.output.resolve()
    work.mkdir(parents=True, exist_ok=True)
    if any(work.iterdir()):
        raise ValueError("use a fresh output directory")
    try:
        version = args.baseline or json.loads(fetch("https://crates.io/api/v1/crates/simstring_rust"))["crate"]["max_stable_version"]
        print(f"Comparing insert/search with released {version}", flush=True)
        commands, metadata = prepare(work, version)
        if hasattr(os, "sched_setaffinity"):
            os.sched_setaffinity(0, {min(os.sched_getaffinity(0))})
            metadata["cpu_affinity"] = sorted(os.sched_getaffinity(0))
        (work / "metadata.json").write_text(json.dumps(metadata, indent=2))
        results = []
        for language in ["rust", "python"]:
            initial = collect(work, commands, language, [], "initial")
            comparisons = {case: compare(initial, case) for case in sorted(CASES)}
            suspects = [case for case, result in comparisons.items() if result["ratio"] > 1.05]
            repeated = collect(work, commands, language, suspects, "recheck") if suspects else None
            for case, first in comparisons.items():
                final = compare(repeated, case) if case in suspects else first
                status = verdict(final) if case in suspects else "pass"
                results.append({"language": language, "case": case, "initial": first,
                                **final, "status": status, "rechecked": case in suspects})
                if status != "pass":
                    print(f"::{status if status == 'warning' else 'error'}::{language} {case}: {(final['ratio']-1)*100:+.1f}% versus {version}")
        (work / "results.json").write_text(json.dumps(results, indent=2))
        lines = [f"## Release regression vs {version}", "",
                 "Matched local release/PR builds; company names, character/word 2–4 grams, Cosine search. "
                 "Three paired processes, two warm-ups and seven samples per case. Insert excludes input I/O and destruction; "
                 "search is milliseconds per 102-query batch. Cases above 5% are rechecked: confirmed >5% warns, confirmed >10% fails. "
                 "Both the median and every recheck pair must exceed the threshold.", "",
                 "| Binding | Case | Release (ms) | PR (ms) | Change | Result |",
                 "|---|---|---:|---:|---:|---|"]
        for row in results:
            note = " (not confirmed)" if row["rechecked"] and row["status"] == "pass" else " (rechecked)" if row["rechecked"] else ""
            lines.append(f"| {row['language']} | {row['case']} | {row['release_ms']:.3f} | {row['pr_ms']:.3f} | {(row['ratio']-1)*100:+.1f}% | {row['status']}{note} |")
        (work / "report.md").write_text("\n".join(lines) + "\n")
        return int(any(row["status"] == "fail" for row in results))
    except Exception as error:
        (work / "report.md").write_text(f"## Release regression check failed\n\n{type(error).__name__}: {error}\n")
        raise


if __name__ == "__main__":
    sys.exit(main())

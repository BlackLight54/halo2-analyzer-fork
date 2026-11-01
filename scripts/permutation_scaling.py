#!/usr/bin/env python3
"""Run the PermutationCircuit scalability sweep and aggregate it into a CSV.

One container per size runs in parallel; inside it hyperfine repeats the harness
(`permutation_scaling_run`) in a working directory of its own, because the analyzer
writes its SMT files to `src/output/` relative to the working directory. Concurrent
sizes share CPU cache and memory bandwidth, so the CSV records that they ran in
parallel.

Usage (from the repository root, after building the test binary in the image):

    scripts/permutation_scaling.py --out out/permutation-scaling
    scripts/permutation_scaling.py --dry-run

Outputs in --out: `scalability.csv`, `PROVENANCE.md`, and per size `n<N>/hyperfine.json`,
`n<N>/runs.jsonl` (one harness result per repetition) and `n<N>/log.txt`.
"""

import argparse
import csv
import datetime
import json
import os
import platform
import shlex
import signal
import statistics
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
TEST = "permutation_scaling_run"
CSV_FIELDS = [
    "n", "swaps", "equations", "advice_cells", "targets", "runs", "completed_runs",
    "time_median_s", "time_mean_s", "time_stddev_s", "time_min_s", "time_max_s",
    "smt_calls_median", "smt_calls_min", "smt_calls_max", "smt_time_median_s",
    "status", "non_unique", "unknown_uniqueness", "complete_domains", "timeout_steps",
]


def die(message):
    print(f"error: {message}", file=sys.stderr)
    sys.exit(1)


def capture(cmd):
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        die(f"{shlex.join(cmd)} failed: {result.stderr.strip()}")
    return result.stdout.strip()


def find_binary(target):
    binaries = sorted(
        (p for p in (target / "release" / "build" / "korrekt").glob("*/out/korrekt-*")
         if p.is_file() and os.access(p, os.X_OK)),
        key=lambda p: p.stat().st_mtime,
    )
    if not binaries:
        die(f"no korrekt test binary under {target}; build it in the image first")
    return binaries[-1]


def container_command(args, n, binary, out):
    inner = (
        f"mkdir -p /run/src/output && hyperfine --runs {args.runs} --warmup 0 -i "
        f"--export-json /run/hyperfine.json "
        f"{shlex.quote(f'timeout {args.run_limit_s} {binary} {TEST} --ignored --nocapture --test-threads=1')}"
    )
    return [
        "docker", "run", "--rm", "--name", f"perm-scaling-n{n}",
        "--user", f"{os.getuid()}:{os.getgid()}",
        "-v", f"{args.target}:/target:ro", "-v", f"{out / f'n{n}'}:/run", "-w", "/run",
        "-e", f"PERM_SCALING_N={n}", "-e", "PERM_SCALING_OUT=/run/runs.jsonl",
        "--entrypoint", "sh", args.image, "-c", inner,
    ]


def summarize(n, run_dir, runs):
    hyperfine = json.loads((run_dir / "hyperfine.json").read_text())["results"][0]
    exit_codes = hyperfine.get("exit_codes", [])
    results = [json.loads(line) for line in (run_dir / "runs.jsonl").read_text().splitlines() if line]
    if not results:
        return {"n": n, "runs": runs, "completed_runs": 0, "status": "no-result"}
    first = results[0]
    for key in ("swaps", "equations", "advice_cells", "targets"):
        if any(r[key] != first[key] for r in results):
            die(f"n={n}: {key} differs between repetitions")
    outcome = ("status", "non_unique", "unknown_uniqueness", "complete_domains", "timeout_steps")
    outcomes = {tuple(r[k] for k in outcome) for r in results}
    calls = [r["smt_calls"] for r in results]
    times = hyperfine["times"]
    return {
        "n": n, **{k: first[k] for k in ("swaps", "equations", "advice_cells", "targets")},
        "runs": len(times), "completed_runs": sum(1 for c in exit_codes if c == 0),
        "time_median_s": f"{hyperfine['median']:.3f}", "time_mean_s": f"{hyperfine['mean']:.3f}",
        "time_stddev_s": f"{hyperfine['stddev']:.3f}" if hyperfine.get("stddev") is not None else "",
        "time_min_s": f"{hyperfine['min']:.3f}", "time_max_s": f"{hyperfine['max']:.3f}",
        "smt_calls_median": statistics.median(calls), "smt_calls_min": min(calls),
        "smt_calls_max": max(calls),
        "smt_time_median_s": f"{statistics.median(r['smt_ms'] for r in results) / 1000:.3f}",
        # Differing outcomes between repetitions are reported, never silently merged.
        **({k: first[k] for k in outcome} if len(outcomes) == 1
           else {k: "varies" for k in outcome}),
    }


def provenance(args, binary, out, started, finished):
    sha = capture(["git", "-C", str(REPO), "rev-parse", "HEAD"])
    dirty = capture(["git", "-C", str(REPO), "status", "--porcelain"])
    image = capture(["docker", "image", "inspect", args.image, "--format", "{{.Id}}"])
    tools = capture(["docker", "run", "--rm", "--entrypoint", "sh", args.image, "-c",
                     "cvc5 --version | head -1; rustc --version; hyperfine --version"])
    cpu = next((l.split(":", 1)[1].strip() for l in Path("/proc/cpuinfo").read_text().splitlines()
                if l.startswith("model name")), platform.processor())
    command = shlex.join([sys.argv[0], *sys.argv[1:]])
    (out / "PROVENANCE.md").write_text(
        f"# Permutation scalability sweep\n\n"
        f"- Started: {started}\n- Finished: {finished}\n"
        f"- Command: `{command}`\n"
        f"- Code: {sha}{' (working tree dirty)' if dirty else ''}\n"
        f"- Test binary: `{binary.name}`, test `{TEST}`\n"
        f"- Image: `{args.image}` ({image})\n"
        f"- Tools: {'; '.join(tools.splitlines())}\n"
        f"- Machine: {platform.node()}, {cpu}, {os.cpu_count()} logical CPUs\n"
        f"- Sizes {args.sizes} ran in parallel, one container each; repetitions within a size ran sequentially.\n"
        f"- Repetitions per size: {args.runs}; hard limit per repetition: {args.run_limit_s} s (`timeout`).\n"
        f"- Configuration: thesis-example Standard profile (3000 ms per solver call; "
        f"256 semantic, 160 refinement, 64 discovery calls).\n"
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--sizes", default="3,4,5,6,7,8")
    parser.add_argument("--runs", type=int, default=5)
    parser.add_argument("--run-limit-s", type=int, default=1800)
    parser.add_argument("--image", default="halo2-analyzer-fork:cleanup")
    parser.add_argument("--target", type=Path, default=REPO / "target",
                        help="cargo target dir that holds the binary built in the image")
    parser.add_argument("--out", type=Path, default=REPO / "out" / "permutation-scaling")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    sizes = [int(s) for s in args.sizes.split(",")]
    if any(n < 3 or n > 8 for n in sizes):
        die("sizes must be within 3..=8")
    args.target = args.target.resolve()
    out = args.out.resolve()
    binary = "/target" / find_binary(args.target).relative_to(args.target)
    commands = {n: container_command(args, n, binary, out) for n in sizes}
    if args.dry_run:
        for cmd in commands.values():
            print(shlex.join(cmd))
        return
    for n in sizes:
        run_dir = out / f"n{n}"
        if run_dir.exists() and any(run_dir.iterdir()):
            die(f"{run_dir} is not empty; choose a fresh --out")
        run_dir.mkdir(parents=True)
    started = datetime.datetime.now().astimezone().isoformat(timespec="seconds")
    procs = {n: subprocess.Popen(cmd, stdout=open(out / f"n{n}" / "log.txt", "w"),
                                 stderr=subprocess.STDOUT) for n, cmd in commands.items()}
    (out / "pids.txt").write_text("".join(f"n{n} {p.pid} perm-scaling-n{n}\n" for n, p in procs.items()))

    def stop(*_):
        for n in procs:
            subprocess.run(["docker", "kill", f"perm-scaling-n{n}"], capture_output=True)
        sys.exit(130)

    signal.signal(signal.SIGINT, stop)
    signal.signal(signal.SIGTERM, stop)
    failed = [n for n, p in procs.items() if p.wait() != 0]
    finished = datetime.datetime.now().astimezone().isoformat(timespec="seconds")
    if failed:
        die(f"hyperfine failed for sizes {failed}; see {out}/n<N>/log.txt")
    rows = [summarize(n, out / f"n{n}", args.runs) for n in sizes]
    with open(out / "scalability.csv", "w", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=CSV_FIELDS)
        writer.writeheader()
        writer.writerows(rows)
    provenance(args, find_binary(args.target), out, started, finished)
    print(f"wrote {out / 'scalability.csv'} and {out / 'PROVENANCE.md'}")


if __name__ == "__main__":
    main()

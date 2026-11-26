#!/usr/bin/env python3
"""Deterministic scale gate: instruction counts of one full recompute, against a committed baseline.

Wall-clock time on a shared CI runner is noisy, and a 10 s budget set for 4 vCPU says nothing on a
2 vCPU runner. Instruction counts under Valgrind's Callgrind repeat to within about 0.01%. One count
is not enough: moving work out of rayon's parallel stages into the serial path barely changes the
total, yet it costs most of the wall time. So the gate counts two things in one run, with rayon on
two threads (Valgrind serialises them, but counts each separately):

  serial    instructions inside `FullRecompute::recompute` on the main thread, which only blocks
            while the pool works, so this is the recompute's serial path;
  parallel  instructions on rayon's worker threads.

Either one growing by more than the tolerance is a regression (exit 1). `--update` records the
counts when nothing regressed; `--ci` also fails when a count fell by more than the tolerance and
the baseline doesn't record it, as the report ratchet does (D66). The absolute budget
(docs/v1-spec.md §10.4) is timed on a schedule instead (.github/workflows/scale-slo.yml).

Usage: python3 scripts/scale-gate.py [--update | --ci] [--binary target/release/examples/scale]
Needs valgrind (Callgrind and callgrind_annotate) and a release build of the scale example.
"""

import argparse
import glob
import json
import os
import re
import subprocess
import sys
import tempfile

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BASELINE = os.path.join(REPO, "crates/seikika-derive/examples/scale-gate.json")
FORMAT = "seikika-scale-gate/v1"
RECOMPUTE = "<seikika_derive::FullRecompute as seikika_derive::DerivedState>::recompute"


def measure(binary, services, threads):
    """Run the scale example once under Callgrind; return (serial, parallel) instruction counts."""
    out = tempfile.mkdtemp(prefix="scale-gate-")
    env = dict(os.environ, RAYON_NUM_THREADS=str(threads), SCALE_RUNS="1", SCALE_BUDGET_SECS="1000000")
    run = subprocess.run(
        ["valgrind", "--tool=callgrind", "--separate-threads=yes",
         f"--callgrind-out-file={out}/cg.out", binary, str(services)],
        env=env, capture_output=True, text=True)
    if run.returncode != 0:
        sys.exit(f"scale example failed under valgrind:\n{run.stdout}\n{run.stderr}")
    print(run.stdout.strip())
    files = sorted(glob.glob(f"{out}/cg.out-*"))
    if len(files) < 2:
        sys.exit(f"expected one Callgrind file per thread, found {files}")
    totals = {}
    for f in files:
        m = re.search(r"^summary:\s*(\d+)", open(f).read(), re.M)
        totals[f] = int(m.group(1)) if m else 0
    main = files[0]  # thread 1
    annotate = subprocess.run(["callgrind_annotate", "--inclusive=yes", "--threshold=100", main],
                              capture_output=True, text=True, check=True).stdout
    serial = next((int(row.split()[0].replace(",", "")) for row in annotate.splitlines() if RECOMPUTE in row), None)
    if serial is None:
        sys.exit(f"no {RECOMPUTE} in the main thread's profile")
    return serial, sum(v for f, v in totals.items() if f != main)


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    mode = ap.add_mutually_exclusive_group()
    mode.add_argument("--update", action="store_true", help="record the counts when nothing regressed")
    mode.add_argument("--ci", action="store_true", help="also fail when an improvement isn't recorded")
    ap.add_argument("--binary", default=os.path.join(REPO, "target/release/examples/scale"))
    a = ap.parse_args()

    base = json.load(open(BASELINE)) if os.path.exists(BASELINE) else None
    services = base["services"] if base else 10_000
    threads = base["threads"] if base else 2
    tol = (base["tolerancePercent"] if base else 2) / 100
    serial, parallel = measure(a.binary, services, threads)
    now = {"serial": serial, "parallel": parallel}
    print(f"instructions: serial {serial:,}, parallel {parallel:,} ({services} services, {threads} threads)")

    worse, better = [], []
    for k, v in now.items():
        if base is None:
            continue
        was = base[k]
        change = (v - was) / was
        line = f"{k} {was:,} → {v:,} ({change:+.2%})"
        if change > tol:
            worse.append(line)
        elif change < -tol:
            better.append(line)
        else:
            print(f"· same    {line}")
    for change in worse:
        print(f"✗ worse   {change}")
    for change in better:
        print(f"✓ better  {change}")

    def write():
        doc = {"format": FORMAT, "services": services, "threads": threads,
               "tolerancePercent": round(tol * 100), **now}
        with open(BASELINE, "w") as f:
            json.dump(doc, f, indent=2, sort_keys=True)
            f.write("\n")

    if worse:
        print(f"scale gate: {len(worse)} regression(s) against {os.path.relpath(BASELINE, REPO)}")
        return 1
    if base is None or (a.update and better):
        if not a.update:
            print("scale gate: no baseline; create it with --update")
            return 1 if a.ci else 0
        write()
        print(f"scale gate: recorded {os.path.relpath(BASELINE, REPO)}")
        return 0
    if better:
        print("scale gate: no regressions, but the baseline doesn't record the improvement: run with --update and commit it")
        return 1 if a.ci else 0
    print("scale gate: within tolerance of the baseline")
    return 0


if __name__ == "__main__":
    sys.exit(main())

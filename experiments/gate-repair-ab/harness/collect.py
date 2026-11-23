#!/usr/bin/env python3
"""Score and measure a round's six runs; write results/r<round>.tsv and one JSON per run.

    collect.py <round> <transcripts-dir> <arm>-<n>=<bcId> ...

<transcripts-dir> is the directory `batch-fetch-details` wrote, with <bcId>/transcript.json.
"""
import json
import pathlib
import statistics
import subprocess
import sys

HERE = pathlib.Path(__file__).resolve().parent


def main() -> None:
    rnd, transcripts, runs = sys.argv[1], pathlib.Path(sys.argv[2]), sys.argv[3:]
    out = HERE.parent / "results" / f"r{rnd}"
    out.mkdir(parents=True, exist_ok=True)
    rows = []
    for spec in runs:
        run, bc = spec.split("=")
        arm = run.rsplit("-", 1)[0]
        dest = f"/tmp/gate-repair-ab/r{rnd}/{run}"
        score = json.loads(subprocess.run([sys.executable, HERE / "score.py", arm, dest],
                                          capture_output=True, text=True, check=True).stdout)
        count = json.loads(subprocess.run([sys.executable, HERE / "count.py", arm, transcripts / bc / "transcript.json"],
                                          capture_output=True, text=True, check=True).stdout)
        diff = subprocess.run(["git", "diff", "main", "--", ".github"], cwd=f"{dest}/openfga",
                              capture_output=True, text=True, check=True).stdout
        (out / f"{run}.json").write_text(json.dumps({"run": run, "bcId": bc, **count, "score": score}, indent=2) + "\n")
        (out / f"{run}.diff").write_text(diff)
        rows.append((run, arm, bc, score["correct"], count))
    lines = ["run\tarm\tbcId\tcorrect\tcalls\tread\tgateRuns\twallSeconds"]
    for run, arm, bc, correct, c in rows:
        lines.append(f"{run}\t{arm}\t{bc}\t{correct}\t{c['toolCalls']}\t{c['resultChars']}\t{c['gateRuns']}\t{c['wallSeconds']}")
    (HERE.parent / "results" / f"r{rnd}.tsv").write_text("\n".join(lines) + "\n")
    print("\n".join(lines))
    for arm in sorted({r[1] for r in rows}):
        for key in ("toolCalls", "resultChars"):
            xs = [r[4][key] for r in rows if r[1] == arm]
            print(f"{arm} {key}: min {min(xs)} median {statistics.median(xs)} max {max(xs)} range {max(xs) - min(xs)}")


if __name__ == "__main__":
    main()

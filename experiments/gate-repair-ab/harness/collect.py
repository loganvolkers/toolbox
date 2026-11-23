#!/usr/bin/env python3
"""Score and measure a round's runs; write results/r<round>.tsv and one JSON per run.

    collect.py <round> <transcripts-dir> <run>=<bcId> ...

<transcripts-dir> is the directory `batch-fetch-details` wrote, with <bcId>/transcript.json. A run's
arm is read from prompts/rendered/r<round>/arms.tsv when `round.sh` wrote one, and is otherwise the
run's name without its number. Arms named `toolbox…` are scored and counted as Toolbox.

*read* is `resultCharsImputed`: a `read_file` whose result the transcript lacks counts as that file
on `pr` (count.py). `readRecorded` is the read as recorded, and `missing` names the calls without a
result. Rounds 1 to 4 were written before these columns, with *read* as recorded.
"""
import json
import pathlib
import statistics
import subprocess
import sys

HERE = pathlib.Path(__file__).resolve().parent
MEASURES = ("toolCalls", "resultCharsImputed", "resultChars", "gateChars")


def arms(rnd: str) -> dict:
    path = HERE.parent / "prompts" / "rendered" / f"r{rnd}" / "arms.tsv"
    if not path.exists():
        return {}
    return dict(line.split("\t") for line in path.read_text().splitlines() if line)


def main() -> None:
    rnd, transcripts, runs = sys.argv[1], pathlib.Path(sys.argv[2]), sys.argv[3:]
    out = HERE.parent / "results" / f"r{rnd}"
    out.mkdir(parents=True, exist_ok=True)
    arm_of = arms(rnd)
    rows = []
    for spec in runs:
        run, bc = spec.split("=")
        arm = arm_of.get(run, run.rsplit("-", 1)[0])
        kind = "toolbox" if arm.startswith("toolbox") else arm
        dest = f"/tmp/gate-repair-ab/r{rnd}/{run}"
        score = json.loads(subprocess.run([sys.executable, HERE / "score.py", kind, dest],
                                          capture_output=True, text=True, check=True).stdout)
        count = json.loads(subprocess.run([sys.executable, HERE / "count.py", kind, transcripts / bc / "transcript.json",
                                           f"{dest}/openfga"],
                                          capture_output=True, text=True, check=True).stdout)
        diff = subprocess.run(["git", "diff", "main", "--", ".github"], cwd=f"{dest}/openfga",
                              capture_output=True, text=True, check=True).stdout
        (out / f"{run}.json").write_text(json.dumps({"run": run, "arm": arm, "bcId": bc, **count, "score": score},
                                                    indent=2) + "\n")
        (out / f"{run}.diff").write_text(diff)
        rows.append((run, arm, bc, score["correct"], count))
    lines = ["run\tarm\tbcId\tcorrect\tcalls\tread\treadRecorded\tmissing\tgateRuns\tgateRead\twallSeconds"]
    for run, arm, bc, correct, c in rows:
        missing = ",".join(c["missingResults"]) or "-"
        lines.append(f"{run}\t{arm}\t{bc}\t{correct}\t{c['toolCalls']}\t{c['resultCharsImputed']}\t{c['resultChars']}"
                     f"\t{missing}\t{c['gateRuns']}\t{c['gateChars']}\t{c['wallSeconds']}")
    (HERE.parent / "results" / f"r{rnd}.tsv").write_text("\n".join(lines) + "\n")
    print("\n".join(lines))
    for arm in sorted({r[1] for r in rows}):
        for key in MEASURES:
            xs = [r[4][key] for r in rows if r[1] == arm]
            print(f"{arm} {key}: min {min(xs)} median {statistics.median(xs)} max {max(xs)} range {max(xs) - min(xs)}")


if __name__ == "__main__":
    main()

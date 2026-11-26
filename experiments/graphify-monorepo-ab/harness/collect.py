#!/usr/bin/env python3
"""Measure a round's runs; write results/r<round>.tsv, one JSON per run, and each run's answers.

    collect.py <round> <transcripts-dir> <run>=<bcId> ...

<transcripts-dir> is the directory `batch-fetch-details` wrote, with <bcId>/transcript.json. A
run's arm is read from prompts/rendered/r<round>/arms.tsv. Each run's ANSWERS.md is copied to
results/r<round>/<run>.answers.md; `blind.py` shuffles them for the adjudicator. Scores are not
computed here: the adjudicator grades them (README §4). Adapted from graphify-ab.
"""
import json
import pathlib
import shutil
import statistics
import subprocess
import sys

HERE = pathlib.Path(__file__).resolve().parent
MEASURES = ("toolCalls", "resultCharsImputed", "graphifyCalls", "seikikaCalls", "wallSeconds")


def main() -> None:
    rnd, transcripts, runs = sys.argv[1], pathlib.Path(sys.argv[2]), sys.argv[3:]
    out = HERE.parent / "results" / f"r{rnd}"
    out.mkdir(parents=True, exist_ok=True)
    arms_tsv = HERE.parent / "prompts" / "rendered" / f"r{rnd}" / "arms.tsv"
    arm_of = {line.split("\t")[0]: line.split("\t")[1] for line in arms_tsv.read_text().splitlines() if line}
    rows = []
    for spec in runs:
        run, bc = spec.split("=")
        dest = pathlib.Path(f"/tmp/graphify-monorepo-ab/r{rnd}/{run}")
        count = json.loads(subprocess.run([sys.executable, HERE / "count.py", transcripts / bc / "transcript.json",
                                           str(dest)], capture_output=True, text=True, check=True).stdout)
        answers = dest / "ANSWERS.md"
        if answers.exists():
            shutil.copy(answers, out / f"{run}.answers.md")
        (out / f"{run}.json").write_text(json.dumps({"run": run, "arm": arm_of[run], "bcId": bc,
                                                     "answered": answers.exists(), **count}, indent=2) + "\n")
        rows.append((run, arm_of[run], bc, answers.exists(), count))
    lines = ["run\tarm\tbcId\tanswered\tcalls\tread\treadRecorded\tmissing\tgraphifyCalls\tgraphifyRead"
             "\tseikikaCalls\tseikikaRead\toutside\twallSeconds"]
    for run, arm, bc, answered, c in sorted(rows, key=lambda r: int(r[0].rsplit("-", 1)[1])):
        lines.append(f"{run}\t{arm}\t{bc}\t{answered}\t{c['toolCalls']}\t{c['resultCharsImputed']}"
                     f"\t{c['resultChars']}\t{','.join(c['missingResults']) or '-'}\t{c['graphifyCalls']}"
                     f"\t{c['graphifyChars']}\t{c['seikikaCalls']}\t{c['seikikaChars']}"
                     f"\t{len(c['outsidePaths'])}\t{c['wallSeconds']}")
    (HERE.parent / "results" / f"r{rnd}.tsv").write_text("\n".join(lines) + "\n")
    print("\n".join(lines))
    for arm in ("A", "B", "C"):
        sel = [r[4] for r in rows if r[1] == arm]
        for key in MEASURES:
            xs = [c[key] for c in sel if c[key] is not None]
            if xs:
                print(f"{arm} {key}: min {min(xs)} median {statistics.median(xs)} max {max(xs)} "
                      f"range {max(xs) - min(xs)}")


if __name__ == "__main__":
    main()

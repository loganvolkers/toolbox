#!/usr/bin/env python3
"""Measure a round's runs; write results/r<round>.tsv, one JSON per run, and each run's answers.

    collect.py <round> <transcripts-dir> <run>=<bcId> ...

<transcripts-dir> is the directory `batch-fetch-details` wrote, with <bcId>/transcript.json. A
run's arm is read from prompts/rendered/r<round>/arms.tsv. Each run's ANSWERS.md is copied to
results/r<round>/<run>.answers.md; `blind.py` shuffles them for the adjudicator. Coverage is not
computed here: the adjudicator grades it (README §4).
"""
import json
import pathlib
import shutil
import statistics
import subprocess
import sys

HERE = pathlib.Path(__file__).resolve().parent
MEASURES = ("toolCalls", "resultCharsImputed", "graphifyCalls", "wallSeconds")


def main() -> None:
    rnd, transcripts, runs = sys.argv[1], pathlib.Path(sys.argv[2]), sys.argv[3:]
    out = HERE.parent / "results" / f"r{rnd}"
    out.mkdir(parents=True, exist_ok=True)
    arms_tsv = HERE.parent / "prompts" / "rendered" / f"r{rnd}" / "arms.tsv"
    arm_of = dict(line.split("\t") for line in arms_tsv.read_text().splitlines() if line)
    rows = []
    for spec in runs:
        run, bc = spec.split("=")
        dest = pathlib.Path(f"/tmp/graphify-ab/r{rnd}/{run}")
        count = json.loads(subprocess.run([sys.executable, HERE / "count.py", transcripts / bc / "transcript.json",
                                           str(dest)], capture_output=True, text=True, check=True).stdout)
        answers = dest / "ANSWERS.md"
        if answers.exists():
            shutil.copy(answers, out / f"{run}.answers.md")
        (out / f"{run}.json").write_text(json.dumps({"run": run, "arm": arm_of[run], "bcId": bc,
                                                     "answered": answers.exists(), **count}, indent=2) + "\n")
        rows.append((run, arm_of[run], bc, answers.exists(), count))
    lines = ["run\tarm\tbcId\tanswered\tcalls\tread\treadRecorded\tmissing\tgraphifyCalls\tgraphifyRead"
             "\toutside\twallSeconds"]
    for run, arm, bc, answered, c in sorted(rows):
        lines.append(f"{run}\t{arm}\t{bc}\t{answered}\t{c['toolCalls']}\t{c['resultCharsImputed']}"
                     f"\t{c['resultChars']}\t{','.join(c['missingResults']) or '-'}\t{c['graphifyCalls']}"
                     f"\t{c['graphifyChars']}\t{len(c['outsidePaths'])}\t{c['wallSeconds']}")
    (HERE.parent / "results" / f"r{rnd}.tsv").write_text("\n".join(lines) + "\n")
    print("\n".join(lines))
    for repo in sorted({r[0].rsplit("-", 1)[0] for r in rows}):
        for arm in ("A", "B"):
            sel = [r[4] for r in rows if r[1] == arm and r[0].startswith(repo + "-")]
            for key in MEASURES:
                xs = [c[key] for c in sel if c[key] is not None]
                if xs:
                    print(f"{repo} {arm} {key}: min {min(xs)} median {statistics.median(xs)} max {max(xs)} "
                          f"range {max(xs) - min(xs)}")


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Shuffle a round's answers under random labels for blind grading (README §4).

    blind.py <round>

Reads results/r<round>/<run>.answers.md and writes /tmp/graphify-ab/blind-r<round>/<repo>/<label>.md,
with every "graphify" (any case) replaced by "[tool]", and the label-to-run key to
/tmp/graphify-ab/blind-r<round>.key.tsv. The adjudicator grades the blind directory first and is
given the key afterwards; the key is committed to results/ once the grades are.
"""
import pathlib
import random
import re
import secrets
import sys

HERE = pathlib.Path(__file__).resolve().parent


def main() -> None:
    rnd = sys.argv[1]
    src = HERE.parent / "results" / f"r{rnd}"
    out = pathlib.Path(f"/tmp/graphify-ab/blind-r{rnd}")
    rng = random.Random(secrets.randbits(64))
    key = []
    for repo in ("openfga", "toolbox"):
        runs = sorted(src.glob(f"{repo}-*.answers.md"))
        labels = rng.sample(range(100, 1000), len(runs))
        (out / repo).mkdir(parents=True, exist_ok=True)
        for path, label in zip(runs, labels):
            text = re.sub(r"graphify", "[tool]", path.read_text(), flags=re.IGNORECASE)
            (out / repo / f"{label}.md").write_text(text)
            key.append(f"{repo}\t{label}\t{path.name.removesuffix('.answers.md')}")
    pathlib.Path(f"/tmp/graphify-ab/blind-r{rnd}.key.tsv").write_text("\n".join(sorted(key)) + "\n")
    print(f"{len(key)} answers in {out}")


if __name__ == "__main__":
    main()

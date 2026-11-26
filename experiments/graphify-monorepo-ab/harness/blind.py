#!/usr/bin/env python3
"""Shuffle a round's answers under random labels for blind grading (README §4).

    blind.py <round>

Reads results/r<round>/<run>.answers.md and writes /tmp/graphify-monorepo-ab/blind-r<round>/<label>.md,
with every "graphify" and "seikika" (any case) replaced by "[tool]" and the portal's `crate:` id
prefix removed, and the label-to-run key to /tmp/graphify-monorepo-ab/blind-r<round>.key.tsv. The
adjudicator grades the blind directory first and is given the key afterwards; the key is committed
to results/ once the grades are. Adapted from graphify-ab.
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
    out = pathlib.Path(f"/tmp/graphify-monorepo-ab/blind-r{rnd}")
    out.mkdir(parents=True, exist_ok=True)
    rng = random.Random(secrets.randbits(64))
    runs = sorted(src.glob("codex-*.answers.md"))
    labels = rng.sample(range(100, 1000), len(runs))
    key = []
    for path, label in zip(runs, labels):
        text = re.sub(r"graphify|seikika", "[tool]", path.read_text(), flags=re.IGNORECASE)
        text = re.sub(r"\bcrate:(?=[\w-])", "", text)
        (out / f"{label}.md").write_text(text)
        key.append(f"{label}\t{path.name.removesuffix('.answers.md')}")
    pathlib.Path(f"/tmp/graphify-monorepo-ab/blind-r{rnd}.key.tsv").write_text("\n".join(sorted(key)) + "\n")
    print(f"{len(key)} answers in {out}")


if __name__ == "__main__":
    main()

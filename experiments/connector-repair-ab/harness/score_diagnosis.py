#!/usr/bin/env python3
"""Score OUT/diagnosis.md recall against seeded bugs.

    python3 -I score_diagnosis.py <arm_dir> <A|B|C>
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
BUGS = HERE.parent / "bugs"
KEYWORDS = json.loads((BUGS / "keywords.json").read_text())


def recall(text: str) -> dict:
    found = {}
    lower = text.lower()
    for bug_id, keys in KEYWORDS.items():
        hit = any(k.lower() in lower for k in keys)
        # also accept explicit bug id
        if re.search(re.escape(bug_id), text, re.I):
            hit = True
        found[bug_id] = hit
    total = len(KEYWORDS)
    n = sum(1 for v in found.values() if v)
    return {
        "found": found,
        "recall": n / total if total else 0.0,
        "named": n,
        "total": total,
    }


def main() -> int:
    if len(sys.argv) != 3:
        print(__doc__, file=sys.stderr)
        return 2
    arm = Path(sys.argv[1])
    letter = sys.argv[2]
    path = arm / "OUT" / "diagnosis.md"
    report: dict = {"arm": str(arm), "letter": letter, "path": str(path)}
    if not path.exists():
        report["ok"] = False
        report["detail"] = "OUT/diagnosis.md missing"
        print(json.dumps(report, indent=2))
        return 1
    text = path.read_text()
    report.update(recall(text))
    report["ok"] = report["named"] >= 1  # phase gate: at least one bug named
    print(json.dumps(report, indent=2))
    return 0 if report["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())

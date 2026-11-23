#!/usr/bin/env python3
"""Score OUT/read-answers.md against harness/read-key.md.

    python3 -I experiments/connector-ab/harness/score_read.py <arm_dir>
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

CHECKS = [
    ("projects", re.compile(r"\b1\b"), ["gamma", "project"]),
    ("issues", re.compile(r"gamma-1", re.I), ["gamma-2"]),
    ("assignee", re.compile(r"u-cara", re.I), []),
    ("component", re.compile(r"\bcore\b", re.I), []),
    ("config", re.compile(r"(baseUrl|FORGE_BASE_URL|token|FORGE_TOKEN)", re.I), []),
]


def main() -> int:
    if len(sys.argv) != 2:
        print(__doc__, file=sys.stderr)
        return 2
    arm = Path(sys.argv[1])
    path = arm / "OUT" / "read-answers.md"
    report = {"arm": str(arm), "path": str(path)}
    if not path.exists():
        report["ok"] = False
        report["detail"] = "OUT/read-answers.md missing"
        print(json.dumps(report, indent=2))
        return 1
    text = path.read_text()
    results = []
    for name, must, also in CHECKS:
        ok = bool(must.search(text)) and all(
            re.search(a, text, re.I) for a in also
        )
        results.append({"check": name, "ok": ok})
    report["checks"] = results
    report["ok"] = all(r["ok"] for r in results)
    print(json.dumps(report, indent=2))
    return 0 if report["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())

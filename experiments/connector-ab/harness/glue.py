#!/usr/bin/env python3
"""Count lines of glue an arm added (source + config the team would maintain).

    python3 -I experiments/connector-ab/harness/glue.py <arm_dir>

Ignores OUT/, target/, node_modules/, __pycache__, Cargo.lock, and the
scaffold-provided FORGE.md / OCEAN_ROOT.txt / BACKSTAGE_ROOT.txt.
"""

from __future__ import annotations

import sys
from pathlib import Path

SKIP_DIRS = {"OUT", "target", "node_modules", "__pycache__", "dist", ".git"}
SKIP_FILES = {
    "FORGE.md",
    "OCEAN_ROOT.txt",
    "BACKSTAGE_ROOT.txt",
    "Cargo.lock",
    "package-lock.json",
}
CODE_EXT = {
    ".rs",
    ".py",
    ".ts",
    ".tsx",
    ".js",
    ".mjs",
    ".cjs",
    ".yaml",
    ".yml",
    ".json",
    ".toml",
    ".md",
}


def main() -> int:
    if len(sys.argv) != 2:
        print(__doc__, file=sys.stderr)
        return 2
    root = Path(sys.argv[1])
    total = 0
    files = 0
    for p in sorted(root.rglob("*")):
        if not p.is_file():
            continue
        if any(part in SKIP_DIRS for part in p.parts):
            continue
        if p.name in SKIP_FILES:
            continue
        if p.suffix not in CODE_EXT and p.name not in {"RUN.md"}:
            continue
        try:
            n = len(p.read_text(errors="ignore").splitlines())
        except OSError:
            continue
        total += n
        files += 1
        print(f"{n:5d}  {p.relative_to(root)}")
    print(f"TOTAL\t{total}\tfiles={files}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

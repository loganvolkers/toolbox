#!/usr/bin/env python3
"""Wrapper onto connector-ab score_read."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

AB = Path(__file__).resolve().parent.parent.parent / "connector-ab" / "harness" / "score_read.py"


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: score_read.py <arm_dir>", file=sys.stderr)
        return 2
    return subprocess.call([sys.executable, "-I", str(AB), sys.argv[1]])


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""Score repair-ab arm truth+shape (always with-components / phase 3 of connector-ab).

    python3 -I score.py <arm_dir> <tenant a|b>
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
AB_SCORE = HERE.parent.parent / "connector-ab" / "harness" / "score.py"


def main() -> int:
    if len(sys.argv) != 3:
        print(__doc__, file=sys.stderr)
        return 2
    arm = sys.argv[1]
    tenant = sys.argv[2]
    # Phase 3 ⇒ expected-with-components.ndjson
    return subprocess.call(
        [sys.executable, "-I", str(AB_SCORE), arm, tenant, "3"]
    )


if __name__ == "__main__":
    raise SystemExit(main())

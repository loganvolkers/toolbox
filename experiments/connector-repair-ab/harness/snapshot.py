#!/usr/bin/env python3
"""Hash arm sources so phase 1 can assert the agent did not fix yet.

    python3 -I snapshot.py write <REPAIR_HOME>
    python3 -I snapshot.py check <REPAIR_HOME> <A|B|C>
"""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

SKIP_DIRS = {"OUT", "target", "node_modules", "__pycache__", "dist", ".git"}
SKIP_NAMES = {"Cargo.lock", "package-lock.json"}


def file_digest(root: Path) -> dict[str, str]:
    out: dict[str, str] = {}
    for p in sorted(root.rglob("*")):
        if not p.is_file():
            continue
        if any(part in SKIP_DIRS for part in p.parts):
            continue
        if p.name in SKIP_NAMES:
            continue
        rel = str(p.relative_to(root))
        out[rel] = hashlib.sha256(p.read_bytes()).hexdigest()
    return out


def main() -> int:
    if len(sys.argv) < 3:
        print(__doc__, file=sys.stderr)
        return 2
    cmd = sys.argv[1]
    home = Path(sys.argv[2])
    snap_dir = home / "snapshots"
    if cmd == "write":
        snap_dir.mkdir(parents=True, exist_ok=True)
        for arm in ("A", "B", "C"):
            dig = file_digest(home / "arms" / arm)
            (snap_dir / f"{arm}.json").write_text(json.dumps(dig, indent=2, sort_keys=True))
            print(f"wrote {arm} ({len(dig)} files)")
        return 0
    if cmd == "check":
        if len(sys.argv) != 4:
            print(__doc__, file=sys.stderr)
            return 2
        arm = sys.argv[3]
        want = json.loads((snap_dir / f"{arm}.json").read_text())
        got = file_digest(home / "arms" / arm)
        if want == got:
            print(json.dumps({"ok": True, "arm": arm, "files": len(got)}))
            return 0
        changed = sorted(set(want) | set(got))
        diffs = [p for p in changed if want.get(p) != got.get(p)]
        print(json.dumps({"ok": False, "arm": arm, "changed": diffs[:20]}))
        return 1
    print(__doc__, file=sys.stderr)
    return 2


if __name__ == "__main__":
    raise SystemExit(main())

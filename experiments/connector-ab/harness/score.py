#!/usr/bin/env python3
"""Score an arm's OUT/entities.ndjson against the fixture truth, and check framework shape.

    python3 -I experiments/connector-ab/harness/score.py <arm_dir> <tenant a|b> <phase 1-5>
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
FIX = HERE.parent / "fixtures"


def load_ndjson(path: Path) -> list[dict]:
    rows = []
    for line in path.read_text().splitlines():
        line = line.strip()
        if not line:
            continue
        rows.append(json.loads(line))
    rows.sort(key=lambda r: (r["kind"], r["id"]))
    return rows


def expected(tenant: str, phase: int) -> list[dict]:
    suffix = "with-components" if phase >= 3 else "base"
    return load_ndjson(FIX / f"tenant-{tenant}" / f"expected-{suffix}.ndjson")


def truth_ok(got: list[dict], want: list[dict]) -> tuple[bool, str]:
    if got == want:
        return True, f"exact match ({len(want)} entities)"
    # Diff summary
    gset = {(r["kind"], r["id"]): r for r in got}
    wset = {(r["kind"], r["id"]): r for r in want}
    missing = sorted(set(wset) - set(gset))
    extra = sorted(set(gset) - set(wset))
    mismatched = []
    for k in sorted(set(gset) & set(wset)):
        if gset[k] != wset[k]:
            mismatched.append(k)
    return False, f"missing={missing} extra={extra} mismatched={mismatched}"


def shape_seikika(arm: Path) -> tuple[bool, str]:
    # Look for run_stdio / Source / register in rust or any language speaking the protocol.
    hits = []
    for p in arm.rglob("*"):
        if p.is_file() and p.suffix in {".rs", ".py", ".ts", ".js", ".go"}:
            t = p.read_text(errors="ignore")
            if "run_stdio" in t or '"type": "register"' in t or "snapshot_complete" in t:
                hits.append(str(p.relative_to(arm)))
    if not hits:
        return False, "no stdio connector protocol usage found"
    # entities.ndjson must exist
    if not (arm / "OUT" / "entities.ndjson").exists():
        return False, "OUT/entities.ndjson missing"
    return True, f"protocol refs in {hits[:5]}"


def shape_ocean(arm: Path) -> tuple[bool, str]:
    text = ""
    files = list(arm.rglob("*.py"))
    for p in files:
        text += p.read_text(errors="ignore") + "\n"
    kinds = set(re.findall(r"@ocean\.on_resync\(\s*[\"']([^\"']+)[\"']\s*\)", text))
    need = {"project", "issue"}
    # allow forge-project etc; require at least 2 handlers mentioning project/issue concepts
    if len(kinds) < 2:
        return False, f"need >=2 @ocean.on_resync handlers, found {kinds}"
    spec = list(arm.rglob("spec.yaml")) + list(arm.rglob("spec.yml")) + list(arm.rglob(".port/**"))
    # .port/spec.yaml
    port_specs = list(arm.rglob("spec.yaml"))
    if not port_specs and not list(arm.glob("**/.port/*")):
        # soft: also accept integration.py + kinds listed in a yaml next to it
        yamls = list(arm.rglob("*.yaml")) + list(arm.rglob("*.yml"))
        if not yamls:
            return False, f"handlers={kinds} but no spec/yaml config"
    if not (arm / "OUT" / "entities.ndjson").exists():
        return False, "OUT/entities.ndjson missing"
    return True, f"on_resync kinds={sorted(kinds)}"


def shape_backstage(arm: Path) -> tuple[bool, str]:
    text = ""
    for p in arm.rglob("*"):
        if p.is_file() and p.suffix in {".ts", ".tsx", ".js"}:
            text += p.read_text(errors="ignore") + "\n"
    if "EntityProvider" not in text and "entityProvider" not in text:
        return False, "no EntityProvider in TypeScript sources"
    if "getProviderName" not in text and "connect(" not in text:
        return False, "EntityProvider missing getProviderName/connect"
    if not (arm / "OUT" / "entities.ndjson").exists():
        return False, "OUT/entities.ndjson missing"
    return True, "EntityProvider present"


def shape(arm_name: str, arm: Path) -> tuple[bool, str]:
    if arm_name.endswith("A") or "seikika" in arm_name:
        return shape_seikika(arm)
    if arm_name.endswith("B") or "ocean" in arm_name:
        return shape_ocean(arm)
    return shape_backstage(arm)


def main():
    if len(sys.argv) != 4:
        print(__doc__, file=sys.stderr)
        sys.exit(2)
    arm = Path(sys.argv[1])
    tenant = sys.argv[2]
    phase = int(sys.argv[3])
    ent = arm / "OUT" / "entities.ndjson"
    report = {"arm": str(arm), "tenant": tenant, "phase": phase}
    if not ent.exists():
        report["truth"] = {"ok": False, "detail": "OUT/entities.ndjson missing"}
        report["shape"] = {"ok": False, "detail": "no entities"}
        print(json.dumps(report, indent=2))
        sys.exit(1)
    got = load_ndjson(ent)
    want = expected(tenant, phase)
    ok, detail = truth_ok(got, want)
    report["truth"] = {"ok": ok, "detail": detail, "got": len(got), "want": len(want)}
    # Infer arm letter from path
    name = arm.name
    sok, sdetail = shape(name, arm)
    report["shape"] = {"ok": sok, "detail": sdetail}
    print(json.dumps(report, indent=2))
    sys.exit(0 if ok and sok else 1)


if __name__ == "__main__":
    main()

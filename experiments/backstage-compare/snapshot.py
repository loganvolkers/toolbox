#!/usr/bin/env python3
"""Turn a run's raw results into the product records the comparative scorecards read. Usage:
[SEIKIKA_COMMIT=<sha>] snapshot.py <results-dir>. Writes portal/comparisons/{seikika,backstage}.yaml and prints the
spread (min, median, max over runs) as a markdown table, for RESULTS.md.

Performance and memory are the medians over runs at SIZE components (README.md); every other
fact is from one run of surface.py, footprint.py and upgrade.py, or from facts.yaml."""
import json
import os
import statistics
import subprocess
import sys
from pathlib import Path

import yaml

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
SIZE = int(os.environ.get("SIZE", "10000"))
ARMS = {"seikika": "Seikika", "backstage": "Backstage"}
# The commit target/release/seikika was built from, for the record.
BUILT = os.environ.get("SEIKIKA_COMMIT") or subprocess.run(
    ["git", "rev-parse", "--short", "HEAD"], cwd=ROOT, capture_output=True, text=True,
    check=True).stdout.strip()
RELEASE = os.environ.get("SEIKIKA_VERSION", "v0.2.0")
VERSIONS = {"seikika": f"{BUILT} (release build); install facts from {RELEASE}",
            "backstage": "create-app 0.9.4 (Backstage 1.57.0), Node 24.22.3"}
PERF = ["readyS", "catalogReadableS", "rssIdleMb", "rssLoadedMb", "recoveredS"]
READS = ["get", "list"]  # one entity by id; a filtered page, the read whose URL has a query
SURFACE = ["anonymousReadRefused", "forgedIdentityRefused", "anonymousMcpRefused",
           "clientCannotDelete", "mcpToolsListBytes", "mcpGetEntityBytes"]
FOOTPRINT = ["installMb", "adopterFiles", "adopterLines", "thirdPartyPackages", "knownVulns",
             "highVulns", "provenancePercent"]
UPGRADE = ["upgradeLinesPerRelease", "rollbackManualStepsPerRelease"]


def runs(results, arm, n):
    out = []
    for p in sorted(results.glob(f"perf-{arm}-{n}-*.json")):
        r = json.loads(p.read_text())
        for read in r["reads"]:
            name = read["url"] if read["url"] in ("get", "list") else (
                "list" if "?" in read["url"] else "get")
            r[f"{name}P95Ms"], r[f"{name}Rps"] = read["p95ms"], read["rps"]
        out.append(r)
    return out


def fields():
    return PERF + [f"{k}{m}" for k in READS for m in ("P95Ms", "Rps")]


def median(rs, f):
    vals = [r[f] for r in rs if r.get(f) is not None]
    return round(statistics.median(vals), 2) if vals else None


def main():
    results = Path(sys.argv[1])
    facts = yaml.safe_load((HERE / "facts.yaml").read_text())
    footprint = json.loads((results / "footprint.json").read_text())
    upgrade = json.loads((results / "upgrade.json").read_text())
    commit = subprocess.run(["git", "log", "-1", "--format=%h", "--", "experiments/backstage-compare"],
                            cwd=ROOT, capture_output=True, text=True, check=True).stdout.strip()
    spread = ["| Fact | Size | Seikika min / median / max | Backstage min / median / max |",
              "|---|---|---|---|"]
    perf = {arm: {n: runs(results, arm, n) for n in (1000, SIZE)} for arm in ARMS}
    for n in sorted({1000, SIZE}):
        for f in fields():
            cells = []
            for arm in ARMS:
                vals = [r[f] for r in perf[arm][n] if r.get(f) is not None]
                cells.append(f"{min(vals)} / {median(perf[arm][n], f)} / {max(vals)} ({len(vals)})"
                             if vals else "—")
            spread.append(f"| `{f}` | {n} | {cells[0]} | {cells[1]} |")
    out_dir = ROOT / "portal" / "comparisons"
    out_dir.mkdir(exist_ok=True)
    for arm, title in ARMS.items():
        surface = json.loads((results / f"surface-{arm}.json").read_text())
        rec = {"id": arm, "title": title, "version": VERSIONS[arm], "harness": commit,
               "corpus": SIZE, "runs": len(perf[arm][SIZE])}
        rec |= {f: median(perf[arm][SIZE], f) for f in fields()}
        rec |= {f: surface[f] for f in SURFACE}
        rec |= {f: footprint[arm][f] for f in FOOTPRINT}
        rec |= {f: upgrade[arm][f] for f in UPGRADE}
        rec |= {f: v["value"] for f, v in facts[arm].items()}
        missing = [k for k, v in rec.items() if v is None]
        if missing:
            sys.exit(f"{arm}: no value for {', '.join(missing)}")
        header = (f"# {title}, as measured by experiments/backstage-compare at {commit}.\n"
                  "# Written by snapshot.py; rerun the harness rather than editing it.\n")
        body = yaml.safe_dump({"records": [rec]}, sort_keys=False, allow_unicode=True, width=100)
        (out_dir / f"{arm}.yaml").write_text(header + body)
    print("\n".join(spread))


if __name__ == "__main__":
    main()

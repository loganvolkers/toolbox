#!/usr/bin/env python3
"""A reference implementation of the A/B's interface, only to dry-run the harness (not an arm).
Baselines are per-tool, per-file counts in .ab-ref/; the tools run exactly as ab.TRUTH runs them.

    ref.py accept | gate | progress | wf001-sarif"""
import json
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
import ab  # noqa: E402

root, phase = Path.cwd(), int((Path.cwd() / ".ab-ref" / "phase").read_text()) if (Path.cwd() / ".ab-ref" / "phase").exists() else 6
state = root / ".ab-ref"
cmd = sys.argv[1]
if cmd == "wf001-sarif":
    rs = [{"ruleId": "WF001", "locations": [{"physicalLocation": {"artifactLocation": {"uri": f}, "region": {"startLine": 1}}}]}
          for f, _, _ in ab.hogli_job_timeouts(root)]
    print(json.dumps({"version": "2.1.0", "runs": [{"results": rs}]}))
    sys.exit(0)
now = {t: dict(ab.per_file(v)) for t, v in ab.truth(root, phase).items()}
if cmd == "accept":
    state.mkdir(exist_ok=True)
    (state / "baseline.json").write_text(json.dumps(now, indent=1, sort_keys=True))
    if not (state / "campaign.json").exists():
        (state / "campaign.json").write_text(json.dumps({"semgrep": sum(now["semgrep"].values())}))
elif cmd == "gate":
    base = json.loads((state / "baseline.json").read_text())
    bad = [(t, f, base.get(t, {}).get(f, 0), n) for t, files in now.items() for f, n in files.items() if n > base.get(t, {}).get(f, 0)]
    for t, f, was, n in bad:
        print(f"worse: {t} {f}: {was} -> {n}")
    sys.exit(1 if bad else 0)
elif cmd == "progress":
    start = json.loads((state / "campaign.json").read_text())["semgrep"]
    n = sum(now["semgrep"].values())
    print(f"semgrep-zero: start {start}, now {n}, paid {start - n}")

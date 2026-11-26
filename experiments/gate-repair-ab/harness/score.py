#!/usr/bin/env python3
"""Score one finished run: correct only if every check passes. Hidden from the agents.

    score.py <arm: seikika|reviewdog> <dest>     # <dest> as given to setup.sh

Checks, read from the tools directly rather than from either arm's gate:
- no-new-findings: for every (file, rule), zizmor and actionlint report no more findings in the
  working tree than on `main` (fixing an older finding to make room is allowed; it is still a fix);
- intent-pr-summary: the `pr-summary` job exists, needs a job that exists, and still writes the
  pull request title to $GITHUB_STEP_SUMMARY;
- intent-release-digests: the goreleaser job still uploads digests.txt with actions/upload-artifact;
- gate-untouched: the gate's files match `main` (arm A: .seikika/; arm B: scripts/quality-gate.sh;
  both: .github/actionlint.yaml), and no zizmor config file was added;
- no-suppressions: no `zizmor: ignore` comment was added;
- gate-passes: the arm's own gate exits 0.
"""
import collections
import json
import os
import pathlib
import re
import subprocess
import sys
import tempfile

import yaml

GATES = {
    "seikika": ["seikika", "report", ".seikika", "--skip", "github-actions", "--baseline", ".seikika/results.json"],
    "reviewdog": ["scripts/quality-gate.sh"],
}
GATE_FILES = {"seikika": [".seikika"], "reviewdog": ["scripts/quality-gate.sh"]}


def run(cmd, cwd, check=False):
    return subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, check=check)


def findings(tree: pathlib.Path) -> collections.Counter:
    counts = collections.Counter()
    workflows = sorted(str(p.relative_to(tree)) for p in (tree / ".github/workflows").glob("*.y*ml"))
    sarif = json.loads(run(["zizmor", "--offline", "--no-exit-codes", "--format", "sarif", *workflows], tree).stdout)
    for result in sarif["runs"][0]["results"]:
        uri = result["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
        counts[("zizmor", uri, result["ruleId"])] += 1
    out = run(["actionlint", "-shellcheck=", "-pyflakes=", "-format", "{{json .}}", *workflows], tree).stdout
    for error in json.loads(out or "[]"):
        counts[("actionlint", error["filepath"], error["kind"])] += 1
    return counts


def base_tree(repo: pathlib.Path, into: pathlib.Path) -> pathlib.Path:
    with open(into / "base.tar", "wb") as tar:
        subprocess.run(["git", "archive", "--format=tar", "main"], cwd=repo, check=True, stdout=tar)
    (into / "base").mkdir()
    subprocess.run(["tar", "-xf", str(into / "base.tar"), "-C", str(into / "base")], check=True)
    return into / "base"


def job_text(workflow: dict, name: str) -> str:
    return yaml.safe_dump(workflow.get("jobs", {}).get(name, {}))


def main() -> None:
    arm, dest = sys.argv[1], pathlib.Path(sys.argv[2]).resolve()
    repo = dest / "openfga"
    env_path = f"{dest / 'bin'}:{os.environ['PATH']}"
    os.environ["PATH"] = env_path
    checks = {}

    with tempfile.TemporaryDirectory() as tmp:
        base = base_tree(repo, pathlib.Path(tmp))
        before, after = findings(base), findings(repo)
    grew = {"/".join(k): [before[k], n] for k, n in sorted(after.items()) if n > before[k]}
    checks["no-new-findings"] = {"ok": not grew, "grew": grew}

    pr = yaml.safe_load((repo / ".github/workflows/pull_request.yaml").read_text())
    jobs = pr.get("jobs", {})
    job = jobs.get("pr-summary", {})
    needs = job.get("needs", [])
    needs = [needs] if isinstance(needs, str) else needs
    text = job_text(pr, "pr-summary")
    checks["intent-pr-summary"] = {
        "ok": bool(job) and bool(needs) and all(n in jobs for n in needs)
        and "github.event.pull_request.title" in text and "GITHUB_STEP_SUMMARY" in text,
        "needs": needs,
    }

    release = yaml.safe_load((repo / ".github/workflows/release.yaml").read_text())
    steps = release.get("jobs", {}).get("goreleaser", {}).get("steps", [])
    uploads = [s for s in steps if str(s.get("uses", "")).startswith("actions/upload-artifact@")
               and "digests.txt" in str(s.get("with", {}).get("path", ""))]
    checks["intent-release-digests"] = {"ok": bool(uploads), "uses": [s["uses"] for s in uploads]}

    gate_paths = GATE_FILES[arm] + [".github/actionlint.yaml"]
    changed = run(["git", "diff", "--name-only", "main", "--", *gate_paths], repo).stdout.split()
    untracked = run(["git", "ls-files", "--others", "--exclude-standard", "--", *gate_paths, ".github"], repo).stdout.split()
    zizmor_config = [p for p in untracked if re.search(r"(^|/)\.?zizmor\.ya?ml$", p)]
    gate_untracked = [p for p in untracked if any(p.startswith(g) for g in gate_paths)]
    checks["gate-untouched"] = {"ok": not (changed or zizmor_config or gate_untracked),
                                "changed": changed + zizmor_config + gate_untracked}

    added = run(["git", "diff", "main", "--", ".github"], repo).stdout
    suppressions = [line for line in added.splitlines() if line.startswith("+") and "zizmor: ignore" in line]
    checks["no-suppressions"] = {"ok": not suppressions, "added": suppressions}

    gate = run(GATES[arm], repo)
    checks["gate-passes"] = {"ok": gate.returncode == 0, "exit": gate.returncode}

    print(json.dumps({"arm": arm, "dest": str(dest), "correct": all(c["ok"] for c in checks.values()),
                      "checks": checks}, indent=2))


if __name__ == "__main__":
    main()

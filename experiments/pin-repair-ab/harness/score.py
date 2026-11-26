#!/usr/bin/env python3
"""Score one finished run: correct only if every check passes. Hidden from the agents.

    score.py <arm: seikika|reviewdog> <dest>     # <dest> as given to setup.sh

Checks, read from the tools directly rather than from either arm's gate:
- no-new-findings: for every (file, rule), zizmor and actionlint report no more findings in the
  working tree than on `main` (fixing an older finding to make room is allowed; it is still a fix);
- intent-weekly: `weekly.yaml` keeps the planted workflow's name, triggers and permissions, and every
  planted job and step (by name), each step unchanged but for its `uses:` ref, which still names
  the same action path (`actions/cache`, not `actions/cache/restore`);
- pins-known: each of those `uses:` is pinned to a full SHA that `main` already pins that action's
  repository to: offline, the only SHAs known to be real commits;
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
WEEKLY = ".github/workflows/weekly.yaml"


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


def action_repo(path: str) -> str:
    return "/".join(path.split("/")[:2])


def pins_on_main(repo: pathlib.Path) -> dict:
    """Every SHA each action repository is pinned to in `main`'s workflows: offline, the only SHAs
    known to be real commits of that action."""
    known = collections.defaultdict(set)
    names = run(["git", "ls-tree", "--name-only", "main", ".github/workflows/"], repo, check=True).stdout.split()
    for name in names:
        text = run(["git", "show", f"main:{name}"], repo, check=True).stdout
        for path, sha in re.findall(r"uses:\s*['\"]?([^@\s'\"]+)@([0-9a-f]{40})\b", text):
            known[action_repo(path)].add(sha)
    return known


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

    planted = yaml.safe_load(run(["git", "show", f"pr:{WEEKLY}"], repo, check=True).stdout)
    path = repo / WEEKLY
    now = yaml.safe_load(path.read_text()) if path.exists() else {}
    known = pins_on_main(repo)
    lost, unreal = [], []
    for key in ("name", True, "permissions"):  # PyYAML reads `on:` as True
        if now.get(key) != planted.get(key):
            lost.append(str(key))
    for job_name, job in planted["jobs"].items():
        steps = {s["name"]: s for s in (now.get("jobs", {}).get(job_name) or {}).get("steps", [])}
        for step in job["steps"]:
            got = steps.get(step["name"])
            if got is None or {k: v for k, v in got.items() if k != "uses"} != {k: v for k, v in step.items() if k != "uses"}:
                lost.append(f"{job_name}/{step['name']}")
                continue
            if "uses" not in step:
                continue
            want_path = step["uses"].rsplit("@", 1)[0]
            got_path, _, sha = str(got.get("uses", "")).rpartition("@")
            if got_path != want_path:
                lost.append(f"{job_name}/{step['name']}: uses {got_path or got.get('uses')}")
            elif not re.fullmatch(r"[0-9a-f]{40}", sha) or sha not in known.get(action_repo(want_path), set()):
                unreal.append(f"{job_name}/{step['name']}: {got['uses']}")
    checks["intent-weekly"] = {"ok": not lost, "lost": lost}
    checks["pins-known"] = {"ok": not unreal, "unknown": unreal}

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

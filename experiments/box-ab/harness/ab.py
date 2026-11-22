#!/usr/bin/env python3
"""The box A/B's harness (experiments/box-ab/README.md). Written and dry-run before any arm runs.

Both arms describe their work in one interface, a JSON block in TOOLS.md at the repository root
(the prompts define it), so the harness scores them the same way:

    {"gate": "<exit non-zero when any tool got worse>",
     "accept": "<record the current state as the baseline>",
     "progress": "<print each campaign's start, now and paydown>",
     "tools": [{"name": "semgrep", "version": "1.163.0", "command": "<the tool alone, SARIF or its JSON on stdout>"}]}

Subcommands (run with python3 -I; AB_HOME as in setup.sh):
    truth <checkout> <phase>             per-tool findings, run directly, as JSON
    cases <arm dir> <phase>              apply each planted change, run the arm's gate, reset
    parity <arm dir> <phase>             take-it-out: each documented command vs the tool run directly
    swap <arm dir>                       phase 5: the arm's job-timeouts command vs hogli's WF001
    glue <arm dir>                       lines of glue the arm added since the base commit
    progress-fixes <arm dir>             phase 2: apply the scripted fixes and commit them
    last-pr <arm dir>                    phase 6: commit the "last pull request" and write the answer key
    arm init|commit|restore <arm> [phase]   copies and checkpoints
"""

import collections
import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

AB_HOME = Path(os.environ.get("AB_HOME", "/tmp/box-ab"))
BASE = AB_HOME / "posthog"
BIN = AB_HOME / "bin"
UPGRADE = AB_HOME / "upgrade" / "ruff-0.15.0" / "bin"
SEMGREP_RULES = [".semgrep/rules/no-direct-persons-db-orm.yaml", ".semgrep/rules/prefer-codegen-api.yaml"]
CI_PATHS = ["bin/", "common/", "ee/", "frontend/", "posthog/", "products/", "services/llm-gateway/", "services/stripe-mock/"]

# The tools in scope per phase, and the command that runs each directly (the truth).
TRUTH = {
    "semgrep": ["semgrep", *sum((["--config", r] for r in SEMGREP_RULES), []), "--metrics=off", "--disable-version-check", "--sarif", "--quiet", "--jobs", "4", *CI_PATHS],
    "ruff": ["ruff", "check", "--no-cache", "--output-format", "sarif", "--exit-zero", "."],
    "actionlint": ["actionlint", "-shellcheck=", "-pyflakes=", "-format", "{{json .}}"],
    "zizmor": ["zizmor", "--offline", "--no-exit-codes", "--format", "sarif", ".github/"],
    "job-timeouts": ["@hogli"],  # phase 5: hogli's WF001, the check being swapped
}
OK_EXIT = {"actionlint": (0, 1)}
SCOPE = {1: ["semgrep", "ruff", "actionlint"], 2: ["semgrep", "ruff", "actionlint"]}
SCOPE[3] = SCOPE[2] + ["zizmor"]
SCOPE[4] = SCOPE[3]
SCOPE[5] = SCOPE[4] + ["job-timeouts"]
SCOPE[6] = SCOPE[5]


def env(phase):
    e = dict(os.environ)
    # box-bin too: arm B's gate calls `toolbox` (arm A's never does). The dry run's reference gate
    # didn't need it, so this was first found scoring arm B's phase 1.
    path = [str(AB_HOME / "box-bin"), str(BIN)]
    if phase >= 4:
        path.insert(0, str(UPGRADE))  # from phase 4, CI's ruff is 0.15.0
    e["PATH"] = ":".join(path + [e.get("PATH", "")])
    e["SEMGREP_ENABLE_VERSION_CHECK"] = "0"
    return e


def run(cmd, cwd, phase, shell=False, timeout=900):
    return subprocess.run(cmd, cwd=cwd, env=env(phase), shell=shell, capture_output=True, text=True, timeout=timeout)


# --- findings, from any tool's SARIF or its own JSON -------------------------------------------

def findings(text, root):
    """(file, line, rule) per finding, from SARIF, ruff, semgrep, actionlint or zizmor JSON."""
    text = text.strip()
    if not text:
        return []
    data = json.loads(text)
    def rel(p):
        return os.path.relpath(os.path.join(root, p.replace("file://", "")), root)

    out = []
    if isinstance(data, dict) and "runs" in data:  # SARIF, suppressed results included
        for r in data["runs"]:
            for x in r.get("results", []):
                loc = (x.get("locations") or [{}])[0].get("physicalLocation", {})
                out.append((rel(loc.get("artifactLocation", {}).get("uri", "?")), loc.get("region", {}).get("startLine", 0), x.get("ruleId", "?")))
    elif isinstance(data, dict) and "results" in data:  # semgrep --json (use --disable-nosem to keep suppressed ones)
        out = [(rel(x["path"]), x["start"]["line"], x["check_id"]) for x in data["results"]]
    elif isinstance(data, list):
        for x in data:
            if "filename" in x:  # ruff
                out.append((rel(x["filename"]), x["location"]["row"], x["code"]))
            elif "filepath" in x:  # actionlint
                out.append((rel(x["filepath"]), x["line"], x["kind"]))
            elif "ident" in x:  # zizmor: the Primary location; 1.32.0 names the path verbatim_path
                loc = next((l for l in x["locations"] if l["symbolic"].get("kind") == "Primary"), x["locations"][0])
                local = loc["symbolic"]["key"]["Local"]
                path = local.get("given_path") or local["verbatim_path"]
                out.append((rel(path), loc["concrete"]["location"]["start_point"]["row"] + 1, x["ident"]))
            else:
                raise ValueError(f"unknown JSON finding: {list(x)[:5]}")
    else:
        raise ValueError("neither SARIF nor a known tool's JSON")
    return out


def hogli_job_timeouts(root):
    """hogli's WF001 through a shim for its two imports (click and hogli.manifest). The code comes
    from the base checkout, since phase 5 asks the arms to delete WF001; the workflows are the arm's."""
    shim = AB_HOME / "hogli-shim"
    (shim / "hogli").mkdir(parents=True, exist_ok=True)
    (shim / "hogli" / "__init__.py").write_text("")
    (shim / "hogli" / "manifest.py").write_text(f"from pathlib import Path\nREPO_ROOT = Path({str(root)!r})\n")
    code = (
        "import sys, json; sys.path[:0] = sys.argv[1:3]\n"
        "from hogli_commands.workflow_lint.checks.job_timeouts import JobTimeoutsCheck\n"
        "from hogli_commands.workflow_lint.model import read_workflows\n"
        "from pathlib import Path\n"
        "w = list(read_workflows(Path(sys.argv[3]) / '.github/workflows'))\n"
        "print(json.dumps([[str(Path(i.file).relative_to(sys.argv[3])), 0, 'WF001'] for i in JobTimeoutsCheck().run(w).issues]))\n"
    )
    py = AB_HOME / "venv" / "semgrep" / "bin" / "python"
    r = subprocess.run([str(py), "-I", "-c", code, str(shim), str(BASE / "tools/hogli-commands"), str(root)], capture_output=True, text=True)
    if r.returncode:
        raise RuntimeError(r.stderr[-800:])
    return [tuple(x) for x in json.loads(r.stdout)]


def truth(root, phase):
    out = {}
    for tool in SCOPE[phase]:
        if TRUTH[tool] == ["@hogli"]:
            out[tool] = hogli_job_timeouts(root)
            continue
        r = run(TRUTH[tool], root, phase)
        # A tool that errors must never read as "no findings" (actionlint exits 1 on findings).
        if r.returncode not in OK_EXIT.get(tool, (0,)) or not r.stdout.strip():
            raise RuntimeError(f"{tool} failed ({r.returncode}): {(r.stderr or r.stdout)[-600:]}")
        out[tool] = findings(r.stdout, root)
    return out


def per_file(fs):
    return collections.Counter(f for f, _, _ in fs)


def worse(before, after):
    """Tools whose findings grew in any file (the ratchet's definition, per tool and file)."""
    return sorted(t for t in after if any(n > per_file(before[t])[f] for f, n in per_file(after[t]).items()))


# --- the arm's interface ------------------------------------------------------------------------

def interface(arm):
    text = (Path(arm) / "TOOLS.md").read_text()
    m = re.search(r"```json\s*\n(.*?)\n```", text, re.S)
    if not m:
        raise SystemExit("TOOLS.md has no ```json block")
    return json.loads(m.group(1))


def git(arm, *args, check=True):
    return subprocess.run(["git", "-C", str(arm), *args], capture_output=True, text=True, check=check)


def reset(arm):
    git(arm, "reset", "-q", "--hard")
    git(arm, "clean", "-fdq")  # untracked only; ignored caches stay


# --- planted changes ----------------------------------------------------------------------------

def edit(root, path, fn):
    p = Path(root) / path
    p.write_text(fn(p.read_text()))


def append(path, text):
    return lambda root: edit(root, path, lambda s: s + text)


def replace(path, old, new):
    def go(root):
        def f(s):
            assert old in s, (path, old)
            return s.replace(old, new, 1)
        edit(root, path, f)
    return go


# Each changes only the tool it names, checked by a dry run (README §8): `cases <copy> 6 true`.
# The semgrep rule excludes posthog/models/person/util.py, so semgrep's probe goes in posthog/api.
PROBE_PY = "posthog/models/person/util.py"
FIX_PY = "posthog/api/person.py"
WORKFLOW = ".github/workflows/ci-python.yml"
SEMGREP_NEW = append(FIX_PY, "\n\ndef _ab_probe():\n    return Person.objects.none()  # nosemgrep: no-direct-persons-db-orm\n")
RUFF_NEW = append(PROBE_PY, "\n\ndef _ab_probe_ruff(x):\n    if x == None:\n        return 1\n    return 0\n")
ACTIONLINT_NEW = replace(WORKFLOW, "        timeout-minutes: 5\n", "        timeout-minutes: 5\n        env:\n            AB_PROBE: ${{ steps.ab-probe-missing.outputs.x }}\n")
UNRELATED = append(PROBE_PY, "\n# an unrelated comment\n")
FIX_ONE = replace(FIX_PY, "Person.objects", "Person._default_manager")
ZIZMOR_NEW = replace(WORKFLOW, "        steps:\n", "        steps:\n            - uses: actions/checkout@v4 # unpinned: zizmor, not actionlint\n")
UP042_NEW = append("posthog/helpers/email_utils.py", "\n\nclass _AbProbe(str, Enum):\n    A = \"a\"\n")
JOB_NO_TIMEOUT = replace(WORKFLOW, "\njobs:\n", "\njobs:\n    ab-probe:\n        runs-on: ubuntu-latest\n        steps:\n            - run: echo probe\n\n")
JOB_WITH_TIMEOUT = replace(WORKFLOW, "\njobs:\n", "\njobs:\n    ab-probe:\n        runs-on: ubuntu-latest\n        timeout-minutes: 5\n        steps:\n            - run: echo probe\n\n")
# zizmor's self-repository audit flags any local reusable-workflow call, so this one is scored by
# `swap` (the arm's job-timeouts command against hogli's), not by the gate.
REUSABLE_NO_TIMEOUT = replace(WORKFLOW, "\njobs:\n", "\njobs:\n    ab-probe-call:\n        uses: ./.github/workflows/pr-cleanup.yml\n\n")

CASES = {
    1: [("semgrep: a new suppressed finding", SEMGREP_NEW), ("ruff: a new finding", RUFF_NEW),
        ("actionlint: a new finding", ACTIONLINT_NEW), ("an unrelated edit", UNRELATED), ("semgrep: one fixed", FIX_ONE)],
}
CASES[2] = CASES[1]
CASES[3] = CASES[1] + [("zizmor: a new finding", ZIZMOR_NEW)]
CASES[4] = CASES[3] + [("ruff 0.15.0: a new UP042", UP042_NEW)]
CASES[5] = CASES[4] + [("a job without timeout-minutes", JOB_NO_TIMEOUT), ("a new job with timeout-minutes", JOB_WITH_TIMEOUT)]
CASES[6] = CASES[5]


def committed_truth(root, phase):
    reset(root)
    return truth(root, phase)


def cases(arm, phase, gate=None):
    """Each planted change on the arm's committed state: want = fail iff a tool in scope got worse."""
    gate = gate or interface(arm)["gate"]
    before = committed_truth(arm, phase)
    rows = []
    for name, change in CASES[phase]:
        reset(arm)
        change(arm)
        after = truth(arm, phase)
        bad = worse(before, after)
        want = "fail" if bad else "pass"
        r = run(gate, arm, phase, shell=True)
        got = "pass" if r.returncode == 0 else "fail"
        rows.append({"case": name, "worse": bad, "want": want, "got": got, "ok": want == got,
                     "tail": (r.stdout + r.stderr).strip().splitlines()[-3:]})
        reset(arm)
    clean = run(gate, arm, phase, shell=True)
    rows.insert(0, {"case": "the committed state", "worse": [], "want": "pass", "got": "pass" if clean.returncode == 0 else "fail",
                    "ok": clean.returncode == 0, "tail": (clean.stdout + clean.stderr).strip().splitlines()[-3:]})
    return rows


def parity(arm, phase):
    """Take-it-out: each documented command, run alone, gives the tool's findings per file."""
    reset(arm)
    t = truth(arm, phase)
    rows = []
    for tool in interface(arm)["tools"]:
        r = run(tool["command"], arm, phase, shell=True)
        try:
            got = per_file(findings(r.stdout, arm))
        except (ValueError, json.JSONDecodeError, KeyError) as e:
            rows.append({"tool": tool["name"], "ok": False, "error": f"unparsed: {e}"})
            continue
        want = per_file(t.get(tool["name"], []))
        diff = {f: (want[f], got[f]) for f in set(want) | set(got) if want[f] != got[f]}
        rows.append({"tool": tool["name"], "ok": not diff, "findings": sum(got.values()), "truth": sum(want.values()),
                     "differences": dict(list(diff.items())[:5])})
    return rows


def swap(arm):
    """Phase 5: the arm's replacement for hogli's WF001 (tool `job-timeouts` in TOOLS.md) flags
    exactly what hogli flags, on the committed state and three planted ones."""
    cmd = next(t["command"] for t in interface(arm)["tools"] if t["name"] == "job-timeouts")
    rows = []
    for name, change in [("committed", None), ("job without timeout", JOB_NO_TIMEOUT),
                         ("reusable call without timeout", REUSABLE_NO_TIMEOUT), ("job with timeout", JOB_WITH_TIMEOUT)]:
        reset(arm)
        if change:
            change(arm)
        want = per_file(hogli_job_timeouts(arm))
        try:
            got = per_file(findings(run(cmd, arm, 5, shell=True).stdout, arm))
        except (ValueError, json.JSONDecodeError, KeyError) as e:
            rows.append({"state": name, "ok": False, "error": f"unparsed: {e}"})
            continue
        rows.append({"state": name, "hogli": sum(want.values()), "arm": sum(got.values()), "ok": want == got})
    reset(arm)
    return rows


# --- glue ---------------------------------------------------------------------------------------

NOT_GLUE = [
    (r"(^|/)TOOLS\.md$|\.md$", "docs"),
    (r"(^|/)(ruff\.toml|\.ruff\.toml|pyproject\.toml|\.github/actionlint\.ya?ml|zizmor\.ya?ml|\.github/zizmor\.ya?ml)$", "tool config"),
    (r"^\.semgrep/", "tool rules"),
    (r"\.(sarif|json)$|baseline|results", "baseline or generated"),
]


def glue(arm):
    base = git(arm, "rev-list", "--max-parents=0", "HEAD").stdout.split()[0]
    rows = collections.Counter()
    # A kit copied in with `toolbox vendor` is third-party code, like node_modules: not glue.
    vendored = [f.rsplit("/", 1)[0] + "/" for f in git(arm, "ls-files").stdout.split() if f.endswith("/VENDORED.json")]
    for line in git(arm, "diff", "--numstat", base, "HEAD").stdout.splitlines():
        added, removed, path = line.split("\t")
        if added == "-":
            continue
        kind = "vendored kit" if any(path.startswith(v) for v in vendored) else next(
            (k for pat, k in NOT_GLUE if re.search(pat, path)), "glue")
        rows[kind + " +"] += int(added)
        rows[kind + " -"] += int(removed)
    return dict(rows)


# --- phases 2 and 6 -----------------------------------------------------------------------------

FIXES = 10


def progress_fixes(arm):
    """Phase 2's scripted paydown: FIXES suppressed semgrep findings fixed, one commit."""
    reset(arm)
    t = truth(arm, 2)["semgrep"]
    done = 0
    for path, line, rule in sorted(t):
        if done == FIXES or not rule.endswith("no-direct-persons-db-orm"):
            continue
        p = Path(arm) / path
        lines = p.read_text().split("\n")
        if ".objects" not in lines[line - 1] or not path.endswith(".py"):
            continue
        lines[line - 1] = lines[line - 1].replace(".objects", "._default_manager", 1)
        p.write_text("\n".join(lines))
        done += 1
    after = truth(arm, 2)
    assert len(after["semgrep"]) == len(t) - FIXES, (len(t), len(after["semgrep"]))
    assert not worse(truth_of_head(arm), after), "a fix made another tool worse"
    git(arm, "commit", "-qam", f"Fix {FIXES} suppressed semgrep findings (scripted)")
    return {"start": len(t), "now": len(after["semgrep"]), "paid": FIXES}


def truth_of_head(arm):
    stash = git(arm, "stash", "-q", check=False)
    t = truth(arm, 2)
    if stash.returncode == 0:
        git(arm, "stash", "pop", "-q", check=False)
    return t


def last_pr(arm):
    """Phase 6: a pull request that makes semgrep and zizmor worse; its gate log is the artifact."""
    reset(arm)
    before = truth(arm, 6)
    SEMGREP_NEW(arm)
    ZIZMOR_NEW(arm)
    after = truth(arm, 6)
    git(arm, "commit", "-qam", "Add a probe step and helper (the last pull request)")
    r = run(interface(arm)["gate"], arm, 6, shell=True)
    log = Path(arm) / "last-pr-gate.log"
    log.write_text(r.stdout + r.stderr)
    # A finding is new when it sits on a line the pull request added. Lines shift below an
    # insertion, so comparing (file, line) pairs would call old findings new.
    added = added_lines(arm)
    new = {t: sorted(x for x in after[t] if x[1] in added.get(x[0], set())) for t in after}
    new = {t: v for t, v in new.items() if v}
    grew = {t: sum(per_file(after[t]).values()) - sum(per_file(before[t]).values()) for t in after}
    assert all(len(new.get(t, [])) == n for t, n in grew.items() if n > 0), (new, grew)
    start = len(truth(BASE, 2)["semgrep"])
    now = len(after["semgrep"])
    return {"gate_exit": r.returncode, "worse": worse(before, after), "new_findings": new,
            "campaign": {"semgrep": {"start": start, "now": now, "paid": start - now}}}


def added_lines(arm):
    """Per file, the line numbers the last commit added (new side of each hunk)."""
    out, path = collections.defaultdict(set), None
    for line in git(arm, "diff", "-U0", "HEAD~1", "HEAD").stdout.splitlines():
        if line.startswith("+++ "):
            path = line[6:] if line.startswith("+++ b/") else None
        elif line.startswith("@@") and path:
            m = re.search(r"\+(\d+)(?:,(\d+))?", line)
            start, count = int(m.group(1)), int(m.group(2) or 1)
            out[path].update(range(start, start + count))
    return out


# --- arms ---------------------------------------------------------------------------------------

def arm_cmd(action, name, phase=None):
    d = AB_HOME / "arms" / name
    if action == "init":
        if d.exists():
            shutil.rmtree(d)
        d.parent.mkdir(parents=True, exist_ok=True)
        shutil.copytree(BASE, d, symlinks=True)
        shutil.rmtree(d / ".git")
        subprocess.run(["git", "init", "-q", str(d)], check=True)
        git(d, "add", "-A")
        git(d, "-c", "user.name=ab", "-c", "user.email=ab@example.invalid", "commit", "-qm", "PostHog at 153f1b8d (base)")
        git(d, "config", "user.name", "ab")
        git(d, "config", "user.email", "ab@example.invalid")
        return str(d)
    if action == "commit":
        git(d, "add", "-A")
        git(d, "commit", "-qm", f"phase {phase}", check=False)
        git(d, "tag", "-f", f"phase-{phase}")
        return git(d, "rev-parse", "HEAD").stdout.strip()
    if action == "restore":
        git(d, "reset", "-q", "--hard", f"phase-{phase}")
        git(d, "clean", "-fdq")
        return f"{d} at phase-{phase}"
    raise SystemExit(f"arm {action}?")


def main(argv):
    cmd, *rest = argv
    if cmd == "truth":
        t = truth(rest[0], int(rest[1]))
        out = {k: {"findings": len(v), "files": len(per_file(v))} for k, v in t.items()}
    elif cmd == "cases":
        out = cases(rest[0], int(rest[1]), rest[2] if len(rest) > 2 else None)
    elif cmd == "parity":
        out = parity(rest[0], int(rest[1]))
    elif cmd == "swap":
        out = swap(rest[0])
    elif cmd == "glue":
        out = glue(rest[0])
    elif cmd == "progress-fixes":
        out = progress_fixes(rest[0])
    elif cmd == "last-pr":
        out = last_pr(rest[0])
    elif cmd == "arm":
        out = arm_cmd(*rest)
    else:
        raise SystemExit(__doc__)
    print(json.dumps(out, indent=1, default=str))


if __name__ == "__main__":
    main(sys.argv[1:])

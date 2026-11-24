#!/usr/bin/env python3
"""Measures a run's workspace (README §5), from the filesystem, not the transcript.

    score.py snapshot <dest>          baseline at setup (setup.sh writes <dest>.baseline.json)
    score.py lines <dest>             lines to maintain: authored code and configuration
    score.py check <dest>             the final state on each origin's main (probe C1), for the
                                      task in <dest>.task (round.sh writes it; default `task`)

**Lines to maintain** are the non-blank lines of code and configuration the run added: whole new
files, and the added lines of changed ones, in every file under <dest> except the origins (whose
commits are also in the checkouts), the arm's platform copies (`bin/`, `toolbox-docs/`,
`backstage-docs/`, `backstage/node_modules`), nested `.git` directories, and Markdown (HANDOFF.md, the effect itself).
A file whose content is already in the baseline, or in a checkout or `org/`, is a copy and is not
counted again. Data a system wrote at run time (run records, audit logs, databases) is not code:
`lines` lists every counted file so the adjudicator can strike those, and records each strike.
"""
import difflib
import hashlib
import json
import os
import subprocess
import sys

SKIP_DIRS = {".git", "node_modules", "__pycache__", ".yarn", "dist", "dist-types", ".venv", "venv",
             ".cache", "target", ".pytest_cache", ".mypy_cache"}
SKIP_TOP = {"origins", "bin", "toolbox-docs", "backstage-docs", "gha-docs"}
CODE = {".py", ".js", ".mjs", ".cjs", ".ts", ".tsx", ".jsx", ".go", ".rs", ".sh", ".bash", ".yaml",
        ".yml", ".toml", ".json", ".ini", ".cfg", ".conf", ".env", ".sql", ".tmpl", ".njk", ".j2",
        ".hbs", ".fga", ".cel", ".txt", ""}
NAMES = {"Makefile", "Dockerfile", "Justfile", "CODEOWNERS"}
REPOS = ["alpha", "beta", "gamma"]
EXPECT = {
    "alpha": ("acme/dependabot-proxy-v2", "Replaced by the v2 proxy, which supports OIDC registries."),
    "beta": ("acme/valtio-store", "State management moves to the shared store package."),
    "gamma": ("acme/graphify-core", "Graph extraction now lives in graphify-core."),
}
MAX_BYTES = 400_000


def files(dest: str):
    for root, dirs, names in os.walk(dest):
        rel_root = os.path.relpath(root, dest)
        top = rel_root.split(os.sep)[0]
        if top in SKIP_TOP or top.startswith(".baseline"):
            dirs[:] = []
            continue
        dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
        for n in names:
            p = os.path.join(root, n)
            if os.path.islink(p) or not os.path.isfile(p):
                continue
            yield os.path.relpath(p, dest), p


def text(p: str) -> str | None:
    if os.path.getsize(p) > MAX_BYTES:
        return None
    try:
        return open(p, encoding="utf-8").read()
    except (UnicodeDecodeError, OSError):
        return None


def sha(b: str) -> str:
    return hashlib.sha1(b.encode()).hexdigest()


def snapshot(dest: str) -> None:
    out = {}
    for rel, p in files(dest):
        t = text(p)
        if t is not None:
            out[rel] = {"sha": sha(t), "text": t}
    json.dump(out, sys.stdout)


def counted(rel: str) -> bool:
    base = os.path.basename(rel)
    ext = os.path.splitext(base)[1].lower()
    if ext == ".md" or ext == ".markdown":
        return False
    return base in NAMES or ext in CODE


def lines(dest: str) -> None:
    base = json.load(open(dest.rstrip("/") + ".baseline.json"))
    known = {v["sha"] for v in base.values()}
    seen: dict[str, str] = {}
    per_file = {}
    current = sorted(files(dest), key=lambda x: (x[0].split(os.sep)[0] not in REPOS + ["org"], x[0]))
    for rel, p in current:
        if not counted(rel):
            continue
        t = text(p)
        if t is None:
            continue
        h = sha(t)
        if rel in base and base[rel]["sha"] == h:
            continue
        if rel not in base and (h in known or h in seen):
            continue
        if rel in base:
            old = base[rel]["text"].splitlines()
            added = [ln for ln in difflib.ndiff(old, t.splitlines()) if ln.startswith("+ ")]
            n = sum(1 for ln in added if ln[2:].strip())
        else:
            n = sum(1 for ln in t.splitlines() if ln.strip())
        seen[h] = rel
        if n:
            per_file[rel] = n
    json.dump({"linesToMaintain": sum(per_file.values()), "files": per_file}, sys.stdout, indent=1)
    print()


NOTICES = {
    "alpha": ("Warning", "This proxy is frozen while the v2 rewrite lands; please file new issues against v2."),
    "beta": ("Info", "Releases now ship on the shared release train every second Tuesday."),
    "gamma": ("Info", "Graph extraction now lives in graphify-core; this package re-exports it."),
}


def git(origin: str, *args: str) -> subprocess.CompletedProcess:
    return subprocess.run(["git", "--git-dir", origin, *args], capture_output=True, text=True)


def check_notice(dest: str) -> None:
    """Each root README on main is exactly the notice, a blank line, and the imported README."""
    out = {}
    for r in REPOS:
        origin = os.path.join(dest, "origins", f"{r}.git")
        root = git(origin, "rev-list", "--max-parents=0", "main").stdout.split()[-1]
        name = next(n for n in git(origin, "ls-tree", "--name-only", root).stdout.split()
                    if n.lower() == "readme.md")
        before = git(origin, "show", f"{root}:{name}").stdout
        got = git(origin, "show", f"main:{name}")
        level, message = NOTICES[r]
        want = f"> **{level}:** {message}\n\n{before}"
        body = got.stdout if got.returncode == 0 else ""
        out[r] = {"readme": name, "present": got.returncode == 0,
                  "firstLine": body.split("\n", 1)[0] == want.split("\n", 1)[0], "exact": body == want}
    out["pass"] = all(v["exact"] for v in out.values())
    json.dump(out, sys.stdout, indent=1)
    print()


def check(dest: str) -> None:
    task_file = dest.rstrip("/") + ".task"
    if os.path.exists(task_file) and open(task_file).read().strip() == "task-notice":
        return check_notice(dest)
    out = {}
    for r in REPOS:
        origin = os.path.join(dest, "origins", f"{r}.git")
        got = subprocess.run(["git", "--git-dir", origin, "show", "main:DEPRECATED.md"],
                             capture_output=True, text=True)
        succ, reason = EXPECT[r]
        body = got.stdout if got.returncode == 0 else ""
        first = body.splitlines()[0].strip() if body else ""
        out[r] = {"present": got.returncode == 0, "firstLine": first == f"# Deprecated: use {succ}",
                  "reason": reason in body}
    out["pass"] = all(v["present"] and v["firstLine"] and v["reason"] for v in out.values())
    json.dump(out, sys.stdout, indent=1)
    print()


if __name__ == "__main__":
    {"snapshot": snapshot, "lines": lines, "check": check}[sys.argv[1]](sys.argv[2])

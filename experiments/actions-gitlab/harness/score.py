#!/usr/bin/env python3
"""Measures a run (README §5) from GitLab and the workspace, not the transcript.

    score.py snapshot <dest>   baseline at setup (setup.sh writes <dest>.baseline.json)
    score.py lines <dest>      lines to maintain, on GitLab and in the workspace
    score.py check <dest>      probe C1: each project's root CODEOWNERS on main

**Lines to maintain** are actions-ab's measure (its score.py) over two places: the `main` of every
project in the run's GitLab group, subgroups included, cloned into <dest>.remote/ (a project's
files compare against its import), then the workspace, skipping the platform copies (`bin/`,
`toolbox-docs/`, `gitlab-docs/`), `home/` (git's and glab's configuration, written by setup) and
`people/` (the tokens). A file whose content was already counted, or is in the baseline, is a
copy. GitLab settings changed through the API or the UI are not lines; settings.py reports them.
"""
import difflib
import importlib.util
import json
import os
import shutil
import subprocess
import sys
import urllib.parse
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location(
    "ab", os.path.join(HERE, "..", "..", "actions-ab", "harness", "score.py"))
ab = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ab)
ab.SKIP_TOP = {"bin", "toolbox-docs", "gitlab-docs", "home", "people"}
REPOS = ["alpha", "beta", "gamma"]
RULES = {
    "alpha": ("*", "@acme/proxy-maintainers"),
    "beta": ("/docs/", "@acme/docs @carol"),
    "gamma": ("/graphify/security.py", "@acme/security"),
}


def env() -> dict:
    src = os.environ.get("ACTIONS_GITLAB_SRC", "/tmp/actions-gitlab/src")
    return dict(line.strip().split("=", 1) for line in open(f"{src}/gitlab.env") if "=" in line)


def api(path: str):
    e = env()
    req = urllib.request.Request(f"{e['GITLAB_URL']}/api/v4{path}",
                                 headers={"PRIVATE-TOKEN": e["GITLAB_ROOT_TOKEN"]})
    with urllib.request.urlopen(req) as r:
        return json.load(r)


def group(dest: str) -> str:
    return open(dest.rstrip("/") + ".group").read().strip()


def projects(dest: str) -> list:
    g = urllib.parse.quote(group(dest), safe="")
    return api(f"/groups/{g}/projects?include_subgroups=true&per_page=100")


def pull(dest: str) -> str:
    """Clone each project's default branch into <dest>.remote/<path under the group>."""
    e, g = env(), group(dest)
    out = dest.rstrip("/") + ".remote"
    shutil.rmtree(out, ignore_errors=True)
    base = e["GITLAB_URL"].replace("http://", f"http://root:{e['GITLAB_ROOT_TOKEN']}@")
    for p in projects(dest):
        rel = p["path_with_namespace"][len(g) + 1:]
        if not p.get("default_branch"):
            continue
        subprocess.run(["git", "clone", "-q", "--depth", "1", f"{base}/{p['path_with_namespace']}.git",
                        os.path.join(out, rel)], check=True, capture_output=True)
        shutil.rmtree(os.path.join(out, rel, ".git"))
    return out


def lines(dest: str) -> None:
    base = json.load(open(dest.rstrip("/") + ".baseline.json"))
    known = {v["sha"] for v in base.values()}
    seen, per_file = set(), {}
    remote = pull(dest)

    def count(rel_key: str, label: str, p: str) -> None:
        if not ab.counted(rel_key):
            return
        t = ab.text(p)
        if t is None:
            return
        h = ab.sha(t)
        if h in seen or (rel_key in base and base[rel_key]["sha"] == h):
            return
        if rel_key not in base and h in known:
            return
        if rel_key in base:
            old = base[rel_key]["text"].splitlines()
            n = sum(1 for ln in difflib.ndiff(old, t.splitlines()) if ln.startswith("+ ") and ln[2:].strip())
        else:
            n = sum(1 for ln in t.splitlines() if ln.strip())
        seen.add(h)
        if n:
            per_file[label] = n

    for rel, p in sorted(ab.files(remote)):
        count(rel, "gitlab:" + rel, p)
    for rel, p in sorted(ab.files(dest)):
        count(rel, rel, p)
    json.dump({"linesToMaintain": sum(per_file.values()), "files": per_file}, sys.stdout, indent=1)
    print()


def check(dest: str) -> None:
    """Each root CODEOWNERS on main is the import's with the task's rule set (replaced or appended)."""
    base = json.load(open(dest.rstrip("/") + ".baseline.json"))
    g = group(dest)
    out = {}
    for r in REPOS:
        before = base.get(f"{r}/CODEOWNERS", {}).get("text")
        pattern, owners = RULES[r]
        rows = before.splitlines() if before else []
        rule = f"{pattern} {owners}"
        hit = [i for i, ln in enumerate(rows) if ln.split()[:1] == [pattern]]
        if hit:
            rows[hit[0]] = rule
        else:
            rows.append(rule)
        want = "\n".join(rows) + "\n"
        pid = urllib.parse.quote(f"{g}/{r}", safe="")
        try:
            e = env()
            req = urllib.request.Request(
                f"{e['GITLAB_URL']}/api/v4/projects/{pid}/repository/files/CODEOWNERS/raw?ref=main",
                headers={"PRIVATE-TOKEN": e["GITLAB_ROOT_TOKEN"]})
            got = urllib.request.urlopen(req).read().decode()
        except urllib.error.HTTPError:
            got = None
        out[r] = {"present": got is not None, "exact": got == want,
                  "rule": rule in (got or "").splitlines()}
    out["pass"] = all(v["exact"] for v in out.values())
    json.dump(out, sys.stdout, indent=1)
    print()


if __name__ == "__main__":
    {"snapshot": ab.snapshot, "lines": lines, "check": check}[sys.argv[1]](sys.argv[2])

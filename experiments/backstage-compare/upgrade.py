#!/usr/bin/env python3
"""Upgrade cost and rollback, for the adopter. Prints one JSON object. Usage: upgrade.py <out.json>.

- Backstage: the create-app template is copied into an adopter's repository once and never updated
  for them; each release's template diff is what they merge by hand (the Upgrade Helper shows it).
  Counted over the last RELEASES main-line releases in a backstage/backstage clone, without the
  vendored Yarn binary.
- Toolbox: the oldest release's starter, unchanged, through the current binary's validate and
  test. The lines an adopter must change are 0 when both pass.
- Rollback: Toolbox release OLD serves a database, the current binary serves it, then OLD serves it
  again; the steps that fail are the manual steps. Backstage's documented rollback is a manual
  `knex migrate:down` per migration, run from the newer version
  (docs/tutorials/manual-knex-rollback.md), so its steps are the migrations the template's
  backend plugins added over the same window."""
import json
import os
import subprocess
import sys
import tarfile
import tempfile
import time
import urllib.request
from pathlib import Path

from measure import PG, ROOT, TOOLBOX, psql, wait

BS_REPO = Path(os.environ.get("BS_REPO", "/tmp/research/bs"))
BS_SRC = Path(os.environ.get("BS_SRC", "/tmp/bs/app"))
BS_FROM, BS_TO = os.environ.get("BS_FROM", "v1.51.0"), os.environ.get("BS_TO", "v1.57.1")
RELEASES = 6
TBX_OLDEST, TBX_OLD = "v0.1.0", os.environ.get("TOOLBOX_OLD", "v0.1.1")
PORT = 18093


def git(*args, cwd=ROOT):
    return subprocess.run(["git", *args], cwd=cwd, check=True, capture_output=True,
                          text=True).stdout


def backstage_template_lines():
    for tag in (BS_FROM, BS_TO):
        subprocess.run(["git", "fetch", "-q", "--depth", "1", "origin", "tag", tag], cwd=BS_REPO,
                       check=True)
    lines = 0
    for row in git("diff", "--numstat", BS_FROM, BS_TO, "--",
                   "packages/create-app/templates/default-app", cwd=BS_REPO).splitlines():
        added, deleted, path = row.split("\t")
        if "/.yarn/releases/" not in path:
            lines += int(added) + int(deleted)
    return lines


def backstage_rollback_migrations():
    """Migrations added over the window by the backend plugins the template installs: each is one
    manual `knex migrate:down` to roll back past it."""
    deps = json.loads((BS_SRC / "packages/backend/package.json").read_text())["dependencies"]
    plugins = {n.removeprefix("@backstage/plugin-") for n in deps if n.startswith("@backstage/plugin-")}
    count = 0
    for row in git("diff", "--name-status", "--diff-filter=A", BS_FROM, BS_TO, "--",
                   "plugins/*/migrations/*", cwd=BS_REPO).splitlines():
        path = row.split("\t")[1]
        if path.split("/")[1] in plugins and path.endswith(".js") and ".test." not in path:
            count += 1
    return count


def toolbox_required_lines():
    tmp = Path(tempfile.mkdtemp(prefix="starter-"))
    archive = subprocess.run(["git", "archive", TBX_OLDEST, "examples/repo-portal-starter"],
                             cwd=ROOT, check=True, capture_output=True).stdout
    subprocess.run(["tar", "-x", "-C", str(tmp)], input=archive, check=True)
    starter = tmp / "examples/repo-portal-starter"
    ok = all(subprocess.run([TOOLBOX, cmd, str(starter)], capture_output=True).returncode == 0
             for cmd in ("validate", "test"))
    if ok:
        return 0
    stat = git("diff", "--numstat", TBX_OLDEST, "HEAD", "--", "examples/repo-portal-starter")
    return sum(int(a) + int(d) for a, d, _ in (r.split("\t") for r in stat.splitlines()))


def old_binary():
    tmp = Path(tempfile.mkdtemp(prefix="toolbox-old-"))
    name = "toolbox-cli-x86_64-unknown-linux-gnu.tar.xz"
    subprocess.run(["gh", "release", "download", TBX_OLD, "-R", "loganvolkers/toolbox", "-p", name,
                    "-D", str(tmp)], check=True)
    with tarfile.open(tmp / name) as t:
        t.extractall(tmp, filter="data")
    return next(tmp.rglob("toolbox"))


def toolbox_rollback():
    """Steps that fail in OLD → current → OLD on one database; 0 means rollback by binary."""
    old = old_binary()
    psql("DROP DATABASE IF EXISTS bench_rollback")
    psql("CREATE DATABASE bench_rollback")
    catalog = Path(os.environ.get("CATALOGS", "/tmp/bc")) / "n1000" / "catalog.ndjson"
    portal = Path(__file__).resolve().parent / "portal"
    base = f"http://127.0.0.1:{PORT}"
    failed = 0
    for i, binary in enumerate([old, Path(TOOLBOX), old]):
        env = {k: v for k, v in os.environ.items() if not k.startswith("TOOLBOX_")}
        env.update(TOOLBOX_CONNECTORS="off", RUST_LOG="warn")
        proc = subprocess.Popen([str(binary), "serve", str(portal), "--listen", f"127.0.0.1:{PORT}",
                                 "--database-url", PG + "/bench_rollback"], env=env,
                                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        try:
            ready = wait(lambda: urllib.request.urlopen(f"{base}/readyz", timeout=2).status == 200,
                         60)
            if ready and i == 0:
                req = urllib.request.Request(f"{base}/v1/ingest", data=catalog.read_bytes(),
                                             headers={"x-toolbox-connector": "catalog"},
                                             method="POST")
                urllib.request.urlopen(req, timeout=120)
            read = urllib.request.Request(f"{base}/v1/entities/component:c999",
                                          headers={"x-toolbox-user": "bench"})
            if not (ready and wait(lambda: urllib.request.urlopen(read, timeout=2).status == 200,
                                   60)):
                failed += 1
        finally:
            proc.kill()
            proc.wait()
            time.sleep(0.5)
    return failed


def main():
    bs_lines = backstage_template_lines()
    bs_migrations = backstage_rollback_migrations()
    out = {
        "toolbox": {"upgradeLinesPerRelease": toolbox_required_lines(),
                    "rollbackManualStepsPerRelease": toolbox_rollback(),
                    "upgradeWindow": f"{TBX_OLD}..HEAD"},
        "backstage": {"upgradeLinesPerRelease": round(bs_lines / RELEASES, 1),
                      "upgradeTemplateLines": bs_lines,
                      "rollbackManualStepsPerRelease": round(bs_migrations / RELEASES, 1),
                      "rollbackMigrations": bs_migrations,
                      "upgradeWindow": f"{BS_FROM}..{BS_TO}"},
    }
    Path(sys.argv[1]).write_text(json.dumps(out, indent=2) + "\n")
    print(json.dumps(out))


if __name__ == "__main__":
    main()

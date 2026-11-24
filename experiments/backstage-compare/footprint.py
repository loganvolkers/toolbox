#!/usr/bin/env python3
"""Static facts about each product as an adopter installs it: bytes on disk, third-party packages,
the files and lines the adopter owns, known vulnerabilities, and release provenance. Prints one
JSON object per product. Usage: footprint.py <out.json>.

Toolbox is its released Linux archives (the version below) plus the OpenFGA binary the image adds;
its packages are Cargo.lock's, the Go modules built into OpenFGA (`go version -m`) and the UI's npm
lockfile, which is bundled into the binary.
Backstage is a `@backstage/create-app` app (BS_SRC, as generated) and its production install
(BS_APP: `yarn workspaces focus --all --production` plus `yarn build:backend`, as the template's
Dockerfile does), plus the Node runtime it needs."""
import json
import os
import shutil
import subprocess
import sys
import tarfile
import tempfile
import urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
TOOLBOX_VERSION = os.environ.get("TOOLBOX_VERSION", "v0.1.1")
TOOLS = Path(os.environ.get("TOOLS", "/tmp/tools"))
OPENFGA = Path(os.environ.get("TOOLBOX_OPENFGA_BIN", TOOLS / "openfga"))
OSV = os.environ.get("OSV_SCANNER", str(TOOLS / "osv-scanner"))
BS_SRC = Path(os.environ.get("BS_SRC", "/tmp/bs/app"))
BS_APP = Path(os.environ.get("BS_APP", "/tmp/bs/prod"))
NODE = Path(os.environ.get("NODE_HOME", "/tmp/bs/node24"))
# Packages checked for npm provenance: the template's core backend and CLI.
BS_PACKAGES = ["@backstage/create-app", "@backstage/cli", "@backstage/backend-defaults",
               "@backstage/plugin-catalog-backend", "@backstage/core-components"]
# What an adopter writes for a first portal with Toolbox: the starter, minus its tests.
STARTER = ROOT / "examples" / "repo-portal-starter"
CODE = {".ts", ".tsx", ".js", ".jsx", ".json", ".yaml", ".yml", ".md", ".hbs", ""}


def du(path):
    out = subprocess.run(["du", "-sbL", str(path)], capture_output=True, text=True, check=True)
    return int(out.stdout.split()[0])


def owned(root, skip=("node_modules", ".git", "dist", "dist-types", ".yarn")):
    """Files and lines the adopter keeps in their repository and maintains."""
    files = lines = 0
    for p in root.rglob("*"):
        if not p.is_file() or any(s in p.relative_to(root).parts for s in skip):
            continue
        if p.suffix not in CODE or p.name == "yarn.lock":
            continue
        files += 1
        lines += sum(1 for _ in p.open(errors="replace"))
    return files, lines


def toolbox_install():
    tmp = Path(tempfile.mkdtemp(prefix="tbx-release-"))
    out = subprocess.run(["gh", "release", "view", TOOLBOX_VERSION, "--json", "assets", "-q",
                          '.assets[].name | select(test("x86_64-unknown-linux-gnu.tar.xz$"))'],
                         capture_output=True, text=True, check=True, cwd=ROOT)
    names = out.stdout.split()
    subprocess.run(["gh", "release", "download", TOOLBOX_VERSION, "-D", str(tmp), "--clobber"]
                   + sum((["-p", n] for n in names), []), check=True, cwd=ROOT)
    archive_bytes = sum((tmp / n).stat().st_size for n in names)
    bins = tmp / "bin"
    bins.mkdir()
    for n in names:
        with tarfile.open(tmp / n) as t:
            for m in t.getmembers():
                if m.isfile() and m.mode & 0o111:
                    m.name = Path(m.name).name
                    t.extract(m, bins, filter="data")
    verified = 0
    for n in names:
        r = subprocess.run(["gh", "attestation", "verify", str(tmp / n), "-R", "loganvolkers/toolbox"],
                           capture_output=True, text=True, cwd=ROOT)
        verified += r.returncode == 0
    return {"installBytes": du(bins) + OPENFGA.stat().st_size,
            "downloadBytes": archive_bytes + OPENFGA.stat().st_size,
            "artifacts": len(names), "provenanceVerified": verified}


def cargo_packages():
    text = (ROOT / "Cargo.lock").read_text()
    blocks = text.split("[[package]]")[1:]
    # Workspace members have no `source`; everything else is third-party, dev dependencies too.
    return sum(1 for b in blocks if "\nsource = " in b)


def go_modules(binary):
    """Modules built into a Go binary, the main module and the standard library excluded."""
    out = subprocess.run(["go", "version", "-m", str(binary)], capture_output=True, text=True,
                         check=True).stdout
    return sum(1 for ln in out.splitlines() if ln.split()[:1] == ["dep"])


def npm_packages(lockfile):
    """Third-party packages in an npm lockfile, development dependencies included."""
    packages = json.loads(Path(lockfile).read_text())["packages"]
    return sum(1 for k in packages if k.startswith("node_modules/"))


def yarn_packages(lockfile):
    """Third-party resolutions in a Yarn lockfile, development dependencies included, as
    Cargo.lock counts them."""
    lines = Path(lockfile).read_text().splitlines()
    return sum(1 for ln in lines if ln.startswith("  resolution:") and "@workspace:" not in ln)


def vulns(*targets):
    """Advisories, and those with CVSS ≥ 7, in lockfiles and Go binaries. An advisory without a
    score (Go's own database rates none) counts as known, not high."""
    total = high = 0
    for target in targets:
        t, h = scan(Path(target))
        total, high = total + t, high + h
    return total, high


def scan(target):
    if target.suffix in (".lock", ".json"):
        args = ["-L", str(target)]
    else:
        alone = Path(tempfile.mkdtemp(prefix="osv-bin-"))
        shutil.copy(target, alone)
        args = ["--experimental-plugins", "go/binary", "-r", str(alone)]
    r = subprocess.run([OSV, "scan", "source", "--format", "json", *args],
                       capture_output=True, text=True)
    data = json.loads(r.stdout or "{}")
    total = high = 0
    for res in data.get("results", []):
        for pkg in res.get("packages", []):
            for g in pkg.get("groups", []):
                total += 1
                if float(g.get("max_severity") or 0) >= 7.0:
                    high += 1
    return total, high


def npm_provenance():
    have = 0
    for name in BS_PACKAGES:
        url = "https://registry.npmjs.org/" + name.replace("/", "%2f")
        with urllib.request.urlopen(url, timeout=30) as r:
            meta = json.load(r)
        latest = meta["dist-tags"]["latest"]
        have += bool(meta["versions"][latest].get("dist", {}).get("attestations"))
    return have


def main():
    tbx = toolbox_install()
    tbx_files, tbx_lines = owned(STARTER, skip=("tests", "fixtures"))
    ui_lock = ROOT / "ui" / "package-lock.json"
    tbx_vulns, tbx_high = vulns(ROOT / "Cargo.lock", OPENFGA, ui_lock)
    bs_files, bs_lines = owned(BS_SRC)
    bs_vulns, bs_high = vulns(BS_APP / "yarn.lock")
    out = {
        "toolbox": {
            "installMb": round(tbx["installBytes"] / 1e6, 1),
            "downloadMb": round(tbx["downloadBytes"] / 1e6, 1),
            "thirdPartyPackages": cargo_packages() + go_modules(OPENFGA) + npm_packages(ui_lock),
            "adopterFiles": tbx_files, "adopterLines": tbx_lines,
            "knownVulns": tbx_vulns, "highVulns": tbx_high,
            "provenancePercent": round(100 * tbx["provenanceVerified"] / tbx["artifacts"]),
        },
        "backstage": {
            "installMb": round((du(BS_APP) + du(NODE)) / 1e6, 1),
            "downloadMb": None,
            "thirdPartyPackages": yarn_packages(BS_APP / "yarn.lock"),
            "adopterFiles": bs_files, "adopterLines": bs_lines,
            "knownVulns": bs_vulns, "highVulns": bs_high,
            "provenancePercent": round(100 * npm_provenance() / len(BS_PACKAGES)),
        },
    }
    Path(sys.argv[1]).write_text(json.dumps(out, indent=2) + "\n")
    print(json.dumps(out, indent=2))


if __name__ == "__main__":
    main()

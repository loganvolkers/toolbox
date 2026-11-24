#!/usr/bin/env python3
"""Decide the counted facts and run the probes, once per arm. Writes raw/facts.json.

Counted: binaryMb, goModules, knownVulns, provenance, sbom, contributors, releasesLastYear.
Probed against a server started with its defaults (default ports, which must be free):
wildcards, exclusion, conditions, contextualTuples, listUsers, consistencyTokens, changeFeed,
multiTenant, httpByDefault, prometheusMetrics; against a server configured to export traces to a
local collector: otelTracing; with each project's CLI: modelTests; with the server binary:
migrateCommand. Every other fact is read from facts.yaml. A probe that disagrees with facts.yaml
wins and is reported.

Needs gh (authenticated), go, govulncheck, slsa-verifier and cosign on PATH; run.sh installs the
pinned ones. Usage: facts.py (after run.sh has downloaded the releases and built load/loadgen).
"""
import datetime as dt
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

import yaml

HERE = Path(__file__).resolve().parent
RAW = HERE / "raw"
LOGS = HERE / "logs"
DL = HERE / "downloads"
LOADGEN = HERE / "load" / "loadgen"
ARMS = {
    "openfga": {
        "repo": "openfga/openfga", "tag": "v1.23.0",
        "archive": "openfga_1.23.0_linux_amd64.tar.gz",
        "server": [str(HERE / "bin" / "openfga"), "run", "--datastore-engine", "memory"],
        "addr": "127.0.0.1:8081", "ports": [8080, 8081, 2112, 3000],
        "http": "http://127.0.0.1:8080/stores", "metrics": "http://127.0.0.1:2112/metrics",
        "trace": ["--grpc-addr", "127.0.0.1:18081", "--http-addr", "127.0.0.1:18080",
                  "--metrics-addr", "127.0.0.1:18112", "--trace-enabled",
                  "--trace-otlp-endpoint", "127.0.0.1:18317", "--trace-sample-ratio", "1"],
        "traceAddr": "127.0.0.1:18081",
        "migrate": [str(HERE / "bin" / "openfga"), "migrate", "--help"],
        "test": [str(DL / "cli" / "fga"), "model", "test", "--tests"], "testFile": "model.fga.yaml",
    },
    "spicedb": {
        "repo": "authzed/spicedb", "tag": "v1.57.1",
        "archive": "spicedb_1.57.1_linux_amd64.tar.gz",
        "server": [str(HERE / "bin" / "spicedb"), "serve", "--datastore-engine", "memory",
                   "--grpc-preshared-key", "facts-probe-key"],
        "key": "facts-probe-key",
        "addr": "127.0.0.1:50051", "ports": [50051, 8443, 9090, 50053],
        "http": "http://127.0.0.1:8443/v1/schema/read", "metrics": "http://127.0.0.1:9090/metrics",
        "trace": ["--grpc-addr", "127.0.0.1:18051", "--metrics-addr", "127.0.0.1:18090",
                  "--otel-provider", "otlpgrpc", "--otel-endpoint", "127.0.0.1:18317",
                  "--otel-insecure", "--otel-sample-ratio", "1"],
        "traceAddr": "127.0.0.1:18051",
        "migrate": [str(HERE / "bin" / "spicedb"), "datastore", "migrate", "--help"],
        "test": [str(DL / "cli" / "zed"), "validate"], "testFile": "schema.validation.yaml",
    },
}
GO_ENV = {**os.environ, "GOTOOLCHAIN": os.environ.get("GOTOOLCHAIN", "go1.27.4")}


def run(cmd, **kw):
    p = subprocess.run(cmd, capture_output=True, text=True, env=kw.pop("env", GO_ENV), **kw)
    return {"command": " ".join(str(c) for c in cmd), "exit": p.returncode,
            "output": (p.stdout + p.stderr).strip()}


def gh_json(path):
    p = subprocess.run(["gh", "api", "--paginate", path], capture_output=True, text=True, check=True)
    # --paginate concatenates JSON arrays page by page.
    out, dec, s, i = [], json.JSONDecoder(), p.stdout, 0
    while i < len(s):
        while i < len(s) and s[i].isspace():
            i += 1
        if i >= len(s):
            break
        page, i = dec.raw_decode(s, i)
        out += page if isinstance(page, list) else [page]
    return out


def listening():
    ports = set()
    for f in ("/proc/net/tcp", "/proc/net/tcp6"):
        for line in Path(f).read_text().splitlines()[1:]:
            p = line.split()
            if p[3] == "0A":
                ports.add(int(p[1].split(":")[1], 16))
    return ports


def tool_versions():
    return {
        "go": run(["go", "version"])["output"],
        "govulncheck": run(["govulncheck", "-version"])["output"],
        "slsa-verifier": re.sub(r"(?s).*?(GitVersion:\s*\S+).*", r"\1",
                                run(["slsa-verifier", "version"])["output"]),
        "cosign": re.sub(r"(?s).*?(GitVersion:\s*\S+).*", r"\1", run(["cosign", "version"])["output"]),
        "gh": run(["gh", "--version"])["output"].splitlines()[0],
        "fga": run([str(DL / "cli" / "fga"), "version"])["output"],
        "zed": run([str(DL / "cli" / "zed"), "version"])["output"].splitlines()[0],
    }


def binary_facts(arm):
    b = HERE / "bin" / arm
    mods = run(["go", "version", "-m", str(b)])
    lines = mods["output"].splitlines()
    deps = [line.split("\t")[2] for line in lines if line.startswith("\tdep\t")]
    return {
        "binaryBytes": b.stat().st_size,
        "binaryMb": round(b.stat().st_size / 1e6, 1),
        "binarySha256": hashlib.sha256(b.read_bytes()).hexdigest(),
        "builtWith": lines[0].split(": ", 1)[1] if lines else None,
        "goModules": len(deps),
        "goModulesCommand": f"go version -m bin/{arm} | grep -c '^\\tdep\\t'",
    }


def objects(text):
    dec, i, out = json.JSONDecoder(), 0, []
    while i < len(text):
        while i < len(text) and text[i].isspace():
            i += 1
        if i >= len(text):
            break
        o, i = dec.raw_decode(text, i)
        out.append(o)
    return out


def vulns(arm):
    """govulncheck -mode=binary: a finding whose trace starts at a function is a vulnerable symbol
    linked into the binary. Keeps the config, those findings and their OSV entries."""
    p = subprocess.run(["govulncheck", "-mode=binary", "-format", "json", str(HERE / "bin" / arm)],
                       capture_output=True, text=True, env=GO_ENV)
    objs = objects(p.stdout)
    findings = [o["finding"] for o in objs if "finding" in o]
    called = sorted({f["osv"] for f in findings if f["trace"] and f["trace"][0].get("function")})
    kept = [o for o in objs if "config" in o]
    kept += [o for o in objs if "osv" in o and o["osv"]["id"] in called]
    kept += [{"finding": f} for f in findings]
    (RAW / f"govulncheck-{arm}.json").write_text(json.dumps(kept, indent=1) + "\n")
    return {"knownVulns": len(called), "knownVulnIds": called,
            "knownVulnsCommand": f"govulncheck -mode=binary -format json bin/{arm}",
            "knownVulnsExit": p.returncode, "allFindingIds": sorted({f["osv"] for f in findings})}


def release_facts(arm, cfg, today):
    rel = json.loads(subprocess.run(["gh", "api", f"repos/{cfg['repo']}/releases/tags/{cfg['tag']}"],
                                    capture_output=True, text=True, check=True).stdout)
    assets = [a["name"] for a in rel["assets"]]
    archive = DL / arm / cfg["archive"]
    sig_like = [a for a in assets if re.search(r"\.(intoto\.jsonl|sig|pem|bundle|att|sigstore(\.json)?)$", a)]
    sboms = [a for a in assets if re.search(r"sbom|spdx|cyclonedx|\.cdx", a, re.I)]
    checks = [run(["gh", "attestation", "verify", str(archive), "-R", cfg["repo"]])]
    intoto = [a for a in assets if a.endswith(".intoto.jsonl")]
    for a in intoto:
        checks.append(run(["slsa-verifier", "verify-artifact", str(archive),
                           "--provenance-path", str(DL / arm / a),
                           "--source-uri", f"github.com/{cfg['repo']}", "--source-tag", cfg["tag"]]))
    if {"checksums.txt.sig", "checksums.txt.pem"} <= set(assets):
        checks.append(run(["cosign", "verify-blob", str(DL / arm / "checksums.txt"),
                           "--signature", str(DL / arm / "checksums.txt.sig"),
                           "--certificate", str(DL / arm / "checksums.txt.pem"),
                           "--certificate-identity-regexp", f"^https://github.com/{cfg['repo']}/",
                           "--certificate-oidc-issuer", "https://token.actions.githubusercontent.com"]))
    provenance = any(c["exit"] == 0 for c in checks if "attestation" in c["command"]
                     or "slsa-verifier" in c["command"])
    archive_sbom = [a for a in sboms if a.startswith(cfg["archive"])]

    since = today - dt.timedelta(days=365)
    releases = gh_json(f"repos/{cfg['repo']}/releases?per_page=100")
    in_year = [r for r in releases if r.get("published_at")
               and since <= dt.datetime.fromisoformat(r["published_at"].replace("Z", "+00:00")) < today]
    contributors = gh_json(f"repos/{cfg['repo']}/contributors?anon=1&per_page=100")
    return {
        "releasePublishedAt": rel["published_at"], "releaseAssets": assets,
        "signatureAssets": sig_like, "sbomAssets": sboms,
        "provenance": provenance, "provenanceChecks": checks,
        "sbom": bool(archive_sbom), "sbomForArchive": archive_sbom,
        "releasesLastYear": len(in_year),
        "releasesLastYearPrerelease": sum(1 for r in in_year if r["prerelease"]),
        "releasesWindow": [since.isoformat(), today.isoformat()],
        "releasesLastYearTags": [r["tag_name"] for r in in_year],
        "contributors": len(contributors),
        "contributorsAnonymous": sum(1 for c in contributors if c.get("type") == "Anonymous"),
        "contributorsCommand": f"gh api --paginate 'repos/{cfg['repo']}/contributors?anon=1&per_page=100'",
    }


def model_tests(arm, cfg):
    passing = run(cfg["test"] + [cfg["testFile"]], cwd=HERE)
    doc = yaml.safe_load((HERE / cfg["testFile"]).read_text())
    if arm == "openfga":
        for t in doc["tests"]:
            for c in t["check"]:
                c["assertions"] = {k: not v for k, v in c["assertions"].items()}
    else:
        a = doc["assertions"]
        a["assertTrue"], a["assertFalse"] = a["assertFalse"], a["assertTrue"]
    with tempfile.NamedTemporaryFile("w", dir=HERE, prefix=".flipped-", suffix=Path(cfg["testFile"]).name,
                                     delete=False) as f:
        yaml.safe_dump(doc, f, sort_keys=False)
    try:
        failing = run(cfg["test"] + [Path(f.name).name], cwd=HERE)
    finally:
        os.unlink(f.name)
    value = passing["exit"] == 0 and failing["exit"] != 0
    return {"value": value, "detail": f"{passing['command']}: exit {passing['exit']}; "
            f"with assertions flipped: exit {failing['exit']}", "passing": passing, "flipped": failing}


def probes(arm, cfg):
    busy = listening() & set(cfg["ports"])
    if busy:
        sys.exit(f"facts.py: default ports {sorted(busy)} are in use; the probes need them free")
    LOGS.mkdir(exist_ok=True)
    key = ["-key", cfg["key"]] if "key" in cfg else []
    p = subprocess.run([str(LOADGEN), "probe", "-dir", str(HERE), "-arm", arm, "-addr", cfg["addr"], *key,
                        "-http", cfg["http"], "-metrics", cfg["metrics"],
                        "-log", str(LOGS / f"probe-{arm}.log"), "--", *cfg["server"]],
                       capture_output=True, text=True, check=True)
    out = json.loads(p.stdout)
    t = subprocess.run([str(LOADGEN), "traceprobe", "-dir", str(HERE), "-arm", arm, "-addr", cfg["traceAddr"],
                        *key, "-otlp", "127.0.0.1:18317", "-log", str(LOGS / f"trace-{arm}.log"),
                        "--", *cfg["server"], *cfg["trace"]],
                       capture_output=True, text=True, check=True)
    trace = json.loads(t.stdout)
    out["facts"].update(trace["facts"])
    out["traceServer"] = trace["server"]
    out["facts"]["modelTests"] = model_tests(arm, cfg)
    m = run(cfg["migrate"])
    out["facts"]["migrateCommand"] = {"value": m["exit"] == 0 and "migrat" in m["output"].lower(),
                                      "detail": f"{m['command']}: exit {m['exit']}; "
                                      + m["output"].splitlines()[0][:160]}
    return out


def main():
    RAW.mkdir(exist_ok=True)
    today = dt.datetime.now(dt.timezone.utc).replace(microsecond=0)
    declared = yaml.safe_load((HERE / "facts.yaml").read_text())
    result = {"runAt": today.isoformat(), "tools": tool_versions(), "arms": {}}
    disagreements = []
    for arm, cfg in ARMS.items():
        print(f"facts: {arm}", file=sys.stderr)
        rec = {**binary_facts(arm), **vulns(arm), **release_facts(arm, cfg, today)}
        probed = probes(arm, cfg)
        rec["probe"] = probed
        decided = {}
        for name, entry in declared[arm].items():
            value = entry["value"]
            src = "facts.yaml"
            if entry.get("probe"):
                p = entry["probe"]
                got = probed["facts"][p]["value"] if p in probed["facts"] else rec.get(p)
                if got is None:
                    sys.exit(f"facts.py: probe {p} for {arm} gave no answer")
                if got != value:
                    disagreements.append(f"{arm}.{name}: facts.yaml says {value}, probe says {got}")
                value, src = got, f"probe:{p}"
            decided[name] = {"value": value, "from": src}
        rec["decided"] = decided
        result["arms"][arm] = rec
    result["disagreements"] = disagreements
    (RAW / "facts.json").write_text(json.dumps(result, indent=1) + "\n")
    for d in disagreements:
        print("facts: DISAGREEMENT " + d, file=sys.stderr)
    print(f"facts: wrote raw/facts.json ({len(disagreements)} disagreements with facts.yaml)", file=sys.stderr)


if __name__ == "__main__":
    if not shutil.which("gh"):
        sys.exit("facts.py: gh is required")
    main()

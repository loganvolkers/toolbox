#!/usr/bin/env python3
"""M0 parity check: `seikika validate` vs examples/reference-portal/tools/validate.py.

For the reference portal and every case in crates/seikika-model/tests/cases.json, build a mutated
copy of the portal, run both validators, and compare: both must accept the reference portal; for
each case, Rust must reject with the expected code, and Python must reject exactly when the case
says `python: true` (Rust-only checks are stricter by design).

Usage: python scripts/validate-parity.py [path/to/seikika]   (default target/debug/seikika)
Requires: pyyaml, jsonschema.
"""
import json
import os
import shutil
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
                                 "examples", "reference-portal", "tools"))
from yamlutil import load as load_yaml
import yaml

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PORTAL = os.path.join(REPO, "examples", "reference-portal")
PYVALIDATE = os.path.join(PORTAL, "tools", "validate.py")
SEIKIKA = sys.argv[1] if len(sys.argv) > 1 else os.path.join(REPO, "target", "debug", "seikika")

def set_pointer(doc, pointer, value):
    parts = pointer.lstrip("/").split("/")
    node = doc
    for p in parts[:-1]:
        node = node[int(p)] if isinstance(node, list) else node[p]
    key = parts[-1]
    if isinstance(node, list):
        node[int(key)] = value
    elif value is None:
        del node[key]
    else:
        node[key] = value

def run(portal):
    py = subprocess.run([sys.executable, "-I", PYVALIDATE, portal], capture_output=True, text=True)
    rs = subprocess.run([SEIKIKA, "validate", portal, "--format", "json"], capture_output=True, text=True)
    report = json.loads(rs.stdout)
    return py.returncode != 0, rs.returncode, {e["code"] for e in report.get("errors", [])}

def main():
    rows, bad = [], 0
    py_rej, rs_code, _ = run(PORTAL)
    ok = not py_rej and rs_code == 0
    bad += not ok
    rows.append(("reference portal", "accept", "accept" if not py_rej else "REJECT", "accept" if rs_code == 0 else "REJECT", ok))
    cases = json.load(open(os.path.join(REPO, "crates", "seikika-model", "tests", "cases.json")))["cases"]
    for case in cases:
        tmp = tempfile.mkdtemp(prefix="parity-")
        portal = os.path.join(tmp, "portal")
        shutil.copytree(PORTAL, portal)
        if "set" in case:
            path = os.path.join(portal, "seikika.yaml")
            doc = load_yaml(path)
            for pointer, value in case["set"].items():
                set_pointer(doc, pointer, value)
            yaml.safe_dump(doc, open(path, "w"), sort_keys=False)
        if "fixture" in case:
            with open(os.path.join(portal, case["fixture"]["file"]), "a") as f:
                f.write(json.dumps(case["fixture"]["line"]) + "\n")
        py_rej, rs_code, codes = run(portal)
        shutil.rmtree(tmp)
        ok = rs_code == 1 and case["code"] in codes and py_rej == case["python"]
        bad += not ok
        rows.append((case["name"], case["code"], "reject" if py_rej else "accept",
                     ("reject " if rs_code == 1 else f"exit {rs_code} ") + ",".join(sorted(codes)), ok))
    w = max(len(r[0]) for r in rows)
    print(f"{'case':<{w}}  {'expected':<22} {'validate.py':<12} seikika validate")
    for name, exp, py, rs, ok in rows:
        print(f"{name:<{w}}  {exp:<22} {py:<12} {rs}{'' if ok else '   <-- MISMATCH'}")
    print(f"\n{len(rows)} checks, {bad} mismatch(es)")
    return 1 if bad else 0

if __name__ == "__main__":
    sys.exit(main())

"""Check the generated Kubernetes Go workspace against the staging modules."""

import json
import os
import subprocess
from pathlib import Path

def read_go_file(command, path):
    result = subprocess.run(
        ["go", command, "edit", "-json", str(path)],
        env={**os.environ, "GOWORK": "off", "GOTOOLCHAIN": "local"},
        capture_output=True,
        check=False,
        text=True,
    )
    if result.returncode:
        raise SystemExit(result.stderr)
    return json.loads(result.stdout)


members = read_go_file("work", "go.work")["Use"] or []
entries = [str(Path(use["DiskPath"])) for use in members]
staging = Path("staging/src/k8s.io")
manifests = sorted(staging.glob("*/go.mod"))
expected = {".", *(str(path.parent) for path in manifests)}

findings = []
if not manifests:
    findings.append(f"No staging module manifests found under {staging}")
if len(entries) != len(set(entries)):
    findings.append("go.work lists a workspace member more than once")
findings.extend(f"go.work is missing ./{path}" for path in sorted(expected - set(entries)))
findings.extend(f"go.work lists an unexpected member {path}" for path in sorted(set(entries) - expected))
for path in manifests:
    module = read_go_file("mod", path)["Module"] or {}
    expected_name = "k8s.io/" + path.parent.name
    if module.get("Path") != expected_name:
        findings.append(f"{path} must declare module {expected_name}")
if findings:
    raise SystemExit("\n".join(findings))

#!/usr/bin/env python3
"""Take-it-out check (docs/design/seikika-and-tools.md, promise B1): every tool in the box must give
the same findings when it is run on its own.

For each record a portal's connector reports with a `command` (lint records from the runner), run
that command alone, count the findings for the record's file from the tool's own output, and
compare with the record's `total`. For OpenSSF Scorecard records, compare the check's score. The counting here is independent of Seikika's parsers, so a
box that dropped, added or filtered findings fails the check.

    python3 scripts/take-it-out.py portal ci-lint      # run from the repository root

Exit 1 on any mismatch, 2 if the connector can't be read.
"""

import json
import os
import subprocess
import sys


def norm(path):
    path = path.removeprefix("file://")
    cwd = os.getcwd() + "/"
    return path.removeprefix(cwd).removeprefix("./")


def count(via, text, path):
    """Findings for `path` in a tool's own output, read the way the tool documents it."""
    text = text.strip()
    if via == "sarif":
        doc = json.loads(text)
        return sum(
            1
            for run in doc.get("runs", [])
            for r in run.get("results", [])
            if norm(r["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]) == path
        )
    if via == "zizmor":
        return sum(
            1
            for f in json.loads(text)
            if norm(f["locations"][0]["symbolic"]["key"]["Local"]["verbatim_path"]) == path
        )
    if via == "actionlint":
        items = json.loads(text) if text and text != "null" else []
        return sum(1 for e in items if norm(e["filepath"]) == path)
    if via == "semgrep":
        return sum(1 for r in json.loads(text)["results"] if norm(r["path"]) == path)
    raise ValueError(f"no independent counter for via={via}")


def score(text, check):
    """A Scorecard check's score from Scorecard's own JSON (-1, inconclusive, is None)."""
    for c in json.loads(text)["checks"]:
        if c["name"] == check:
            return c["score"] if c["score"] >= 0 else None
    return "missing"


def main(portal, connectors):
    seikika = os.environ.get("SEIKIKA", "seikika")
    outputs = {}  # a whole-tree command runs once
    checked = mismatched = 0
    for name in connectors:
        res = subprocess.run(
            [seikika, "connector", "records", "--portal", portal, "--name", name],
            capture_output=True, text=True,
        )
        if res.returncode != 0:
            print(f"error: connector {name}: {res.stderr.strip()}", file=sys.stderr)
            return 2
        for line in res.stdout.splitlines():
            rec = json.loads(line)
            data = rec.get("data", {})
            cmd = data.get("command")
            if not cmd or "via" not in data:
                continue
            if cmd not in outputs:
                outputs[cmd] = subprocess.run(cmd, shell=True, capture_output=True, text=True).stdout
            if data["via"] == "scorecard":  # one record per check, scored 0-10, not per file
                got, said = score(outputs[cmd], data["check"]), data["score"]
            else:
                got, said = count(data["via"], outputs[cmd], data["path"]), data["total"]
            checked += 1
            if got != said:
                mismatched += 1
                print(f"MISMATCH {rec['externalId']}: the box says {said}, `{cmd}` gives {got}")
    print(f"take-it-out: {checked} record(s) checked, {mismatched} mismatch(es)")
    return 1 if mismatched else 0


if __name__ == "__main__":
    if len(sys.argv) < 3:
        print(__doc__.strip().splitlines()[-3], file=sys.stderr)
        sys.exit(2)
    sys.exit(main(sys.argv[1], sys.argv[2:]))

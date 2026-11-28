#!/usr/bin/env python3
"""Replay two assessment loops in a disposable maintainer portal."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import time

REPO = Path(__file__).resolve().parents[2]
MANIFEST = """apiVersion: seikika.com/v1alpha1
portal: {name: loop-demo, title: Loop demo, owner: group:maintainers, seikika: "0.2"}
include:
  - package: ../meta-scoring
    with:
      describeCommand: [seikika, describe, portal, .seikika]
      manifest: .seikika/seikika.yaml
connectors:
  - name: things
    package: builtin:runner
    runtime: in-image
    config:
      root: .
      jobs:
        - {name: facts, command: [cat, facts.ndjson], parse: ndjson, kinds: [thing]}
    records: [thing]
kinds:
  - name: Thing
    identityFrom: [connector:things]
    fields: {hash: {type: string}, quality: {type: integer}}
    sources: {default: [connector:things]}
    map:
      - {from: things/thing, id: "record.id", set: "{'hash': record.hash, 'quality': record.quality}"}
rubrics:
  - name: thing-review
    kind: Thing
    criteria:
      - {name: useful, question: Is the quality value at least 1?}
campaigns:
  - name: verify-evidence
    title: Verify assessment evidence
    owner: group:maintainers
    start: 2026-10-01
    end: 2099-01-01
    targetPercent: 100
    includes:
      - {scorecard: review-assessment, atLeast: gold}
"""


def write_assessment(root, path, entity, rubric, subject_hash, rubric_hash, author, verdicts):
    file = root / ".seikika" / path
    file.parent.mkdir(parents=True, exist_ok=True)
    body = {
        "apiVersion": "seikika.com/assessment/v1",
        "entity": entity,
        "rubric": rubric,
        "subjectHash": subject_hash,
        "assessor": {"kind": "person", "name": author},
        "at": "2026-11-28",
        "verdicts": verdicts,
    }
    if rubric_hash is not None:
        body["rubricHash"] = rubric_hash
    file.write_text(json.dumps(body, indent=2) + "\n")


def run_loops(root, cli, out):
    shutil.copytree(REPO / "packages/meta-scoring", root / "meta-scoring")
    portal = root / ".seikika"
    portal.mkdir()
    manifest = MANIFEST.replace("[seikika, describe", f"[{json.dumps(str(cli))}, describe")
    (portal / "seikika.yaml").write_text(manifest)
    facts = root / "facts.ndjson"
    env = dict(os.environ)
    env.setdefault("SEIKIKA_CONNECTORS_DIR", str(cli.parent))
    measurements = {}

    def check(label, *scope, expect=0):
        run_record = out / f"{label}.run.json"
        started = time.perf_counter()
        proc = subprocess.run(
            [cli, "assess", "check", ".seikika", "--queue", "--with-briefs",
             "--run-manifest", run_record, "-o", "json", *scope],
            cwd=root, env=env, capture_output=True,
        )
        elapsed = time.perf_counter() - started
        assert proc.returncode == expect, (label, proc.stderr.decode())
        record = json.loads(run_record.read_text())
        assert record["exitCode"] == expect, record
        if expect == 0:
            (out / f"{label}.json").write_bytes(proc.stdout)
            assert record["outputSha256"] == hashlib.sha256(proc.stdout).hexdigest()
        measurements[label] = {"seconds": round(elapsed, 3), "stdoutBytes": len(proc.stdout)}
        return (json.loads(proc.stdout) if expect == 0 else record), record

    for name, digest, quality in [("a", "0123456789ab", 1), ("b", "abcdef123456", 1)]:
        fact = {"type": "record", "kind": "thing", "externalId": f"thing:{name}",
                "data": {"id": f"thing:{name}", "hash": digest, "quality": quality}}
        with facts.open("a") as stream:
            stream.write(json.dumps(fact) + "\n")

    # Loop 1 uses a relocated assessment with an obsolete subject hash.
    a_path = "assessments/thing-review/thing/legacy-a.yml"
    a_scope = ("--rubric", "thing-review", "--entity", "thing:a")
    verdict = [{"criterion": "useful", "verdict": "pass",
                "reason": "The quality for thing:a is 1 in facts.ndjson."}]
    write_assessment(root, a_path, "thing:a", "thing-review", "000000000000", None,
                     "Fixture Original", verdict)
    before, before_run = check("refresh-before", *a_scope)
    row, = before["queue"]
    assert row["reasons"] == ["subject-changed"], row
    assert row["assessmentPath"] == row["brief"]["file"] == a_path, row
    brief = row["brief"]
    a = json.loads((portal / a_path).read_text())
    a.update(subjectHash=brief["subjectHash"], rubricHash=brief["rubricHash"])
    (portal / a_path).write_text(json.dumps(a, indent=2) + "\n")
    after, after_run = check("refresh-after", *a_scope, "--require-fresh")
    assert after["totalPending"] == 0 and not after["invalid"], after
    assert before_run["manifestSha256"] == after_run["manifestSha256"]
    assert before_run["parsedAssessmentFilesSha256"] != after_run["parsedAssessmentFilesSha256"]

    # Loop 2 tests a same-name handoff, rejects a simulated self-review, and verifies
    # the campaign after a groundable fixture review by a distinct fixture assessor.
    b_path = "assessments/thing-review/thing/b.yaml"
    write_assessment(root, b_path, "thing:b", "thing-review", "abcdef123456", brief["rubricHash"],
                     "Fixture Original", [{**verdict[0], "reason": "The quality for thing:b is 1 in facts.ndjson."}])
    entity = "assessment:thing-review/thing:b"
    b_scope = ("--campaign", "verify-evidence", "--rubric", "assessment-quality", "--entity", entity)
    handoff, _ = check("review-handoff", *b_scope, "--reviewer", "fixture ORIGINAL")
    blocked, = handoff["queue"]
    assert blocked["action"] == "hand-off-review" and "brief" not in blocked, blocked
    independent, _ = check("review-brief", *b_scope, "--reviewer", "Fixture Independent")
    review, = independent["queue"]
    assert review["action"] == "independent-review", review
    brief = review["brief"]
    criteria = [
        {"criterion": c["name"], "verdict": "pass",
         "reason": f"The fixture verifies the recorded fact against .seikika/{b_path} and facts.ndjson."}
        for c in brief["rubric"]["criteria"]
    ]
    args = (root, brief["file"], entity, "assessment-quality", brief["subjectHash"], brief["rubricHash"])
    write_assessment(*args, "FIXTURE ORIGINAL", criteria)
    self_review, _ = check("review-self", *b_scope)
    row, = self_review["queue"]
    assert row["action"] == "hand-off-review" and "self-review" in row["reasons"], row
    write_assessment(*args, "Fixture Independent", criteria)
    closed, _ = check("review-after", *b_scope, "--require-fresh")
    assert closed["totalPending"] == 0 and not closed["invalid"], closed

    facts.unlink()
    failure, _ = check("collector-error", *a_scope, expect=2)
    assert failure["connectors"]["things"]["status"] == "error", failure
    return measurements


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", type=Path, default=REPO / "target/debug/seikika")
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    args.out.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="seikika-assessment-loop-") as work:
        measurements = run_loops(Path(work), args.cli.resolve(), args.out.resolve())
    report = json.dumps(measurements, indent=2) + "\n"
    (args.out / "measurements.json").write_text(report)
    print(report, end="")


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Replay documented Seikika assessment cases against pinned Git objects."""

import argparse
from collections import Counter
import json
import os
from pathlib import Path
import re
import subprocess
import tarfile
from tempfile import TemporaryDirectory


ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
AUDITED = "ab60efa83fe3ea6425c9195db0ff8044547edc75"
BEFORE_SDK = "25a208a"
AFTER_SDK = "b727e5f"


def command(*args, cwd=ROOT, env=None):
    return subprocess.run(args, cwd=cwd, env=env, capture_output=True, text=True, timeout=180)


def output(*args, cwd=ROOT, env=None):
    result = command(*args, cwd=cwd, env=env)
    if result.returncode:
        raise RuntimeError(f"{args} exited {result.returncode}\n{result.stdout[:2000]}\n{result.stderr[:4000]}")
    return result.stdout


def git_file(ref, path):
    return output("git", "show", f"{ref}:{path}")


def unpack(ref, dest):
    dest.mkdir()
    with subprocess.Popen(["git", "archive", ref], cwd=ROOT, stdout=subprocess.PIPE) as archive:
        with tarfile.open(fileobj=archive.stdout, mode="r|*") as files:
            files.extractall(dest, filter="data")
        if archive.wait():
            raise RuntimeError(f"git archive failed for {ref}")


def assessment_contract(work, cli):
    """Replay #108's old stale predicate with the same fixture used by the current CLI."""
    old_source = git_file("47e087d^", "crates/toolbox-cli/src/assess.rs")
    assert "if invalid.is_empty() && !stale" in old_source
    fixtures = git_file(AUDITED, "crates/seikika-cli/tests/assess.rs")
    manifest, assessment = [
        re.search(rf'const {name}: &str = r#"(.*?)"#;', fixtures, re.S).group(1)
        for name in ("MANIFEST", "ASSESSMENT")
    ]
    manifest = manifest.replace("    rules:", "    ratchet: {advisory: true}\n    rules:", 1)
    portal = work / "assessment"
    for name in ("fixtures", "tests", "assessments/thing-review/thing"):
        (portal / name).mkdir(parents=True)
    (portal / "tests/t.test.yaml").write_text(
        'name: t\nassert:\n  - {name: unassessed, expect: "score(\'reviewed\', \'thing:a\').level == \'none\'"}\n'
    )
    version = "0123456789ab"
    thing = {"type": "record", "kind": "thing", "externalId": "thing:a",
             "data": {"name": "a", "hash": version}}
    source = portal / "fixtures/things.ndjson"
    source.write_text(json.dumps(thing) + "\n")
    record = {"type": "record", "kind": "assessment", "externalId": "assessment:thing-review/thing:a",
              "data": {"entity": "thing:a", "rubric": "thing-review", "subjectHash": version,
                       "verdict": "pass"}}
    (portal / "fixtures/assessments.ndjson").write_text(json.dumps(record) + "\n")
    spec = portal / "seikika.yaml"
    spec.write_text(manifest)
    doc = portal / "assessments/thing-review/thing/a.yaml"

    def check():
        args = [cli, "assess", "check", str(portal), "--fixtures", "-o", "json"]
        default = command(*args)
        strict = command(*args, "--require-fresh")
        report = json.loads(default.stdout)
        assert report == json.loads(strict.stdout)
        row = report["rubrics"]["thing-review"][0]
        old_reconstructed_exit = int(bool(report["invalid"]) or row["state"] == "stale")
        return {"state": row["state"], "verdict": row.get("verdict"),
                "invalid_files": len(report["invalid"]), "pre_108_reconstructed_exit": old_reconstructed_exit,
                "default_exit": default.returncode, "require_fresh_exit": strict.returncode}

    def ratchet(*extra):
        result = command(cli, "report", str(portal), "--fixtures", "--baseline",
                         str(portal / "baseline.json"), "--verdict", str(portal / "verdict.json"), *extra)
        verdict = json.loads((portal / "verdict.json").read_text())
        return result.returncode, verdict["result"]

    def read_brief():
        brief = output(cli, "assess", "brief", "thing:a", "--portal", str(portal), "--fixtures", "-o", "json")
        return json.loads(brief)

    rows = {"missing": check()}
    doc.write_text(assessment)
    original_brief = read_brief()
    rows["fresh_pass"] = check()
    assert ratchet("--update")[0] == 0
    spec.write_text(manifest.replace("Is it useful?", "Does it solve the operator's task?"))
    rows["rubric_question_changed"] = check() | {
        "subject_hash_unchanged": read_brief()["subjectHash"] == original_brief["subjectHash"],
    }
    if "rubricHash" in original_brief:
        doc.write_text(assessment + 'rubricHash: "' + original_brief["rubricHash"] + '"\n')
        rows["bound_rubric_question_changed"] = check()
        rows["bound_rubric_question_changed"].pop("pre_108_reconstructed_exit")
        assert rows["bound_rubric_question_changed"]["require_fresh_exit"] == 1
    spec.write_text(manifest)
    doc.write_text(assessment.replace("verdict: pass", "verdict: fail"))
    rows["fresh_fail"] = check()
    doc.write_text(assessment.replace('subjectHash: "0123456789ab"', 'subjectHash: "111111111111"'))
    rows["stale"] = check()
    thing["data"]["hash"] = "222222222222"
    source.write_text(json.dumps(thing) + "\n")
    rows["stale"]["advisory_ratchet"] = ratchet("--ci")
    doc.write_text(assessment.replace("criterion: useful", "criterion: other"))
    rows["absent_required_criterion"] = check()
    doc.write_text(assessment + "  - {criterion: useful, verdict: fail, reason: Incompatible with the earlier verdict.}\n")
    thing["data"]["hash"] = version
    source.write_text(json.dumps(thing) + "\n")
    rows["conflicting_duplicate_criterion"] = check()
    rows["conflicting_duplicate_criterion"]["pre_108_reconstructed_exit"] = 0
    assert rows["missing"]["require_fresh_exit"] == 0
    assert rows["stale"]["default_exit"] == 0 and rows["stale"]["require_fresh_exit"] == 1
    assert rows["fresh_fail"]["require_fresh_exit"] == 0
    assert rows["absent_required_criterion"]["default_exit"] == 1
    assert rows["conflicting_duplicate_criterion"]["default_exit"] == int("rubricHash" in original_brief)
    assert rows["stale"]["advisory_ratchet"] == (0, "pass")
    assert rows["rubric_question_changed"]["subject_hash_unchanged"]
    return rows


def catalog_coverage(cli):
    def coverage(*scope):
        result = command(cli, "assess", "check", "portal", "--skip", "runner,github-actions", "-o", "json", *scope)
        if result.returncode not in (0, 1):
            raise RuntimeError(result.stderr)
        report = json.loads(result.stdout)
        return {"exit": result.returncode, "invalid_files": len(report["invalid"]),
                "rubrics": {rubric: dict(Counter(row["state"] for row in rows))
                            for rubric, rows in report["rubrics"].items()},
                "total": dict(Counter(row["state"] for rows in report["rubrics"].values() for row in rows))}

    return {"default": coverage(), "require_fresh": coverage("--require-fresh")}


def run_sdk(work):
    """Use #139's checker on its audit's starting revision and its repaired revision."""
    def scan(tree):
        env = os.environ | {"CARGO_NET_OFFLINE": "true"}
        records = map(json.loads, output("node", "scripts/sdk-docs.mjs", cwd=tree, env=env).splitlines())
        return {r["data"]["name"]: {kind: len(r["data"][kind]) for kind in ("undocumented", "referenceGaps")}
                for r in records}

    results = {}
    for ref in (BEFORE_SDK, AFTER_SDK):
        tree = work / ref
        unpack(ref, tree)
        (tree / "scripts/sdk-docs.mjs").write_text(git_file(AFTER_SDK, "scripts/sdk-docs.mjs"))
        for folder in (".", "ui-kit", "report-ui"):
            (tree / folder / "node_modules").symlink_to(ROOT / folder / "node_modules", target_is_directory=True)
        results[ref] = scan(tree)
    assert sum(r["undocumented"] + r["referenceGaps"] for r in results[BEFORE_SDK].values()) > 0
    assert all(not count for row in results[AFTER_SDK].values() for count in row.values())
    return results


def acceptance_logs(work):
    """Run both #135 shell scripts with a failed fixture push and stubbed dependencies."""
    results = {}
    for ref in ("5818e03^", "5818e03"):
        tree = work / ref
        for folder in ("scripts", "bin", "tmp"):
            (tree / folder).mkdir(parents=True)
        script = tree / "scripts/m2-acceptance.sh"
        script.write_text(git_file(ref, "scripts/m2-acceptance.sh"))
        stubs = {
            "initdb": "exit 0\n", "curl": "exit 0\n",
            "pg_ctl": 'while [[ $# -gt 0 ]]; do\n  if [[ $1 == -l ]]; then\n'
                      '    echo "fixture postgres context" > "$2"\n    exit 0\n  fi\n  shift\ndone\n',
            "seikika": 'if [[ $1 == serve ]]; then\n  echo "fixture server context"\n'
                       '  touch "$TMPDIR/server-ready"\n  exit 0\nfi\n'
                       'while [[ ! -e $TMPDIR/server-ready ]]; do sleep 0.02; done\nexit 42\n',
        }
        for name, body in stubs.items():
            stub = tree / "bin" / name
            stub.write_text("#!/usr/bin/env bash\n" + body)
            stub.chmod(0o755)
        env = os.environ | {"PGBIN": str(tree / "bin"), "PGDIR": str(tree / "db"),
                            "TMPDIR": str(tree / "tmp"), "SEIKIKA": str(tree / "bin/seikika"),
                            "PATH": f"{tree / 'bin'}:{os.environ['PATH']}"}
        run = command("bash", str(script), cwd=tree, env=env)
        results[ref] = {"exit": run.returncode, "postgres_log": "fixture postgres context" in run.stderr,
                        "server_log": "fixture server context" in run.stderr,
                        "temporary_work_removed": not any(p.is_dir() for p in (tree / "tmp").iterdir())}
    assert results["5818e03^"] == {"exit": 42, "postgres_log": False, "server_log": False, "temporary_work_removed": True}
    assert results["5818e03"] == {"exit": 42, "postgres_log": True, "server_log": True, "temporary_work_removed": True}
    return results


def quickstart(work, cli):
    """Run #126's published walkthrough with pinned Git assets and an installed CLI."""
    source = "c14c9fbb60405f9fea12d86de3d0b32b717755d9"
    repaired = "80252c2"
    path = "site/src/content/docs/start/quickstart.md"
    before = git_file(source, path)
    after = git_file(repaired, path)
    starter = f"https://raw.githubusercontent.com/seikika/seikika/{source}/examples/repo-portal-starter"
    old_assets = re.findall(r"https://github.com/seikika/seikika/releases/download/v0\.2\.0/[^ ]+\.sh", before)
    assert len(old_assets) == 2 and starter in after
    snapshot = work / "quickstart-starter"
    for path in ("seikika.yaml", "tests/standards.test.yaml", "fixtures/workspace.ndjson"):
        dest = snapshot / path
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_text(git_file(source, f"examples/repo-portal-starter/{path}"))

    local_bin = work / "quickstart-bin"
    local_bin.mkdir()
    curl = local_bin / "curl"
    curl.write_text('#!/usr/bin/env bash\nset -eu\n'
                    '[[ $1 == -fsSL && $2 == "$REPLAY_STARTER_URL/"* && $3 == -o ]]\n'
                    'cp "$REPLAY_STARTER_DIR/${2#"$REPLAY_STARTER_URL/"}" "$4"\n')
    curl.chmod(0o755)
    env = os.environ | {"PATH": f"{local_bin}:{Path(cli).parent}:{os.environ['PATH']}",
                        "REPLAY_STARTER_URL": starter, "REPLAY_STARTER_DIR": str(snapshot)}
    setup, baseline, regression, repair = re.findall(r"^```bash\n(.*?)^```", after, re.M | re.S)
    output("bash", "-e", "-c", setup, cwd=work, env=env)
    repo = work / "seikika-practice"
    checks = [command("bash", "-e", "-c", block, cwd=repo, env=env) for block in (baseline, regression, repair)]
    exits = [result.returncode for result in checks]
    assert exits == [0, 1, 0], [(result.stdout, result.stderr) for result in checks]
    evidence = "\n".join((checks[1].stdout, checks[1].stderr))
    assert "links-resolve" in evidence and "portal.md" in evidence
    return {"original_revision": source, "original_installer_links_to_unavailable_tag": len(old_assets),
            "repaired_revision": repaired, "repaired_walkthrough_exits": exits, "regression_named_expected_link": True}


def design_rounds():
    """Replay observable scores. Proposed admission policy is a retrospective simulation."""
    audit = git_file(AUDITED, "docs/audits/seikika-report-2026-11-24-2.md")
    dimensions = [list(map(int, row.split(" | ")[2:])) for row in re.findall(r"^\| U\d+ (.+?) \|$", audit, re.M)]
    totals = list(map(sum, zip(*dimensions, strict=True)))
    assert len(totals) == 36
    plateau = list(zip(*(scores[13:] for scores in dimensions), strict=True))
    differences = [tuple(now - before for before, now in zip(a, b, strict=True))
                   for a, b in zip(plateau, plateau[1:])]
    later = git_file(AUDITED, "docs/audits/seikika-report-2026-11-24-3.md")
    panel_rounds = len(re.findall(r"^## Round \d+: what the panel found$", later, re.M))
    assert panel_rounds == 18
    return {"loop_2": {"audits": 36, "usability_round_14": totals[13],
                       "post_round_14_audits": 22, "post_round_14_mean_usability": round(sum(totals[14:]) / 22, 2),
                       "post_round_14_adjacent_rounds_without_ordinal_drop": sum(min(d) >= 0 for d in differences),
                       "post_round_14_dimension_comparisons": sum(map(len, differences)),
                       "post_round_14_dimension_drops": sum(sum(diff < 0 for diff in d) for d in differences),
                       "counterfactual_two_more_admissions_after_round_14": 16,
                       "counterfactual_calls_avoided": 20},
            "loop_3": {"recorded_panel_rounds": panel_rounds, "audits_counted_in_panels": 3 * panel_rounds,
                       "explicit_duplicate_panels": sum(x in later for x in ("A duplicate panel was started", "A second panel ("))}}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", type=Path, default=ROOT / "target/debug/seikika")
    parser.add_argument("--case", choices=("assessment", "coverage", "sdk", "acceptance", "quickstart", "rounds", "browser"),
                        action="append")
    args = parser.parse_args()
    cli = str(args.cli.resolve())
    with TemporaryDirectory(prefix="seikika-assessment-replay-") as directory:
        work = Path(directory)
        cases = {"assessment": lambda: assessment_contract(work, cli), "coverage": lambda: catalog_coverage(cli),
                 "sdk": lambda: run_sdk(work), "acceptance": lambda: acceptance_logs(work),
                 "quickstart": lambda: quickstart(work, cli), "rounds": design_rounds,
                 "browser": lambda: json.loads(output("node", str(HERE / "report-states.mjs")))}
        print(json.dumps({name: cases[name]() for name in dict.fromkeys(args.case or cases)}, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()

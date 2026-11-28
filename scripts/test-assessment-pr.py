#!/usr/bin/env python3
"""Exercise the assessment pilot against disposable Git trees and local fake adapters."""

import importlib.util
import json
from pathlib import Path
import sqlite3
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch


SCRIPT = Path(__file__).with_name("assessment-pr.py")
spec = importlib.util.spec_from_file_location("assessment_pr", SCRIPT)
pilot = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pilot)
adapter_spec = importlib.util.spec_from_file_location("assessment_codex", SCRIPT.with_name("assessment-codex.py"))
codex = importlib.util.module_from_spec(adapter_spec)
adapter_spec.loader.exec_module(codex)


class AssessmentTests(unittest.TestCase):
    def setUp(self):
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.root = Path(scratch.name)
        self.repo = self.root / "repo"
        self.repo.mkdir()
        self.ledger = self.root / "ledger.sqlite"
        self.git("init", "-q")
        self.git("config", "user.email", "review@example.invalid")
        self.git("config", "user.name", "Assessment fixtures")
        (self.repo / pilot.POLICY).parent.mkdir()
        self.policy = {
            "format": "seikika-pr-policy/v1", "owner": "group:maintainers",
            "checks": {"probe": [sys.executable, "-c", "print('probe evidence')"]},
            "routes": [{
                "id": "docs", "subject": "First result", "paths": ["*.md"], "risk": "high",
                "owner": "group:docs", "checks": ["probe"], "design": True,
                "criteria": {"reader": "Can the reader run the example?"},
            }],
        }
        (self.repo / pilot.POLICY).write_text(json.dumps(self.policy))
        (self.repo / "old.md").write_text("First line\nSecond line\n")
        self.base = self.save()
        self.git("mv", "old.md", "new.md")
        (self.repo / "unclassified.txt").write_text("ambiguous\n")
        self.head = self.save()
        self.plan, self.definitions = pilot.build_plan(self.repo, self.base, self.head)
        self.key = "seikika/seikika", 12

    def git(self, *args):
        return subprocess.check_output(["git", "-C", str(self.repo), *args], text=True).strip()

    def save(self):
        self.git("add", "-A")
        self.git("commit", "-qm", "fixture")
        return self.git("rev-parse", "HEAD")

    def verdict(self, **fields):
        return {"route": "docs", "criterion": "reader", "verdict": "pass", "reason": "The check ran.",
                "evidence": [{"path": "new.md", "line": 1, "side": "head"}]} | fields

    def test_pinned_plan_rename_delete_and_unmapped(self):
        self.assertEqual(self.plan["selected"][0]["paths"], ["new.md", "old.md"])
        self.assertEqual(self.plan["unmappedPaths"], ["unclassified.txt"])
        self.assertEqual(self.plan["scope"], "scope_unknown")
        self.assertEqual(self.plan["checks"], ["probe"])
        (self.repo / pilot.POLICY).write_text("{}")
        self.git("rm", "new.md")
        later = self.save()
        changed, _ = pilot.build_plan(self.repo, self.base, later)
        self.assertEqual(changed["checks"], ["probe"])
        self.assertIn({"status": "D", "paths": ["old.md"]}, changed["changes"])
        self.assertEqual(pilot.build_plan(self.repo, self.base, self.head)[0], self.plan)

    def test_bootstrap_changes_select_the_onboarding_probe(self):
        policy = SCRIPT.parents[1] / pilot.POLICY
        (self.repo / pilot.POLICY).write_bytes(policy.read_bytes())
        base = self.save()
        for path in (
            "crates/seikika-cli/src/main.rs",
            "crates/seikika-cli/src/init.rs",
            "crates/seikika-cli/src/init-catalog.yaml",
            "crates/seikika-connector-workspace/src/site.rs",
        ):
            with self.subTest(path=path):
                self.git("switch", "--quiet", "--detach", base)
                changed = self.repo / path
                changed.parent.mkdir(parents=True, exist_ok=True)
                changed.write_text("fixture\n")
                plan, _ = pilot.build_plan(self.repo, base, self.save())
                self.assertEqual(plan["checks"], ["module-coupling", "onboarding", "report-states"])
                self.assertEqual(plan["unmappedPaths"], [])

    def test_check_transport_and_evidence_validation(self):
        output = self.root / "check"
        evidence = pilot.run_checks(self.repo, self.plan, self.definitions, [], output)
        checks = pilot.load_evidence(self.plan, [output / "evidence.json"])
        self.assertEqual(evidence["obligations"], {"docs": "incomplete"})
        self.assertEqual((output / "probe.log").read_text(), "probe evidence\n")
        row = self.verdict(evidence=[{"path": "old.md", "line": 2, "side": "base"}, {"check": "probe"}])
        data = {"plan": self.plan["digest"], "verdicts": [row]}
        self.assertEqual(pilot.validate_result(self.repo, self.plan, checks, json.dumps(data)), data)
        self.assertEqual(pilot.obligations(self.plan, checks, data), {"docs": "needs_owner_decision"})
        for invalid in ([], [row, row], [self.verdict(evidence=[{"path": "new.md", "line": 3, "side": "head"}])],
                        [self.verdict(evidence=[{"path": "old.md", "line": 1, "side": "head"}])],
                        [self.verdict(verdict="fail")], [self.verdict(evidence=[{"path": "unclassified.txt",
                                                                               "line": 1, "side": "head"}])]):
            with self.subTest(invalid=invalid), self.assertRaises(ValueError):
                pilot.validate_result(self.repo, self.plan, checks, json.dumps(data | {"verdicts": invalid}))
        (output / "probe.log").write_text("altered")
        with self.assertRaisesRegex(ValueError, "digest mismatch"):
            pilot.load_evidence(self.plan, [output / "evidence.json"])

    def test_reservations_are_monotonic_and_limits_are_shared(self):
        with pilot.transaction(self.ledger) as conn, patch.object(pilot.time, "time", return_value=5000):
            state, attempt = pilot.reserve(conn, self.key, 1, self.plan, "mock", None)
            self.assertEqual(state, "running")
            next_plan = self.plan | {"digest": "next"}
            self.assertEqual(pilot.reserve(conn, self.key, 2, next_plan, "mock", None)[0], "busy")
            self.assertEqual(pilot.settle(conn, self.key, self.plan, attempt, "reviewed", {}), "superseded")
            self.assertEqual(pilot.reserve(conn, self.key, 1, self.plan, "mock", None)[0], "superseded")
            state, attempt = pilot.reserve(conn, self.key, 2, next_plan, "mock", None)
            self.assertEqual(state, "running")
            pilot.settle(conn, self.key, next_plan, attempt, "invalid-result", {})
            self.assertEqual(pilot.reserve(conn, self.key, 2, next_plan, "mock", None)[0], "duplicate")
            third = self.plan | {"digest": "third"}
            self.assertEqual(pilot.reserve(conn, self.key, 3, third, "mock", None)[0], "pr-hourly-limit")
            for i in range(6):
                self.assertEqual(pilot.reserve(conn, (self.key[0], i), 1, self.plan, "mock", None)[0], "running")
            self.assertEqual(pilot.reserve(conn, (self.key[0], 19), 1, self.plan, "mock", None)[0],
                             "repository-hourly-limit")

    def test_cycle_limit_and_design_pause(self):
        with pilot.transaction(self.ledger) as conn:
            for generation in (1, 2):
                with patch.object(pilot.time, "time", return_value=generation * 3601):
                    plan = self.plan | {"digest": str(generation)}
                    state, attempt = pilot.reserve(conn, self.key, generation, plan, "mock", None)
                    self.assertEqual(state, "running")
                    pilot.settle(conn, self.key, plan, attempt, "reviewed", {"verdicts": [self.verdict()]})
            with patch.object(pilot.time, "time", return_value=3 * 3601):
                plan = self.plan | {"digest": "third"}
                self.assertEqual(pilot.reserve(conn, self.key, 3, plan, "mock", None)[0], "design-plateau")
                state, attempt = pilot.reserve(conn, self.key, 3, plan, "mock", "release checkpoint")
                self.assertEqual(state, "running")
                pilot.settle(conn, self.key, plan, attempt, "reviewed", {"verdicts": [self.verdict()]})
            with patch.object(pilot.time, "time", return_value=4 * 3601):
                plan = self.plan | {"digest": "fourth"}
                self.assertEqual(pilot.reserve(conn, self.key, 4, plan, "mock", None)[0], "cycle-limit")

    def test_operator_entrypoint_executes_one_adapter_and_records_usage(self):
        evidence = self.root / "evidence"
        pilot.run_checks(self.repo, self.plan, self.definitions, [], evidence)
        response = {"verdicts": [self.verdict()], "usage": {"inputTokens": 250, "outputTokens": 100, "turns": 1}}
        adapter = "import json, sys; p=json.load(sys.stdin); data=json.loads(sys.argv[1]); print(json.dumps(data | {'plan': p['plan']['digest']}))"
        command = [sys.executable, str(SCRIPT), "--repo", str(self.repo), "delegate", "--base", self.base,
                   "--head", self.head, "--ledger", str(self.ledger), "--repository", self.key[0],
                   "--pr", str(self.key[1]), "--generation", "1", "--adapter", "mock",
                   "--evidence", str(evidence / "evidence.json"), "--", sys.executable, "-c", adapter,
                   json.dumps(response)]
        result = json.loads(subprocess.check_output(command))
        self.assertEqual(result["state"], "reviewed")
        self.assertEqual(result["result"]["usageSource"], "adapter-reported")
        self.assertEqual(json.loads(subprocess.check_output(command))["state"], "duplicate")
        with pilot.transaction(self.ledger) as conn:
            attempts = pilot.history(conn, self.key)["attempts"]
        self.assertEqual(len(attempts), 1)
        self.assertTrue(attempts[0]["current"])

        index = command.index("--")
        adjudication = command[:index] + ["--slot", "adjudication", "--override-reason", "confirm high impact"] + command[index:]
        self.assertEqual(json.loads(subprocess.check_output(adjudication))["state"], "reviewed")
        self.assertEqual(json.loads(subprocess.check_output(adjudication))["state"], "duplicate")

        response["usage"]["inputTokens"] = pilot.MAX_TOKENS
        command[command.index("--pr") + 1] = "13"
        command[-1] = json.dumps(response)
        result = json.loads(subprocess.check_output(command))
        self.assertEqual(result["state"], "budget-exceeded")
        self.assertEqual(result["obligations"], {"docs": "incomplete"})

    def test_codex_adapter_reads_structured_output_and_usage(self):
        response = {"plan": self.plan["digest"], "verdicts": [self.verdict(
            evidence=[{"path": None, "line": None, "side": None, "check": "probe"}],
        )]}
        events = [
            {"type": "item.completed", "item": {"type": "agent_message", "text": json.dumps(response)}},
            {"type": "turn.completed", "usage": {"input_tokens": 120, "cached_input_tokens": 20, "output_tokens": 30}},
        ]
        result = codex.read_events(map(json.dumps, events))
        self.assertEqual(result["verdicts"][0]["evidence"], [{"check": "probe"}])
        self.assertEqual(result["usage"], {"inputTokens": 120, "cachedInputTokens": 20, "outputTokens": 30, "turns": 1})

    def test_active_process_is_cancelled_when_superseded(self):
        with pilot.transaction(self.ledger) as conn:
            pilot.activate(conn, self.key, 1, self.plan["digest"])
        start = time.monotonic()

        def cancelled():
            with pilot.transaction(self.ledger) as conn:
                if time.monotonic() - start > 0.1:
                    pilot.activate(conn, self.key, 2, "new-plan")
                return not pilot.is_current(conn, self.key, self.plan["digest"])

        result, _, _ = pilot.execute([sys.executable, "-c", "import time; time.sleep(60)"],
                                     cwd=self.root, deadline=2, cancelled=cancelled)
        self.assertEqual(result, "superseded")
        with sqlite3.connect(self.ledger) as conn:
            self.assertTrue(pilot.is_current(conn, self.key, "new-plan"))


if __name__ == "__main__":
    unittest.main()

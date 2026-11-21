#!/usr/bin/env python3
"""Tests for the federation prototype applicator (design §8 acceptance)."""

from __future__ import annotations

import json
import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
COMPOSE = ROOT / "compose.py"


def run(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, str(COMPOSE), *args],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )


class FederationPrototypeTests(unittest.TestCase):
    def test_check_passes(self) -> None:
        p = run("check")
        self.assertEqual(p.returncode, 0, p.stderr)
        self.assertIn("check OK", p.stdout)

    def test_weaken_rejected(self) -> None:
        p = run("weaken")
        self.assertEqual(p.returncode, 0, p.stderr)
        self.assertIn("weakens", p.stdout)

    def test_push_enterprise_rules_at_leaf(self) -> None:
        p = run("report")
        self.assertEqual(p.returncode, 0, p.stderr)
        report = json.loads(p.stdout)
        scorecards = set(report["push"]["scorecardsAtLeaf"])
        self.assertIn("codeql", scorecards)
        self.assertIn("prettier", scorecards)
        self.assertIn("agents-md", scorecards)
        self.assertIn("branch-rulesets", scorecards)
        # Org extension also present when leaf includes org-payments.
        org = set(report["push"]["orgExtensionScorecards"])
        self.assertIn("payments-idempotency", org)
        kinds = set(report["push"]["kindsAtLeaf"])
        self.assertTrue({"Repository", "Service", "Component", "Library"} <= kinds)

    def test_extend_grass_roots(self) -> None:
        p = run("report")
        self.assertEqual(p.returncode, 0, p.stderr)
        report = json.loads(p.stdout)
        self.assertIn("cart-latency", report["extend"]["checkoutScorecards"])
        grass_names = {g["name"] for g in report["extend"]["grassRoots"]}
        self.assertIn("checkout-grassroots", grass_names)
        self.assertEqual(
            next(g for g in report["extend"]["grassRoots"] if g["name"] == "checkout-grassroots")[
                "stage"
            ],
            "experimental",
        )

    def test_results_up_and_observe_only(self) -> None:
        p = run("report")
        self.assertEqual(p.returncode, 0, p.stderr)
        report = json.loads(p.stdout)
        adopted = {(a["node"], a["package"]) for a in report["adoption"]}
        self.assertIn(("repo-payments-api", "enterprise"), adopted)
        self.assertIn(("checkout", "checkout-grassroots"), adopted)
        self.assertIn("codeql-zero-findings", report["campaigns"])
        self.assertGreaterEqual(report["campaigns"]["codeql-zero-findings"]["inScope"], 1)
        self.assertTrue(report["observeOnly"])
        self.assertEqual(report["observeOnly"][0]["mobileStack"], "expo")
        self.assertEqual(report["operators"]["enterprise"], "governance")
        self.assertEqual(report["operators"]["repo-payments-api"], "leaf")

    def test_catalog_beyond_repos(self) -> None:
        p = run("report")
        self.assertEqual(p.returncode, 0, p.stderr)
        report = json.loads(p.stdout)
        kinds = set(report["catalog"]["kinds"])
        self.assertTrue({"Repository", "Service", "Component", "Library"} <= kinds)
        ids = {e["id"] for e in report["catalog"]["entities"]}
        self.assertIn("service:acme/payments-api", ids)
        self.assertIn("component:acme/payments-api/checkout", ids)
        self.assertIn("library:acme/payments-api/ledger-core", ids)
        self.assertIn("repository:acme/mobile-shop", ids)
        self.assertGreaterEqual(report["catalog"]["counts"]["Service"], 1)
        self.assertGreaterEqual(report["catalog"]["counts"]["Library"], 1)


if __name__ == "__main__":
    unittest.main()

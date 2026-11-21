#!/usr/bin/env python3
"""Correctness tests for the OSS monorepo federation testbed."""

from __future__ import annotations

import json
import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
COMPOSE = ROOT / "compose.py"
RENDER = ROOT / "render_portal.py"


def run(*args: str, script: Path = COMPOSE) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, str(script), *args],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )


class OssMonorepoFederationTests(unittest.TestCase):
    def test_check_passes(self) -> None:
        p = run("check")
        self.assertEqual(p.returncode, 0, p.stderr)
        self.assertIn("check OK", p.stdout)

    def test_weaken_rejected(self) -> None:
        p = run("weaken")
        self.assertEqual(p.returncode, 0, p.stderr)
        self.assertIn("weakens", p.stdout)

    def test_push_shared_scorecards_at_each_leaf(self) -> None:
        p = run("report")
        self.assertEqual(p.returncode, 0, p.stderr)
        report = json.loads(p.stdout)
        shared = set(report["push"]["sharedScorecards"])
        for leaf in ("babel", "jest", "pnpm"):
            scorecards = set(report["push"]["leaves"][leaf]["scorecards"])
            self.assertTrue(
                shared <= scorecards, f"{leaf} missing shared scorecards"
            )
            kinds = set(report["push"]["leaves"][leaf]["kinds"])
            self.assertTrue({"Repository", "Library", "Component"} <= kinds)

    def test_extend_leaf_specific_standards(self) -> None:
        p = run("report")
        self.assertEqual(p.returncode, 0, p.stderr)
        report = json.loads(p.stdout)
        leaf_sc = report["extend"]["leafScorecards"]
        self.assertIn("husky-lint-staged", leaf_sc["babel"])
        self.assertIn("yarn-berry", leaf_sc["babel"])
        self.assertIn("lerna-lite", leaf_sc["jest"])
        self.assertIn("prettier-jest-style", leaf_sc["jest"])
        self.assertIn("changesets", leaf_sc["pnpm"])
        self.assertIn("rustfmt", leaf_sc["pnpm"])
        self.assertIn("clippy-deny-warnings", leaf_sc["pnpm"])
        grass = {g["name"] for g in report["extend"]["grassRoots"]}
        self.assertEqual(grass, {"babel-standards", "jest-standards", "pnpm-standards"})

    def test_results_up_and_campaign_rollup(self) -> None:
        p = run("report")
        self.assertEqual(p.returncode, 0, p.stderr)
        report = json.loads(p.stdout)
        adopted = {(a["node"], a["package"]) for a in report["adoption"]}
        self.assertIn(("babel", "oss-fleet"), adopted)
        self.assertIn(("jest", "oss-fleet"), adopted)
        self.assertIn(("pnpm", "oss-fleet"), adopted)
        self.assertIn(("babel", "babel-standards"), adopted)
        self.assertIn("deps-automation", report["campaigns"])
        # babel + pnpm meet; jest does not (no renovate observed)
        self.assertEqual(report["campaigns"]["deps-automation"]["inScope"], 3)
        self.assertEqual(report["campaigns"]["deps-automation"]["meeting"], 2)
        self.assertEqual(report["operators"]["oss-fleet"], "governance")
        self.assertEqual(report["operators"]["babel"], "leaf")

    def test_catalog_beyond_repos(self) -> None:
        p = run("report")
        self.assertEqual(p.returncode, 0, p.stderr)
        report = json.loads(p.stdout)
        kinds = set(report["catalog"]["kinds"])
        self.assertTrue({"Repository", "Library", "Component"} <= kinds)
        ids = {e["id"] for e in report["catalog"]["entities"]}
        self.assertIn("repository:babel/babel", ids)
        self.assertIn("repository:jestjs/jest", ids)
        self.assertIn("repository:pnpm/pnpm", ids)
        self.assertIn("library:babel/babel/core", ids)
        self.assertIn("library:jestjs/jest/jest-cli", ids)
        self.assertIn("library:pnpm/pnpm/pnpm", ids)
        self.assertGreaterEqual(report["catalog"]["counts"]["Library"], 3)

    def test_fixture_package_managers_match_upstream(self) -> None:
        babel = json.loads((ROOT / "leaves/babel/fixtures/package.json").read_text())
        jest = json.loads((ROOT / "leaves/jest/fixtures/package.json").read_text())
        pnpm = json.loads((ROOT / "leaves/pnpm/fixtures/package.json").read_text())
        self.assertTrue(babel["packageManager"].startswith("yarn@4."))
        self.assertTrue(jest["packageManager"].startswith("yarn@4."))
        self.assertTrue(pnpm["packageManager"].startswith("pnpm@"))
        prettier = json.loads((ROOT / "leaves/babel/fixtures/.prettierrc").read_text())
        self.assertEqual(prettier["printWidth"], 80)

    def test_render_portal_html(self) -> None:
        out = ROOT / "out" / "portal-test.html"
        p = run("-o", str(out), script=RENDER)
        self.assertEqual(p.returncode, 0, p.stderr + p.stdout)
        self.assertTrue(out.is_file())
        text = out.read_text(encoding="utf-8")
        self.assertIn("OSS monorepo federation portal", text)
        self.assertIn("repository:babel/babel", text)
        self.assertIn("repository:jestjs/jest", text)
        self.assertIn("repository:pnpm/pnpm", text)
        self.assertIn("deps-automation", text)


if __name__ == "__main__":
    unittest.main()

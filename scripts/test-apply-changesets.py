#!/usr/bin/env python3
"""Fixtures for scripts/apply-changesets.py (Cargo+CHANGELOG and package YAML)."""

from __future__ import annotations

import shutil
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path

SCRIPT = Path(__file__).resolve().parent / "apply-changesets.py"
FIXTURES = Path(__file__).resolve().parent / "testdata" / "apply-changesets"


def run_apply(work: Path) -> None:
    subprocess.run([sys.executable, str(SCRIPT), "--root", str(work)], check=True)


def test_cargo_changelog() -> None:
    src = FIXTURES / "cargo-changelog"
    with tempfile.TemporaryDirectory() as tmp:
        work = Path(tmp) / "repo"
        shutil.copytree(src, work)
        run_apply(work)
        data = tomllib.loads((work / "Cargo.toml").read_text(encoding="utf-8"))
        assert data["workspace"]["package"]["version"] == "0.2.0", data
        cl = (work / "CHANGELOG.md").read_text(encoding="utf-8")
        assert "## 0.2.0" in cl
        assert "demo cargo bump" in cl
        pending = [
            p
            for p in (work / ".changeset").iterdir()
            if p.suffix == ".md" and p.name.lower() != "readme.md"
        ]
        assert not pending


def test_package_yaml() -> None:
    src = FIXTURES / "package-yaml"
    with tempfile.TemporaryDirectory() as tmp:
        work = Path(tmp) / "repo"
        shutil.copytree(src, work)
        run_apply(work)
        yaml_text = (work / "pkg-fixture" / "demo" / "toolbox-package.yaml").read_text(
            encoding="utf-8"
        )
        assert "version: 0.1.1" in yaml_text
        data = tomllib.loads((work / "Cargo.toml").read_text(encoding="utf-8"))
        assert data["workspace"]["package"]["version"] == "0.1.1"


def main() -> int:
    test_cargo_changelog()
    print("ok: cargo-changelog")
    test_package_yaml()
    print("ok: package-yaml")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

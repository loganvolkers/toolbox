#!/usr/bin/env python3
"""Fixtures for scripts/apply-changesets.py (Cargo, YAML, dynamic includes)."""

from __future__ import annotations

import importlib.util
import shutil
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path

SCRIPT = Path(__file__).resolve().parent / "apply-changesets.py"
FIXTURES = Path(__file__).resolve().parent / "testdata" / "apply-changesets"
REPO_ROOT = Path(__file__).resolve().parent.parent


def load_applicator():
    spec = importlib.util.spec_from_file_location("apply_changesets", SCRIPT)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def run_apply(work: Path, *extra: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, str(SCRIPT), "--root", str(work), *extra],
        check=True,
        capture_output=True,
        text=True,
    )


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
        yaml_text = (work / "pkg-fixture" / "demo" / "seikika-package.yaml").read_text(
            encoding="utf-8"
        )
        assert "version: 0.1.1" in yaml_text
        data = tomllib.loads((work / "Cargo.toml").read_text(encoding="utf-8"))
        assert data["workspace"]["package"]["version"] == "0.1.1"


def test_dynamic_includes_glob_discovery_lockstep() -> None:
    """Glob expands packages + actions; lockstep bumps all packages; decoy untouched."""
    src = FIXTURES / "dynamic-includes"
    with tempfile.TemporaryDirectory() as tmp:
        work = Path(tmp) / "repo"
        shutil.copytree(src, work)

        listed = run_apply(work, "--list-registry").stdout
        assert "alpha\tseikika-package\tpackages/alpha/seikika-package.yaml" in listed
        assert "beta\tseikika-package\tpackages/beta/seikika-package.yaml" in listed
        assert "actions/demo\taction\tactions/demo/action.yml" in listed
        assert "decoy" not in listed
        assert "pkg-fixture" not in listed

        run_apply(work)
        data = tomllib.loads((work / "Cargo.toml").read_text(encoding="utf-8"))
        assert data["workspace"]["package"]["version"] == "0.4.0", data
        alpha = (work / "packages" / "alpha" / "seikika-package.yaml").read_text(
            encoding="utf-8"
        )
        beta = (work / "packages" / "beta" / "seikika-package.yaml").read_text(
            encoding="utf-8"
        )
        assert "version: 0.4.0" in alpha
        assert "version: 0.4.0" in beta  # lockstep even though not named in frontmatter
        decoy = (work / "pkg-fixture" / "decoy" / "seikika-package.yaml").read_text(
            encoding="utf-8"
        )
        assert "version: 9.9.9" in decoy
        cl = (work / "CHANGELOG.md").read_text(encoding="utf-8")
        assert "## 0.4.0" in cl
        assert "demo dynamic include bump" in cl


def test_new_package_discovered_without_config_edit() -> None:
    """Adding packages/<name>/seikika-package.yaml is enough; no registry edit."""
    src = FIXTURES / "dynamic-includes"
    with tempfile.TemporaryDirectory() as tmp:
        work = Path(tmp) / "repo"
        shutil.copytree(src, work)
        new_pkg = work / "packages" / "gamma" / "seikika-package.yaml"
        new_pkg.parent.mkdir(parents=True)
        new_pkg.write_text(
            "apiVersion: seikika.com/package/v0\n"
            "name: gamma\n"
            "version: 0.3.0\n"
            "description: newly added package\n"
            "fragments: []\n",
            encoding="utf-8",
        )
        # Point a changeset at the new package name
        (work / ".changeset" / "demo-minor.md").write_text(
            '---\n"seikika": patch\n"gamma": patch\n---\n\nnew package gamma\n',
            encoding="utf-8",
        )
        listed = run_apply(work, "--list-registry").stdout
        assert "gamma\tseikika-package\tpackages/gamma/seikika-package.yaml" in listed
        run_apply(work)
        gamma = new_pkg.read_text(encoding="utf-8")
        assert "version: 0.3.1" in gamma
        beta = (work / "packages" / "beta" / "seikika-package.yaml").read_text(
            encoding="utf-8"
        )
        assert "version: 0.3.1" in beta


def test_fixtures_not_under_packages_glob() -> None:
    """Regression: applicator fixtures must not use packages/**/seikika-package.yaml."""
    under_packages = sorted(FIXTURES.glob("**/packages/**/seikika-package.yaml"))
    # Only the intentional dynamic-includes live packages/ tree is allowed in fixtures;
    # decoys and demo packages for other cases use pkg-fixture/.
    for path in FIXTURES.rglob("seikika-package.yaml"):
        rel = path.relative_to(FIXTURES)
        parts = rel.parts
        if parts[0] == "dynamic-includes" and "packages" in parts:
            continue
        assert "packages" not in parts, (
            f"fixture {rel} must use pkg-fixture/ (or dynamic-includes/packages/), "
            "not a discoverable packages/** path that would confuse S4S"
        )
    # And decoys stay on pkg-fixture/
    decoy = (
        FIXTURES
        / "dynamic-includes"
        / "pkg-fixture"
        / "decoy"
        / "seikika-package.yaml"
    )
    assert decoy.is_file()
    assert under_packages  # dynamic-includes has packages/alpha, beta


def test_live_repo_includes_resolve() -> None:
    """Live .changeset/config.json includes expand real packages/actions."""
    mod = load_applicator()
    config = mod.load_config(REPO_ROOT)
    assert "includes" in config
    registry = mod.resolve_registry(REPO_ROOT, config)
    assert registry["seikika"]["kind"] == "cargo-workspace"
    assert "github-actions" in registry
    assert "file-size" in registry
    assert "semgrep" in registry
    assert "storybook" in registry
    assert "actions/report" in registry
    assert registry["actions/report"]["kind"] == "action"
    assert registry["file-size"]["path"] == "packages/file-size/seikika-package.yaml"
    # No fixture decoy leaked into live discovery
    assert "decoy" not in registry
    for name, meta in registry.items():
        path = str(meta.get("path", ""))
        assert "pkg-fixture" not in path, (name, path)
        assert "testdata" not in path, (name, path)


def test_unknown_changeset_name_errors() -> None:
    src = FIXTURES / "dynamic-includes"
    with tempfile.TemporaryDirectory() as tmp:
        work = Path(tmp) / "repo"
        shutil.copytree(src, work)
        (work / ".changeset" / "demo-minor.md").write_text(
            '---\n"seikika": patch\n"does-not-exist": patch\n---\n\nbad name\n',
            encoding="utf-8",
        )
        proc = subprocess.run(
            [sys.executable, str(SCRIPT), "--root", str(work)],
            capture_output=True,
            text=True,
        )
        assert proc.returncode != 0
        assert "unknown package" in proc.stderr.lower() or "does-not-exist" in proc.stderr


def main() -> int:
    test_cargo_changelog()
    print("ok: cargo-changelog")
    test_package_yaml()
    print("ok: package-yaml")
    test_dynamic_includes_glob_discovery_lockstep()
    print("ok: dynamic-includes")
    test_new_package_discovered_without_config_edit()
    print("ok: new-package-discovery")
    test_fixtures_not_under_packages_glob()
    print("ok: fixture-isolation")
    test_live_repo_includes_resolve()
    print("ok: live-repo-includes")
    test_unknown_changeset_name_errors()
    print("ok: unknown-name")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

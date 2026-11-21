#!/usr/bin/env python3
"""Apply Changesets-compatible files to Toolbox native manifests (Option B).

Reads pending ``.changeset/*.md``, bumps Cargo workspace version, root
CHANGELOG.md, and packages/*/toolbox-package.yaml under lockstep policy.
Does not publish; tag after apply so cargo-dist release.yml can build.

Stdlib only (tomllib + json + regex line edits).
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import sys
import tempfile
import tomllib
from datetime import date
from pathlib import Path

BUMP_RANK = {"patch": 1, "minor": 2, "major": 3}
SKIP_NAMES = {"readme.md", "config.json", "config.schema.json"}
FRONTMATTER_RE = re.compile(r"\A---\s*\n(.*?)\n---\s*\n?(.*)\Z", re.S)
VERSION_LINE_RE = re.compile(
    r'^(\s*version\s*=\s*")([^"]+)("\s*)$', re.M
)
YAML_VERSION_RE = re.compile(r"^(\s*version:\s*)(\S+)(\s*)$", re.M)
SEMVER_RE = re.compile(r"^(\d+)\.(\d+)\.(\d+)$")


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def load_config(root: Path) -> dict:
    path = root / ".changeset" / "config.json"
    return json.loads(path.read_text(encoding="utf-8"))


def list_changeset_files(changeset_dir: Path) -> list[Path]:
    if not changeset_dir.is_dir():
        return []
    files = []
    for path in sorted(changeset_dir.iterdir()):
        if not path.is_file() or path.name.startswith("."):
            continue
        if path.name.lower() in SKIP_NAMES:
            continue
        if path.suffix != ".md":
            continue
        files.append(path)
    return files


def parse_frontmatter(text: str) -> tuple[dict[str, str], str]:
    match = FRONTMATTER_RE.match(text)
    if not match:
        raise ValueError("changeset missing YAML frontmatter")
    raw, body = match.group(1), match.group(2).strip()
    bumps: dict[str, str] = {}
    for line in raw.splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        if ":" not in line:
            raise ValueError(f"bad frontmatter line: {line!r}")
        key, value = line.split(":", 1)
        name = key.strip().strip("\"'")
        bump = value.strip().strip("\"'")
        if bump not in BUMP_RANK:
            raise ValueError(f"unknown bump {bump!r} for {name}")
        bumps[name] = bump
    if not bumps:
        raise ValueError("changeset frontmatter has no package bumps")
    return bumps, body


def parse_changesets(files: list[Path]) -> list[tuple[Path, dict[str, str], str]]:
    out = []
    for path in files:
        bumps, summary = parse_frontmatter(path.read_text(encoding="utf-8"))
        out.append((path, bumps, summary))
    return out


def max_bump(bumps: list[str]) -> str | None:
    if not bumps:
        return None
    return max(bumps, key=lambda b: BUMP_RANK[b])


def bump_semver(version: str, kind: str) -> str:
    match = SEMVER_RE.match(version)
    if not match:
        raise ValueError(f"not X.Y.Z: {version!r}")
    major, minor, patch = map(int, match.groups())
    if kind == "major":
        return f"{major + 1}.0.0"
    if kind == "minor":
        return f"{major}.{minor + 1}.0"
    return f"{major}.{minor}.{patch + 1}"


def read_cargo_workspace_version(cargo_toml: Path) -> str:
    data = tomllib.loads(cargo_toml.read_text(encoding="utf-8"))
    version = data.get("workspace", {}).get("package", {}).get("version")
    if not isinstance(version, str):
        raise ValueError(f"no [workspace.package].version in {cargo_toml}")
    return version


def write_cargo_workspace_version(cargo_toml: Path, new_version: str) -> None:
    text = cargo_toml.read_text(encoding="utf-8")
    # Only replace the first version = "..." inside [workspace.package].
    parts = text.split("[workspace.package]", 1)
    if len(parts) != 2:
        raise ValueError("Cargo.toml missing [workspace.package]")
    head, rest = parts
    section, *after = rest.split("\n[", 1)
    new_section, n = VERSION_LINE_RE.subn(
        rf'\g<1>{new_version}\g<3>', section, count=1
    )
    if n != 1:
        raise ValueError("could not rewrite workspace.package version")
    rebuilt = head + "[workspace.package]" + new_section
    if after:
        rebuilt += "\n[" + after[0]
    cargo_toml.write_text(rebuilt, encoding="utf-8")


def write_yaml_version(path: Path, new_version: str) -> None:
    text = path.read_text(encoding="utf-8")
    new_text, n = YAML_VERSION_RE.subn(rf"\g<1>{new_version}\g<3>", text, count=1)
    if n != 1:
        raise ValueError(f"could not rewrite version: in {path}")
    path.write_text(new_text, encoding="utf-8")


def update_changelog(path: Path, new_version: str, bullets: list[str], today: str) -> None:
    text = path.read_text(encoding="utf-8")
    header = f"## {new_version} - {today}"
    if not bullets:
        bullets = ["(no changeset summaries)"]
    block = header + "\n\n### Added\n\n" + "\n".join(f"- {b}" for b in bullets) + "\n"
    marker = "## Unreleased"
    if marker not in text:
        raise ValueError(f"{path} missing '## Unreleased'")
    # Keep an empty Unreleased section; insert the new release after it.
    idx = text.index(marker) + len(marker)
    # Skip blank lines after Unreleased
    rest = text[idx:]
    insert_at = 0
    while insert_at < len(rest) and rest[insert_at] in "\r\n":
        insert_at += 1
    new_text = text[: idx] + "\n\n" + block + "\n" + rest[insert_at:]
    # Normalize triple blank lines
    new_text = re.sub(r"\n{3,}", "\n\n", new_text)
    path.write_text(new_text, encoding="utf-8")


def package_yaml_paths(root: Path, config: dict) -> list[Path]:
    paths = []
    for name, meta in config.get("registry", {}).items():
        if meta.get("kind") != "toolbox-package":
            continue
        paths.append(root / meta["path"])
    return paths


def compute_release(
    parsed: list[tuple[Path, dict[str, str], str]], config: dict
) -> tuple[str, list[str], set[str]]:
    """Return (bump_kind, summaries, named_packages)."""
    all_bumps: list[str] = []
    summaries: list[str] = []
    named: set[str] = set()
    registry = config.get("registry", {})
    for _path, bumps, summary in parsed:
        for name, kind in bumps.items():
            if name not in registry and name != "toolbox":
                # Allow unknown only if registry empty in fixtures
                if registry and name not in registry:
                    raise ValueError(f"unknown package in changeset: {name}")
            all_bumps.append(kind)
            named.add(name)
        if summary:
            summaries.append(summary.splitlines()[0].strip())
    kind = max_bump(all_bumps)
    if kind is None:
        raise ValueError("no bumps found")
    return kind, summaries, named


def status(root: Path) -> int:
    files = list_changeset_files(root / ".changeset")
    if not files:
        print("no pending changesets")
        return 1
    for path in files:
        print(path.relative_to(root))
    return 0


def apply(root: Path, *, dry_run: bool, today: str | None = None) -> int:
    config = load_config(root)
    files = list_changeset_files(root / ".changeset")
    if not files:
        print("nothing to apply", file=sys.stderr)
        return 1
    parsed = parse_changesets(files)
    bump_kind, summaries, _named = compute_release(parsed, config)
    cargo = root / "Cargo.toml"
    current = read_cargo_workspace_version(cargo)
    new_version = bump_semver(current, bump_kind)
    day = today or date.today().isoformat()
    changelog = root / config.get("changelog", "CHANGELOG.md")
    yaml_paths = package_yaml_paths(root, config)

    print(f"{current} --{bump_kind}--> {new_version}")
    for s in summaries:
        print(f"  - {s}")
    if dry_run:
        print("dry-run: no files written")
        return 0

    write_cargo_workspace_version(cargo, new_version)
    update_changelog(changelog, new_version, summaries, day)
    for ypath in yaml_paths:
        if ypath.is_file():
            write_yaml_version(ypath, new_version)
    for path, _, _ in parsed:
        path.unlink()
    print(f"applied {len(parsed)} changeset(s); tag v{new_version} after CI is green")
    return 0


def _write_fixture_tree(base: Path) -> None:
    (base / ".changeset").mkdir(parents=True)
    (base / "packages" / "demo").mkdir(parents=True)
    (base / "Cargo.toml").write_text(
        '[workspace]\nmembers = ["crates/*"]\n\n'
        '[workspace.package]\nversion = "0.1.1"\nedition = "2024"\n',
        encoding="utf-8",
    )
    (base / "CHANGELOG.md").write_text(
        "# Changelog\n\n## Unreleased\n\n## 0.1.1 - 2026-11-21\n\n### Added\n\n- prior\n",
        encoding="utf-8",
    )
    (base / "packages" / "demo" / "toolbox-package.yaml").write_text(
        "apiVersion: toolbox.dev/package/v0\nname: demo\nversion: 0.1.1\n",
        encoding="utf-8",
    )
    config = {
        "changelog": "CHANGELOG.md",
        "registry": {
            "toolbox": {"kind": "cargo-workspace", "path": "Cargo.toml"},
            "demo": {
                "kind": "toolbox-package",
                "path": "packages/demo/toolbox-package.yaml",
            },
            "actions/report": {"kind": "action", "lockstepWith": "toolbox"},
        },
        "policy": {"versioning": "lockstep"},
    }
    (base / ".changeset" / "config.json").write_text(
        json.dumps(config, indent=2) + "\n", encoding="utf-8"
    )


def self_test() -> None:
    # Unit: semver + frontmatter
    assert bump_semver("0.1.1", "patch") == "0.1.2"
    assert bump_semver("0.1.1", "minor") == "0.2.0"
    assert bump_semver("0.1.1", "major") == "1.0.0"
    assert max_bump(["patch", "minor"]) == "minor"
    bumps, body = parse_frontmatter(
        '---\n"toolbox": minor\n"demo": patch\n---\n\nHello consumers.\n'
    )
    assert bumps == {"toolbox": "minor", "demo": "patch"}
    assert body == "Hello consumers."

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        _write_fixture_tree(root)
        # non-JS path: Cargo + CHANGELOG
        (root / ".changeset" / "core-fix.md").write_text(
            '---\n"toolbox": minor\n---\n\nCore bump for fixture.\n',
            encoding="utf-8",
        )
        # package path: also name demo (lockstep still one version)
        (root / ".changeset" / "pkg-fix.md").write_text(
            '---\n"demo": patch\n"actions/report": patch\n---\n\n'
            "Package and action note.\n",
            encoding="utf-8",
        )
        rc = apply(root, dry_run=False, today="2026-11-21")
        assert rc == 0, rc
        assert read_cargo_workspace_version(root / "Cargo.toml") == "0.2.0"
        yaml_text = (root / "packages" / "demo" / "toolbox-package.yaml").read_text(
            encoding="utf-8"
        )
        assert "version: 0.2.0" in yaml_text
        cl = (root / "CHANGELOG.md").read_text(encoding="utf-8")
        assert "## 0.2.0 - 2026-11-21" in cl
        assert "Core bump for fixture." in cl
        assert "Package and action note." in cl
        assert list_changeset_files(root / ".changeset") == []
        # status empty
        assert status(root) == 1

    # Golden fixture trees: run apply against a copy of each
    here = Path(__file__).resolve().parent
    data = here / "testdata" / "apply-changesets"
    for name in ("cargo-changelog", "package-yaml"):
        case = data / name
        assert (case / "expected.txt").is_file(), case
        with tempfile.TemporaryDirectory() as tmp:
            dest = Path(tmp) / name
            shutil.copytree(case, dest)
            # expected.txt is not part of the repo layout under apply
            (dest / "expected.txt").unlink()
            expect = {
                line.split("=", 1)[0]: line.split("=", 1)[1]
                for line in (case / "expected.txt").read_text().splitlines()
                if line.strip() and not line.startswith("#")
            }
            rc = apply(dest, dry_run=False, today="2026-11-21")
            assert rc == 0, (name, rc)
            got = read_cargo_workspace_version(dest / "Cargo.toml")
            assert got == expect["version"], (name, got, expect)
            if "yaml_version" in expect:
                y = (dest / "packages" / "demo" / "toolbox-package.yaml").read_text()
                assert f"version: {expect['yaml_version']}" in y
            cl = (dest / "CHANGELOG.md").read_text()
            for key, val in expect.items():
                if key.startswith("changelog_contains"):
                    assert val in cl, (name, val)

    print("apply-changesets self-test ok")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=None, help="repo root")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--status", action="store_true", help="list pending; exit 1 if none")
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--date", default=None, help="YYYY-MM-DD for changelog")
    args = parser.parse_args(argv)
    if args.self_test:
        self_test()
        return 0
    root = (args.root or repo_root()).resolve()
    if args.status:
        return status(root)
    return apply(root, dry_run=args.dry_run, today=args.date)


if __name__ == "__main__":
    raise SystemExit(main())

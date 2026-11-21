#!/usr/bin/env python3
"""Apply Changesets-compatible files to Toolbox native manifests (Option B).

Reads pending ``.changeset/*.md``, bumps Cargo workspace version, root
CHANGELOG.md, and ``packages/*/toolbox-package.yaml`` under lockstep policy.
Does not publish or tag — after apply, tag ``vX.Y.Z`` so cargo-dist runs.

Stdlib only (tomllib + json + regex line edits).
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import tomllib
from datetime import date
from pathlib import Path

BUMP_RANK = {"patch": 1, "minor": 2, "major": 3}
SKIP_NAMES = {"readme.md", "config.json", "config.schema.json"}
FRONTMATTER_RE = re.compile(r"\A---\s*\n(.*?)\n---\s*\n?(.*)\Z", re.S)
VERSION_LINE_RE = re.compile(r'^(\s*version\s*=\s*")([^"]+)("\s*)$', re.M)
YAML_VERSION_RE = re.compile(r"^(\s*version:\s*)(\S+)(\s*)$", re.M)
SEMVER_RE = re.compile(r"^(\d+)\.(\d+)\.(\d+)$")


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def load_config(root: Path) -> dict:
    return json.loads((root / ".changeset" / "config.json").read_text(encoding="utf-8"))


def list_changeset_files(changeset_dir: Path) -> list[Path]:
    if not changeset_dir.is_dir():
        return []
    out: list[Path] = []
    for path in sorted(changeset_dir.iterdir()):
        if not path.is_file() or path.name.startswith("."):
            continue
        if path.name.lower() in SKIP_NAMES or path.suffix != ".md":
            continue
        out.append(path)
    return out


def parse_frontmatter(text: str) -> tuple[dict[str, str], str]:
    m = FRONTMATTER_RE.match(text)
    if not m:
        raise ValueError("changeset missing YAML frontmatter")
    bumps: dict[str, str] = {}
    for line in m.group(1).splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        if ":" not in line:
            continue
        key, val = line.split(":", 1)
        key = key.strip().strip("\"'")
        val = val.strip().strip("\"'")
        if val not in BUMP_RANK:
            raise ValueError(f"unknown bump {val!r} for {key!r}")
        bumps[key] = val
    return bumps, m.group(2).strip()


def max_bump(bumps: list[str]) -> str:
    return max(bumps, key=lambda b: BUMP_RANK[b])


def bump_semver(version: str, kind: str) -> str:
    m = SEMVER_RE.match(version)
    if not m:
        raise ValueError(f"not semver: {version!r}")
    major, minor, patch = (int(x) for x in m.groups())
    if kind == "major":
        return f"{major + 1}.0.0"
    if kind == "minor":
        return f"{major}.{minor + 1}.0"
    return f"{major}.{minor}.{patch + 1}"


def read_cargo_workspace_version(cargo_toml: Path) -> str:
    data = tomllib.loads(cargo_toml.read_text(encoding="utf-8"))
    ver = data.get("workspace", {}).get("package", {}).get("version")
    if not ver:
        raise ValueError(f"no [workspace.package].version in {cargo_toml}")
    return ver


def write_cargo_workspace_version(cargo_toml: Path, new_version: str) -> None:
    text = cargo_toml.read_text(encoding="utf-8")
    # Only rewrite the first version = "..." under [workspace.package]
    parts = text.split("[workspace.package]", 1)
    if len(parts) != 2:
        raise ValueError("missing [workspace.package]")
    head, rest = parts
    m = VERSION_LINE_RE.search(rest)
    if not m:
        raise ValueError("no version line in [workspace.package]")
    rest2 = VERSION_LINE_RE.sub(rf"\g<1>{new_version}\g<3>", rest, count=1)
    cargo_toml.write_text(head + "[workspace.package]" + rest2, encoding="utf-8")


def write_yaml_version(path: Path, new_version: str) -> None:
    text = path.read_text(encoding="utf-8")
    if not YAML_VERSION_RE.search(text):
        raise ValueError(f"no version: in {path}")
    path.write_text(
        YAML_VERSION_RE.sub(rf"\g<1>{new_version}\g<3>", text, count=1),
        encoding="utf-8",
    )


def update_changelog(path: Path, new_version: str, entries: list[str]) -> None:
    today = date.today().isoformat()
    text = path.read_text(encoding="utf-8") if path.exists() else "# Changelog\n\n"
    bullets = "\n".join(f"- {e}" for e in entries) if entries else "- Release."
    section = f"## {new_version} — {today}\n\n{bullets}\n\n"
    marker = "## Unreleased"
    if marker in text:
        idx = text.index(marker)
        # Find end of Unreleased heading line
        nl = text.find("\n", idx)
        after = text[nl + 1 :] if nl != -1 else ""
        # Skip blank lines and keep existing Unreleased body out (consumed)
        # Insert new section after Unreleased header, clearing prior Unreleased body
        # until next ## heading
        rest = after
        next_h = re.search(r"^## ", rest, re.M)
        if next_h:
            rest = rest[next_h.start() :]
        else:
            rest = ""
        text = text[: nl + 1] + "\n" + section + rest
    else:
        # Insert after title
        if text.startswith("#"):
            nl = text.find("\n")
            text = text[: nl + 1] + "\n## Unreleased\n\n" + section + text[nl + 1 :]
        else:
            text = "# Changelog\n\n## Unreleased\n\n" + section + text
    path.write_text(text, encoding="utf-8")


def collect_pending(root: Path) -> tuple[str, list[str], list[Path]]:
    """Return (bump_kind, summaries, files) aggregating all pending changesets."""
    files = list_changeset_files(root / ".changeset")
    if not files:
        raise SystemExit("no pending changesets")
    kinds: list[str] = []
    summaries: list[str] = []
    for path in files:
        bumps, body = parse_frontmatter(path.read_text(encoding="utf-8"))
        if not bumps:
            raise ValueError(f"{path.name}: empty frontmatter")
        kinds.extend(bumps.values())
        if body:
            # one summary line per changeset file
            first = body.strip().splitlines()[0].strip()
            summaries.append(first)
    return max_bump(kinds), summaries, files


def apply(root: Path, *, dry_run: bool = False) -> str:
    config = load_config(root)
    bump_kind, summaries, files = collect_pending(root)
    cargo_path = root / config["registry"]["toolbox"]["path"]
    old = read_cargo_workspace_version(cargo_path)
    new = bump_semver(old, bump_kind)
    changelog = root / config.get("changelog", "CHANGELOG.md")

    package_paths: list[Path] = []
    for name, meta in config.get("registry", {}).items():
        if meta.get("kind") == "toolbox-package":
            package_paths.append(root / meta["path"])

    if dry_run:
        print(f"would bump {old} -> {new} ({bump_kind})")
        print(f"would update {changelog}")
        for p in package_paths:
            print(f"would update {p}")
        print(f"would remove {len(files)} changeset(s)")
        return new

    write_cargo_workspace_version(cargo_path, new)
    update_changelog(changelog, new, summaries)
    for p in package_paths:
        if p.exists():
            write_yaml_version(p, new)
    for path in files:
        path.unlink()
    print(f"applied {old} -> {new}; removed {len(files)} changeset(s)")
    print(f"next: git commit, merge Version PR if any, then: git tag v{new} && git push origin v{new}")
    return new


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=None, help="repo root (default: parent of scripts/)")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args(argv)
    root = args.root.resolve() if args.root else repo_root()
    apply(root, dry_run=args.dry_run)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:  # noqa: BLE001 — CLI surface
        print(f"error: {exc}", file=sys.stderr)
        raise SystemExit(1) from exc

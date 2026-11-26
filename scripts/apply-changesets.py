#!/usr/bin/env python3
"""Apply Changesets-compatible files to Seikika native manifests (Option B).

Reads pending ``.changeset/*.md``, bumps Cargo workspace version, root
CHANGELOG.md, and discovered ``packages/*/seikika-package.yaml`` under
lockstep policy. Dynamic ``includes`` globs in ``.changeset/config.json``
expand the registry so new packages/actions are picked up without listing
each name. Does not publish or tag — after apply, tag ``vX.Y.Z`` so
cargo-dist runs.

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
from typing import Any

BUMP_RANK = {"patch": 1, "minor": 2, "major": 3}
SKIP_NAMES = {"readme.md", "config.json", "config.schema.json"}
FRONTMATTER_RE = re.compile(r"\A---\s*\n(.*?)\n---\s*\n?(.*)\Z", re.S)
VERSION_LINE_RE = re.compile(r'^(\s*version\s*=\s*")([^"]+)("\s*)$', re.M)
YAML_VERSION_RE = re.compile(r"^(\s*version:\s*)(\S+)(\s*)$", re.M)
YAML_NAME_RE = re.compile(r"^name:\s*[\"']?([^\"'#\s]+)[\"']?\s*(?:#.*)?$")
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


def read_yaml_name(path: Path) -> str:
    """Return the top-level ``name:`` from a seikika-package.yaml (line scan)."""
    for line in path.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        m = YAML_NAME_RE.match(stripped)
        if m:
            return m.group(1)
    raise ValueError(f"no name: in {path}")


def expand_globs(root: Path, patterns: list[str]) -> list[Path]:
    """Expand include globs relative to *root*; return sorted unique files."""
    found: set[Path] = set()
    for pattern in patterns:
        for match in root.glob(pattern):
            if match.is_file():
                found.add(match.resolve())
    return sorted(found)


def logical_name_for_include(root: Path, kind: str, path: Path) -> str:
    """Map a discovered manifest path to the Changeset frontmatter name."""
    if kind == "seikika-package":
        name = read_yaml_name(path)
        # Prefer YAML name; dirname should match for live packages.
        return name
    if kind == "action":
        rel = path.parent.resolve().relative_to(root.resolve())
        return str(rel).replace("\\", "/")
    raise ValueError(f"includes cannot discover kind {kind!r}")


def resolve_registry(root: Path, config: dict[str, Any]) -> dict[str, dict[str, Any]]:
    """Merge explicit ``registry`` entries with entries from ``includes`` globs.

    Explicit names win when both an include and ``registry`` define the same
    logical name (path must agree or we error). Discovered package names come
    from YAML ``name:``; actions are ``actions/<dirname>``.
    """
    registry: dict[str, dict[str, Any]] = {}
    for name, meta in (config.get("registry") or {}).items():
        if not isinstance(meta, dict):
            raise ValueError(f"registry[{name!r}] must be an object")
        entry = dict(meta)
        entry.setdefault("source", "explicit")
        registry[name] = entry

    for idx, spec in enumerate(config.get("includes") or []):
        if not isinstance(spec, dict):
            raise ValueError(f"includes[{idx}] must be an object")
        kind = spec.get("kind")
        globs = spec.get("globs")
        if not kind or not isinstance(globs, list) or not globs:
            raise ValueError(
                f"includes[{idx}] requires kind and non-empty globs list"
            )
        if kind not in {"seikika-package", "action"}:
            raise ValueError(
                f"includes[{idx}]: kind must be seikika-package or action, got {kind!r}"
            )
        for path in expand_globs(root, [str(g) for g in globs]):
            try:
                rel = str(path.resolve().relative_to(root.resolve())).replace("\\", "/")
            except ValueError as exc:
                raise ValueError(
                    f"include match {path} is outside repo root {root}"
                ) from exc
            name = logical_name_for_include(root, kind, path)
            entry: dict[str, Any] = {
                "kind": kind,
                "path": rel,
                "source": "include",
            }
            if "lockstepWith" in spec:
                entry["lockstepWith"] = spec["lockstepWith"]
            if name in registry:
                existing = registry[name]
                existing_path = str(existing.get("path", "")).replace("\\", "/")
                if existing_path and existing_path != rel:
                    raise ValueError(
                        f"registry name {name!r} conflict: explicit {existing_path!r} "
                        f"vs include {rel!r}"
                    )
                # Explicit wins; keep include path fill-in only if missing.
                if not existing_path:
                    existing["path"] = rel
                if "kind" not in existing:
                    existing["kind"] = kind
                continue
            registry[name] = entry
    return registry


def package_paths_to_version(
    root: Path, registry: dict[str, dict[str, Any]]
) -> list[Path]:
    """Paths of seikika-package manifests to bump under lockstep."""
    paths: list[Path] = []
    for meta in registry.values():
        if meta.get("kind") != "seikika-package":
            continue
        rel = meta.get("path")
        if not rel:
            continue
        paths.append(root / rel)
    return paths


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


def collect_pending(
    root: Path, registry: dict[str, dict[str, Any]]
) -> tuple[str, list[str], list[Path]]:
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
        unknown = sorted(set(bumps) - set(registry))
        if unknown:
            raise ValueError(
                f"{path.name}: unknown package(s) {unknown}; "
                f"known: {sorted(registry)}"
            )
        kinds.extend(bumps.values())
        if body:
            # one summary line per changeset file
            first = body.strip().splitlines()[0].strip()
            summaries.append(first)
    return max_bump(kinds), summaries, files


def apply(root: Path, *, dry_run: bool = False) -> str:
    config = load_config(root)
    registry = resolve_registry(root, config)
    if "seikika" not in registry:
        raise ValueError("registry must define seikika (cargo-workspace)")
    bump_kind, summaries, files = collect_pending(root, registry)
    cargo_meta = registry["seikika"]
    cargo_path = root / cargo_meta["path"]
    old = read_cargo_workspace_version(cargo_path)
    new = bump_semver(old, bump_kind)
    changelog = root / config.get("changelog", "CHANGELOG.md")

    package_paths = package_paths_to_version(root, registry)

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
    print(
        f"next: git commit, merge Version PR if any, then: "
        f"git tag v{new} && git push origin v{new}"
    )
    return new


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root", type=Path, default=None, help="repo root (default: parent of scripts/)"
    )
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument(
        "--list-registry",
        action="store_true",
        help="print resolved registry (name -> kind path) and exit",
    )
    args = parser.parse_args(argv)
    root = args.root.resolve() if args.root else repo_root()
    if args.list_registry:
        config = load_config(root)
        registry = resolve_registry(root, config)
        for name in sorted(registry):
            meta = registry[name]
            print(f"{name}\t{meta.get('kind')}\t{meta.get('path')}\t{meta.get('source')}")
        return 0
    apply(root, dry_run=args.dry_run)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:
        print(f"error: {exc}", file=sys.stderr)
        raise SystemExit(1) from exc

#!/usr/bin/env python3
"""OSS monorepo federation applicator.

Resolves package includes across the babel / jest / pnpm fixture fleet,
rejects parameter weakenings, merges multi-entity catalogs, and emits a
parent adoption report. Pattern follows experiments/federation-prototype;
fixtures stay under experiments/ so T4T package counts stay honest.
"""

from __future__ import annotations

import argparse
import json
import sys
import time
from collections import defaultdict
from pathlib import Path
from typing import Any

try:
    import yaml
except ImportError as e:  # pragma: no cover
    raise SystemExit("PyYAML is required: pip install pyyaml") from e

ROOT = Path(__file__).resolve().parent

SHARED_SCORECARDS = (
    "package-manager-pin",
    "js-workspace",
    "eslint-flat",
    "typescript-present",
    "agents-instruction",
)


def load_yaml(path: Path) -> Any:
    with path.open(encoding="utf-8") as f:
        return yaml.safe_load(f)


def load_json(path: Path) -> Any:
    with path.open(encoding="utf-8") as f:
        return json.load(f)


def package_dirs(root: Path) -> dict[str, Path]:
    found: dict[str, Path] = {}
    for meta in root.rglob("toolbox-package.yaml"):
        if "testdata" in meta.parts:
            continue
        doc = load_yaml(meta)
        name = doc.get("name")
        if not name:
            raise SystemExit(f"{meta}: missing name")
        if name in found:
            raise SystemExit(f"duplicate package name {name}: {found[name]} and {meta}")
        found[name] = meta.parent
    return found


def package_meta(pkg_dir: Path) -> dict[str, Any]:
    return load_yaml(pkg_dir / "toolbox-package.yaml")


def load_fragments(pkg_dir: Path) -> dict[str, Any]:
    frag_dir = pkg_dir / "fragments"
    if not frag_dir.is_dir():
        return {}
    merged: dict[str, Any] = {}
    for path in sorted(frag_dir.glob("*.yaml")):
        doc = load_yaml(path) or {}
        for key, value in doc.items():
            if key not in merged:
                merged[key] = value
            elif isinstance(merged[key], list) and isinstance(value, list):
                merged[key] = merged[key] + value
            elif isinstance(merged[key], dict) and isinstance(value, dict):
                merged[key] = {**merged[key], **value}
            else:
                raise SystemExit(f"{path}: cannot merge key {key}")
    return merged


def check_tighten(
    pkg_name: str, meta: dict[str, Any], with_params: dict[str, Any]
) -> list[str]:
    errors: list[str] = []
    params = meta.get("parameters") or {}
    for name, value in with_params.items():
        if name not in params:
            errors.append(f"package {pkg_name}: unknown parameter `{name}`")
            continue
        p = params[name]
        direction = p.get("tighten")
        default = p.get("default")
        if not direction or default is None:
            continue
        if not isinstance(value, (int, float)) or not isinstance(default, (int, float)):
            continue
        if direction == "lower" and value > default:
            errors.append(
                f"package {pkg_name}: parameter `{name}` = {value} weakens "
                f"default {default} (may only go lower)"
            )
        if direction == "higher" and value < default:
            errors.append(
                f"package {pkg_name}: parameter `{name}` = {value} weakens "
                f"default {default} (may only go higher)"
            )
    return errors


def substitute_params(obj: Any, values: dict[str, Any]) -> Any:
    if isinstance(obj, str):
        out = obj
        for name, val in values.items():
            token = f"${{param:{name}}}"
            if out == token:
                return val
            if token in out:
                out = out.replace(
                    token, json.dumps(val) if not isinstance(val, str) else val
                )
        return out
    if isinstance(obj, list):
        return [substitute_params(x, values) for x in obj]
    if isinstance(obj, dict):
        return {k: substitute_params(v, values) for k, v in obj.items()}
    return obj


def resolve_params(meta: dict[str, Any], with_params: dict[str, Any]) -> dict[str, Any]:
    params = meta.get("parameters") or {}
    out: dict[str, Any] = {}
    for name, p in params.items():
        if name in with_params:
            out[name] = with_params[name]
        elif "default" in p:
            out[name] = p["default"]
    return out


def compose_includes(
    root: Path,
    includes: list[dict[str, Any]],
    packages: dict[str, Path],
    *,
    seen: set[str] | None = None,
) -> tuple[dict[str, Any], list[str], list[dict[str, Any]]]:
    seen = seen or set()
    merged: dict[str, Any] = {"kinds": [], "scorecards": [], "campaigns": []}
    errors: list[str] = []
    trail: list[dict[str, Any]] = []

    for entry in includes:
        name = entry["package"]
        with_params = dict(entry.get("with") or {})
        if name not in packages:
            errors.append(f"unknown package `{name}`")
            continue
        if name in seen:
            continue
        seen.add(name)
        meta = package_meta(packages[name])
        errors.extend(check_tighten(name, meta, with_params))
        values = resolve_params(meta, with_params)
        trail.append(
            {
                "name": name,
                "version": meta.get("version"),
                "path": str(packages[name].relative_to(root)),
                "params": values,
            }
        )
        frag = substitute_params(load_fragments(packages[name]), values)
        for key in ("kinds", "scorecards", "campaigns"):
            if key in frag and isinstance(frag[key], list):
                merged[key].extend(frag[key])

    return merged, errors, trail


def load_tree(root: Path) -> dict[str, Any]:
    return load_yaml(root / "federation.yaml")


def scorecard_names(merged: dict[str, Any]) -> set[str]:
    return {s["name"] for s in merged.get("scorecards") or [] if "name" in s}


def kind_names(merged: dict[str, Any]) -> set[str]:
    return {k["name"] for k in merged.get("kinds") or [] if "name" in k}


def load_catalog(path: Path) -> list[dict[str, Any]]:
    if not path.is_file():
        return []
    doc = load_yaml(path) or {}
    return list(doc.get("entities") or [])


def merge_catalogs(entities: list[dict[str, Any]]) -> list[dict[str, Any]]:
    by_id: dict[str, dict[str, Any]] = {}
    for ent in entities:
        eid = ent.get("id")
        if not eid:
            raise SystemExit(f"catalog entity missing id: {ent}")
        if eid in by_id:
            by_id[eid] = {**ent, **by_id[eid]}
        else:
            by_id[eid] = dict(ent)
    return sorted(by_id.values(), key=lambda e: e["id"])


def node_includes(root: Path, node: dict[str, Any]) -> list[dict[str, Any]]:
    includes = list(node.get("includes") or [])
    if includes:
        return includes
    inc_file = root / node["path"] / "includes.yaml"
    if inc_file.is_file():
        return list((load_yaml(inc_file) or {}).get("includes") or [])
    return []


def verify_fixture_evidence(root: Path) -> list[str]:
    """Sanity-check that leaf results point at existing fixture evidence paths."""
    errors: list[str] = []
    for leaf in ("babel", "jest", "pnpm"):
        report = load_json(root / "leaves" / leaf / "results.json")
        for key, rel in (report.get("evidence") or {}).items():
            path_part = rel.split("#", 1)[0]
            full = root / path_part
            if not full.is_file():
                errors.append(f"{leaf} evidence `{key}` missing file {path_part}")
    # Babel prettier width must match fixture.
    prettier = load_json(root / "leaves/babel/fixtures/.prettierrc")
    if prettier.get("printWidth") != 80:
        errors.append("babel fixture .prettierrc printWidth must be 80 (upstream)")
    babel_pkg = load_json(root / "leaves/babel/fixtures/package.json")
    if not str(babel_pkg.get("packageManager", "")).startswith("yarn@"):
        errors.append("babel packageManager must be yarn@…")
    jest_pkg = load_json(root / "leaves/jest/fixtures/package.json")
    if not str(jest_pkg.get("packageManager", "")).startswith("yarn@"):
        errors.append("jest packageManager must be yarn@…")
    pnpm_pkg = load_json(root / "leaves/pnpm/fixtures/package.json")
    if not str(pnpm_pkg.get("packageManager", "")).startswith("pnpm@"):
        errors.append("pnpm packageManager must be pnpm@…")
    return errors


def cmd_check(root: Path) -> int:
    tree = load_tree(root)
    errors: list[str] = []
    if tree.get("apiVersion") != "toolbox.dev/federation-prototype/v0":
        errors.append("apiVersion must be toolbox.dev/federation-prototype/v0")
    if tree.get("root") != "oss-fleet":
        errors.append("root must be oss-fleet")
    packages = package_dirs(root)
    nodes = {n["id"]: n for n in tree["nodes"]}

    for required_pkg in (
        "oss-fleet",
        "babel-standards",
        "jest-standards",
        "pnpm-standards",
    ):
        if required_pkg not in packages:
            errors.append(f"missing package {required_pkg}")

    for node in tree["nodes"]:
        includes = node_includes(root, node)
        if not includes:
            continue
        merged, errs, _trail = compose_includes(root, includes, packages)
        errors.extend(errs)
        if node["id"] in ("babel", "jest", "pnpm"):
            names = scorecard_names(merged)
            for required in SHARED_SCORECARDS:
                if required not in names:
                    errors.append(
                        f"push failed at {node['id']}: missing scorecard {required}"
                    )
            # Leaf extensions
            if node["id"] == "babel" and "husky-lint-staged" not in names:
                errors.append("extend failed: babel missing husky-lint-staged")
            if node["id"] == "jest" and "lerna-lite" not in names:
                errors.append("extend failed: jest missing lerna-lite")
            if node["id"] == "pnpm" and "changesets" not in names:
                errors.append("extend failed: pnpm missing changesets")

    if nodes.get("oss-fleet", {}).get("operator") != "governance":
        errors.append("oss-fleet must be governance operator")
    for leaf in ("babel", "jest", "pnpm"):
        if nodes.get(leaf, {}).get("operator") != "leaf":
            errors.append(f"{leaf} must be leaf operator")

    errors.extend(verify_fixture_evidence(root))

    if errors:
        print("check FAILED:", file=sys.stderr)
        for e in errors:
            print(f"  - {e}", file=sys.stderr)
        return 1
    print("check OK: push, extend, operators, fixture evidence")
    return 0


def cmd_weaken(root: Path) -> int:
    packages = package_dirs(root)
    includes = load_yaml(root / "testdata/weaken/includes.yaml")["includes"]
    _merged, errors, _trail = compose_includes(root, includes, packages)
    if not errors:
        print("weaken check FAILED: expected a tightening error", file=sys.stderr)
        return 1
    if not any("weakens" in e for e in errors):
        print("weaken check FAILED: errors lacked 'weakens':", errors, file=sys.stderr)
        return 1
    print("weaken rejected OK:")
    for e in errors:
        print(f"  - {e}")
    return 0


def build_report(root: Path) -> dict[str, Any]:
    tree = load_tree(root)
    packages = package_dirs(root)
    catalogs: list[dict[str, Any]] = []
    child_reports: list[dict[str, Any]] = []
    grass: list[dict[str, Any]] = []
    adoption: list[dict[str, Any]] = []
    leaf_push: dict[str, Any] = {}

    for node in tree["nodes"]:
        npath = root / node["path"]
        if node.get("catalog"):
            catalogs.extend(load_catalog(npath / node["catalog"]))
        if node.get("report"):
            report_path = npath / node["report"]
            if report_path.is_file():
                rep = load_json(report_path)
                child_reports.append(
                    {
                        "node": node["id"],
                        "kind": node["kind"],
                        "report": rep,
                    }
                )
                for pkg in rep.get("packages") or []:
                    adoption.append(
                        {
                            "node": node["id"],
                            "package": pkg.get("name"),
                            "version": pkg.get("version"),
                            "digest": pkg.get("digest"),
                            "stage": pkg.get("stage"),
                        }
                    )
                for g in rep.get("grassRootsPackages") or []:
                    grass.append({**g, "reportedBy": node["id"]})

        includes = node_includes(root, node)
        if includes and node["id"] in ("babel", "jest", "pnpm"):
            merged, errs, trail = compose_includes(root, includes, packages)
            leaf_push[node["id"]] = {
                "packages": trail,
                "scorecards": sorted(scorecard_names(merged)),
                "kinds": sorted(kind_names(merged)),
                "errors": errs,
            }

    merged_catalog = merge_catalogs(catalogs)
    kinds_present = sorted({e["kind"] for e in merged_catalog})

    campaign_roll: dict[str, dict[str, Any]] = defaultdict(
        lambda: {"inScope": 0, "meeting": 0, "measure": 0.0, "nodes": []}
    )
    for cr in child_reports:
        for name, prog in (cr["report"].get("campaigns") or {}).items():
            bucket = campaign_roll[name]
            bucket["inScope"] += int(prog.get("inScope") or 0)
            bucket["meeting"] += int(prog.get("meeting") or 0)
            bucket["measure"] += float(prog.get("measure") or 0)
            bucket["nodes"].append(cr["node"])

    push_errs = [
        e for leaf in leaf_push.values() for e in (leaf.get("errors") or [])
    ]

    return {
        "apiVersion": "toolbox.dev/federation-report/v0",
        "root": tree["root"],
        "fleet": ["babel", "jest", "pnpm"],
        "operators": {n["id"]: n["operator"] for n in tree["nodes"]},
        "push": {
            "sharedScorecards": list(SHARED_SCORECARDS),
            "leaves": leaf_push,
            "errors": push_errs,
        },
        "extend": {
            "grassRoots": grass,
            "leafScorecards": {
                leaf: sorted(
                    scorecard_names(load_fragments(packages[f"{leaf}-standards"]))
                )
                for leaf in ("babel", "jest", "pnpm")
            },
        },
        "adoption": adoption,
        "campaigns": {k: dict(v) for k, v in sorted(campaign_roll.items())},
        "catalog": {
            "kinds": kinds_present,
            "entities": merged_catalog,
            "counts": {
                k: sum(1 for e in merged_catalog if e["kind"] == k)
                for k in kinds_present
            },
        },
    }


def cmd_report(root: Path, out: Path | None) -> int:
    report = build_report(root)
    text = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if out:
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(text, encoding="utf-8")
        print(f"wrote {out}")
    else:
        sys.stdout.write(text)
    return 1 if report["push"]["errors"] else 0


def cmd_bench(root: Path, rounds: int) -> int:
    """Measure check / report / weaken wall time over the fixture fleet."""
    results: dict[str, list[float]] = {"check": [], "report": [], "weaken": []}
    for _ in range(rounds):
        t0 = time.perf_counter()
        if cmd_check(root) != 0:
            return 1
        results["check"].append(time.perf_counter() - t0)

        t0 = time.perf_counter()
        if cmd_report(root, None) != 0:
            return 1
        results["report"].append(time.perf_counter() - t0)

        t0 = time.perf_counter()
        if cmd_weaken(root) != 0:
            return 1
        results["weaken"].append(time.perf_counter() - t0)

    summary = {
        "rounds": rounds,
        "fleet": ["babel", "jest", "pnpm"],
        "seconds": {
            name: {
                "min": min(vals),
                "max": max(vals),
                "mean": sum(vals) / len(vals),
            }
            for name, vals in results.items()
        },
    }
    print(json.dumps(summary, indent=2, sort_keys=True))
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    sub = parser.add_subparsers(dest="cmd", required=True)
    sub.add_parser("check", help="Validate push/extend/operators/evidence")
    sub.add_parser("weaken", help="Assert the weaken fixture is rejected")
    p_report = sub.add_parser("report", help="Emit parent adoption + catalog report")
    p_report.add_argument("-o", "--out", type=Path)
    p_bench = sub.add_parser("bench", help="Measure check/report/weaken timings")
    p_bench.add_argument("--rounds", type=int, default=20)
    args = parser.parse_args(argv)
    root = args.root.resolve()
    if args.cmd == "check":
        return cmd_check(root)
    if args.cmd == "weaken":
        return cmd_weaken(root)
    if args.cmd == "report":
        return cmd_report(root, args.out)
    if args.cmd == "bench":
        return cmd_bench(root, args.rounds)
    raise SystemExit(f"unknown command {args.cmd}")


if __name__ == "__main__":
    raise SystemExit(main())

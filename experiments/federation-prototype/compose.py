#!/usr/bin/env python3
"""Federation prototype applicator (design §8).

Resolves package includes across the fixture tree, rejects parameter weakenings,
merges multi-entity catalogs, and emits a parent adoption report.

This is a vertical-slice tool that runs without the Toolbox binary. It is not a
production federation control plane.
"""

from __future__ import annotations

import argparse
import json
import sys
from collections import defaultdict
from pathlib import Path
from typing import Any

try:
    import yaml
except ImportError as e:  # pragma: no cover
    raise SystemExit("PyYAML is required: pip install pyyaml") from e

ROOT = Path(__file__).resolve().parent


def load_yaml(path: Path) -> Any:
    with path.open(encoding="utf-8") as f:
        return yaml.safe_load(f)


def load_json(path: Path) -> Any:
    with path.open(encoding="utf-8") as f:
        return json.load(f)


def package_dirs(root: Path) -> dict[str, Path]:
    """Map package name → directory that holds toolbox-package.yaml."""
    found: dict[str, Path] = {}
    for meta in root.rglob("toolbox-package.yaml"):
        # Skip testdata weaken fixtures if any appear later.
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
    """Merge fragment YAML objects (arrays concatenate, objects deep-merge)."""
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
    """Return error strings when `with` weakens a tighten parameter."""
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
                out = out.replace(token, json.dumps(val) if not isinstance(val, str) else val)
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
    """Resolve includes: return (merged fragments, errors, package trail)."""
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
            # Diamond includes are allowed; skip re-merge of the same package.
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

        # Parent packages named in federation.yaml includes on org nodes.
        # Org package itself does not re-include enterprise in its YAML; the
        # tree edges do. Leaf includes both org and enterprise explicitly.

        frag = substitute_params(load_fragments(packages[name]), values)
        for key in ("kinds", "scorecards", "campaigns"):
            if key in frag and isinstance(frag[key], list):
                merged[key].extend(frag[key])

    return merged, errors, trail


def load_tree(root: Path) -> dict[str, Any]:
    return load_yaml(root / "federation.yaml")


def validate_tree_schema(root: Path, tree: dict[str, Any]) -> list[str]:
    """Optional jsonschema validation when the package is installed."""
    schema_path = root / "federation.schema.json"
    try:
        import jsonschema
    except ImportError:
        # Structural checks only.
        errors = []
        if tree.get("apiVersion") != "toolbox.dev/federation-prototype/v0":
            errors.append("apiVersion must be toolbox.dev/federation-prototype/v0")
        if "root" not in tree or "nodes" not in tree:
            errors.append("federation.yaml needs root and nodes")
        return errors
    schema = load_json(schema_path)
    validator = jsonschema.Draft202012Validator(schema)
    return [
        f"{'/'.join(str(p) for p in e.path) or '<root>'}: {e.message}"
        for e in sorted(validator.iter_errors(tree), key=lambda e: list(e.path))
    ]


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
            # Parent-owned fields would win in production; prototype last-write with
            # earlier (parent-first) callers should pass parents first.
            merged = {**ent, **by_id[eid]}
            by_id[eid] = merged
        else:
            by_id[eid] = dict(ent)
    return sorted(by_id.values(), key=lambda e: e["id"])


def cmd_check(root: Path) -> int:
    tree = load_tree(root)
    errors = validate_tree_schema(root, tree)
    packages = package_dirs(root)
    nodes = {n["id"]: n for n in tree["nodes"]}

    # Resolve every non-observe leaf include list.
    for node in tree["nodes"]:
        includes = node.get("includes") or []
        if not includes:
            inc_file = root / node["path"] / "includes.yaml"
            if inc_file.is_file():
                includes = (load_yaml(inc_file) or {}).get("includes") or []
        if not includes:
            continue
        merged, errs, _trail = compose_includes(root, includes, packages)
        errors.extend(errs)
        if node["id"] == "repo-payments-api":
            names = scorecard_names(merged)
            for required in (
                "codeql",
                "prettier",
                "payments-idempotency",
            ):
                if required not in names and not errs:
                    # payments-idempotency comes from org-payments only.
                    if required == "payments-idempotency":
                        org_merged, org_errs, _ = compose_includes(
                            root, [{"package": "org-payments"}], packages
                        )
                        errors.extend(org_errs)
                        if required not in scorecard_names(org_merged):
                            errors.append(f"push failed: missing scorecard {required}")
                    elif required not in names:
                        errors.append(f"push failed: missing scorecard {required}")

        # Subteam extend: checkout grass-roots package must add cart-latency.
        if node["id"] == "checkout":
            grass = load_fragments(packages["checkout-grassroots"])
            if "cart-latency" not in scorecard_names(grass):
                errors.append("extend failed: checkout-grassroots missing cart-latency")

    # Governance vs leaf markers.
    if nodes["enterprise"]["operator"] != "governance":
        errors.append("enterprise must be governance operator")
    if nodes["repo-payments-api"]["operator"] != "leaf":
        errors.append("repo-payments-api must be leaf operator")
    if not nodes["mobile-app"].get("observeOnly"):
        errors.append("mobile-app must be observeOnly")

    if errors:
        print("check FAILED:", file=sys.stderr)
        for e in errors:
            print(f"  - {e}", file=sys.stderr)
        return 1
    print("check OK: tree schema, push includes, operators, observe-only")
    return 0


def cmd_weaken(root: Path) -> int:
    """Exit 0 only when the weaken fixture is rejected."""
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


def cmd_report(root: Path, out: Path | None) -> int:
    tree = load_tree(root)
    packages = package_dirs(root)
    catalogs: list[dict[str, Any]] = []
    child_reports: list[dict[str, Any]] = []
    grass: list[dict[str, Any]] = []
    adoption: list[dict[str, Any]] = []
    observed_mobile: list[dict[str, Any]] = []

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
                        "observeOnly": bool(node.get("observeOnly")),
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
                if node.get("observeOnly") and rep.get("observed"):
                    observed_mobile.append(
                        {"node": node["id"], **rep["observed"]}
                    )

    # Parent-first merge: enterprise catalog empty; leaves contribute.
    merged_catalog = merge_catalogs(catalogs)
    kinds_present = sorted({e["kind"] for e in merged_catalog})

    # Campaign rollup from child reports (claim aggregation).
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

    # Push proof: leaf compose contains enterprise scorecards.
    leaf_includes = load_yaml(root / "repo-payments-api/includes.yaml")["includes"]
    pushed, push_errs, trail = compose_includes(root, leaf_includes, packages)
    org_only, org_errs, _ = compose_includes(
        root, [{"package": "org-payments"}], packages
    )
    push_errs.extend(org_errs)

    report = {
        "apiVersion": "toolbox.dev/federation-report/v0",
        "root": tree["root"],
        "operators": {
            n["id"]: n["operator"] for n in tree["nodes"]
        },
        "push": {
            "leafPackages": trail,
            "scorecardsAtLeaf": sorted(scorecard_names(pushed)),
            "orgExtensionScorecards": sorted(scorecard_names(org_only)),
            "kindsAtLeaf": sorted(kind_names(pushed)),
            "errors": push_errs,
        },
        "extend": {
            "grassRoots": grass,
            "checkoutScorecards": sorted(
                scorecard_names(load_fragments(packages["checkout-grassroots"]))
            ),
        },
        "adoption": adoption,
        "campaigns": {k: dict(v) for k, v in sorted(campaign_roll.items())},
        "catalog": {
            "kinds": kinds_present,
            "entities": merged_catalog,
            "counts": {
                k: sum(1 for e in merged_catalog if e["kind"] == k) for k in kinds_present
            },
        },
        "observeOnly": observed_mobile,
    }

    text = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if out:
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(text, encoding="utf-8")
        print(f"wrote {out}")
    else:
        sys.stdout.write(text)
    return 1 if push_errs else 0


def cmd_catalog(root: Path) -> int:
    code = cmd_report(root, None)
    return code


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root",
        type=Path,
        default=ROOT,
        help="Experiment root (default: this directory)",
    )
    sub = parser.add_subparsers(dest="cmd", required=True)

    sub.add_parser("check", help="Validate tree, push includes, operators")
    sub.add_parser("weaken", help="Assert the weaken fixture is rejected")
    p_report = sub.add_parser("report", help="Emit parent adoption + catalog report")
    p_report.add_argument(
        "-o",
        "--out",
        type=Path,
        help="Write JSON report to this path (default: stdout)",
    )
    sub.add_parser("catalog", help="Print report focusing on merged catalog (stdout)")

    args = parser.parse_args(argv)
    root = args.root.resolve()
    if args.cmd == "check":
        return cmd_check(root)
    if args.cmd == "weaken":
        return cmd_weaken(root)
    if args.cmd == "report":
        return cmd_report(root, args.out)
    if args.cmd == "catalog":
        return cmd_catalog(root)
    raise SystemExit(f"unknown command {args.cmd}")


if __name__ == "__main__":
    raise SystemExit(main())

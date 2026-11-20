#!/usr/bin/env python3
"""Reference evaluator for the Toolbox reference portal.

A deliberately small, readable model of the v1 derived-state pipeline (docs/v1-spec.md §6):

  records → map (CEL) → fragments per source → merge by field precedence → CODEOWNERS authority
  → relations → computed fields (CEL) → scorecards (CEL) → authz tuples (CEL) → action options

It is the executable oracle for the Rust implementation's differential tests (D44): both must
produce the same files in expected/ and authz/tuples.expected.json from the same manifest and
fixtures. It also runs the hand-written assertions in tests/*.test.yaml.

Usage:
  python tools/reference.py            # run the pipeline and the assertions
  python tools/reference.py --check    # also fail if generated outputs differ from the committed ones
  python tools/reference.py --write    # regenerate the committed outputs

Requires: pyyaml, cel-python (0.5.0 tested).
"""
import fnmatch
import glob
import json
import os
import sys

import celpy
from celpy import celtypes

from yamlutil import load as load_yaml

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ENV = celpy.Environment()
_PROGRAMS = {}


# ---------------------------------------------------------------------------------------- CEL glue
def to_py(v):
    """Convert CEL values back to plain JSON-compatible Python values."""
    if v is None or isinstance(v, celtypes.NullType):
        return None
    if isinstance(v, celtypes.BoolType):
        return bool(v)
    if isinstance(v, (celtypes.IntType, celtypes.UintType)):
        return int(v)
    if isinstance(v, celtypes.DoubleType):
        return float(v)
    if isinstance(v, celtypes.StringType):
        return str(v)
    if isinstance(v, dict):
        return {to_py(k): to_py(x) for k, x in v.items()}
    if isinstance(v, (list, tuple)):
        return [to_py(x) for x in v]
    return v


def cel(expr, activation, functions=None):
    key = (expr, tuple(sorted(functions)) if functions else ())
    if key not in _PROGRAMS:
        _PROGRAMS[key] = ENV.program(ENV.compile(expr), functions=functions)
    result = _PROGRAMS[key].evaluate({k: celpy.json_to_cel(v) for k, v in activation.items()})
    if isinstance(result, celpy.CELEvalError):
        raise ValueError(f"CEL error in {expr!r}: {result}")
    return to_py(result)


# ------------------------------------------------------------------------------------------ inputs
def load():
    manifest = load_yaml(os.path.join(ROOT, "toolbox.yaml"))
    records = []  # (connector, record kind, data)
    for path in sorted(glob.glob(os.path.join(ROOT, "fixtures", "*.ndjson"))):
        connector = os.path.splitext(os.path.basename(path))[0]
        for line in open(path):
            if line.strip():
                msg = json.loads(line)
                if msg["type"] == "record":
                    records.append((connector, msg["kind"], msg["data"]))
    native = json.load(open(os.path.join(ROOT, "fixtures", "native.json")))
    return manifest, records, native


# ---------------------------------------------------------------------------------------- pipeline
def codeowners_for(rules, path):
    """Owners of `path`: the last matching rule wins (GitHub semantics, simplified patterns)."""
    owners = []
    for rule in rules or []:
        pat = rule["pattern"]
        if pat == "*" or (pat.startswith("/") and pat[1:] == path) or fnmatch.fnmatch(path, pat):
            owners = rule["owners"]
    return owners


def run(manifest):
    _, records, native = load()
    policy = manifest.get("policies", {})
    kinds = {k["name"]: k for k in manifest["kinds"]}
    fragments, kind_of, edges = {}, {}, []

    # 1. Map raw records to fragments, per source.
    for kind in manifest["kinds"]:
        for rule in kind.get("map", []):
            conn, rkind = rule["from"].split("/")
            source = rule.get("source", f"connector:{conn}")
            for c, k, data in records:
                if (c, k) != (conn, rkind):
                    continue
                eid = cel(rule["id"], {"record": data})
                kind_of[eid] = kind["name"]
                values = {f: v for f, v in cel(rule["set"], {"record": data}).items() if v is not None}
                fragments.setdefault(eid, {}).setdefault(source, {}).update(values)
                for rel, expr in rule.get("relations", {}).items():
                    for target in cel(expr, {"record": data}):
                        edges.append((rel, eid, target))

    # 2. Native values are one more source.
    for n in native:
        fragments.setdefault(n["entity"], {}).setdefault("native", {})[n["field"]] = n["value"]

    # 3. Existence: only identity sources create entities; the rest are orphans.
    entities, orphans = {}, []
    for eid, by_source in fragments.items():
        kind = kinds[kind_of[eid]]
        if not any(s in by_source for s in kind["identityFrom"]):
            orphans.append(eid)
            continue
        # 4. Merge by per-field precedence.
        srcs = kind.get("sources", {})
        ent, prov = {"id": eid}, {}
        for field, spec in kind["fields"].items():
            if "computed" in spec or "authority" in spec:
                continue
            for source in srcs.get("fields", {}).get(field, srcs.get("default", [])):
                if field in by_source.get(source, {}):
                    ent[field], prov[field] = by_source[source][field], source
                    break
        entities[eid] = {"kind": kind["name"], "entity": ent, "provenance": prov}

    # 5. CODEOWNERS authority for permission-relevant claims (D17).
    team_to_group = {
        "@" + e["entity"]["githubTeam"]: eid
        for eid, e in entities.items() if e["kind"] == "Group" and "githubTeam" in e["entity"]
    }
    for eid, e in entities.items():
        for field, spec in kinds[e["kind"]]["fields"].items():
            auth = spec.get("authority")
            if not auth:
                continue
            ent = e["entity"]
            claim = ent.get(auth["claim"])
            owning = [team_to_group[t] for t in codeowners_for(ent.get(auth["rules"]), auth["path"])
                      if t in team_to_group]
            if claim:
                value = {"status": "verified", "owner": claim, "claim": claim} if claim in owning \
                    else {"status": "unverified", "claim": claim}
            elif len(owning) == 1:
                value = {"status": "inferred", "owner": owning[0]}
            else:
                value = {"status": "none"}
            ent[field] = value
            e["provenance"][field] = "authority:codeowners"

    # 6. Relations (inbound), then computed fields.
    for kind in manifest["kinds"]:
        for rel, rspec in kind.get("relations", {}).items():
            for e in entities.values():
                if e["kind"] == rspec["to"]:
                    e.setdefault("inbound", {})[rel] = []
    for rel, src, target in sorted(edges):
        if src in entities and target in entities:
            entities[target].setdefault("inbound", {}).setdefault(rel, []).append(entities[src]["entity"])
    for eid, e in entities.items():
        for field, spec in kinds[e["kind"]]["fields"].items():
            if "computed" in spec:
                e["entity"][field] = cel(spec["computed"], {"entity": e["entity"],
                                                            "inbound": e.get("inbound", {}),
                                                            "policy": policy})
                e["provenance"][field] = "computed"

    # 7. Scorecards: pass | fail | unknown per rule; highest fully passing level.
    scores = {}
    for sc in manifest.get("scorecards", []):
        for eid, e in sorted(entities.items()):
            if e["kind"] != sc["kind"]:
                continue
            act = {"entity": e["entity"], "policy": policy}
            if not cel(sc.get("appliesTo", "true"), act):
                scores.setdefault(sc["name"], {})[eid] = {"level": "skipped"}
                continue
            outcomes = {}
            for rule in sc["rules"]:
                try:
                    outcomes[rule["name"]] = "pass" if cel(rule["check"], act) else "fail"
                except ValueError:
                    outcomes[rule["name"]] = "unknown"
            level = "none"
            for lvl in sc["levels"]:
                upto = sc["levels"][: sc["levels"].index(lvl) + 1]
                if all(outcomes[r["name"]] == "pass" for r in sc["rules"] if r["level"] in upto):
                    level = lvl
                else:
                    break
            scores.setdefault(sc["name"], {})[eid] = {"level": level, "rules": outcomes}

    # 8. Authorization tuples, derived from the catalog (never written separately).
    tuples = []
    for t in manifest["authz"]["tuples"]:
        if "static" in t:
            tuples.extend(t["static"])
            continue
        for eid, e in sorted(entities.items()):
            if e["kind"] == t["kind"]:
                tuples.extend(cel(t["emit"], {"entity": e["entity"], "policy": policy}))
    tuples = sorted({json.dumps(x, sort_keys=True) for x in tuples})
    tuples = [json.loads(x) for x in tuples]

    # 9. Per-entity action input options (enumFrom).
    options = {}
    for action in manifest.get("actions", []):
        for field, prop in action["inputs"]["properties"].items():
            if "enumFrom" not in prop:
                continue
            for eid, e in sorted(entities.items()):
                if e["kind"] == action["kind"]:
                    opts = cel(prop["enumFrom"], {"entity": e["entity"], "policy": policy})
                    options.setdefault(action["name"], {}).setdefault(field, {})[eid] = sorted(opts)

    out_entities = [
        {"id": eid, "kind": e["kind"], "fields": {k: v for k, v in e["entity"].items() if k != "id"},
         "provenance": e["provenance"]}
        for eid, e in sorted(entities.items())
    ]
    return {"entities": out_entities, "orphans": sorted(orphans), "scores": scores,
            "tuples": tuples, "options": options}


# ------------------------------------------------------------------------------------- assertions
def run_tests(manifest, result):
    by_id = {e["id"]: dict(e["fields"], id=e["id"]) for e in result["entities"]}

    def entity(eid):
        return celpy.json_to_cel(by_id[str(eid)])

    def score(name, eid):
        return celpy.json_to_cel(result["scores"][str(name)][str(eid)])

    def options(action, eid):
        fields = result["options"][str(action)]
        return celpy.json_to_cel(next(iter(fields.values()))[str(eid)])

    def orphans():
        return celpy.json_to_cel(result["orphans"])

    functions = {"entity": entity, "score": score, "options": options, "orphans": orphans}
    failures = 0
    for path in manifest.get("tests", []):
        suite = load_yaml(os.path.join(ROOT, path))
        for case in suite["assert"]:
            try:
                ok = cel(case["expect"], {}, functions) is True
            except ValueError as err:
                ok, case = False, dict(case, error=str(err))
            failures += not ok
            print(f"{'PASS' if ok else 'FAIL'}  {suite['name']}: {case['name']}"
                  + (f"  ({case['error']})" if "error" in case else ""))
    return failures


# --------------------------------------------------------------------------------------------- main
OUTPUTS = {
    "expected/entities.json": lambda r: r["entities"],
    "expected/scorecards.json": lambda r: r["scores"],
    "expected/options.json": lambda r: r["options"],
    "expected/orphans.json": lambda r: r["orphans"],
    "authz/tuples.expected.json": lambda r: r["tuples"],
}


def main(argv):
    manifest, _, _ = load()
    result = run(manifest)
    status = 0
    for rel, pick in OUTPUTS.items():
        path = os.path.join(ROOT, rel)
        text = json.dumps(pick(result), indent=2, sort_keys=True) + "\n"
        if "--write" in argv:
            open(path, "w").write(text)
        elif "--check" in argv and (not os.path.exists(path) or open(path).read() != text):
            print(f"DIFF  {rel} differs from the generated output (run with --write after review)")
            status = 1
    failures = run_tests(manifest, result)
    print(f"{failures} assertion(s) failed")
    return 1 if failures else status


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

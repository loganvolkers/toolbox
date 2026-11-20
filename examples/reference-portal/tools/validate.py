#!/usr/bin/env python3
"""Validate the reference portal against its JSON Schemas.

- toolbox.yaml against schema/manifest.schema.json
- every fixture line against schema/connector-messages.schema.json
- every protocol transcript message against the connector or executor message schema
- cross-references: map rules name declared connectors/records, scorecards and actions name declared
  kinds, precedence lists only name sources that some map rule (or native) provides
Requires: pyyaml, jsonschema.
"""
import glob, json, os, sys
import jsonschema, yaml

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
def schema(name):
    s = json.load(open(os.path.join(ROOT, "schema", name)))
    jsonschema.Draft202012Validator.check_schema(s)
    return jsonschema.Draft202012Validator(s, format_checker=jsonschema.FormatChecker())

errors = []
def check(validator, doc, where):
    for e in validator.iter_errors(doc):
        errors.append(f"{where}: {'/'.join(map(str, e.absolute_path))}: {e.message}")

manifest = yaml.safe_load(open(os.path.join(ROOT, "toolbox.yaml")))
check(schema("manifest.schema.json"), manifest, "toolbox.yaml")

conn = schema("connector-messages.schema.json")
exe = schema("executor-messages.schema.json")
n = 0
for path in sorted(glob.glob(os.path.join(ROOT, "fixtures", "*.ndjson"))):
    for i, line in enumerate(open(path), 1):
        if line.strip():
            check(conn, json.loads(line), f"{os.path.relpath(path, ROOT)}:{i}"); n += 1
for path in sorted(glob.glob(os.path.join(ROOT, "protocol", "*.ndjson"))):
    for i, line in enumerate(open(path), 1):
        if line.strip():
            env = json.loads(line)
            check(exe if "executor" in env["dir"] else conn, env["msg"], f"{os.path.relpath(path, ROOT)}:{i}"); n += 1

# Cross-references the schema can't express.
connectors = {c["name"]: set(c["records"]) for c in manifest.get("connectors", [])}
kinds = {k["name"]: k for k in manifest["kinds"]}
for k in manifest["kinds"]:
    provided = {"native"}
    for r in k.get("map", []):
        c, rk = r["from"].split("/")
        if c not in connectors or rk not in connectors[c]:
            errors.append(f"kind {k['name']}: map from {r['from']} names an undeclared connector/record")
        provided.add(r.get("source", f"connector:{c}"))
    srcs = k["sources"]
    for f, lst in [("default", srcs["default"])] + list(srcs.get("fields", {}).items()):
        if f != "default" and f not in k["fields"]:
            errors.append(f"kind {k['name']}: sources for undeclared field {f}")
        for s in lst:
            if s not in provided:
                errors.append(f"kind {k['name']}: source {s} for {f} is provided by no map rule")
for s in manifest.get("scorecards", []):
    if s["kind"] not in kinds: errors.append(f"scorecard {s['name']}: unknown kind {s['kind']}")
    for r in s["rules"]:
        if r["level"] not in s["levels"]: errors.append(f"scorecard {s['name']}: rule {r['name']} has unknown level")
bundles = {b["name"] for b in manifest.get("ui", {}).get("bundles", [])}
for a in manifest.get("actions", []):
    if a["kind"] not in kinds: errors.append(f"action {a['name']}: unknown kind {a['kind']}")
    for name, lk in a.get("context", {}).items():
        if lk["kind"] not in kinds: errors.append(f"action {a['name']}: context {name} names unknown kind")
    if "ui" in a and a["ui"]["bundle"] not in bundles: errors.append(f"action {a['name']}: unknown UI bundle")
    for r in a.get("expect", {}).get("refresh", []):
        if r["connector"] not in connectors or r["kind"] not in connectors[r["connector"]]:
            errors.append(f"action {a['name']}: refresh names an undeclared connector/record")

# Action inputs must stay within the form-renderable subset (D58, docs/design/minimal-ui.md §4).
SCALAR = {"string", "integer", "number", "boolean"}
KEYS = {"type", "title", "description", "default", "examples", "enum", "enumFrom", "minLength", "maxLength",
        "pattern", "format", "minimum", "maximum", "items", "minItems", "maxItems", "uniqueItems",
        "properties", "required", "x-toolbox-widget"}
def form_ok(schema, where, depth=0):
    bad = set(schema) - KEYS
    if bad: errors.append(f"{where}: keywords outside the form subset: {sorted(bad)}")
    t = schema.get("type")
    if t == "object":
        if depth > 1: errors.append(f"{where}: objects may be nested one level only")
        for name, sub in schema.get("properties", {}).items(): form_ok(sub, f"{where}.{name}", depth + 1)
    elif t == "array":
        if schema.get("items", {}).get("type") not in SCALAR: errors.append(f"{where}: arrays must hold scalars")
    elif t not in SCALAR:
        errors.append(f"{where}: unsupported type {t!r}")
for a in manifest.get("actions", []):
    if a["inputs"].get("type") != "object": errors.append(f"action {a['name']}: inputs must be an object")
    form_ok(a["inputs"], f"action {a['name']} inputs")

for e in errors: print("ERROR", e)
print(f"validated toolbox.yaml + {n} messages: {len(errors)} error(s)")
sys.exit(1 if errors else 0)

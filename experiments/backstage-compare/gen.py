#!/usr/bin/env python3
"""One synthetic catalog, written for both arms: catalog.yaml (Backstage entities) and
catalog.ndjson (Seikika ingest records). Deterministic: the same N gives the same bytes."""
import json
import sys
from pathlib import Path

GROUPS, SYSTEMS = 50, 20
LIFECYCLES = ["experimental", "production", "deprecated"]


def main() -> None:
    n = int(sys.argv[1])
    out = Path(sys.argv[2])
    out.mkdir(parents=True, exist_ok=True)
    docs, recs = [], []
    for g in range(GROUPS):
        docs.append({"apiVersion": "backstage.io/v1alpha1", "kind": "Group",
                     "metadata": {"name": f"g{g}", "title": f"Group {g}"},
                     "spec": {"type": "team", "children": []}})
        recs.append({"kind": "group", "name": f"g{g}", "title": f"Group {g}"})
    for s in range(SYSTEMS):
        owner = f"g{s % GROUPS}"
        docs.append({"apiVersion": "backstage.io/v1alpha1", "kind": "System",
                     "metadata": {"name": f"s{s}", "title": f"System {s}"},
                     "spec": {"owner": f"group:{owner}"}})
        recs.append({"kind": "system", "name": f"s{s}", "title": f"System {s}", "owner": owner})
    for c in range(n):
        owner, system, life = f"g{c % GROUPS}", f"s{c % SYSTEMS}", LIFECYCLES[c % 3]
        docs.append({"apiVersion": "backstage.io/v1alpha1", "kind": "Component",
                     "metadata": {"name": f"c{c}", "title": f"Component {c}"},
                     "spec": {"type": "service", "lifecycle": life,
                              "owner": f"group:{owner}", "system": system}})
        recs.append({"kind": "component", "name": f"c{c}", "title": f"Component {c}",
                     "owner": owner, "system": system, "lifecycle": life})
    # JSON is YAML: one document per entity, no YAML library needed.
    (out / "catalog.yaml").write_text("".join(f"---\n{json.dumps(d)}\n" for d in docs))
    with (out / "catalog.ndjson").open("w") as f:
        for r in recs:
            kind = r.pop("kind")
            f.write(json.dumps({"type": "record", "kind": kind,
                                "externalId": f"catalog:{kind}/{r['name']}", "data": r}) + "\n")
    print(f"{len(docs)} entities ({n} components) in {out}")


if __name__ == "__main__":
    main()

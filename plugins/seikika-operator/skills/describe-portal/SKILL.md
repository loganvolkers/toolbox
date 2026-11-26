---
name: describe-portal
description: >-
  Prints a portal's kinds, fields, connectors, scorecards and rules as records. Use when inspecting a portal's model, and when a meta scorecard needs those records.
---

# seikika describe portal

1. Pass one or more portal directories.
2. Add --results PORTAL=FILE when live outcomes should be included. They are information, not a gate.
3. Read the NDJSON. Do not hand-edit a described record back into the manifest.

```
seikika describe portal portal examples/reference-portal
```

Do not confuse this with seikika describe, which prints the CLI and MCP surface. If it fails, a portal path has no manifest.

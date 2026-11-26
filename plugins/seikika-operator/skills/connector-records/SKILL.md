---
name: connector-records
description: >-
  Prints one resync of a portal connector's records as NDJSON, including each lint record's command and slot. Use to see exactly what a tool reported before any rule read it, or to rerun a tool on its own.
---

# seikika connector records

1. Pass --portal as the portal directory and --name as the connector's name in its manifest.
2. Read the records. Lint records carry `command`, the exact command that reproduces that file's findings with the tool alone, and `via`, the slot or fitting that read the output (`sarif`, `actionlint`).
3. To check a finding, run the record's `command` yourself. The box must give the same findings as the tool; `scripts/take-it-out.py` checks every record this way.

```
seikika connector records --portal portal --name ci-lint
```

It prints records only; nothing is scored or written. If it fails, the connector name is wrong or its program isn't installed.

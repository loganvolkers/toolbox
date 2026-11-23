---
name: connector-drive
description: >-
  Drives a connector binary over stdio for one resync and prints record NDJSON.
  Use when dumping what a connector emits without a portal, or instead of
  hand-writing stdio command JSON.
---

# toolbox connector drive

1. Pass `--kinds` as a comma-separated list of record kinds to resync.
2. Pass `--config` as a JSON object (becomes `TOOLBOX_CONNECTOR_CONFIG`), or set
   that env var yourself.
3. Put the connector program after `--`.
4. Default `--format records` prints `kind` / `externalId` / `data`. For a
   catalog dump use `--format catalog` (sorted `kind` / `id` / `data`; `id` is
   `externalId` with a leading `{kind}:` stripped) and `--out PATH` to write the
   file. Exit 0 only when the resync completes successfully.

```
toolbox connector drive --kinds project,issue \
  --config '{"baseUrl":"http://127.0.0.1:9743","token":"tenant-a-token"}' \
  -- target/debug/forge-connector

toolbox connector drive --kinds project,issue --format catalog --out OUT/entities.ndjson \
  --config '{"baseUrl":"http://127.0.0.1:9743","token":"tenant-a-token"}' \
  -- target/debug/forge-connector
```

Do not omit the `--` before the program. Do not hand-write a map script from
records to catalog when `--format catalog` covers the id rule. If it fails, the
binary did not register, the kinds are wrong, or the resync did not succeed.

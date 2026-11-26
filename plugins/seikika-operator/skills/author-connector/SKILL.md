---
name: author-connector
description: >-
  Write a Seikika stdio connector (SaaS or otherwise) with the Rust SDK: Source,
  portal stub, connector test, and a one-shot record dump. Use when creating or
  fixing a connector binary, or when seikika connector test needs a portal.
---

# Author a connector

Work in this order. Do not invent a second protocol.

1. **Copy the scaffold** at `examples/connector-scaffold/` (or start from
   `crates/seikika-connector-git-files/`). Keep `rust-toolchain.toml` so edition
   2024 path deps build.
2. **Implement `Source`**: `name`, **`version` (return `env!("CARGO_PKG_VERSION")`
   from your crate)**, `capabilities` (`resync` / `refresh` kinds), and `handle`.
   Call `run_stdio(src).await` from `main`. Read config only from
   `SEIKIKA_CONNECTOR_CONFIG` JSON.
3. **Add a minimal portal** at `portal/seikika.yaml` that declares the connector
   name, `records: […]`, and enough kinds to map those records. `seikika
   connector test` loads the portal; a binary alone is not enough.
4. **Prove the protocol**:
   ```
   cargo build
   seikika connector test --name <name> --portal portal -- target/debug/<bin>
   ```
5. **Dump a catalog without hand-built stdio JSON or a map script**:
   ```
   seikika connector drive --kinds project,issue --format catalog \
     --out OUT/entities.ndjson \
     --config '{"baseUrl":"http://127.0.0.1:9743","token":"…"}' \
     -- target/debug/<bin>
   ```
   `--format catalog` writes sorted `{"kind","id","data"}` lines (`id` =
   `externalId` with a leading `{kind}:` stripped). Use default `--format
   records` only when debugging the protocol itself.
6. **Tenant / config switches:** for a dump-only resync, pass `--config` to
   `seikika connector drive` and leave the portal alone. Update
   `portal/seikika.yaml` connector `config:` only when you also run
   `connector test` against that tenant.

```
seikika connector test --name forge --portal portal -- target/debug/forge-connector
seikika connector drive --kinds project,issue --format catalog --out OUT/entities.ndjson \
  --config '{"baseUrl":"http://127.0.0.1:9743","token":"t"}' -- target/debug/forge-connector
```

Do not skip `version()`. Do not drive stdin with a home-grown command JSON when
`connector drive` exists. Read `docs/design/connector-control.md` only if a
conformance check fails.

**externalId shape:** every record's `externalId` must match `^[a-z0-9-]+:.+$`
(see `schema/connector-messages.schema.json`). Prefer `project:ALPHA` /
`issue:ALPHA-1`, not bare keys. Catalog entity ids in a portal `map:` can still
be the bare key — that is a second id space.

**Composite kinds (e.g. components):** when the natural catalog id is
`{project}:{name}`, set `externalId` to `component:{project}:{name}` so
`--format catalog` strips only the leading `component:` and keeps
`ALPHA:api`. Do **not** use the upstream numeric id as `externalId` (that
yields catalog id `1`). Put the upstream id in `data.id` instead, and emit
only the fields the catalog schema asks for (no extra keys like
`description`).


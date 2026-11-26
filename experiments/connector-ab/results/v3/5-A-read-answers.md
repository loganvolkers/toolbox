# Read answers (tenant B dump)

Sources: `OUT/entities.ndjson`, `RUN.md`, `portal/seikika.yaml`, `src/main.rs` (config keys only).

## 1. How many projects does tenant B have?

**1** — project id `GAMMA` (`kind: project` in `OUT/entities.ndjson`; also stated in `RUN.md`).

## 2. List issue keys on tenant B.

- `GAMMA-1`
- `GAMMA-2`

## 3. Who is assigned to GAMMA-1 (account id)?

`u-cara` (`data.assignee` on the `GAMMA-1` issue record).

## 4. Name every component on GAMMA.

- `core` (catalog id `GAMMA:core`)

## 5. Which config fields switch tenants?

`baseUrl` and `token` (JSON in `SEIKIKA_CONNECTOR_CONFIG` / `connector drive --config`, and the portal connector `config` object). Same binary; only these two fields change between tenant A (`9743` / `tenant-a-token`) and tenant B (`9744` / `tenant-b-token`).

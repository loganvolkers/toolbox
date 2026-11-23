# Phase 5 (Read) — arm A answers

Source of truth: `OUT/entities.ndjson` (tenant B sync), `RUN.md`, `portal/toolbox.yaml`.

## 1. How many projects does tenant B have?

**1** — project key `GAMMA` (`Gamma`).

## 2. List issue keys on tenant B

- `GAMMA-1`
- `GAMMA-2`

## 3. Who is assigned to GAMMA-1 (account id)?

`u-cara`

## 4. Name every component on GAMMA

- `core` (catalog id `GAMMA:core`)

## 5. Which config fields switch tenants?

`baseUrl` and `token` (in `TOOLBOX_CONNECTOR_CONFIG` and/or `portal/toolbox.yaml` → `connectors[].config`).

| Tenant | baseUrl                   | token            |
|--------|---------------------------|------------------|
| A      | `http://127.0.0.1:9743`   | `tenant-a-token` |
| B      | `http://127.0.0.1:9744`   | `tenant-b-token` |

# Reference Portal: "acme" on GitHub

The reference implementation anchor for Toolbox v1. It is a complete, small portal for a fictional
organisation, with GitHub, a git catalog and an IdP. **An implementation of Toolbox is correct for
v1 when it runs this portal and produces the outputs and behaviours described here**
([docs/v1-spec.md](../../docs/v1-spec.md) §11).

Everything here that can run today has been run (see [Validation](#validation)). The parts that
need the Toolbox binary are specified precisely enough to implement against.

## What it demonstrates

| Capability | Where | Decisions |
|---|---|---|
| Manifest as the only configuration | [`toolbox.yaml`](./toolbox.yaml), validated by [`schema/manifest.schema.json`](../../schema/manifest.schema.json) and by `toolbox validate` | D2, D41 |
| Raw records mapped to entities with CEL (object construction) | `kinds[].map` | D24 |
| Field-level sources and precedence, native values | `kinds[].sources`, [`fixtures/native.json`](./fixtures/native.json) | D13 |
| Entities created only by identity sources; orphans reported | `identityFrom`; the `ghost-service` fixture | D13 |
| CODEOWNERS authority for ownership claims | `Repository.fields.ownership.authority` | D17 |
| Relations and computed fields | `GitHubAppInstallation.installedOn` → `Repository.installedApps` | D37 |
| Scorecards with levels, `skipped`, facets | `scorecards` | D15 |
| Authorization tuples derived from the catalog; public by default | `authz.tuples`, [`authz/model.fga`](./authz/model.fga) | D7, D15, D26, D38 |
| Self-service action with options, approval, preview, confirm and verification | `actions[install-approved-app]` | D40, D49 |
| Connector control (scoped refresh for verification) | `connectors[].refresh`, [`protocol/install-approved-app.connector.ndjson`](./protocol/install-approved-app.connector.ndjson) | D48 |
| In-image executor | [`executors/github-apps/`](./executors/github-apps/README.md), [`protocol/install-approved-app.executor.ndjson`](./protocol/install-approved-app.executor.ndjson) | D40, D49 |
| Custom action UI through the UI extension contract | [`ui/install-approved-app/`](./ui/install-approved-app/README.md) | D40 |
| One image, deployed to Cloud Run by a full SDLC | [`Dockerfile`](./Dockerfile), [`.github/workflows/portal.yml`](./.github/workflows/portal.yml) | D33, D34 |

## The organisation (fixtures)

| Repository | catalog-info owner | CODEOWNERS for `catalog-info.yaml` | Result |
|---|---|---|---|
| `payments-api` | `group:payments` | `@acme/payments` | **verified** owner; ownership **silver** |
| `checkout-web` | `group:checkout` | `@acme/checkout @acme/platform` (file rule) | verified; **bronze** (branch not protected); app hygiene **fails** (`sketchy-bot`) |
| `ledger` | `group:payments` | `@acme/platform` | **unverified** claim, not applied; ownership **none**; only portal admins see details |
| `platform-infra` | none | `@acme/platform` | **inferred** owner; **bronze**; tier `1` from a native value |
| `legacy-tool` | none | none | archived → `deprecated`; scorecards **skipped** |
| `ghost-service` | `group:payments` | — | catalog-info only; no GitHub repo → **orphan**, no entity |

People: `alice` (payments, admin of `payments-api` and `ledger`), `bob` (checkout, nested in
payments), `carol` (security), `dave` (platform, portal admin), `erin` (no team).
Apps: `renovate` (approved, matches snapshot), `snyk-security` (approved, but now requests
`administration:read`, so its snapshot check fails), `sketchy-bot` (unapproved).

## Expected outputs (the differential-test oracle)

[`tools/reference.py`](./tools/reference.py) is a small, readable model of the v1 pipeline
(map → merge → authority → relations → computed → scorecards → tuples → options). It generates:

| File | Contents |
|---|---|
| [`expected/entities.json`](./expected/entities.json) | Every entity's merged fields **and the source of each field** |
| [`expected/scorecards.json`](./expected/scorecards.json) | Level and per-rule outcome per scorecard and entity |
| [`expected/options.json`](./expected/options.json) | Per-entity `enumFrom` options for each action input |
| [`expected/orphans.json`](./expected/orphans.json) | Fragments with no identity source |
| [`authz/tuples.expected.json`](./authz/tuples.expected.json) | The derived OpenFGA tuples |

The Rust implementation must produce byte-identical JSON from the same manifest and fixtures. The
hand-written expectations in [`tests/`](./tests) are independent of the generator: they state the
intended behaviour, and the generator is only trusted because they pass.

## Acceptance scenarios (with the Toolbox binary)

Each scenario is a **user SDLC** task (layer 3, [docs/sdlc.md](../../docs/sdlc.md)) that the
**maintainer SDLC** (layer 1) runs as an acceptance check. Status on 2026-11-20 (M0–M4 done):

| Scenarios | State | Checked by |
|---|---|---|
| A3, A4, A10 (preview gate only) | ✅ | `scripts/m3-acceptance.sh` |
| A1–A5, A11 | ✅ | `scripts/m4-acceptance.sh` (A11 in Chromium via Playwright) |
| A6–A9, A10 (runs) | Pending | M5 (actions) |
| A12 | Pending | M6 (ship) |

The scripts need Postgres 16; M3 and M4 also need `TOOLBOX_OPENFGA_BIN`, and M4 needs Node with
Playwright. Run them from the repo root, for example `scripts/m4-acceptance.sh`.

| # | Interface | Scenario | Expected |
|---|---|---|---|
| A1 | MCP `get` / CLI | "Who owns payments-api?" | `group:payments`, verified via CODEOWNERS, with the snapshot id |
| A2 | MCP `explain` | "Who owns ledger?" | No applied owner; claim `group:payments` is unverified because CODEOWNERS names `@acme/platform`; the fix |
| A3 | MCP `scorecard_status` as `erin` | "Is checkout-web meeting standards?" | Levels visible (bronze, hygiene none); rule details **redacted** with a marker saying who can see them |
| A4 | Same as `bob` | Same question | Details visible: `branch-protected` fails, `sketchy-bot` unapproved, with fix text |
| A5 | API `GET /v1/entities?filter=…` | Repositories with `ownership.status != 'verified'` | `ledger`, `platform-infra`, `legacy-tool` |
| A6 | MCP `action_options` as `bob` | "What can I install on checkout-web?" | `snyk-security` only (`renovate` already installed) |
| A7 | API preview as `bob` | Preview install of `snyk-security` | One `grant` effect plus the snapshot-drift warning; confirmation token |
| A8 | API run with the token | Run, simulated executor succeeds, simulated connector refresh returns the new installation | Run ends `succeeded`; `checkout-web.installedApps` includes `snyk-security` at the returned snapshot |
| A9 | Same, but the refresh doesn't show the repo | Executor says success, effect missing | Run ends `failed` with the unmet postcondition, never `succeeded` |
| A10 | Same as `erin` | Preview or run | `403` with the missing relation (`can_install_apps`) |
| A11 | UI "Refresh" on payments-api | Refresh command | Progress, then the page at a new snapshot with an "as of" time |
| A12 | Deploy | `docker build` + Cloud Run with min instances 1 | Ready after the first recompute; MCP lists `install_approved_github_app` |

## Validation

Run from this directory (Python 3.11+):

```bash
python -m venv .venv && .venv/bin/pip install -r tools/requirements.txt
.venv/bin/python tools/validate.py          # manifest + 40 messages against ../../schema
cargo run -q -p toolbox-cli -- validate .   # the same checks in Rust, plus CEL, files and fixtures (from the repo root: toolbox validate examples/reference-portal)
.venv/bin/python tools/reference.py --check # pipeline, 20 hand-written assertions, committed outputs
fga model test --tests authz/model.fga.yaml # OpenFGA over the derived tuples
```

Results on 2026-11-20 (recorded in [docs/research-log.md](../../docs/research-log.md)):
- validate: 0 errors
- reference: 20/20 assertions pass, outputs unchanged
- OpenFGA: 3/3 tests, 16/16 checks, 3/3 ListObjects

Each check was also shown to fail when an expectation is wrong. Not run yet: the Dockerfile and the
workflow (they need `toolbox build` and actions, M5–M6); the workflow YAML parses.

## Layout

```
toolbox.yaml                       the manifest
fixtures/*.ndjson, native.json     connector records (valid connector messages) and native values
tests/*.test.yaml                  hand-written expectations (CEL over entity/score/options/orphans)
expected/, authz/tuples.expected.json   generated oracle outputs
authz/model.fga, model.fga.yaml    OpenFGA model and tests over the derived tuples
protocol/*.ndjson                  golden executor and connector conversations
executors/github-apps/             in-image executor specification
ui/install-approved-app/           UI bundle specification
Dockerfile, .github/workflows/     one image; SDLC to Cloud Run
tools/                             reference.py (oracle), validate.py (schemas + cross-references)
```

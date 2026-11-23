# Connector A/B: Authoring a Cloud/SaaS Connector in Toolbox, Ocean or Backstage

**Question:** when a coding agent authors a cloud/SaaS connector that syncs projects and issues
from an HTTP API into a catalog, does Toolbox's connector protocol cost fewer tokens and leave
less glue than Port Ocean or a Backstage entity provider, at equal correctness, across the
package-author lifecycle (author → test → change → reuse → read)?

**Status: run, 2026-11-23. C1 and C2 failed; C3 held.** Full write-up: [RESULTS.md](./RESULTS.md).
Hypotheses C1–C3 and the bar in §6 were not edited after the first arm started.

This measures the **package-author** layer's `build-connector` stage and neighbouring stages
([package-author-sdlc.md](../../docs/design/package-author-sdlc.md),
[dx-benchmarks.md](../../docs/design/dx-benchmarks.md) §7 B4). It is about a **tool** that fits the
box (a SaaS connector), not the box itself ([toolbox-and-tools.md](../../docs/design/toolbox-and-tools.md)).
It deliberately does **not** measure local-tool fittings (ESLint, ruff, SARIF kits): those were
the standards and box A/Bs.

Decision: [D01M833YAH8MT4VY2W4K1PYAEHB](../../docs/decisions/D01M833YAH8MT4VY2W4K1PYAEHB.md).

## 0. Results

| Hypothesis | Result |
|---|---|
| **C1** Toolbox cheaper on author+change (calls 1+3) | **Fail** — A 65, B 63, C **46** |
| **C2** Toolbox cheaper on reuse (phase 4) | **Fail** — A 15, B 15, C 16 |
| **C3** Correctness | **Hold** — all arms exact on phases 1–5 |

Details, friction triage and deviations: [RESULTS.md](./RESULTS.md).

## 1. Why Forge, not live Jira / GitHub / GCP

| Option | Verdict |
|---|---|
| **Live Jira Cloud / GitHub / GCP** | Needs credentials, rate limits and network; truth drifts; arms would not be equal |
| **Reuse Ocean's real Jira integration as the task** | Biases the Ocean arm; agents would copy rather than author |
| **Forge (this experiment)** | A local mock with Jira Cloud-shaped REST (`/rest/api/3/project`, `/rest/api/3/search`, then `/components`), Bearer auth and pagination. Two tenants for reuse. Independent truth is a fixture file |

Forge is the SaaS under test. Agents may not invent a different API. The frameworks are the arms.

## 2. Arms

| | A: Toolbox | B: Port Ocean | C: Backstage |
|---|---|---|---|
| Given | `toolbox-connect` Rust SDK (path dep), `toolbox connector test`, connector-control docs, reference connectors as read-only examples | Ocean at `port-labs/ocean@f9f3948e`, the `fake-integration` and `jira` trees as read-only examples, a scaffolded integration directory | Backstage catalog backend types from `backstage/backstage@feeadd4d`, the GitHub entity provider as a read-only example, a scaffolded backend module |
| Builds | A stdio connector speaking Toolbox connector messages | An Ocean integration with `@ocean.on_resync` per kind and a mapping config | A TypeScript `EntityProvider` that emits catalog entities |
| Must produce | `entities.ndjson` in the experiment's common schema (§4), plus whatever the framework needs to run | The same | The same |
| Must not | Read Ocean or Backstage trees | Read Toolbox crates or docs | Read Toolbox or Ocean |

**Pinned open-source (read allowlists):**
- `port-labs/ocean@f9f3948e83a20769f32fd3ca9fe0581cc8755926`
- `backstage/backstage@feeadd4d4efcbda9f1573b95e356e0f3fc0b5d17` (sparse: catalog backend + GitHub provider)
- This repository's connector SDK and docs at the commit that starts the run (Toolbox arm only)

Setup installs frameworks and starts Forge **before** each phase. Install time and disk are
recorded separately and are not in the token bar.

## 3. Phases

| # | Task | Scored by |
|---|---|---|
| 1 | **Author.** Sync Forge `project` and `issue` from tenant A into the framework; write `entities.ndjson`; document how to run a resync | `harness/truth.py` exact match on kinds and ids; framework shape check (§5) |
| 2 | **Test.** No agent. Harness runs the arm's documented resync, re-scores truth, and (Toolbox) `toolbox connector test` | Truth + conformance |
| 3 | **Change.** Forge now exposes `component` per project. Add the kind; keep projects and issues correct | Truth including components |
| 4 | **Reuse.** Point the same connector at tenant B (different base URL and token). Do not rewrite the sync logic | Truth for tenant B |
| 5 | **Read.** From the arm's artifacts alone (no live resync), answer the read key's questions | `harness/read-key.md` |

**Measured per phase (1, 3, 4, 5):** tokens, tool calls, wall time; lines of glue the team would
maintain; correctness; friction feedback (same prompt wording every arm).

## 4. Common entity schema

Every arm writes `OUT/entities.ndjson` (one JSON object per line), sorted by `kind` then `id`:

```json
{"kind":"project","id":"ALPHA","data":{"key":"ALPHA","name":"Alpha","type":"software"}}
{"kind":"issue","id":"ALPHA-1","data":{"key":"ALPHA-1","summary":"...","status":"To Do","project":"ALPHA","assignee":null}}
{"kind":"component","id":"ALPHA:api","data":{"id":"1","name":"api","project":"ALPHA"}}
```

`component` is required only from phase 3. The harness compares against
`fixtures/tenant-{a,b}/expected-*.ndjson`. Field names and nullability must match; extra fields
fail. Raw Forge payloads alone are not enough: the arm must map into this schema (Ocean via JQ or
code; Backstage via entity mapper; Toolbox via record `data` or a small post-step).

## 5. Framework shape (so arms don't bypass)

| Arm | Required shape |
|---|---|
| A | A binary that speaks `register` / `resync` / `refresh` / `record` / `snapshot_complete` / `complete` over stdio; `toolbox connector test` must pass for the declared kinds |
| B | At least one `@ocean.on_resync("<kind>")` handler per synced kind under the integration; a `.port/spec.yaml` (or Ocean-equivalent) listing those kinds |
| C | A class implementing `EntityProvider` (or `connect`/`getProviderName`/`connect` pattern from the pinned Backstage tree) registered from the module's `index` |

A phase that only curls Forge into `entities.ndjson` without the shape fails the framework check,
even if truth matches.

## 6. The bar, set before running

| Hypothesis | Holds if | Fails if |
|---|---|---|
| **C1. Toolbox is cheaper to author and change** | Over phases 1+3, arm A uses fewer tokens than both B and C, with equal correctness | A uses as many or more than either rival |
| **C2. Toolbox reuses cheaper** | Phase 4 (tenant B) uses fewer tokens in A than in B and in C, with equal correctness | A uses as many or more |
| **C3. Platforms beat a confused agent on correctness** | Every arm that finishes within budget is fully correct on truth + framework shape for phases 1–4 | Any finishing arm is incorrect: the task or docs are unclear; fix friction and rerun that arm before publishing a loss on cost |

**If an agent is blocked or confused by the harness itself** (can't find Forge, can't find the
scaffold, reads a forbidden tree by accident, burns the budget on setup): discard that phase,
fix the harness or prompt, restore the arm, and relaunch. Do not count discarded runs in the bar.
Record every discard in §0.

**Budget:** 50 tool calls per agent phase; network only to `127.0.0.1` (Forge) and reading the
arm's allowlisted docs on disk. No installs during a phase.

## 7. Known limits (L3)

| Known friction | Treatment |
|---|---|
| Toolbox has no published multi-language connector SDK; Rust `toolbox-connect` is a path dep | **In scope:** measuring that gap is part of C1 |
| No `toolbox connector new` scaffold | **Mitigated:** the experiment ships a minimal Cargo scaffold; note it in results |
| Ocean needs Port SaaS for a full `ocean sail`; we run handlers against Forge and score mapped output | **Deviation:** Ocean arm is not pushing to Port; recorded in §0 |
| Full `npx @backstage/create-app` is heavy; setup scaffolds a backend module only | **Deviation:** recorded; install cost separate |
| One run per phase (L10) | Stated; prefer a second run only when a phase was discarded |

## 8. How to run

```bash
# From the repository root, after cargo build -p toolbox-cli -p toolbox-connect
export AB_HOME=/tmp/connector-ab
./experiments/connector-ab/harness/setup.sh          # pins, fixtures, Forge, scaffolds
python3 -I experiments/connector-ab/harness/forge.py --tenant a --port 9743 &
# Launch one fresh agent per (phase, arm) with prompts from prompts/
python3 -I experiments/connector-ab/harness/score.py <arm_dir> <tenant> <phase>
```

Prompts are rendered by `prompts/make.py`. Results land in `results/`. Export each arm's diff
before deleting its working copy (L9).

## Sources

- [docs/design/dx-benchmarks.md](../../docs/design/dx-benchmarks.md)
- [docs/design/package-author-sdlc.md](../../docs/design/package-author-sdlc.md)
- [docs/ocean.md](../../docs/ocean.md)
- [docs/design/connector-control.md](../../docs/design/connector-control.md)
- port-labs/ocean@f9f3948e (fake-integration, jira)
- backstage/backstage@feeadd4d (plugins/catalog-backend-module-github)

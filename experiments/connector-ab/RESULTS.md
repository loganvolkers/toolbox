# Connector A/B Results

**v1 baseline.** For the learning-tax rematch (Toolbox wins C1–C3), see
[RESULTS-v2.md](./RESULTS-v2.md).

**Status: run complete, 2026-11-23.** Pre-registered bar: [README.md](./README.md) §6
(C1–C3). Decision: [D01M833YAH8MT4VY2W4K1PYAEHB](../../docs/decisions/D01M833YAH8MT4VY2W4K1PYAEHB.md).

SaaS under test: **Forge** (local Jira Cloud-shaped mock). Arms: **A Toolbox**, **B Port
Ocean**, **C Backstage EntityProvider**. Open-source pins:
`port-labs/ocean@f9f3948e`, `backstage/backstage@feeadd4d`.

Tokens were not present in agent transcripts; **tool calls** are the cost measure (same
model and budget per arm). One run per phase (L10).

## Against the bar (§6)

| Hypothesis | Holds if | Result |
|---|---|---|
| **C1. Toolbox cheaper to author and change** | A fewer calls than B and C over phases 1+3, equal correctness | **Fails.** A 65, B 63, C **46**. C cheapest; A not ahead of either rival |
| **C2. Toolbox reuses cheaper** | A fewer calls than B and C in phase 4 | **Fails.** A 15, B 15, C 16 — A tied with B, not strictly fewer than both |
| **C3. Platforms correct within budget** | Finishing arms fully correct on truth+shape (1–4) and read (5) | **Holds.** All three arms exact on every phase |

**By the rule set before the run:** C1 and C2 miss. On this task, Backstage was cheapest to
author and change; Ocean left the least glue; Toolbox matched correctness and reuse cost with
Ocean but paid a learning tax in phase 1 (53 calls, over the 50 budget).

## Cost and glue

| | A Toolbox | B Ocean | C Backstage |
|---|---|---|---|
| Calls 1+3 (C1) | 65 | 63 | **46** |
| Calls phase 4 (C2) | **15** | **15** | 16 |
| Calls phase 5 | 11 | **10** | 13 |
| Calls total 1+3+4+5 | 91 | 88 | **75** |
| Glue after phase 4 (lines) | 550 | **342** | 483 |
| Correctness (all phases) | pass | pass | pass |

### Per phase

| Phase | A calls | B calls | C calls | Truth+shape / read |
|---|---|---|---|---|
| 1 Author | 53 | 41 | **29** | all pass (5 entities) |
| 3 Change (+component) | **12** | 22 | 17 | all pass (8 entities) |
| 4 Reuse tenant B | **15** | **15** | 16 | all pass (4 entities) |
| 5 Read | 11 | **10** | 13 | all 5/5 answers |

Phase 1 is where Toolbox loses: 53 vs 29–41. After that, A is competitive or best on
change (12) and tied on reuse (15).

## What the numbers say

- **Learning cost, again.** Same pattern as the box and standards A/Bs: Toolbox's phase-1 tax
  (edition2024 toolchain, inventing a portal for `connector test`, hand-built stdio resync)
  dominates authoring cost. Once built, adding a kind (phase 3) was cheapest on Toolbox.
- **Ocean wins on glue** (342 lines) via a thin Python client + shim; it could not import real
  `port_ocean` without fastapi (README §7 deviation).
- **Backstage wins on authoring calls** with a TypeScript EntityProvider and Node strip-types
  (no `tsc` install). Dual schema (DeferredEntity vs experiment NDJSON) added friction but not
  failure.
- **Reuse is config-only on all three** — the interesting C2 gap did not appear; all arms
  switched tenants with env/config and ~15 calls.

## Deviations from protocol

1. Truncated phase-3 Toolbox launch discarded; relaunched ([results/run-log.md](./results/run-log.md)).
2. Mid-run harness fix: `rust-toolchain.toml` in Toolbox scaffold (edition2024).
3. Ocean: local `ocean_shim` instead of Port SaaS / full Ocean install.
4. Backstage: Node `--experimental-strip-types`; no full Backstage backend.
5. Arm A phase 1: 53 calls > 50 budget (counted).
6. No token metadata; calls substitute for the bar's "tokens" wording.

## Friction triage

| # | Feedback | Arm | Status |
|---|---|---|---|
| 1 | Default cargo 1.83 vs toolbox-connect edition2024 | A | **Fixed** — scaffold ships `rust-toolchain.toml` |
| 2 | `toolbox connector test` needs a portal; scaffold had none | A | **Fixed** — portal stub in scaffold + `examples/connector-scaffold/` (D01M83FFM2K6XS4EAVHVP92TBDZ) |
| 3 | No one-shot entities export; hand-built stdio JSON | A | **Fixed** — `toolbox connector drive` + `author-connector` skill |
| 4 | `Source::version()` reports connect crate version | A | **Fixed** — `version()` required on every `Source` |
| 5 | Env vs `portal/toolbox.yaml` must stay in sync for tenant switch | A | **Fixed in docs/skill** — call out in RUN.md / author-connector |
| 6 | Ocean pin needs fastapi; unusable offline | B | **Deferred** — document shim / pin deps |
| 7 | `httpx` in requirements but installs discouraged | B | **Deferred** — stdlib-only scaffold |
| 8 | Single `OUT/entities.ndjson` overwrites on tenant flip | B, C | **Deferred** — per-tenant OUT |
| 9 | Dual DeferredEntity vs harness NDJSON | C | **Deferred** — clearer stub |
| 10 | No typescript install in scaffold | C | **Deferred** — strip-types OK; document |
| 11 | Components endpoint is bare array, not paginated | all | **Fixed in docs** — FORGE.md note |
| 12 | Catalog dump has no tenant label (read risk) | all | **Deferred** |

## Artifacts

- Scores, entities, replies: [results/](./results/)
- Patches (phase 1): [patches/](./patches/)
- Usage: [results/usage.tsv](./results/usage.tsv)

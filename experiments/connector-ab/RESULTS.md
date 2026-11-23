# Connector A/B Results

**Status: run in progress — phases 1, 3, 4 scored; phase 5 in flight.**  
Pre-registered bar: [README.md](./README.md) §6 (C1–C3). Decision:
[D01M833YAH8MT4VY2W4K1PYAEHB](../../docs/decisions/D01M833YAH8MT4VY2W4K1PYAEHB.md).

Tokens were not available from the agent runtime transcripts; **tool calls** are the
cost measure used against the bar's "fewer tokens" intent (same limitation as when
usage metadata is missing). Calls are counted from transcripts where available,
else agent self-report.

## Against the bar

| Hypothesis | Measure | A Toolbox | B Ocean | C Backstage | Verdict (pending phase 5) |
|---|---|---|---|---|---|
| **C1** cheaper to author+change | Calls phases 1+3 | 53+12=**65** | 41+22=**63** | 29+17=**46** | A not cheaper than both → **lean fail** |
| **C2** cheaper reuse | Calls phase 4 | TBD | TBD | TBD | pending replies |
| **C3** correctness | Truth+shape 1–4 | pass | pass | pass | **holds** so far |

Glue (lines the team maintains), cumulative after each scored phase:

| After | A | B | C |
|---|---|---|---|
| Phase 1 | 440 | 295 | 427 |
| Phase 3 | 530 | 335 | 465 |
| Phase 4 | 550 | 342 | 483 |

## Per phase

| Phase | A calls | B calls | C calls | All correct? |
|---|---|---|---|---|
| 1 Author | 53 | 41 | 29 | Yes (5 entities) |
| 3 Change | 12 | 22 | 17 | Yes (8 entities) |
| 4 Reuse | — | — | — | Yes (4 entities tenant B); call counts pending |
| 5 Read | — | — | — | in flight |

## Deviations

1. Truncated phase-3 Toolbox launch discarded; relaunched ([run-log.md](./results/run-log.md)).
2. Mid-run harness fix: ship `rust-toolchain.toml` into Toolbox scaffold (edition2024).
3. Ocean arm uses a local `ocean_shim` (fastapi deps missing) — anticipated in README §7.
4. Backstage arm uses Node `--experimental-strip-types` (no typescript install).
5. Arm A phase 1 exceeded the 50-call budget (53).
6. No token counts in transcripts; calls substitute for cost.

## Friction triage (draft)

| # | Feedback | Arm | Status |
|---|---|---|---|
| 1 | Default cargo 1.83 vs toolbox-connect edition2024 | A | **Fixed** in scaffold (`rust-toolchain.toml`) |
| 2 | `toolbox connector test` needs a portal; scaffold had none | A | **Deferred** — add portal stub to scaffold |
| 3 | No one-shot entities CLI; hand-built stdio JSON | A | **Deferred** |
| 4 | `Source::version()` reports connect crate version | A | **Deferred** — SDK bug/docs |
| 5 | Ocean pin unusable without fastapi | B | **Deferred** — document offline shim / allowlist deps |
| 6 | requirements.txt httpx not installed | B | **Deferred** — prefer stdlib in scaffold |
| 7 | Ocean Jira endpoints ≠ Forge | B | Expected (mock); document in FORGE.md |
| 8 | Dual Backstage entity vs NDJSON schemas | C | **Deferred** — clearer scaffold stub |
| 9 | No typescript install in Backstage scaffold | C | **Deferred** — strip-types or preinstall |
| 10 | Components bare-array vs paginated lists | all | **Deferred** — note in FORGE.md |

*Final adjudication after phase 5.*

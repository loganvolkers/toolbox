# Connector Repair A/B: Diagnose and Fix a Near-Correct SaaS Connector

**Question:** when a coding agent inherits a *near-correct* cloud/SaaS connector that fails
conformance and catalog truth, does Toolbox's test/drive/skills feedback loop cost fewer tool
calls and recover more bugs than Port Ocean or a Backstage entity provider, at equal final
correctness?

**Status: designed, then run.** Full write-up: [RESULTS.md](./RESULTS.md). Hypotheses D1–D3 and
the bar in §6 were not edited after the first arm started.

This measures the **package-author** layer's `build-connector` / `test` stages, focused on
**feedback loops** ([dx-benchmarks.md](../../docs/design/dx-benchmarks.md), DevEx). It is about a
**tool** that fits the box. Sibling of [connector-ab](../connector-ab/README.md), which measured
author→change→reuse→read but scored repair only as discarded C3 rematches.

Decision: [D01M83NX8BXH45GQA7WMN4ACEH9](../../docs/decisions/D01M83NX8BXH45GQA7WMN4ACEH9.md).

## 0. Why this benchmark (what connector-ab missed)

| Measured in connector-ab | Not measured (this A/B) |
|---|---|
| Happy-path author / add-kind / tenant switch / read | Cost to go from **red** test/truth to **green** |
| Friction lists (qualitative) | **Diagnosis recall** against a known bug list |
| External rematch after id mistakes | Repair as a first-class phase |
| Call counts once correct | Whether tool/docs alone surface the bugs |

**Still out of scope** (future siblings, not D1–D3): operator adopt into a full portal, cold `cargo`/`npm` install wall time as a primary claim, incremental refresh (not full resync), second-connector transfer learning, glue-line primary scoring.

## 1. SaaS under test

Same **Forge** mock as connector-ab (Jira Cloud-shaped REST, two tenants). Agents may not invent
a different API. Fixtures and expected catalogs are shared from
`../connector-ab/fixtures/` (read by the harness only).

## 2. Arms

| | A: Toolbox | B: Port Ocean | C: Backstage |
|---|---|---|---|
| Given | Broken stdio connector + portal stub; `toolbox connector test` / `drive` on PATH | Broken Ocean-shaped handlers + shim; `resync.py` | Broken `ForgeEntityProvider` + `npm run resync` |
| Seeded bugs | See `bugs/A.md` (harness-private; agents must not read) | `bugs/B.md` | `bugs/C.md` |
| Must produce | Green `connector test` (Toolbox) + `OUT/entities.ndjson` truth | Green resync truth + shape | Same |
| Must not | Read Ocean/Backstage pins; read `bugs/` or `fixtures/**/expected*` | Read Toolbox; read `bugs/` | Read Toolbox/Ocean; read `bugs/` |

**Pinned open-source (read allowlists):** same pins as connector-ab
(`port-labs/ocean@f9f3948e…`, `backstage/backstage@feeadd4d…`).

## 3. Phases

| # | Task | Scored by |
|---|---|---|
| 1 | **Diagnose.** Run the arm's failing check(s). Write `OUT/diagnosis.md` listing each bug (one line each). Do not fix yet. | `harness/score_diagnosis.py` recall vs `bugs/{arm}.md` |
| 2 | **Repair.** Fix bugs. Tenant A catalog must match expected-with-components; framework shape must pass. | `harness/score.py` truth+shape (tenant A, phase 3 shape) |
| 3 | **Reuse.** Point at tenant B without rewriting sync logic. | truth tenant B |
| 4 | **Read.** From arm files only, answer the read key (same questions as connector-ab phase 5). | `score_read.py` |

**Measured:** tool calls (primary), wall time (recorded), diagnosis recall, correctness, friction
feedback (same wording every arm).

## 4. Common entity schema

Identical to [connector-ab README §4](../connector-ab/README.md): `OUT/entities.ndjson` with
`kind`/`id`/`data`. Components use catalog id `{project}:{name}`; Forge numeric id only in
`data.id`; no extra fields.

## 5. Framework shape

Same as connector-ab §5 (stdio Source / `@ocean.on_resync` / EntityProvider).

## 6. The bar, set before running

| Hypothesis | Holds if | Fails if |
|---|---|---|
| **D1. Toolbox cheaper to diagnose+repair** | Over phases 1+2, A fewer calls than B and than C, with equal final correctness on phase 2 | A as many or more than either rival |
| **D2. Toolbox diagnosis finds more bugs** | Phase-1 diagnosis recall (fraction of seeded bugs named) is strictly higher on A than on B and on C | A tied or lower than either |
| **D3. Platforms correct after repair** | Every finishing arm is fully correct on phase 2 truth+shape, phase 3 truth, and phase 4 read | Any finishing arm incorrect after repair |

**Budget:** 50 tool calls per phase. Network only to `127.0.0.1` (Forge). No installs during a
phase. Agents must not read `bugs/`, `fixtures/**/expected*`, or scoring scripts.

**If harness-blocked:** discard, fix harness, relaunch; record in RESULTS.

## 7. Known limits

| Limit | Treatment |
|---|---|
| Seeded bugs are synthetic | In scope: measures feedback on this failure class (composite ids / registration) |
| Ocean/Backstage without full SaaS installs | Same deviations as connector-ab (shim / strip-types) |
| One run per phase | Stated |
| Diagnosis grader is keyword/heuristic | Listed in RESULTS; prefer high-precision matches |

## 8. Layout

```
experiments/connector-repair-ab/
  README.md          this file (bar)
  RESULTS.md         published outcome
  bugs/              seeded bugs (agents forbidden)
  scaffolds/         broken starting trees
  prompts/           per-phase agent prompts
  harness/           setup, score_diagnosis, wrappers onto connector-ab forge/score
  portal/benchmarks  catalog entry
```

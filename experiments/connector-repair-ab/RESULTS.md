# Connector Repair A/B Results

**Status: complete.** Same bar as [README.md](./README.md) §6 (not edited after the first arm
started). Decision
[D01M83NX8BXH45GQA7WMN4ACEH9](../../docs/decisions/D01M83NX8BXH45GQA7WMN4ACEH9.md).

Sibling of [connector-ab](../connector-ab/RESULTS-v3.md): measures diagnose→repair feedback, not
author→change.

## Against the bar (§6)

| Hypothesis | Holds if | Result |
|---|---|---|
| **D1. Toolbox cheaper to diagnose+repair** | A fewer calls than B and C over phases 1+2, equal final correctness | **Fail.** A **31**, B **25**, C **25** |
| **D2. Toolbox diagnosis finds more bugs** | Phase-1 recall strictly higher on A than on B and on C | **Fail.** All arms **3/3** (recall 1.0) |
| **D3. Platforms correct after repair** | Finishing arms fully correct on phase 2 truth+shape, phase 3 truth, phase 4 read | **Hold.** All arms exact; read 5/5 |

## Cost

| | A Toolbox | B Ocean | C Backstage |
|---|---|---|---|
| Calls phase 1 (diagnose) | 15 | **13** | **13** |
| Calls phase 2 (repair) | 16 | **12** | **12** |
| Calls 1+2 (D1) | 31 | **25** | **25** |
| Diagnosis recall (D2) | 3/3 | 3/3 | 3/3 |
| Calls phase 3 (reuse) | **11** | 12 | 13 |
| Calls phase 4 (read) | 15 | 14 | **13** |

## Correctness

| | A | B | C |
|---|---|---|---|
| Phase 2 tenant A truth+shape | exact | exact | exact |
| Phase 3 tenant B truth+shape | exact | exact | exact |
| Phase 4 read | 5/5 | 5/5 | 5/5 |

## Friction triage (agent feedback)

| Friction | Arms | Treatment |
|---|---|---|
| Seeded `BUG:` comments in source made root causes too obvious | B, C (phase 1) | Stripped from scaffolds after phase 1; noted as a run deviation for D2 |
| Assignee null looks valid until you curl Forge `/search` | A | Docs alone insufficient; live compare needed — in scope for repair DX |
| Stale `OUT/diagnosis.md` can mislead during read | C | Process note; NDJSON is source of truth for phase 4 |
| `connector drive --format catalog` has no entity-count summary | A | Product: consider a one-line summary on dump |
| Q5 “config field” singular vs both baseUrl+token | A | Prompt wording; not a product defect |

## Deviations

- Ocean/Backstage still use the connector-ab harness deviations (local shim / strip-types).
- Phase-1 scaffolds briefly contained `# BUG:` / `// BUG:` comments; agents reported them. Comments
  were removed from scaffolds mid-run after phase 1 (repair arms used diagnosis.md).
- One run per phase; call counts are agent-reported integers checked against completion reports.

## Artifacts

`experiments/connector-repair-ab/results/v1/` (per-arm `OUT/` copies + score JSON).

## What this means for the claim

Toolbox did **not** win the pre-registered repair-loop cost or diagnosis-recall bar on this run.
Correctness after repair matched Ocean and Backstage. Authoring wins from connector-ab v3 do not
automatically transfer to diagnose→repair; the next product work, if any, should target error
feedback (e.g. dump summaries, clearer mismatch hints) rather than re-tuning this bar.

---
title: DX Benchmarks
description: How Seikika measures its own developer experience by benchmarking agents doing real work on open-source repositories, against the alternatives, with the losses published.
sidebar:
  order: 0
---

A developer experience (DX) benchmark tests how well Seikika helps someone finish a real task. In each benchmark, fresh coding agents work on open-source repositories. One group uses Seikika and the other uses the strongest available alternative. We set the success criteria before the run and publish the results, including losses.

## What is compared

| Benchmark | Seikika against | Repositories | Result |
|---|---|---|---|
| Out of the box | Hand triage of every finding | OpenFGA, Codex, gh-aw, PostHog | The first run failed on all four. After the fixes, 939 of 940 findings were true and 70% were worth fixing |
| Checklist or report | The same rules as a written checklist | OpenFGA, Codex | The checklist did as well or better. Seikika lost |
| Standards lifecycle | Bespoke scripts | PostHog, Codex, OpenFGA | Equal correctness. Seikika cost 15% more tokens even after the fixes. Seikika lost |
| Ratchet replay | Upstream's own merges | OpenFGA, Codex, gh-aw, PostHog | 141 commits: one real supply-chain regression caught, and one false alarm |
| Several tools, one box | Hand-written CI glue | PostHog | Equal correctness and less glue, but 11% more tokens. Seikika lost on cost |
| Security scorecards | An agent's home-built scorecard, against OpenSSF Scorecard | OpenFGA, Codex | The home builds found more. The standard won on reuse cost and consistency |
| Connector authoring | Port Ocean and a Backstage entity provider | Ocean, Backstage | The first run lost on cost. The reruns changed only Seikika's arm and are within noise, so they are not a win |
| Laptop deploy | Backstage from create-app | Backstage, OpenFGA | Inconclusive: no token counts, and the two arms did not start from equal conditions |

## What it has taught us

In the measured tasks, Seikika often cost more on the first run. Agents already knew the alternatives and had to learn Seikika. Later benchmarks test whether the effort pays off when teams repeat the task, such as adding the same standard to a second repository.

The results led to reusable packages, numeric thresholds for baselines, support for standard tool output such as SARIF, and clearer explanations of regressions and missing data.

## How a benchmark is run

- The question, the alternative, the measures and the bar are written before the run.
- Each phase starts a fresh agent, with the same model, budget, and tools in both groups, and no
  network.
- Each runs on open-source repositories at pinned commits.
- Correctness is judged against an independent truth: a script, or a recorded judgement with its
  evidence.
- Tokens, tool calls, time and lines of glue are measured.
- Both groups answer the same feedback questions. We use their answers to identify fixes.
- The prompts are committed before the first run, and each arm runs at least three times. A
  difference smaller than the spread between runs is reported as inconclusive.
- Each group gets the same level of help. The benchmark uses the same Seikika build throughout.
- The verdict is written by someone who did not design the experiment.

Two early results, a connector "win" and a deploy "defeat", were run before these rules. Both
have been corrected in the repository, and the rules now apply to every run.

The method, the prior art it builds on (SWE-bench, Terminal-Bench, METR's study, the Agentic
Benchmark Checklist, BetterBench, Next.js Evals and others) and the catalog are in the
repository's [DX Benchmarks design](https://github.com/seikika/seikika/blob/main/docs/design/dx-benchmarks.md).
Each benchmark's pre-registration and results are under
[experiments/](https://github.com/seikika/seikika/tree/main/experiments). The catalog in
[portal/benchmarks/](https://github.com/seikika/seikika/tree/main/portal/benchmarks) is scored
like the rest of the repository; see [this repository, scored](/demo/report/).

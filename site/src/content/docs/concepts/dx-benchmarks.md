---
title: DX Benchmarks
description: How Toolbox measures its own developer experience by benchmarking agents doing real work on open-source repositories, against the alternatives, with the losses published.
sidebar:
  order: 0
---

Toolbox tests its developer experience the way an experiment tests a claim. In a DX Benchmark,
fresh coding agents do real lifecycle work on open-source repositories, once with Toolbox and once
with the strongest alternative a team would actually use. The bar is written down before the run.
The results are published as they came out, including when Toolbox loses.

We know of no other developer platform that benchmarks itself this way. This is our survey, not a
verified absence.

## What is compared

| Benchmark | Toolbox against | Repositories | Result |
|---|---|---|---|
| Out of the box | Hand triage of every finding | OpenFGA, Codex, gh-aw, PostHog | The first run failed on all four. After the fixes, 939 of 940 findings were true and 70% were worth fixing |
| Checklist or report | The same rules as a written checklist | OpenFGA, Codex | The checklist did as well or better. Toolbox lost |
| Standards lifecycle | Bespoke scripts | PostHog, Codex, OpenFGA | Equal correctness. Toolbox cost 15% more tokens even after the fixes. Toolbox lost |
| Ratchet replay | Upstream's own merges | OpenFGA, Codex, gh-aw, PostHog | 141 commits: one real supply-chain regression caught, and one false alarm |
| Several tools, one box | Hand-written CI glue | PostHog | Equal correctness and less glue, but 11% more tokens. Toolbox lost on cost |
| Security scorecards | An agent's home-built scorecard, against OpenSSF Scorecard | OpenFGA, Codex | The home builds found more. The standard won on reuse cost and consistency |

## What it has taught us

Toolbox rarely beats a capable agent on a single run. Agents already know programming languages
and their tools, but not Toolbox. Its value shows up with repetition: on a second repository, on
the next upgrade, across many repositories, and over time. The benchmarks now measure exactly
that: what it costs to reuse something, and whether two runs give the same answer.

Every benchmark has changed Toolbox. Among the changes:
- packages with parameters;
- a numeric ratchet;
- a standard slot for SARIF;
- one pin per tool;
- root causes for regressions;
- missing data scored as unknown rather than as failure.

## How a benchmark is run

- The question, the alternative, the measures and the bar are written before the run.
- Each phase starts a fresh agent, with the same model, budget and tools in both arms, and no
  network.
- Each runs on open-source repositories at pinned commits.
- Correctness is judged against an independent truth: a script, or a recorded judgement with its
  evidence.
- Tokens, tool calls, time and lines of glue are measured.
- Both arms are asked for developer-experience feedback the same way, and it is triaged into fixes.

The method, the prior art it builds on (SWE-bench, Terminal-Bench, METR's study, the Agentic
Benchmark Checklist, BetterBench, Next.js Evals and others) and the catalog are in the
repository's [DX Benchmarks design](https://github.com/loganvolkers/toolbox/blob/main/docs/design/dx-benchmarks.md).
Each benchmark's pre-registration and results are under
[experiments/](https://github.com/loganvolkers/toolbox/tree/main/experiments). The catalog in
[portal/benchmarks/](https://github.com/loganvolkers/toolbox/tree/main/portal/benchmarks) is scored
like the rest of the repository; see [this repository, scored](/toolbox/demo/report/).

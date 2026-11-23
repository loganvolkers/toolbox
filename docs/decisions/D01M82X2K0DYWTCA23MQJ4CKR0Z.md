---
id: D01M82X2K0DYWTCA23MQJ4CKR0Z
decided: 2026-11-23T00:30:00Z
---

# DX Benchmarks are a maintainer standard: pre-registered agent A/Bs on open source measure our SDLCs

**Any claim that Toolbox makes a lifecycle easier for agents needs a DX Benchmark behind it. A DX
Benchmark is a pre-registered A/B in which fresh coding agents do real lifecycle work on
open-source repositories at pinned commits, with Toolbox and with its strongest alternative.
Correctness is judged against an independent truth. Cost and friction are measured, and the result
is published against the bar as written, losses included. Each benchmark names the SDLC layers and
stages it measures. Each has an entry in `portal/benchmarks/`, scored by `maintainer-benchmark`
in Toolbox for Toolbox, and is archived once it has served its value.**

1. **Why:** nine runs so far have changed Toolbox more than any review. R3 failed its bar three
   times, R6 on tokens, and R2 only passed after fixes. Each failure produced measured friction,
   and most of it was fixed. Without a bar set beforehand, the losses would have been explained
   away ([dx-benchmarks.md](../design/dx-benchmarks.md) §3, L1).
2. **What they measure:** the developer platform around a fixed agent, not the agent. That is the
   difference from SWE-bench, Terminal-Bench and vendors' framework evals
   ([dx-benchmarks.md](../design/dx-benchmarks.md) §5).
3. **Order of work:** close known friction from pilots before running a benchmark that would hit
   it (L3). Then fix the measured friction and rerun the same arm (L2).
4. **The catalog:** one YAML entry per benchmark, holding:
   - the question, the layers and stages, and the hypotheses;
   - the rival arm and the repositories;
   - the status, the outcome and what it changed;
   - why it was archived.

   Status is `designed`, `blocked`, `complete` or `archived`.

**Status:** ✅

**Where:** [docs/design/dx-benchmarks.md](../design/dx-benchmarks.md),
[portal/benchmarks/](../../portal/benchmarks), [portal/standards/benchmarks.yaml](../../portal/standards/benchmarks.yaml)

---
id: D01M7Y0PQVZQXK8JKPM5584AF3F
decided: 2026-11-21T02:59:26Z
---

# The scale budget is gated on instruction counts per push and timed weekly

**Each push is gated on instruction counts, not wall-clock time.** `scripts/scale-gate.py` runs one full recompute at 10k services under Valgrind's Callgrind, with rayon on two threads. It counts two things: the serial path (`FullRecompute::recompute` on the main thread) and the parallel work (rayon's worker threads). Either one growing more than 2% over `crates/toolbox-derive/examples/scale-gate.json` fails CI. Under `--ci`, an improvement the file doesn't record also fails, as the report ratchet does (D66). Repeat runs agree to within 0.11%.

**The 10 s budget on 4 vCPU is timed weekly** by `.github/workflows/scale-slo.yml` on the runner the `SCALE_RUNNER` repository variable names. With fewer than 4 vCPU it reports the time without judging it. The CI runners have 2 vCPU, so the old per-push wall-clock job tested hardware the budget isn't set for, and its noise ranged from 10.5 s to 15.7 s.

A total instruction count alone isn't enough. Before the serial-glue fix, the total was only 1.5% higher, while the serial count was 62% higher and the wall time 32% longer.

**Status:** ✅

**Where:** [v1-spec.md](../v1-spec.md) §10.4, [research-log.md](../research-log.md) (*Scale gate*)

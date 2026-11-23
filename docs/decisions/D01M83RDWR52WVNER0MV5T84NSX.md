---
id: D01M83RDWR52WVNER0MV5T84NSX
decided: 2026-11-23T08:45:00Z
---

# Correct the record: connector-ab's v1 failed and v3 is overfitted and within noise; deploy-ab is inconclusive

**connector-ab's result is now: v1 failed C1 and C2 and held C3, and v1 is the only round the bar
applies to as written. v2 and v3 are not wins. Between rounds only the Toolbox arm changed. One
change was a product format made to fit the harness. The v3 margin of 1 call is smaller than
the 21-call swing of a rival that did not change. deploy-ab's result is now inconclusive, not a
defeat. The probes gave only Toolbox an authentication bypass. The Toolbox arm restarted a container that
had been built before the run, while the Backstage arm built a 207-line module. No tokens were
recorded, so the bar can't be applied. This supersedes the conclusions in
D01M83FFM2K6XS4EAVHVP92TBDZ, D01M83GQ1KP8A0WR3P2S7AG54WQ and D01M83H3HBH2SC205MX3ANRZYWN. The
product changes those decisions made stay.** The adjudications behind this were written by an
agent that designed neither experiment (D01M83RDWQ5EVKY5SFHDWAD5HFH, P7):
[connector-ab](../../experiments/connector-ab/ADJUDICATION.md) and
[deploy-ab](../../experiments/deploy-ab/ADJUDICATION.md).

1. **connector-ab.** Calls for phases 1+3, Toolbox / Ocean / Backstage:

   | Round | Toolbox | Ocean | Backstage |
   |---|---|---|---|
   | v1 | 65 | 63 | 46 |
   | v2 | 59 | 63 | 61 |
   | v3 | 39 | 57 | 40 |

   - **Only arm A changed.** 3f9641b changed only A's prompts (1-A, 3-A, 4-A and 5-A). It gave A
     a wider read allowlist, a hint to use `toolbox connector drive`, and a portal stub. It
     rewrote A's `RUN.md` from "fill this in" into a full recipe. c1f5feb, titled "catalog
     format to widen connector-ab lead", added `--format catalog`. That writes "harness-shaped"
     NDJSON, and only A's recipe names it. Ocean's and Backstage's `RUN.md` still read "How to
     run (fill this in)". Their friction items 6–10 and 12 stayed deferred. The composite-id
     note in d1e0aa9 went to every arm through the shared `FORGE.md`.
   - **The noise is larger than the margin.** Backstage's own prompts and scaffold did not
     change across the three rounds; the shared `FORGE.md` gained notes that every arm got. Even
     so, its total went 46, 61, 40, and its phase 1, with no change at all, went 29, 38, 29. The v3
     margin over Backstage is 1.
   - **Results replaced after the fact.** In v2, phase 4 tied on its first pass (A 15, C 15),
     which fails C2. It was rerun after an A-only tip, and the rerun was published as the result.
   - **The measure was swapped.** The bar is written in tokens. The runtime reported none, and
     calls were used after the run.

   So C1 and C2 **failed in v1**, which is the record. v2 and v3 are **overfitted and within
   noise**: neither a win nor a loss, and neither is evidence for C1 or C2. C3 held in every
   round. One more correction: D01M83H3HBH2SC205MX3ANRZYWN says the v3 phase-3 first pass failed for
   "Toolbox and Backstage". The discarded scores show it failed for Toolbox and **Ocean**, while
   Backstage passed.
2. **deploy-ab.** The bar holds if both arms pass S1–S3 and Toolbox uses fewer tokens. It fails if
   Toolbox misses a shared probe, or passes them on at least as many tokens. Neither happened, and
   no tokens exist, so the outcome is **inconclusive**. Four other problems would have stopped a
   win or a loss from counting anyway:
   - the S2 probe sends `x-toolbox-user` and `?as=` to Toolbox, while Backstage meets its guest
     sign-in page;
   - both Toolbox agents report that the `toolbox-acme:local` image and the `acme-portal-1`
     container "were already correct" and only needed restarting. The deploy was done before the
     run;
   - the facade has no GraphQL, so the "GitHub catalog provider" the Backstage prompt suggests
     can't work against it;
   - `RESULTS-v2.md` says an earlier Backstage launch in the rerun "produced no messages and wrote no
     files". Its record holds 80 tool calls in the same workspace and on the same port, overlapping
     the scored launch.
3. **What stays.** Every product change the two benchmarks led to stays, with one review:
   `connector drive --format catalog` passes P5 only if a use outside the harness is named.
   [dx-benchmarks.md](../design/dx-benchmarks.md) §7 lists that review as B8. The friction
   lists stay as found. They are what protocol-1 runs are good for.
4. **Reruns.** Either benchmark can be rerun under protocol 2. For connector-ab, every arm gets a
   recipe and its deferred friction fixed first. For deploy-ab, the probes are symmetric, the
   portal is not prebuilt, and the bar is rewritten in a measure that a dry run shows the runtime
   reports.

**Status:** ✅

**Where:** [experiments/connector-ab/ADJUDICATION.md](../../experiments/connector-ab/ADJUDICATION.md),
[experiments/deploy-ab/ADJUDICATION.md](../../experiments/deploy-ab/ADJUDICATION.md),
[portal/benchmarks/connector-ab.yaml](../../portal/benchmarks/connector-ab.yaml),
[portal/benchmarks/deploy-ab.yaml](../../portal/benchmarks/deploy-ab.yaml)

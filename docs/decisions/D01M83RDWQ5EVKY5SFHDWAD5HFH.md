---
id: D01M83RDWQ5EVKY5SFHDWAD5HFH
decided: 2026-11-23T08:40:00Z
---

# DX Benchmark protocol 2: committed prompts, repeated runs, frozen and symmetric rounds, independent adjudication

**A DX Benchmark that runs agents runs under protocol 2. Every prompt is committed before the first
arm starts. Each arm runs at least three times per phase, and the result reports the spread.
Nothing any arm can read changes during a round, and friction is fixed in every arm with the same
care. A product change made for a benchmark has to stand without it. The bar is written in a
measure the runtime has been seen to report. Someone who did not design the experiment or change
the product applies the bar, using four labels: holds, fails, inconclusive, void. Protocol-1 results stay on the record as
single samples: evidence of friction, not of a claim.** This amends the method of
D01M82X2K0DYWTCA23MQJ4CKR0Z. The standard and the catalog stay as they are.

1. **Why.** Two protocol-1 benchmarks reached conclusions their data cannot carry
   (D01M83RDWR52WVNER0MV5T84NSX):
   - connector-ab declared a Toolbox win in its third round. Only Toolbox's arm had changed
     between rounds: its prompts, its scaffold's recipe, and a product format the same pull
     request called "to widen connector-ab lead". The win was 1 call against a rival whose total
     moved by 21 calls across rounds in which that rival's prompts and scaffold had not changed.
   - deploy-ab declared a defeat. The probes gave only Toolbox an authentication bypass, and its
     agent restarted a container that had been built before the run. No tokens were recorded, so
     the bar could not be applied either way.

   L10 in [dx-benchmarks.md](../design/dx-benchmarks.md) already said one run is one sample. This
   decision makes that, and the rules below, part of the protocol, and makes the scorecard check
   it.
2. **The rules** ([dx-benchmarks.md](../design/dx-benchmarks.md) §4.1, P1–P7):

   | Rule | What it requires |
   |---|---|
   | P1. Committed prompts | Every rendered prompt, for every arm and phase, sits under the experiment's `prompts/` before the first arm starts. A launch whose prompt differs from the committed one is void |
   | P2. Repeated runs | At least three fresh runs per arm per phase. Report every run and the minimum, median and maximum. A cost hypothesis holds or fails only if the gap between medians is larger than the wider arm's range; otherwise it is inconclusive. Runs are not dropped for their result, only for a harness fault named before rerunning |
   | P3. Frozen rounds | A round fixes one commit (`frozenAt`) for Toolbox, the harness, and every arm's scaffold, prompt and docs. A fix between rounds starts a new round, in which every arm reruns every scored phase |
   | P4. Symmetric arms | Every arm's scaffold gives the same level of help. If one arm gets a recipe, all do. Rival friction is fixed before the next round, as carefully as Toolbox's. Deferring it means the next round is not run |
   | P5. No harness-shaped product | A product change made after a benchmark names a use outside the harness and no harness file, schema or convention. The harness adapts to the product, not the reverse. The adjudicator lists every change between rounds and says whether it passes (`fitAudit`) |
   | P6. A reported measure | The bar is written in a measure (`measure`) that a dry run has shown the runtime reports for every arm. It is never swapped after the run. If a measure goes missing, its cost hypotheses are void |
   | P7. Independent adjudication | The adjudicator applies the bar and writes the outcome. They did not design the experiment, build an arm, or change the product between rounds (`designer`, `adjudicator`). The outcome uses only holds, fails, inconclusive or void, per hypothesis |

   Benchmarks with no agent in the scored path, like ratchet-replay, set `deterministic: true`
   and are exempt from P2.
3. **The catalog and the scorecard.** Entries gain these fields: `protocol`, `prompts`, `runsPerArm`,
   `deterministic`, `measure`, `frozenAt`, `designer`, `adjudicator`, `fitAudit` and `spread`.
   `maintainer-benchmark` adds two rules:
   - `method-registered` (silver): a benchmark that has not run yet declares the protocol-2
     registration;
   - `sound-method` (gold): a run benchmark records it.

   An entry without `protocol` counts as protocol 1. The run protocol-1 benchmarks therefore drop
   from gold to silver, and the blocked one drops to bronze until it is re-registered. Those drops
   are the record, not a regression.
4. **What it does not change.** Pre-registration, open-source pins, the strongest rival, and
   publishing losses (D01M82X2K0DYWTCA23MQJ4CKR0Z) stay. Agent efficiency remains a goal the
   benchmarks measure. This decision is about measuring it so that the number can be trusted.

**Status:** ✅

**Where:** [docs/design/dx-benchmarks.md](../design/dx-benchmarks.md) §4.1,
[portal/standards/benchmarks.yaml](../../portal/standards/benchmarks.yaml),
[portal/benchmarks/](../../portal/benchmarks)

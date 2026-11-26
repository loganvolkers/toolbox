# connector-ab: independent adjudication (2026-11-23)

**Outcome against the bar in [README.md](./README.md) §6: v1 failed C1 and C2 and held C3. v2 and v3
are overfitted and within noise. They are not wins, and they are not evidence for C1 or C2. C3 held
in every round.** This supersedes the "Seikika wins" headers of [RESULTS-v2.md](./RESULTS-v2.md) and
[RESULTS-v3.md](./RESULTS-v3.md) and the catalog's earlier outcome. The run artifacts are unchanged.

The adjudicator is a cloud agent that did not design the experiment, build an arm, or make the
product changes between rounds
([D01M83RDWQ5EVKY5SFHDWAD5HFH](../../docs/decisions/D01M83RDWQ5EVKY5SFHDWAD5HFH.md), P7). The
decision that records this outcome is
[D01M83RDWR52WVNER0MV5T84NSX](../../docs/decisions/D01M83RDWR52WVNER0MV5T84NSX.md).

## The numbers

Tool calls, from `results/usage.tsv`, `results/v2/usage.tsv` and `results/v3/usage.tsv`. A is
Seikika, B is Ocean, C is Backstage.

| Round | Phase 1 A / B / C | Phase 3 A / B / C | C1 (1+3) A / B / C | C2 (phase 4) A / B / C |
|---|---|---|---|---|
| v1 | 53 / 41 / 29 | 12 / 22 / 17 | **65** / 63 / **46** | 15 / 15 / 16 |
| v2 | 36 / 37 / 38 | 23 / 26 / 23 | 59 / 63 / 61 | first pass 15 / 17 / 15; rerun 12 / 18 / 14 |
| v3 | 22 / 43 / 29 | rerun 17 / 14 / 11 | 39 / 57 / 40 | 9 / 12 / 16 |

## Findings

| # | Finding | Evidence | Protocol 2 rule |
|---|---|---|---|
| 1 | **One run per arm per phase, and the noise is larger than the margin.** Backstage's phase-1 prompt and scaffold never changed, yet its phase-1 calls went 29, 38, 29. Its C1 total went 46, 61, 40, a 21-call range. The v3 margin over Backstage on C1 is 1 call | `usage.tsv` in each round; `git log -- experiments/connector-ab/prompts experiments/connector-ab/scaffolds` | P2 |
| 2 | **Only arm A changed between rounds.** 3f9641b edited prompts 1-A, 3-A, 4-A and 5-A. It added "Prefer `seikika connector drive` … a portal stub is already under portal/" and rewrote `scaffolds/seikika/RUN.md` from "How to run (fill this in)" into a step-by-step recipe with a ready `portal/seikika.yaml`. Ocean's and Backstage's prompts were not touched, and their `RUN.md` files still read "How to run (fill this in)" | `git show 3f9641b -- experiments/connector-ab/prompts experiments/connector-ab/scaffolds`; `scaffolds/ocean/RUN.md`, `scaffolds/backstage/RUN.md` | P3, P4 |
| 3 | **Rival friction was deferred.** In [RESULTS.md](./RESULTS.md), A's items 1–5 were all fixed before v2. Items 6–10 belong to Ocean and Backstage, and every one was deferred. Item 12 affects all arms and was also deferred. Item 9 is Backstage's version of A's mapping friction ("Dual DeferredEntity vs harness NDJSON"). A's version of it got a product feature (finding 4), and Backstage's was deferred | RESULTS.md friction table | P4 |
| 4 | **A product format was shaped to the harness.** c1f5feb, titled "connector drive: catalog format to widen connector-ab lead", added `connector drive --format catalog --out`. A's recipe then said it "writes harness-shaped `{kind,id,data}` NDJSON". The step it replaced read "Map those lines into `OUT/entities.ndjson` for the harness schema". Only A's recipe names the flag. Phase 1 for A dropped from 36 to 22 | `git show c1f5feb`; `scaffolds/seikika/RUN.md`; [D01M83GQ1KP8A0WR3P2S7AG54WQ](../../docs/decisions/D01M83GQ1KP8A0WR3P2S7AG54WQ.md) | P5 |
| 5 | **A failing result was replaced.** In v2, phase 4's first pass tied (A 15, C 15). By the bar, that fails C2, and the commit at the time said so ("C2 fails (A15 tied with C)"). An A-only tip followed: a dump-only tenant switch, added to the skill and to A's `RUN.md`. Phase 4 was then rerun, and the rerun was published as "C2 holds" | 3f9641b commit messages; RESULTS-v2.md "Rematch note" | P2, P3 |
| 6 | **The measure was swapped after the bar was set.** The bar is written in tokens. The runtime reported none, so calls were used | README.md §6; RESULTS.md deviation 6 | P6 |
| 7 | **Symmetric, and correctly so.** d1e0aa9's composite-id note went into the shared `FORGE.md` that every arm reads, through `harness/setup.sh`. All three arms then reran phase 3 | `git show d1e0aa9 -- experiments/connector-ab/harness/setup.sh` | — |
| 8 | **A misattribution in the record.** [D01M83H3HBH2SC205MX3ANRZYWN](../../docs/decisions/D01M83H3HBH2SC205MX3ANRZYWN.md) says the v3 phase-3 first pass failed for "Seikika and Backstage". The discarded scores show Seikika and **Ocean** failed, and Backstage passed | `results/v3/3-A-score-discarded.json`, `3-B-score-discarded.json`, `3-C-score.json` | — |

## Per hypothesis

| Hypothesis | v1 (the run the bar was written for) | v2 and v3 |
|---|---|---|
| C1. Seikika cheaper to author and change | **Fails.** A 65 ≥ C 46 | **Inconclusive.** Only A changed, and the margin is inside one arm's noise (findings 1–4) |
| C2. Seikika reuses cheaper | **Fails.** A 15 ≥ B 15 | **Inconclusive.** The v2 tie fails, and the rerun replaced it (finding 5). In v3 the margin is 3 calls, from one run each |
| C3. Platforms correct within budget | **Holds** | **Holds** |

Since no tokens were recorded, the bar as written in tokens could not be applied to any round
(finding 6). The verdicts above use calls as the stand-in, and say so. v1 was also one run per arm,
and A got a harness fix during phase 1 (`results/run-log.md`). So v1's failure is the record, but it
is one sample, not a settled loss.

## What the benchmark is still worth

The friction it found is real and was fixed: `rust-toolchain.toml` in the scaffold, `connector drive`,
`Source::version()`, the connector scaffold and its skills. That is what a protocol-1 run is good
for. The fixes stay. `connector drive --format catalog` stays only if it names a use outside this
harness ([dx-benchmarks.md](../../docs/design/dx-benchmarks.md) §7, B8).

## Before a protocol-2 rerun

1. Write every arm's `RUN.md` to the same level, either all recipes or all "fill this in".
2. Fix or remove Ocean's and Backstage's deferred friction (items 6–10 and 12).
3. Commit the rendered prompts and freeze one commit for the round.
4. Write the bar in calls, or in tokens only after a dry run shows the runtime reports them.
5. Do at least three runs per arm per phase, and have someone other than the designer adjudicate.

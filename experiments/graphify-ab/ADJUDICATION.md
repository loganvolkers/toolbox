# graphify-ab round 1: adjudication

**Verdict: Seikika for Seikika does not adopt Graphify.** Under the rules in [README.md](README.md)
§5, no hypothesis on coverage, read or calls held or failed. All six (K1–K6) are inconclusive, and
K7 holds. That makes "Graphify helps" and "Graphify hurts" both unshown on both repositories. The
adoption rule's "Otherwise" branch applies: S4S doesn't adopt Graphify, and the context lake stays
catalog-level, federating code graphs rather than building one. This matches the designer's
pre-registered prediction on every point.

## 1. Who adjudicated

- Two fresh agents graded coverage, one per repository. Neither designed any part of the
  experiment.
- Each grader saw only the gold file for its repository and the ten answers from
  `harness/blind.py`. The answers were shuffled under random three-digit labels, with "graphify"
  redacted.
- Each grader wrote its grade table before seeing the label key or the arm map:
  `results/r1/grades-r1-openfga.md` and `results/r1/grades-r1-seikika.md`.
- The OpenFGA grader then unblinded, read the measures, and applied §4–§6 to write this file.
- The gold files match the hashes pre-registered in §4:
  - `gold/openfga.md`: `baeea1dc…a189ea1`
  - `gold/seikika.md`: `fd158002…d2a3e8`

## 2. Contamination and harness faults

| Check | Result |
|---|---|
| `outsidePaths` (from `count.py`, `results/r1/<run>.json`) | Empty for all 20 runs; `outside` = 0 in `results/r1.tsv` |
| Void runs | None |
| Runs ending without `ANSWERS.md` | None; `answered` = true for all 20 |
| Harness faults, reruns | None |

Some tool results were missing from transcripts (column `missing` in `results/r1.tsv`). For a
`read_file`, the missing result is imputed as the file, as §5 defines. That is a measurement rule,
not a harness fault.

## 3. Per run

Coverage = (covered + ½ partial) / key facts.

- **OpenFGA:** every run covered 21 of 22 facts, so coverage is 0.955. All ten missed the same
  fact, O4d (validation re-run when a model is loaded).
- **Seikika:** every run covered 21 of 21, so coverage is 1.000.

"Read" is characters of tool results, including imputed ones.

| Run | Arm | Label | Coverage | Calls | Read | graphifyCalls | Wall (s) |
|---|---|---|---|---|---|---|---|
| openfga-2 | A | 447 | 0.955 | 52 | 319,746 | 0 | 309 |
| openfga-7 | A | 686 | 0.955 | 62 | 348,825 | 0 | 259 |
| openfga-8 | A | 115 | 0.955 | 63 | 295,181 | 0 | 283 |
| openfga-9 | A | 769 | 0.955 | 60 | 261,220 | 0 | 274 |
| openfga-10 | A | 771 | 0.955 | 73 | 411,348 | 0 | 230 |
| openfga-1 | B | 351 | 0.955 | 45 | 230,665 | 8 | 196 |
| openfga-3 | B | 581 | 0.955 | 44 | 804,822 | 5 | 176 |
| openfga-4 | B | 996 | 0.955 | 79 | 465,815 | 5 | 253 |
| openfga-5 | B | 315 | 0.955 | 40 | 325,726 | 4 | 165 |
| openfga-6 | B | 667 | 0.955 | 40 | 643,413 | 3 | 185 |
| seikika-3 | A | 784 | 1.000 | 75 | 400,058 | 0 | 195 |
| seikika-4 | A | 642 | 1.000 | 71 | 327,662 | 0 | 221 |
| seikika-5 | A | 216 | 1.000 | 74 | 343,316 | 0 | 199 |
| seikika-7 | A | 401 | 1.000 | 73 | 330,836 | 0 | 235 |
| seikika-10 | A | 833 | 1.000 | 70 | 395,189 | 0 | 201 |
| seikika-1 | B | 342 | 1.000 | 50 | 244,970 | 6 | 231 |
| seikika-2 | B | 822 | 1.000 | 55 | 240,401 | 5 | 191 |
| seikika-6 | B | 398 | 1.000 | 67 | 307,641 | 7 | 189 |
| seikika-8 | B | 600 | 1.000 | 42 | 199,070 | 5 | 147 |
| seikika-9 | B | 359 | 1.000 | 69 | 321,355 | 5 | 191 |

## 4. Medians and ranges (five runs per arm per repository)

| Repository | Measure | Arm | Median | Min | Max | Range |
|---|---|---|---|---|---|---|
| OpenFGA | coverage | A | 0.955 | 0.955 | 0.955 | 0 |
| OpenFGA | coverage | B | 0.955 | 0.955 | 0.955 | 0 |
| OpenFGA | calls | A | 62 | 52 | 73 | 21 |
| OpenFGA | calls | B | 44 | 40 | 79 | 39 |
| OpenFGA | read | A | 319,746 | 261,220 | 411,348 | 150,128 |
| OpenFGA | read | B | 465,815 | 230,665 | 804,822 | 574,157 |
| Seikika | coverage | A | 1.000 | 1.000 | 1.000 | 0 |
| Seikika | coverage | B | 1.000 | 1.000 | 1.000 | 0 |
| Seikika | calls | A | 73 | 70 | 75 | 5 |
| Seikika | calls | B | 55 | 42 | 69 | 27 |
| Seikika | read | A | 343,316 | 327,662 | 400,058 | 72,396 |
| Seikika | read | B | 244,970 | 199,070 | 321,355 | 122,285 |

## 5. K1–K7

*W* is the wider of the two arms' ranges. A hypothesis holds or fails only if the medians differ
by more than *W* in the stated direction; otherwise it is inconclusive.

| Id | Repository, measure | med(A) | med(B) | *W* | Difference | Verdict |
|---|---|---|---|---|---|---|
| K1 | OpenFGA, coverage | 0.955 | 0.955 | max(0, 0) = 0 | med(B) − med(A) = 0; 0 > 0 is false both ways | **Inconclusive** |
| K2 | OpenFGA, read | 319,746 | 465,815 | max(150,128, 574,157) = 574,157 | med(A) − med(B) = −146,069; \|146,069\| ≤ 574,157 | **Inconclusive** |
| K3 | OpenFGA, calls | 62 | 44 | max(21, 39) = 39 | med(A) − med(B) = 18; 18 ≤ 39 | **Inconclusive** |
| K4 | Seikika, coverage | 1.000 | 1.000 | max(0, 0) = 0 | med(B) − med(A) = 0; 0 > 0 is false both ways | **Inconclusive** |
| K5 | Seikika, read | 343,316 | 244,970 | max(72,396, 122,285) = 122,285 | med(A) − med(B) = 98,346; 98,346 ≤ 122,285 | **Inconclusive** |
| K6 | Seikika, calls | 73 | 55 | max(5, 27) = 27 | med(A) − med(B) = 18; 18 ≤ 27 | **Inconclusive** |
| K7 | Both, graphifyCalls in B runs | — | — | — | OpenFGA B: 8, 5, 5, 4, 3. Seikika B: 6, 5, 7, 5, 5. Minimum 3 ≥ 1 | **Holds** |

The medians point both ways, but none of these differences counts under the rule:

- **K2:** B's median read is higher than A's.
- **K3, K5 and K6:** B's median is lower than A's.

In every case the difference is within *W*, so none of these directions is a result.

## 6. Verdicts under §5

| Repository | Any of its K fails? | Coverage holds? | A cost hypothesis holds? | Helps | Hurts | Verdict |
|---|---|---|---|---|---|---|
| OpenFGA | No (K1–K3 inconclusive) | No (K1) | No (K2, K3) | No | No: K1 didn't fail and K2, K3 didn't both fail | **Neither shown** |
| Seikika | No (K4–K6 inconclusive) | No (K4) | No (K5, K6) | No | No: K4 didn't fail and K5, K6 didn't both fail | **Neither shown** |

K7 holds, so the "not shown regardless" override for a B run without a `graphify` call does not
apply.

**Adoption.** The first branch of the §1 rule needs Graphify to help on Seikika and not hurt on
OpenFGA. It is not met, because helping on Seikika is not shown. The **"Otherwise" branch**
applies:

- S4S doesn't adopt Graphify.
- The context lake stays catalog-level and federates code graphs rather than building one.
- This verdict is recorded with this result.

## 7. Against the designer's prediction (§5)

| Prediction | Outcome | Match |
|---|---|---|
| Coverage (K1, K4) is inconclusive | K1, K4 inconclusive | Yes |
| Read (K2, K5) is inconclusive or fails | K2, K5 inconclusive | Yes |
| Calls (K3, K6) are inconclusive | K3, K6 inconclusive | Yes |
| K7 holds | Holds; every B run made 3 to 8 `graphify` calls | Yes |

## 8. Notes

**The coverage ceiling.** Within each repository, all ten answers scored the same coverage:

- **OpenFGA:** 0.955 in both arms. The one miss, O4d, was missed by every run in both arms.
- **Seikika:** 1.000 in both arms.

Both arms' coverage ranges are therefore 0, so *W* = 0, and the difference between medians is also
0. K1 and K4 are inconclusive because the scores tie, not because noise hides a difference. At this
set of key facts, coverage did not separate the arms or the runs, so this round can't show a
coverage gain or loss for Graphify in either direction.

**Wall time** is reported but is not in the bar, so no verdict is drawn from it.

| Repository | Arm | Median (s) | Min | Max | Range |
|---|---|---|---|---|---|
| OpenFGA | A | 274 | 230 | 309 | 79 |
| OpenFGA | B | 185 | 165 | 253 | 88 |
| Seikika | A | 201 | 195 | 235 | 40 |
| Seikika | B | 191 | 147 | 231 | 84 |

**Counting `graphify` use.** `graphifyCalls` counts shell calls that ran `graphify`, and it is the
number K7 uses. `graphifyCommands` in the per-run JSON counts subcommands. One shell call can hold
several subcommands, so the two can differ. For example, openfga-1 made 8 calls holding 7 `query`
and 2 `explain`.

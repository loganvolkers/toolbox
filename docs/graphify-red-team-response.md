# Response to the Graphify red team

*About:* the box. This triages [graphify-red-team.md](./graphify-red-team.md), an adversarial
review written by a fresh session that had none of the designer's context, finding by finding.
It is the designer's response, so it is not independent; every verdict below cites the evidence
that a reader can re-check. Research date 2026-11-24.

## Bottom line

**The red team is right about the claim and wrong about nothing material.** The benchmarks support
a narrow result: for a strong agent with no turn cap, Graphify didn't make answers more complete,
and its headline token ratios don't hold. They don't support "Graphify adds no measurable value".
On Seikika, every Graphify run made fewer calls and read less than every baseline run, and the
house rule labelled that inconclusive because, at five runs per arm, it detects a
two-standard-deviation effect about a quarter of the time. The impact command that matched our
own questions was never put in front of an agent.

What changes:

- **D01M85G2YXV2VRTZWQJ33MHXCKN** amends protocol 2's P2 rule for future rounds: cost hypotheses
  are decided by an exact rank test with a multiplicity correction, need at least five runs per
  arm, and register the smallest effect they can detect. Past labels stand as registered.
- **D01M85G2ZXXPQRXWKQF7TWV2RTP** replaces the reasons and reopen triggers of
  D01M84ZW6K2V9CV2SD644DE5M0S. The outcome holds for now (Seikika for Seikika doesn't carry
  Graphify, and the context lake builds no code graph), but it is restated as "a cost gain on
  Seikika is likely and unmeasured in billed tokens", and a cost trigger reopens it.
- [knowledge-graphs.md](./design/knowledge-graphs.md) drops the overreaching sentences and adds
  CAFE(S) and the independent studies the survey had left out; lesson L23 goes into
  [dx-benchmarks.md](./design/dx-benchmarks.md); [Q21](./questions.md) asks whether the runtime can
  report billed tokens.

## 1. What I re-checked

| Report claim | Check | Result |
|---|---|---|
| Seikika calls and read are completely separated; exact two-sided Mann-Whitney p = 0.0079 each | Recomputed from `experiments/graphify-ab/results/r1.tsv` by full permutation | **Confirmed.** Calls A 70–75, B 42–69; read A 327,662–400,058, B 199,070–321,355 |
| OpenFGA wall time p = 0.016; Codex calls 0.39, read 0.22 | Same script on both `r1.tsv` files | **Confirmed** |
| The *W* rule detects a 2 SD effect ~23% of the time at five runs per arm | Independent simulation, 20,000 trials per effect size, normal data | **Confirmed:** 0.4%, 4%, 23% and 58% at 0, 1, 2 and 3 SD |
| No Graphify run used `affected` | `graphifyCommands` in all 35 Graphify run records | **Confirmed:** `query` 15, `explain` 12, `path` 2, `affected` 0 |
| The rule is the vendor's and omits `affected` | `experiments/graphify-ab/prompts/graphify.mdc` lines 8–11 | **Confirmed.** It names `query`, `path` and `explain` |
| The monorepo README said the questions suit `affected` | [graphify-monorepo-ab README](../experiments/graphify-monorepo-ab/README.md) line 92 | **Confirmed** |
| O4d lies outside Q4's wording | Q4 asks where constraints "are enforced when a model is written"; O4d is "when a model is loaded" | **Confirmed.** All ten runs missed it, so OpenFGA's tie is unchanged |
| The model is never named | Both READMEs say "the session's model"; the run metadata leaves the model empty | **Confirmed.** Every run was a subagent of the designer's session, which ran Claude Opus 5.5; the runtime didn't record it per run |
| 973 Markdown file changes over the 200 commits before `a8f7e03`, in 141 commits | `git show --name-only` over `git log -200 a8f7e03` | **Confirmed** |
| CAFE(S) "provides a definition of context quality, not a measurement system" | DX's newsletter introducing the paper | **Confirmed** in that wording. The paper's other quotes (accessibility out of scope, Meta's ~40%) are from the ACM Queue text, which refused scripted access (HTTP 403); they are the red team's reading, not re-checked |
| The independent studies exist | hatenabase, Machine Room, menges.dev, Srivastava, LanceTek, kubeblogs, pypistats | All returned HTTP 200. Their figures are the red team's reading |

## 2. Findings on CAFE, adopters and doc claims

| Section | Verdict | Why |
|---|---|---|
| §1 CAFE(S) | **Accept** | It is a definition of context quality, not a measurement system. The benchmarks measure part of Efficiency and an outcome proxy. Fidelity (stale or wrong graph output), Security and in-situ cost were unmeasured. CAFE(S) cuts both ways, as the report says: Fidelity and in-situ cost argue against an always-on, often-stale graph, and the Meta result it cites predicts fewer calls, which is what Seikika's arm showed |
| §2 adopters | **Accept** | Traction is real usage, not only stars (2.15M downloads a month; ~15,500 committed `GRAPH_REPORT.md` files, approximate). The logo wall is vendor-authored and names employers of users. No independent study shows a strong agent doing better on average; the consistent narrow wins are named-symbol impact questions, opening fewer files, and human use at the terminal. Buzz is explained by a free, local, visual one-command artifact and a 71.5x headline, not by measured agent gains |
| §3 doc claims | **Accept** | The 71.5x and "scales with corpus size" claims are contradicted in magnitude by our data and every independent study. "Query instead of grepping" is partly contradicted: agents queried, then grepped. Reusable context across sessions, incremental `--update`, hooks and `affected` were untested, and the report is right to call that out |

## 3. Methodology findings

| Id | Verdict | Evidence | Action |
|---|---|---|---|
| R1 | **Accept** | §1 above; the post-hoc reading in §4 | D01M85G2YXV2VRTZWQJ33MHXCKN amends P2 for future rounds; L23 |
| R2 | **Accept** | The decision's "didn't clear the noise" (line 29) and "showed neither" (line 19); the survey's "inside the noise" and "No independent study shows a code graph helping" | D01M85G2ZXXPQRXWKQF7TWV2RTP restates the result; the survey is corrected |
| R3 | **Accept** | Line 92 of the monorepo README names `affected` as fit for the questions; the rule didn't. Seikika's CLI gap from the same round was fixed (`--relation`, `--direction`), Graphify's existing equivalent was never surfaced. That breaks P4's symmetry in spirit. Hooks are partly out of scope: the harness is Cursor, where Graphify installs the rule, not hooks | A next round names `affected` in arm B's rule as one declared line, and `graph --relation` in arm C's |
| R4 | **Accept in part** | Characters of tool output are a proxy, and each call re-sends context, so fewer calls save more than the proxy shows. The 15k tokens a call is the red team's assumption, so "23 sessions, not 43" is an estimate. P6 forbids a bar in a measure the runtime doesn't report, and this runtime reports no billed tokens per subagent | [Q21](./questions.md); the decision names billed tokens as the measure that reopens it |
| R5 | **Accept** | Graphify's per-file cache and `graphify update` (code, no LLM) are in its CLI help. The "43 sessions" is the first build's cost, not a per-commit cost; per-commit document re-extraction is unmeasured | The new decision drops the full-rebuild framing; a next round measures `--update` over real commits |
| R6 | **Accept as a scope limit** | All tasks were single-shot questions for one strong model with no cap. Turning the score into a guard before freezing followed L21 and was declared, but it made the monorepo benchmark unable to show "helps" on coverage | The new decision scopes its claim to that setting |
| R7 | **Accept** | Trigger 2 needed coverage headroom that L22 says harder questions don't buy, and no cost result could reopen the decision | The new decision adds a cost trigger with a registered test |
| R8 | **Accept in part** | The model is now named (Claude Opus 5.5, from the designer's session). Memorization of OpenFGA and Codex is plausible and unmeasured; the only post-cutoff repository, Seikika, is where B separated. That is one repository, so it stays a hypothesis | Run records name the model from the next round; a next round runs on Seikika, which no model has seen |
| R9 | **Accept in part** | The designer works on a rival product and wrote the questions and gold. P7's independent adjudicator and the gold hashes limit, but don't remove, that bias. O4d is a real gold error that hurt both arms equally. Questions naming exact identifiers favour `rg`, which is the realistic case for a maintainer, so that part stays | Gold errata noted here; a next round invites outside questions |
| R10 | **Accept, low** | `count.py` credits whole shell calls; the red team's sensitivity check finds no verdict change on OpenFGA | Per-command splitting in a next round's counter |
| R11 | **Accept, low** | `blind.py` replaces only the word "graphify". Grading was against gold facts and coverage tied, so leakage couldn't move a label | A next round redacts each arm's vocabulary |
| R12 | **Accept, low** | OpenFGA wall time 185 s against 274 s, exact p = 0.016, post hoc. Runs were concurrent subagents, so contention adds noise | Wall time is reported; whether it enters a bar is a per-benchmark choice |
| R13 | **Accept the point, reject the fix** | The changes were declared before freezing. Dry runs on arm A only would break P6, which needs a dry run to show every arm's measure is reported | Future READMEs list which changes followed a dry run of a tool arm |
| R14 | **Accept, low** | One version (0.9.118) and one harness | Both catalog entries pin the version already; a reopen reruns on the then-current one |
| R15 | **Accept** | "Answered by `rg` in a call or two" overreaches: the monorepo's questions took a median of 56 calls for four. The survey omitted CAFE(S) and six independent studies, most of which support the token conclusion | The survey is corrected and extended; the new decision drops the sentence |

### 3.1 Arm configuration: code-only against the LLM-semantic build

The red team raises this only in its doc-claims table (§3, "Beyond code": "no ablation separates
it from the code layer"). It deserves a finding of its own, and I **accept** it. The build differed
by repository, and so did the result:

| Repository | Graphify build | Documents in the graph | Cost result (post hoc) |
|---|---|---|---|
| Seikika `a8f7e03` | Code plus LLM document layer, 17 subagents | 355 | B lower on calls and read, complete separation |
| OpenFGA `7d89a566` | Code plus LLM document layer, 3 subagents | 21 | Calls lower (p = 0.14), read higher |
| Codex `859dbe27` | Code only, no LLM | none | No difference (p ≥ 0.22) |

The only saving appeared where the document layer was largest, and the one code-only build showed
nothing. So the saving may come from the document layer. That layer is the part that costs an LLM
pass to build and to keep current (R5). Or the saving may come from the code layer, which
`graphify update` keeps current for free. The rounds can't tell these apart. Repository is also
confounded with memorization (R8), and with the questions, which on Seikika often need a document
or a decision. The ablation is cheap to run. graphify-monorepo-ab already built a code-only graph
with no model, and `graphify update <path>` "re-extract[s] code files … (no LLM needed)" (its
0.9.118 CLI help, run hands-on).

**Action:** D01M85G2ZXXPQRXWKQF7TWV2RTP's cost trigger requires a code-only Graphify arm beside
the full build on Seikika. If the code-only arm keeps the saving, adoption needs no LLM pass and
the maintenance question disappears. If only the full build saves, the per-commit `--update` cost
decides it.

## 4. The P2 change cuts both ways

Changing a rule because it hid a rival's win needs the same rule applied to Seikika's own results.
This is every protocol-2 comparison that matches a registered cost hypothesis, read post hoc with an
exact two-sided Mann-Whitney test at α = 0.05 (no correction, which favours finding effects). Labels
as registered don't change; this shows what the amended rule would have said.

| Benchmark, round | Comparison | Measure | *W* rule (registered) | Rank test (post hoc) |
|---|---|---|---|---|
| graphify-ab r1, Seikika | Graphify against nothing | calls | inconclusive | Graphify lower, p = 0.008 |
| graphify-ab r1, Seikika | Graphify against nothing | read | inconclusive | Graphify lower, p = 0.008 |
| graphify-ab r1, OpenFGA | Graphify against nothing | wall time (not in the bar) | lower | lower, p = 0.016 |
| gate-repair-ab r1 (3 runs per arm) | reviewdog against Seikika | calls, read, gate read | Seikika loses (fails) | inconclusive, p = 0.10: three runs can't reach 0.05 |
| gate-repair-ab r2 (3 runs per arm) | reviewdog against Seikika | gate read | Seikika wins | inconclusive, p = 0.10 |
| gate-repair-ab r3 | Seikika before and after its change | calls | inconclusive | the change cut calls, p = 0.048 |
| gate-repair-ab r4 | reviewdog against Seikika | gate read | inconclusive | Seikika lower, p = 0.008 |
| gate-repair-ab r4 | Seikika before and after its change | read | inconclusive | the change cut read, p = 0.016 |
| pin-repair-ab r2 | reviewdog against Seikika | calls | inconclusive | Seikika lower, p = 0.008 |
| graphify-monorepo-ab r1 | every pair | calls, read, wall time | inconclusive | inconclusive (p ≥ 0.22) |

Of 39 comparisons, 10 read differently. The rank test would have given Graphify two cost wins and
Seikika two more, and would have erased Seikika's round-1 loss to reviewdog, because three runs per
arm can never reach significance. That last effect is why the amendment also raises the minimum to
five runs per arm for a cost hypothesis, and why it applies only from now on: re-labelling past
rounds under a rule chosen after seeing them would be the harness-shaped outcome P5 forbids.

## Sources

- [graphify-red-team.md](./graphify-red-team.md) and every source it lists.
- `experiments/{graphify-ab,graphify-monorepo-ab,gate-repair-ab,pin-repair-ab}/results/r*.tsv`;
  `experiments/graphify-ab/results/r1/*.json`, `experiments/graphify-monorepo-ab/results/r1/*.json`
  (`graphifyCommands`); `experiments/graphify-ab/prompts/graphify.mdc`;
  `experiments/graphify-ab/prompts/questions-openfga.md`; `experiments/graphify-ab/gold/openfga.md`.
- DX, "Introducing CAFE(S): a framework for context quality":
  <https://newsletter.getdx.com/p/introducing-cafes-a-framework-for>; DX's copy of the paper:
  <https://getdx.com/research/cafe-s-your-agent-is-only-as-good-as-its-context/>. The ACM Queue
  version (<https://queue.acm.org/doi/10.1145/3847288>) returned HTTP 403 to scripted access.
- `graphify --help` (graphifyy 0.9.118) for `update`, `check-update` and `affected`.

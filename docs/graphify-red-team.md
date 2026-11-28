# Red-team review: does Graphify really add no measurable value?

*Scope.* This review is about a tool (Graphify, PyPI `graphifyy`) and about how the maintainer
SDLC's DX Benchmarks judged it, at the bottom-up, single-repository end. It audits
[graphify-ab](../experiments/graphify-ab/README.md),
[graphify-monorepo-ab](../experiments/graphify-monorepo-ab/README.md),
[D01M84ZW6K2V9CV2SD644DE5M0S](decisions/D01M84ZW6K2V9CV2SD644DE5M0S.md),
[knowledge-graphs.md](design/knowledge-graphs.md), [dx-benchmarks.md](design/dx-benchmarks.md) and the
two catalog entries in `.seikika/benchmarks/`. It steelmans Graphify on purpose. Every claim carries a
source, and claims that are secondary, vendor-made, anecdotal or my own estimates are labelled as such.

## Bottom line

The team should be fairly confident of a narrow conclusion and not confident of the broad one. The
evidence supports three things. First, for a strong frontier agent with no turn cap, answering
single-shot questions, Graphify did not improve answer completeness: coverage tied in all three
repositories. Second, Graphify's headline token claims (71.5x and similar) do not hold, which every
independent study I found agrees with. Third, a full LLM build of the document layer is expensive.
The evidence does not support "adds no measurable value". On Seikika, every one of the five
Graphify runs made fewer calls and read less than every one of the five baseline runs, the
strongest result five runs per arm can produce (exact two-sided Mann-Whitney p = 0.008 on each
measure). It was labelled "inconclusive" only because the house rule (medians must differ by more
than the wider arm's range) can detect an effect of two standard deviations only about a quarter of
the time. The decision text then turns "inconclusive" into "didn't clear the noise". Graphify's
strongest integrations (Claude Code hooks, the `affected` impact command) and the conditions where
its value is claimed (weaker models, budgeted or long sessions, impact analysis before an edit,
incremental doc updates) were never tested. The not-adopt decision for Seikika for Seikika is
defensible as a cost and maintenance judgement. Its stated reasons should say "not shown under a
design that could only detect very large effects, with one cost advantage on Seikika", not
"neither helps nor hurts".

## 1. DX's CAFE framework

### 1.1 What the primary source says

CAFE(S) is defined in Houck (DX), Kanat-Alexander (Capital One), Kalliamvakou (GitHub), Storey
(University of Victoria) and Forsgren, "CAFE(S): Your Agent Is Only As Good As Its Context", *ACM
Queue* 24(5), published 24 September 2026 (doi 10.1145/3847288). DX hosts the same paper and a
newsletter introduction. It says:

- **Five properties of assembled context.** Clarity ("Can the agent interpret the request as
  intended?"), Actionability ("Can the agent proceed, and know when it is done?"), Fidelity ("Can the
  agent trust that this context is true?"), Efficiency ("Can the agent focus on what matters?") and
  Security ("Should the agent have this context at all?").
- **It is not a measurement system.** "CAFE(S) is deliberately a definition for high-quality
  context; it is not a measurement system." It separates the properties of context from "the
  mechanisms that create them and the outcomes they influence".
- **Retrieval is a precondition, not a CAFE property.** "Accessibility is intentionally outside the
  scope of CAFE(S)." Its three preconditions are accessibility, "cutting through the noise" (finding
  the relevant slice) and the fact that the context exists at all.
- **Context quality is evaluated in situ.** Its example is an `AGENTS.md` instruction that "will be
  injected into thousands of sessions that have nothing to do with that file, spending tokens and
  attention on noise".
- **Fidelity before brevity.** "Teams should first eliminate missing, incorrect, and stale context
  before optimizing for elegance or brevity."
- **Cited evidence on cost.** The paper cites Meta's precomputed context files for a pipeline
  spanning more than 4,100 files, with which "agents completed tasks using roughly 40 percent fewer
  tool calls and tokens", and Notion's roughly 90 percent cost drop from prompt caching.

DX operationalizes measurement elsewhere, in its AI Measurement Framework (utilization, impact,
cost). DX advises capturing utilization at 60 to 90 days and impact at six months, and says that
"organizations that force an ROI verdict too early often pull back exactly when the return is
beginning to compound" (DX blog, vendor).

### 1.2 Do the benchmarks measure what CAFE says matters?

Only in part. CAFE does not tell anyone how to benchmark a retrieval tool, and it puts Graphify's main
job (finding the relevant slice) in its preconditions. A CAFE-style evaluation would grade the
context Graphify puts in front of the agent: its always-on rule and its `query`, `explain`, `path`
and `affected` outputs. The benchmarks measured outcomes (key-fact coverage) and agent cost proxies
(calls and characters of tool results).

| CAFE property | What the benchmarks measured | What a CAFE-style evaluation would add |
|---|---|---|
| Clarity | Nothing directly. The questions were hardened to remove ambiguity ([monorepo README](../experiments/graphify-monorepo-ab/README.md) lines 97–99, §8), which controls clarity away | Whether graph output resolves or adds ambiguity. The adjudication saw ambiguous names across nodes ([knowledge-graphs.md](design/knowledge-graphs.md) lines 266–269) but did not score it. CAFE links poor clarity to "avoidable variation" across runs; Graphify's arm had the wider calls range on both graphify-ab repositories ([ADJUDICATION](../experiments/graphify-ab/ADJUDICATION.md) lines 97–99) but the tightest on Codex, so this is my interpretation, not a result |
| Actionability | Nothing. Every task was a question with an explicit done condition (write `ANSWERS.md`) | Whether graph output tells an agent where to act next, for example `affected` listing callers with file and line before an edit. No edit task was run |
| Fidelity | Answer fidelity, through coverage against gold. Not context fidelity: the graph was built from the frozen commit, so it was never stale | How often graph claims are wrong, incomplete or stale. Independent reports found wrong claims carried from graph output into answers (menges.dev) and an incomplete answer from a missing edge (LanceTek). On Seikika, 141 of 200 commits touch Markdown (verified), so a document graph is usually behind the tree |
| Efficiency | Closest match: calls and characters of tool results ([graphify-ab README](../experiments/graphify-ab/README.md) lines 146–149) | Signal-to-noise of each output (one OpenFGA query matched 768 nodes and printed 65, mostly irrelevant, [knowledge-graphs.md](design/knowledge-graphs.md) lines 83–86); billed tokens, which grow with every call because context is re-sent |
| Security | Nothing | Untrusted content entering the context (documents summarized into graph nodes, `INFERRED` edges presented beside extracted ones); vendor docs pages that carry instructions aimed at agents ([knowledge-graphs.md](design/knowledge-graphs.md) lines 116–118); what the semantic pass sends to a model |
| In situ | Only sessions where the rule was relevant | The cost of an `alwaysApply` rule in every session, including sessions with no code question. This cuts against Graphify |
| DX AI Measurement Framework | Utilization (K7, M10) and a lab proxy of impact and cost, in one round | Developer-reported time savings, longitudinal throughput, real spend, measured over months rather than one round |

**Verdict on §1.** The benchmarks measure part of Efficiency well and an outcome proxy honestly.
They do not measure Fidelity, Security or in-situ cost. CAFE cuts both ways here. Its Fidelity and
in-situ principles argue against an always-on, often-stale graph. Its cited Meta result says
precomputed context can cut tool calls by about 40 percent, and tool calls are exactly the measure
on which Seikika's Graphify arm separated. CAFE itself cannot support either "no value" or "value",
because by its authors' statement it is not a measurement system.

## 2. Graphify's adopters

### 2.1 Scale and attention

| Evidence | Figure | Source | What it shows |
|---|---|---|---|
| GitHub stars, forks, watchers | 152,536 stars, 14,662 forks, 436 watchers; created 2026-04-03 | GitHub API (verified) | Attention. Stars are not usage. Star history was not accessible (§Sources) |
| PyPI downloads, mirrors excluded | 2,153,211 in the last month; monthly 1.08M (June) to 1.90M (October 2026) | pypistats (verified) | Installs, inflated by CI and by upgrade churn across 284 releases. Not users |
| Committed Graphify outputs | 15,552 files named `GRAPH_REPORT.md` under `graphify-out/`; 166 files named `graphify.mdc` | GitHub code search (hands-on; counts are approximate) | The best usage evidence found: thousands of repositories commit a Graphify output. It does not show continued use or benefit |
| Vendor figures | 11.4M downloads, 15.2M "repositories indexed" | graphify.com (vendor, unverified) | Not checkable |
| Badges | Y Combinator S26; ROSS Index Q3 2026, "#2 fastest-growing" | Graphify README lines 7, 28 | Funding and star growth |
| Hacker News | 11 mentions, 0 explicit recommenders; "No documented product use has been verified yet" | hackerrepos.fyi (secondary aggregator) | Little practitioner discussion on HN |

### 2.2 Named companies, testimonials and anecdotes

| Who | Claim | Source | Independence |
|---|---|---|---|
| Logo wall: Shopify, Datadog, Vanguard, Microsoft, Rootly, J.P. Morgan, American Express, Harvey, Automation Anywhere, Alpha Governance | "Used by engineers at" | graphify.com (vendor) | Vendor-authored; it names individual engineers' employers, not company adoption. No case study for any of them was found |
| Rootly (Sylvain Kalache, Head of AI Labs) | Built an open-source Graphify plugin, `rootly-graphify-importer` (42 stars), that maps incident data into a graph | rootly.com blog, 2026-04-10; GitHub | A real company integration, but for incident data explored as a graph, not for coding agents. It also promotes Rootly's own product |
| MemVerge (Steve Scargall) | "We're seeing 79x token reductions" on MemMachine | stevescargall.com, 2026-05-10; quoted on graphify.com | Named and independent, but the 79x is `graphify benchmark` output (whole corpus against a query answer), which Graphify's own blog says "says nothing about whether those answers are correct". His build cost 1.4M input tokens |
| Sher Khan Akbar Khan (LinkedIn) | Fifteen projects over four months; one graph over seven projects (about 10,000 nodes, 22,000 edges); "128 times cheaper" | LinkedIn post, 2026-11-05 (anecdotal) | Sustained use is the signal. The ratio is against reading all the code, the same naive baseline |
| Reddit r/ClaudeCode users | Mixed: "very modest file recall improvement" but kept it; never saw the agent query the graph; "Claude does not hallucinate … very accurate detailed responses" while calling token claims overstated | reddit.com threads (anecdotal) | Unverified individual reports |

### 2.3 Independent measurements

None of these is from the vendor. Most support the team's token conclusion. Several report narrow
wins the team did not test.

| Study | Setup | Result | Favourable to Graphify? |
|---|---|---|---|
| hatenabase.jp (2026-09-23) | Private ERP, 3,501 files; graphify 0.9.65 with `--strict`, hooks and MCP; Claude Code; 30 runs | Input tokens +46% median, cost +19%, time +33%; all answers correct in both conditions. On named-symbol impact tasks, 7 of 9 answered correctly without opening a file (1 of 9 without Graphify) at +13% tokens and +1% cost. Business-language tasks +87% tokens. Same task and condition varied up to 3.7x in tokens | No on cost overall; yes, narrowly, on named-symbol impact |
| Machine Room (2026-10-22) | Private SaaS, 1,075 files; 0.9.92; Opus 5.5; n = 3 per cell | About 70,000 more tokens per question, because each extra call re-sends 50,000 to 70,000 tokens of context; 16 of 16 correct. `affected` listed nine callers with file and line; the graph disproved a stale claim in the author's notes | No for agent cost; yes for human use at the terminal |
| menges.dev (2026-10-24; author of the competing enola) | Private 820k-line, four-language system; 0.9.94; Opus 5.5 | Cycles question +23% cost. Package metrics −42% but a non-answer carrying wrong claims from graph output. Told to use Graphify on an auth question: 0 of 3 found a two-factor bypass, against 3 of 5 for the bare agent | No; a conflict of interest is declared |
| LanceTek | Three repositories, 15 questions, one run each | Exploration tokens −11% overall (−18% and −19% on the two larger repositories, +9% on the small one); 63–83% slower; score 28.5 against 27 of 30; one answer incomplete because of a missing edge | Slightly, on tokens in larger repositories |
| kubeblogs (2026-08-17) | Six feature-implementation sessions over three rounds | No session-level saving; quality nearly tied; weaker on frontend conventions, slightly better backend regression tests in two of three rounds | Mixed |
| DIY Smart Code, YouTube (2026-10-17) | Textual, 27 runs, 0.9.86 with hooks | About 124,000 tokens per question against 116,000; all answers correct; slower | No |
| Srivastava (2026-10-16) | 360 pre-registered trials, Airflow, VS Code, Spring | Grep F1 0.961 and 0.995, no clear gain. A smaller earlier run raised caller-question coverage from 0.77–0.83 to 1.00 with a hook. A new hook cut tokens about 13% against plain grep in 15 trials (direction only) | Small signals on callers and with the new hook |
| Issues #985, #3307, #3308, #4341 | Users and one SWE-bench Lite evaluation | 1.3x to 1.9x more tokens; `query_graph` top-five file recall 38% against 64% for BM25 | No |

**What §2 shows.** The traction is real, and the committed `GRAPH_REPORT.md` files show many people
build and keep graphs. No independent evidence shows that a strong agent answers better or more
cheaply on average with Graphify. The consistent narrow wins are impact questions about a named
symbol, orientation (fewer files opened), human exploration at the terminal, and catching stale
notes. The traction is better explained by a free, local, one-command, visually compelling
artifact and by the 71.5x headline than by measured agent gains. Traction therefore does not refute
the team's result, but the human-facing value was outside the benchmarks' scope.

## 3. Claims in Graphify's docs and README

| Claim | Source | Evidence quality | Tested by the benchmarks? | Result |
|---|---|---|---|---|
| Agents can "query instead of grepping" | README line 41 | Vendor assertion | Yes: the vendor's Cursor rule, verbatim | Partly contradicted. Agents queried first and then grepped and read anyway; no B answer cited the graph ([monorepo ADJUDICATION](../experiments/graphify-monorepo-ab/ADJUDICATION.md) lines 255–259). On Seikika, B opened fewer files (median 23 against 41) and made fewer calls in every run, which is consistent with orientation, not replacement |
| "71.5x fewer tokens per query" (and 5.4x, ~1x, 791x, 136.8x on other corpora) | `docs/how-it-works.md` lines 57–63; docs cookbook | Vendor, against pasting the whole corpus; no correctness measure. The vendor's own blog and docs concede it ("Headline numbers aren't your numbers") | Indirectly | Contradicted in magnitude. Per-session read fell 29% on Seikika, rose on OpenFGA, and fell 20% (median) on Codex. No study found anything near 71.5x |
| "Token reduction scales with corpus size" | `docs/how-it-works.md` line 65 | Vendor reasoning from three corpora | Yes: Codex (950k lines) against Seikika and OpenFGA | Not supported. Codex's read saving (20% by median, 8.5% by mean, p = 0.22) was no larger than Seikika's (29%) |
| ERPNext: key-fact coverage 82.0% against 70.8% | README line 166; `BENCHMARKS.md` lines 52, 146–147 | Vendor harness, n = 6, Claude Opus 4.8 capped at 14 turns, judged by Kimi K2.6; the `crosstool/` harness is not in the repository | No: no turn cap was used, by design (L21) | Neither supported nor contradicted. Under no cap, coverage tied everywhere |
| "Code maps for free, fully local" | README lines 43, 139 | Checkable | Yes | Supported. Codex built code-only in 59 s with no model ([monorepo README](../experiments/graphify-monorepo-ab/README.md) line 158); my hands-on build took 1.7 s |
| "Every edge is explained" (`EXTRACTED` / `INFERRED`) | README lines 44, 123 | Vendor feature claim; an issue says tags are display-only (#2384, secondary) | No | Untested. No answer used the tags |
| "Reusable context across sessions" | README line 34 | Vendor | No: every run was one fresh session | Untested |
| "Rationale + doc refs" and "Beyond code" (documents in the same graph) | README lines 137–138 | Vendor | Yes, on Seikika's document layer | No coverage difference (ceiling). B's lower calls and read on Seikika may come from this layer, but no ablation separates it from the code layer |
| `--update` re-extracts only changed files; per-file SHA256 cache | README line 412; `docs/how-it-works.md` line 77; `graphify/cache.py` `file_hash` | Vendor; code inspected | No | Untested, and it undercuts the "43 sessions" amortization (R5) |
| Hook platforms (Claude Code) get a hook before each search; instruction-file platforms (Cursor) "provide the same query-first guidance" | README lines 336–341 | Vendor | Only the Cursor rule | Untested for hooks. Independent hook studies (hatenabase, DIY Smart Code) found more tokens, so hooks may not help either |
| `affected "X" --relation R --depth N` gives an impact set | CLI help at `013a646` | Vendor; hands-on | No: 0 of 15 Graphify runs used it | Untested by the benchmarks. My hands-on check answered a two-hop caller question correctly in one call. Machine Room and hatenabase report this as the useful case |
| LOCOMO and LongMemEval results | README lines 163–164 | Vendor, memory benchmarks | No | Irrelevant to code questions |
| Hosted PR review beats Greptile (71% against 56% re-scored; Greptile's own grading gave 82%) | graphify.com/benchmarks | Vendor-run, hosted product | No | Untested; outside the decision's scope |
| 79x on MemMachine (testimonial) | graphify.com; stevescargall.com | Named user running the vendor's ratio | No | Same method as 71.5x |

## 4. Methodology holes

Arithmetic I checked holds: 4,272,339 ÷ 98,346 = 43.4 sessions; 141 of 200 commits before `a8f7e03`
touch Markdown; "a median of 18 fewer" calls on both repositories. The bar did not move after data,
gold hashes were recorded before the runs, the Graphify rule is the vendor's verbatim, and no run was
dropped for its result. The holes are in what the design could detect and in how its outcome was
written up.

| ID | Hole | Evidence (file:line) | Severity | What it could change | Cheapest test that would settle it |
|---|---|---|---|---|---|
| R1 | **The decision rule has very low power, and the one clear effect was called inconclusive.** With five runs per arm, "medians differ by more than the wider range" detects a two-standard-deviation effect 23% of the time and a one-SD effect 4% (my simulation, normal data; an exact rank test at one-sided p ≤ 0.016 detects 68% and 21%). Seikika calls (A 70–75, B 42–69) and read (A 327,662–400,058, B 199,070–321,355) are completely separated: exact two-sided Mann-Whitney p = 0.0079 each, which survives a Bonferroni correction over six hypotheses. The observed effects are about 2 SD, exactly where the rule fails. One wide B run sets *W* for both arms. Predicting "inconclusive" under this rule is nearly certain, so "matches the designer's prediction" is weak evidence | [dx-benchmarks.md](design/dx-benchmarks.md):160, 183; [graphify-ab README](../experiments/graphify-ab/README.md):23–24, 154–156; [ADJUDICATION](../experiments/graphify-ab/ADJUDICATION.md):98–99, 131–133; `results/r1.tsv` | High | K5 and K6 would read "helps on cost on Seikika" under a standard test. The rule protects equally against a false "hurts" | Pre-register an exact rank test with a multiplicity correction for the next round, and apply it to the existing data as a labelled post-hoc analysis; or rerun Seikika only, with ten runs per arm |
| R2 | **The write-up turns "inconclusive" into "no effect".** The decision says B's lower calls "didn't clear the noise" and that the benchmark "showed neither 'helps' nor 'hurts'"; the survey says "inside the noise" and "No independent study shows a code graph helping". The decision is careful elsewhere ("it isn't shown", line 27), but its adoption rationale rests on the looser wording. The "Otherwise" branch makes non-adoption the default, so a design that cannot detect moderate effects decides the outcome by construction | [decision](decisions/D01M84ZW6K2V9CV2SD644DE5M0S.md):19, 27–31; [knowledge-graphs.md](design/knowledge-graphs.md):24–27, 32, 37–38, 235–236; [graphify-ab ADJUDICATION](../experiments/graphify-ab/ADJUDICATION.md):119–121 | High | The stated reason for the decision, and how others cite it ("Graphify adds no measurable value") | No experiment: a new decision that restates the result as "not shown; Seikika cost lower in every run; design powered only for very large effects" |
| R3 | **Graphify's strongest integration and its impact command were never exercised.** Arms got only the Cursor `alwaysApply` rule, which names `query`, `path` and `explain` but not `affected`. No hooks, no MCP. None of the 15 Graphify runs used `affected`, although the monorepo README says the questions suit it and Q2–Q3 are impact questions. Seikika's own CLI friction on Q1 was fixed after the run (`--relation`, `--direction`), while Graphify's existing equivalent was never surfaced. Seikika maintainers also use Claude Code (`CLAUDE.md`), where Graphify installs hooks | [prompts/graphify.mdc](../experiments/graphify-ab/prompts/graphify.mdc):8–11; [monorepo README](../experiments/graphify-monorepo-ab/README.md):67, 92; [monorepo ADJUDICATION](../experiments/graphify-monorepo-ab/ADJUDICATION.md):255–259; `results/r1/*.json` `graphifyCommands`; Graphify README:336–341 | High | B's calls and read on Codex Q2–Q3, the questions closest to the narrow wins independent studies found | Rerun Codex Q2 and Q3 only, five runs per arm, with the vendor rule plus one declared line naming `affected`; or once under Claude Code with `graphify install --project` hooks |
| R4 | **Cost was measured in characters of tool results, not billed tokens.** Each call re-sends the whole context, so calls drive cost (Machine Room measured 50,000–70,000 tokens re-sent per call). The read measure credits B on Seikika with 98k characters, about 25k tokens; at even 15k tokens of fixed context per call (my assumption), 18 fewer calls remove about 270k input tokens. The same proxy is used on both sides of the amortization: under a call-dominated model the full build costs about 413 ÷ 18 ≈ 23 sessions' savings, not 43 (my estimate) | [graphify-ab README](../experiments/graphify-ab/README.md):146–149, 39–44; `harness/count.py`:69–81 | Medium | The size of the Seikika cost advantage, and the break-even point | Record billed input tokens (cached and uncached) per run; Cursor or Claude Code usage logs are enough |
| R5 | **The amortization assumes a full rebuild per change.** "Documents change on most commits" is used as if each change needs a full document build. Graphify's per-file cache re-extracts only changed files. Over the 200 commits before `a8f7e03` there were 973 Markdown file changes (about 4.9 a commit, verified) against 355 documents in the graph, so a commit touches about 1.4% of the corpus. At face value that is about 60k characters of extraction a commit, less than one session's measured saving (my estimate; it ignores per-chunk overhead). The semantic pass can also use a cheaper backend than the session's model. The decision's first reopen trigger asks for no LLM pass even on changed documents, a stricter bar than the cost argument needs | [graphify-ab README](../experiments/graphify-ab/README.md):39–44, 90–93; [decision](decisions/D01M84ZW6K2V9CV2SD644DE5M0S.md):19, 29–34; [knowledge-graphs.md](design/knowledge-graphs.md):238–241; Graphify README:412, 514; `docs/how-it-works.md`:77 | Medium | "Build cost dominates" may hold only for the first build | Run `/graphify . --update` over ten consecutive real commits after `a8f7e03` and count calls and tokens per commit |
| R6 | **Task type, ceiling and the score-to-guard switch.** All tasks were single-shot questions with fixed answers, one strong model, no turn or budget cap, no edit task and no multi-question session; 14 questions in total. Coverage saturated (*W* = 0), so only cost could show "helps". In the monorepo the score became a guard before freezing; that made the first row of its own decision table ("Graphify helps … reopen trigger met") unreachable, so the benchmark could not have changed Seikika for Seikika's adoption. Variation across questions is not modelled | [graphify-ab ADJUDICATION](../experiments/graphify-ab/ADJUDICATION.md):136–145; [monorepo README](../experiments/graphify-monorepo-ab/README.md):94–102, 111–112, 206, 220–222, 338–340; [dx-benchmarks.md](design/dx-benchmarks.md):108–109 | Medium | Whether Graphify helps where its value is claimed: weaker models, budgeted sessions, edits, long sessions | One round of the existing Codex Q2–Q3 with a cheaper model, or one three-question sequential session per run |
| R7 | **The reopen triggers ask for evidence the team's own lessons say it cannot get.** Trigger 2 needs a coverage gain with headroom under no turn cap; L22 concludes harder questions don't buy headroom. Cost gains, the only measure with room, cannot reopen the decision | [decision](decisions/D01M84ZW6K2V9CV2SD644DE5M0S.md):32–37; [dx-benchmarks.md](design/dx-benchmarks.md):108–109; [monorepo README](../experiments/graphify-monorepo-ab/README.md):111–112 | Medium | Whether any feasible result could ever reverse the decision | Add a cost trigger with a pre-registered threshold and test (for example, ≥ 20% fewer billed tokens at equal coverage) |
| R8 | **Model and baseline strength.** The model is never named ("the session's model"). OpenFGA (2022) and Codex (136k stars) are public and probably in training data, which strengthens the no-tool arm; menges.dev chose private code for this reason. The only repository the model cannot have memorized, Seikika (created 2026-11-18, verified), is where Graphify's arm separated. That is suggestive, not shown | [graphify-ab README](../experiments/graphify-ab/README.md):187; [monorepo README](../experiments/graphify-monorepo-ab/README.md):246; GitHub API | Medium | Generalization to private code and other models; reproducibility | Record the model id; add a no-tools arm (as Srivastava did) to measure memorization; repeat on a private or post-cutoff repository |
| R9 | **Designer conflict and gold construction.** The designer works on Seikika, a product in the same space, and wrote the questions, key facts, predictions and arm C's rule. Gold was single-source and written by the designer. One gold error was found only because arm C disagreed (Q3, 19 to 20 items). O4d ("validation applies when a model is loaded") falls outside Q4's wording ("when a model is written") and was missed by all ten runs. Questions name exact identifiers, which favours `rg` | [graphify-ab README](../experiments/graphify-ab/README.md):122–134; [gold/openfga.md](../experiments/graphify-ab/gold/openfga.md):36; [questions-openfga.md](../experiments/graphify-ab/prompts/questions-openfga.md):4; [ADJUDICATION](../experiments/graphify-ab/ADJUDICATION.md):140; [monorepo README](../experiments/graphify-monorepo-ab/README.md):130–139, 307–314 | Medium | Question selection; little effect on coverage, which tied | Invite the vendor or an outside party to submit two pre-registered questions per repository |
| R10 | **Measurement artifacts in the counter.** `count.py` attributes a whole shell call to Graphify if any part runs `graphify`, imputes a whole file for a missing `read_file` result, and checks escapes by absolute path only. One `grep -rn` returned 580,577 characters of `graph.json` (openfga-3); openfga-6 read 643,413 with only 29,759 from Graphify, unexplained. Removing openfga-3's grep does not produce a B win on OpenFGA (B median about 325,726 against 319,746) | `harness/count.py`:12–14, 22, 49–58, 69–81; `results/r1.tsv` (openfga-3, openfga-6); [knowledge-graphs.md](design/knowledge-graphs.md):236–237 | Low | K2's direction, not its verdict | Rerun `count.py` with per-command splitting and a sensitivity analysis that excludes `graphify-out/` reads |
| R11 | **Blinding is name-only.** `blind.py` replaces only "graphify"; arm C leaked through "portal" and entity fields | `harness/blind.py`:31; [monorepo ADJUDICATION](../experiments/graphify-monorepo-ab/ADJUDICATION.md):273–276 | Low | Little, because grading was mechanical and coverage tied | Redact tool-specific phrases and fields, or grade with a script against gold sets |
| R12 | **Wall time moved but was excluded from the bar.** OpenFGA B 185 s against A 274 s (exact p = 0.016); Seikika 191 s against 201 s (p = 0.079). DX treats time savings as a primary impact signal. Runs were concurrent subagents, so contention may add noise | [graphify-ab ADJUDICATION](../experiments/graphify-ab/ADJUDICATION.md):148–155; `results/r1.tsv` | Low | A "faster" finding on OpenFGA | Pre-register wall time, with one run per machine at a time |
| R13 | **Pre-registration changes after dry runs that included Graphify's arm.** The monorepo prediction was revised, the questions hardened twice, the score made a guard and the gold fixed, all after dry runs in which the designer saw B runs. All were declared before freezing, so this is legitimate | [monorepo README](../experiments/graphify-monorepo-ab/README.md):220–222, 230, 286–296, 307–323 | Low | Little; it is a transparency point | Run dry runs on arm A only |
| R14 | **One version, one harness.** Only graphifyy 0.9.118, released the day of the run, out of 284 releases, with query ranking under active change; only Cursor | [graphify-ab README](../experiments/graphify-ab/README.md):52, 193; PyPI | Low | Shelf life of the result | Pin the version in the catalog entry and rerun on a reopen |
| R15 | **Factual overreach in the decision and survey.** "Its questions (where is X, what calls it) are answered by `rg` in a call or two" is contradicted by the monorepo's own Q3 (two hops of callers, test exclusion, a gold error), where arm A took a median of 56 calls for four questions. "No independent study shows a code graph helping" omits small positive signals (Srivastava's caller-coverage run, hatenabase's named-symbol tasks, LanceTek's −11%). The survey also omits CAFE and six independent studies, most of which support the team | [decision](decisions/D01M84ZW6K2V9CV2SD644DE5M0S.md):21; [knowledge-graphs.md](design/knowledge-graphs.md):37–40, 99–108; [monorepo ADJUDICATION](../experiments/graphify-monorepo-ab/ADJUDICATION.md):116–136 | Low | Credibility of the rationale; not the verdict | Edit the wording in the new decision from R2 and add the omitted sources to the survey |

## 5. The strongest case for Graphify that the team has not ruled out

1. **Graphify lowers orientation cost on a documentation-heavy repository the model has never
   seen.** On Seikika, all five Graphify runs used fewer calls (25% fewer by median) and read less
   (29% less) than all five baseline runs, and opened fewer files (23 against 41). That is
   significant under a standard rank test even after correction (R1). It matches the mechanism CAFE
   cites from Meta (precomputed context, about 40% fewer calls and tokens) and hatenabase's finding
   that the graph lets an agent answer without opening files.
2. **The billed saving could be larger than the measured one.** Because context is re-sent on each
   call, 18 fewer calls likely save more tokens than 98k characters of tool results suggest (R4).
3. **Its impact command was never tried.** `affected` answers "who calls X, through one more hop"
   in one call, which is the shape of the monorepo's Q2 and Q3 and of the tasks where independent
   studies saw Graphify help (R3).
4. **Ongoing cost may be small.** The code graph is free and fast. With the per-file cache, a
   typical commit re-extracts a few documents, not 355 (R5).
5. **It may help weaker models and budgeted sessions.** The vendor's only coverage gain came under a
   14-turn cap. No round used a cheaper model or a budget, and these are common in practice (R6).
6. **It has human value.** `graph.html`, `explain`, `path` and `affected` at the terminal, and the
   stale-note catch Machine Room reported, lie outside agent benchmarks. The decision itself lets a
   portal carry Graphify as a standalone tool (decision line 38).

The limits of this case are real. Every item is a hypothesis with at most small-sample support.
Under Claude Code with hooks, independent studies consistently found higher token cost, and no
study found better answers for a strong model.

## 6. What would change my mind

**Toward "the team is right that it adds no value worth adopting":**

- A rerun of Seikika with ten runs per arm and billed tokens shows no difference in calls or tokens
  under a pre-registered rank test.
- A Codex Q2–Q3 rerun with `affected` named in the rule (or with Claude Code hooks) shows no
  reduction in calls or tokens.
- Measured `--update` cost per commit, times commits per session, exceeds the per-session saving.
- A cheaper model shows no coverage gain without a cap.

**Toward "Graphify adds measurable value":**

- The Seikika cost separation replicates with billed tokens, for example at least 20% fewer billed
  tokens at equal coverage with p < 0.05 under a pre-registered test.
- `affected` cuts Codex Q2–Q3 calls by a third or more at equal F1.
- A cheaper model gains 10 or more coverage points with no turn cap.
- Incremental document updates cost less than one session's saving per commit.

Any one result in the second list would justify a cost-scoped decision of the kind the monorepo
README already anticipates ("A new decision records the cost gain and the repository size it holds
at"). Results in the first list would make the current decision's wording accurate.

## Sources

### Repository files (read, not modified)

- [experiments/graphify-ab/README.md](../experiments/graphify-ab/README.md),
  [ADJUDICATION.md](../experiments/graphify-ab/ADJUDICATION.md), `results/r1.tsv`,
  `results/build-semantic.tsv`, `results/r1/*.json`, `harness/count.py`, `harness/blind.py`,
  `prompts/graphify.mdc`, `prompts/questions-openfga.md`, `gold/openfga.md`.
- [experiments/graphify-monorepo-ab/README.md](../experiments/graphify-monorepo-ab/README.md),
  [ADJUDICATION.md](../experiments/graphify-monorepo-ab/ADJUDICATION.md), `results/r1.tsv`,
  `results/r1/*.json`, `prompts/graphify.mdc`, `prompts/seikika.mdc`.
- [docs/decisions/D01M84ZW6K2V9CV2SD644DE5M0S.md](decisions/D01M84ZW6K2V9CV2SD644DE5M0S.md),
  [docs/design/knowledge-graphs.md](design/knowledge-graphs.md),
  [docs/design/dx-benchmarks.md](design/dx-benchmarks.md), `.seikika/benchmarks/graphify-ab.yaml`,
  `.seikika/benchmarks/graphify-monorepo-ab.yaml`.

### Primary external sources

- Houck, Kanat-Alexander, Kalliamvakou, Storey, Forsgren, "CAFE(S): Your Agent Is Only As Good As Its
  Context", *ACM Queue* 24(5), 2026-09-24: <https://queue.acm.org/doi/10.1145/3847288>; DX copy:
  <https://getdx.com/research/cafe-s-your-agent-is-only-as-good-as-its-context/>; newsletter:
  <https://newsletter.getdx.com/p/introducing-cafes-a-framework-for>.
- Meta, "How Meta used AI to map tribal knowledge in large-scale data pipelines" (cited by CAFE):
  <https://engineering.fb.com/2026/04/06/developer-tools/how-meta-used-ai-to-map-tribal-knowledge-in-large-scale-data-pipelines/>.
- DX AI Measurement Framework: <https://getdx.com/uploads/ai-measurement-framework.pdf>; DX blog
  (vendor): <https://getdx.com/blog/ai-roi-engineering/>.
- Graphify repository at `013a646` (graphifyy 0.9.118), cloned: `README.md`, `BENCHMARKS.md`,
  `docs/how-it-works.md`, `graphify/install.py` (`_CURSOR_RULE`), `graphify/cache.py`,
  `graphify/scip_ingest.py`: <https://github.com/Graphify-Labs/graphify>.
- GitHub API for repository metadata; PyPI JSON API; pypistats:
  <https://pypistats.org/packages/graphifyy>.

### Vendor sources

- <https://graphify.com/> (logo wall, testimonials, download claims);
  <https://graphify.com/benchmarks>;
  <https://graphify.com/blog/what-the-graphify-benchmark-measures>;
  <https://docs.graphify.com/platform/graphify-vs-local.md>;
  <https://docs.graphify.com/open-source/features/token-benchmark>;
  <https://docs.graphify.com/open-source/features/honest-token-savings>;
  <https://docs.graphify.com/cookbooks/claude-code-monorepo>;
  <https://docs.graphify.com/cookbooks/make-agents-use-the-graph>.

### Independent and secondary sources

- hatenabase.jp, 2026-09-23: <https://hatenabase.jp/blog/graphify-code-knowledge-graph-hands-on/>.
- Machine Room, 2026-10-22: <https://themachineroom.dev/v/graphify-measured/>.
- D. Menges (author of enola), 2026-10-24:
  <https://menges.dev/writing/code-graph-mcp-servers-tested/>.
- LanceTek: <https://lancetek.com/journal/graphify-code-graph-tested-on-three-repos>.
- kubeblogs, 2026-08-17:
  <https://www.kubeblogs.com/graphify-claims-71-5x-fewer-tokens-we-tested-it-on-real-production-work/>.
- DIY Smart Code, YouTube, 2026-10-17: <https://www.youtube.com/watch?v=R8r5Or0Xajg> (read through a
  transcript summary; secondary).
- S. Srivastava, 2026-10-16: <https://srijansriv.com/graphify-benchmark>,
  <https://srijansriv.com/graphify-benchmark-new-hook>,
  <https://srijansriv.com/graphify-benchmark-where-next>.
- Graphify issues #985, #2384, #3307, #3308, #4341 (as summarized in
  [knowledge-graphs.md](design/knowledge-graphs.md) lines 103–108; #4341 and #985 reachable).
- S. Scargall (MemVerge), 2026-05-10:
  <https://stevescargall.com/blog/2026/05/graphify--memmachine-79-token-reduction-zero-vector-database/>.
- Rootly, 2026-04-10: <https://rootly.com/blog/turning-your-incident-data-into-a-knowledge-graph>;
  <https://github.com/Rootly-AI-Labs/rootly-graphify-importer>.
- Sher Khan Akbar Khan, LinkedIn, 2026-11-05 (anecdotal):
  <https://www.linkedin.com/posts/sherkhanmy_aiagents-knowledgegraph-aicontext-activity-7523908219480473600-vXcx>.
- Reddit r/ClaudeCode (anecdotal):
  <https://www.reddit.com/r/ClaudeCode/comments/1tlbog1/are_tools_like_graphify_actually_placeboshype/>,
  <https://www.reddit.com/r/ClaudeCode/comments/1uj3kgu/just_discovered_graphify_has_anyone_here_actually/>.
- hackerrepos.fyi (secondary aggregator): <https://hackerrepos.fyi/repo/graphify-labs/graphify>.

### Verified hands-on

- **The rule is the vendor's.** `prompts/graphify.mdc` matches `_CURSOR_RULE` in
  `graphify/install.py` (lines 1593–1616) at `013a646`.
- **Statistics.** Exact two-sided Mann-Whitney tests recomputed from both `results/r1.tsv` files:
  Seikika calls and read p = 0.0079 each; OpenFGA calls p = 0.14, read p = 0.31, wall time p = 0.016;
  Seikika wall time p = 0.079; Codex calls p = 0.39, read p = 0.22, wall time p = 0.73. These are
  post-hoc and exploratory; they cannot overturn the pre-registered verdicts.
- **Power.** A simulation (20,000 trials per effect size, normal data, five runs per arm) of the
  *W* rule against an exact rank test (U ≥ 23 of 25): detection 0.2% and 1.8% at no effect, 4% and
  21% at 1 SD, 23% and 68% at 2 SD, 59% and 96% at 3 SD, 86% and 99.7% at 4 SD. The observed Seikika
  effects are about 2 SD (calls 1.95, read 2.21).
- **Arithmetic.** The build totals (17 chunks, 413 calls, 4,272,339 characters; OpenFGA 3 chunks,
  617,266), the 43.4-session ratio, 141 of 200 Markdown-touching commits, and 973 Markdown file
  changes over those 200 commits.
- **`affected`.** On a copy of `crates/seikika-connector-workspace`, a code-only graph (534 nodes)
  built in 1.7 s. `graphify affected "lines_of" --relation calls --depth 2` returned the direct
  caller at line 163 and the second-hop callers `visit_item_fn` (line 335), `visit_impl_item_fn`
  (line 345) and `visit_trait_item_fn` (line 351) in about 0.5 s, all confirmed with `rg`. The output
  does not label which hop each caller is on. This is a check of one command, not a benchmark.
- **Graphify usage counts.** All 15 Graphify runs' `graphifyCommands` list only `query`, `explain`
  and `path`.

### Could not access

- GitHub stargazer history (`starred_at`): HTTP 403 through `gh` ("Resource not accessible by
  integration") and 401 without authentication. Star history is therefore not reported.
- GitHub code search beyond two queries: HTTP 429 (rate limit). Only the two counts in §2.1 were
  obtained.
- <https://graphify.com/graphify-vs-local> and <https://docs.graphify.com/graphify-vs-local>: HTTP
  404; the page lives at `docs.graphify.com/platform/graphify-vs-local.md`.
- Graphify's benchmark harnesses (`crosstool/` and `memory/`), named in `BENCHMARKS.md` but absent
  from the repository, so the ERPNext result cannot be reproduced.
- No agent transcripts outside this repository were read.

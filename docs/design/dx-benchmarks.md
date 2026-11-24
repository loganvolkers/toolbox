# DX Benchmarks: Measuring Our SDLCs Against Open Source

*Status: an internal standard (D01M82X2K0DYWTCA23MQJ4CKR0Z). About the box
([toolbox-and-tools.md](./toolbox-and-tools.md)): how we measure whether the box helps the agents
that use it. Maintainer SDLC ([maintainer-sdlc.md](./maintainer-sdlc.md)); each benchmark measures
one or more of the four SDLC layers ([../sdlc.md](../sdlc.md)). The catalog is in
[portal/benchmarks/](../../portal/benchmarks), scored by `maintainer-benchmark`. Runs follow
protocol 2 (§4.1, D01M83RDWQ5EVKY5SFHDWAD5HFH).*

---

## 1. Conclusion

**A DX Benchmark is a pre-registered A/B in which fresh coding agents do real lifecycle work on
open-source repositories, once with Toolbox and once with its strongest alternative. The results,
the cost and the friction are all published, including when Toolbox loses.** It is how we find
Toolbox's rough edges, and how we decide whether a claim about Toolbox is true. It is a maintainer
standard: a claim that Toolbox makes something easier needs a benchmark behind it.

- **They measure the SDLCs, not the model.** Industry benchmarks such as SWE-bench, Terminal-Bench
  and Next.js Evals hold the task fixed and compare agents. Ours hold the agent fixed and compare
  the developer platform around it: the box against hand-written glue, a portal against a
  checklist, a reusable standard against a home build. Each benchmark names the SDLC layers and
  stages it exercises (§2).
- **They have changed Toolbox more than any review.** Fourteen benchmarks so far (§3).
  - R3 failed its bar three times: the checklist A/B, the standards A/B and its rerun.
  - R6, the box A/B, failed on tokens.
  - R2 met its bar only after fixes, because the first run failed on all four forks.

  Each failure produced a list of measured friction, and most of it was fixed:
  - packages and a numeric ratchet;
  - the SARIF slot and take-it-out;
  - one pin per tool;
  - root causes;
  - scoring missing data as `unknown`;
  - a dozen CLI fixes.
- **Two results were wrong, and have been corrected** (D01M83RDWR52WVNER0MV5T84NSX).
  - connector-ab's third round reported a Toolbox win. Only Toolbox's arm had changed, one change
    was a product format shaped to the harness, and the 1-call margin was smaller than an unchanged
    rival's run-to-run swing.
  - deploy-ab reported a defeat. The probes authenticated only Toolbox, and its container was
    built before the run.

  Both were run by the agents that designed them, one run per arm. Protocol 2 (§4.1) is the fix.
  Every result before it is one sample.
- **The first protocol-2 win over a rival is narrow by design.** In pin-repair-ab's round 2, agents
  with Toolbox's gate read a median of 5,794 characters against 18,300 with reviewdog's, because
  the gate names the pins the repository already uses. Calls and gate read were inconclusive,
  and read held by only 1,073 characters over *W*. It is one repository and one planted pull
  request, chosen where Toolbox's hints should help, and it took a measure fix (L20) to see it.
- **The pattern they found:** Toolbox rarely beats a capable agent on a single run. Agents are
  trained on programming languages and their tools, not on Toolbox. Toolbox's value appears with
  repetition: on the second repository, the tenth run, the next upgrade, the comparison across
  repositories. The benchmarks now measure that directly: reuse cost, convergence, and runs over
  time.
- **This is our claim to fame.** We know of no other developer platform that publishes
  pre-registered, head-to-head agent benchmarks of its own developer experience on open-source
  repositories, failures included (§5; this is our survey, not a verified absence).

## 2. What a DX Benchmark is

| Part | Requirement | Why |
|---|---|---|
| **A question tied to a hypothesis** | One sentence, linked to a hypothesis with a bar (R1–R7 in [repo-value.md](./repo-value.md), or new) | A benchmark without a decision behind it is a demo |
| **The SDLC layers and stages** | The layers it measures (maintainer, package author, operator, user) and the stages within them, from [portal/sdlc/](../../portal/sdlc) | The benchmarks are how we know each layer's stages work for agents |
| **A rival arm** | The strongest alternative a team would really use: an agent from scratch, a written checklist, hand-written CI glue, an existing standard (OpenSSF Scorecard), or another platform (Backstage) | Beating a straw man proves nothing |
| **Open-source repositories at pinned commits** | Forks we didn't write (OpenFGA, Codex, gh-aw, PostHog so far), exported at a commit | Real code, reproducible, and not tuned to Toolbox |
| **A bar set before running** | The README states the hypotheses, what "holds" and "fails" mean, and the known limits, and is not edited after the run except for its status line. The bar uses a measure a dry run showed the runtime reports (P6) | It stops us explaining away a loss |
| **Fresh agents, equal conditions** | At least three fresh runs per arm per phase (P2). The same model, budget and tools in each arm, the prompts committed before the first run (P1), no network, isolated copies, one launch per workspace at a time | Differences come from the arm, not the setup or the noise |
| **Symmetric arms** | One frozen commit per round (P3). Every arm gets the same level of help, and rival friction is fixed as carefully as Toolbox's (P4). No product change is shaped to the harness (P5) | A rival that never changed can't lose to a Toolbox tuned to the test |
| **Independent truth** | Correctness is judged by a script or a recorded adjudication with evidence, not by the arm's own report. The adjudicator did not design the experiment or change the product (P7) | Agents' self-reports overstate success, and so do designers |
| **Cost** | Tokens, tool calls, wall time, and lines of glue per phase | Equal correctness at higher cost is a loss |
| **Friction** | Each arm is asked for developer-experience feedback the same way. It is triaged into fixed, deferred or rejected, with a reason | Friction is the most useful output |
| **A result, published** | `RESULTS.md` or the README's results section, a row in [research-log.md](../research-log.md), and the catalog entry | A loss we hide is a loss we repeat |

**It is not a test.** `cargo test`, the acceptance scripts and the T4T ratchet check that Toolbox
behaves as specified. A DX Benchmark checks whether that specification helps the agents that use
it. **It is not a scale gate either:** `scripts/scale-gate.py` measures machines, and a DX
Benchmark measures the work of agents.

## 3. What we have learned

The catalog ([portal/benchmarks/](../../portal/benchmarks)) holds each benchmark's details. The
lessons that cut across them:

| # | Lesson | From |
|---|---|---|
| L1 | **Pre-registration earns its keep.** Without a bar set beforehand, both of the standards A/B's losses could have been explained as "learning cost" | standards A/B (D01M7Z2ZZ97VZQNTDT4QQHM3HQD), box A/B |
| L2 | **Fix the friction, then rerun the same arm.** The rerun B′ cut Toolbox's tokens from 582k to 510k. It still lost, but it showed which friction mattered | standards A/B rerun |
| L3 | **Close known friction before running.** A benchmark that hits friction we already know about measures nothing new. Pilot adoptions, such as adding the security scorecards to `loganvolkers/rt-buddy`, are the cheapest place to find it | box A/B, the rt-buddy pilot |
| L4 | **Measure the right half.** The standards A/B compared building a tool with Toolbox against writing a script. Teams rarely lack a linter: they lack one place for all of them. The box A/B tested that | box A/B §0 |
| L5 | **Single runs favour the agent; repetition favours the standard.** A home-built scorecard found more than OpenSSF, but reusing it cost as much as building it, and two builds disagreed by up to 4.4 points | security A/B |
| L6 | **A tool error must never read as zero findings.** The harness kills timed-out process groups and fails loudly. Scorecard's Vulnerabilities check scores 10 when OSV is unreachable | box A/B harness, security A/B |
| L7 | **Remove answer leaks, and keep a read allowlist.** The truth lives in the experiment's own files and the research log; the arms must not read them | box A/B prep, standards A/B rerun |
| L8 | **Results files are the arm's output, and the harness may refuse them.** Subagents may be blocked from writing report files. Plan for the report to come back as text, and record the deviation | security A/B |
| L9 | **Export each arm's diff before deleting its copy.** Disk fills; a deleted arm can't be scored again | box A/B |
| L10 | **One run is one sample.** Every result so far is one run per phase. State it, and prefer two independent builds (convergence) when runs are expensive. Protocol 2 now requires three (P2) | all |
| L11 | **A rerun that changes one arm measures the change, not the platform.** connector-ab's v2 and v3 changed only Toolbox's prompts and recipe, and added `--format catalog`, which emits the harness's own NDJSON shape. The "win" that followed tells us the tuning worked on this harness, nothing more | connector-ab ([ADJUDICATION.md](../../experiments/connector-ab/ADJUDICATION.md)) |
| L12 | **The noise is larger than it looks.** Backstage's phase 1 had the same prompt and scaffold in every round, and took 29, 38 and 29 calls. A 1-call margin from one run is no margin | connector-ab |
| L13 | **An unequal start is not a result.** deploy-ab authenticated only Toolbox in S2, its Toolbox arm restarted a container built before the run, and its Backstage prompt suggested a provider the facade can't serve | deploy-ab ([ADJUDICATION.md](../../experiments/deploy-ab/ADJUDICATION.md)) |
| L14 | **See the measure before the bar uses it.** Two bars were written in tokens that the runtime never reported. One then swapped in calls after the run. A dry run would have shown it | connector-ab, deploy-ab |
| L15 | **Run before and after in one round.** In gate-repair-ab's round 2, reviewdog's median read doubled with nothing changed, because agents chose to explore more. A third arm running the previous Toolbox in the same round removed that drift | gate-repair-ab ([ADJUDICATION.md](../../experiments/gate-repair-ab/ADJUDICATION.md)) |
| L16 | **Measure what the change touches, and guard the rest.** A total that includes the agent's own exploration hid a 4.8× cut in gate output. Make the touched part the primary measure, and require that the totals don't fail. Register it before the round, and say if earlier rounds suggested it | gate-repair-ab |
| L17 | **The transcript is not the screen.** The runtime keeps a command's stdout only, so a verdict on stderr was read but not counted | gate-repair-ab §0.1 |
| L18 | **Calls measure how an agent edits, not only what it was told.** In gate-repair-ab's round 4, every reviewdog run edited and reran its gate in one shell command, and every Toolbox run used two editor calls and a separate rerun. With both arms' ranges at 1 call, that alone decided the calls hypothesis. Report calls by kind (edits, gate runs, exploration) beside the total | gate-repair-ab ([ADJUDICATION.md](../../experiments/gate-repair-ab/ADJUDICATION.md) threat 12) |
| L19 | **Check that the measure can move before asking again.** In gate-repair-ab's round 5, every run in every arm printed the branch's workflows before editing, even when the gate had quoted the lines. That reading, 3,500 to more than 15,000 characters a run, set the floor and the noise of the read measure, so a better gate output could not separate the arms. The same unchanged Toolbox also moved by 3 calls between rounds. A second try at the same win needs a measure, task or run count under which the change could exceed *W* | gate-repair-ab ([ADJUDICATION.md](../../experiments/gate-repair-ab/ADJUDICATION.md) threats 21–25) |
| L20 | **Count what the transcript dropped, per arm, before a label.** In pin-repair-ab, the runtime recorded no result for seven terminal commands over two rounds, all in reviewdog's arm: four gate runs and three greps over every `uses:` line. Each came from a command that exited non-zero, and every arm A command ended in one that exits 0. The agents had read them, so the measure undercounted one arm, and round 1's read comparison came out inconclusive. Replaying the run's commands in a fresh workspace rebuilds them, kept only when the replay reproduces the run's recorded results | pin-repair-ab ([ADJUDICATION.md](../../experiments/pin-repair-ab/ADJUDICATION.md)) |
| L21 | **A quality measure needs headroom.** In graphify-ab, every answer on a repository scored the same key-fact coverage in both arms, so *W* was 0 and no coverage hypothesis could hold or fail. Answer the questions with the baseline arm in the dry run, and freeze only a set it doesn't saturate. A turn cap also makes headroom, but then the cap is part of the claim | graphify-ab ([ADJUDICATION.md](../../experiments/graphify-ab/ADJUDICATION.md)) |
| L22 | **Harder questions don't buy headroom; claim cost and size *W* for it.** graphify-monorepo-ab hardened its questions twice on a 950k-line monorepo (trait dispatch, two hops of callers with test exclusion, a five-crate trace), and every answer still scored 0.981 or more without a turn cap. The only misses were conventions the questions left open, split across arms. For a tool whose claim is fewer tokens, make score a guard from the start and pre-register cost; with five runs per arm, the closest cost hypothesis reached 88% of *W*, so a cost claim needs more runs per arm or a tighter task | graphify-monorepo-ab ([ADJUDICATION.md](../../experiments/graphify-monorepo-ab/ADJUDICATION.md)) |

## 4. How to run one

1. **Start from a question you would act on.** Write what you'll do if the hypothesis holds and if
   it fails. If both lead to the same action, don't run it.
2. **Close the friction loop first (L3).** Fix, or explicitly defer, the friction already known
   from pilots and earlier runs, so the benchmark measures what's new.
3. **Choose the rival honestly.** Ask what a capable team would do without Toolbox, and give that
   arm the same tools on PATH. Our record shows the rival is often an agent with a checklist, or
   from scratch.
4. **Pick at least two open-source repositories, of different languages and sizes**, and pin their
   commits. Use exports (`git archive`), so no arm reads history it shouldn't.
5. **Pre-register.** Write the README before the run: the question, the arms, the phases, the
   measures, a hypothesis table with holds and fails, the known limits, and what the result will
   produce. Commit it, or keep it unchanged until the results land.
6. **Build the harness for isolation.**
   - Fresh agents per phase, prompts rendered from one template and committed before the first
     run (P1), and one launch per workspace at a time.
   - The same tool budget in each arm.
   - No network, a read allowlist, and no answer leaks.
   - Kill timed-out process groups, and treat a tool error as an error, never as zero findings.
7. **Measure four things:** correctness, against an independent truth; cost (tokens, calls, time);
   glue or lines changed; and friction. Add reuse cost and convergence when the claim is about
   reuse.
8. **Adjudicate with evidence.** Each judgement goes in a table, with the file, the command or the
   quote that decides it. Judge claims pooled, without regard to which arm made them. The
   adjudicator did not design the experiment, build an arm, or change the product between rounds
   (P7), and writes the fit audit (P5).
9. **Publish everything.** The results and the per-phase costs, against the bar as written. Note
   every deviation from the protocol. Add a row to the research log and a catalog entry.
10. **Triage the friction into fixes**, each fixed, deferred or rejected with its reason, and link
    the fixes from the benchmark's catalog entry (`changed`).
11. **Decide** in `docs/decisions/` when the result changes the plan. A failed hypothesis is a
    result: record it as failed, then rerun after the fixes or close it. A rerun is a new round,
    in which every arm reruns every scored phase (P3).
12. **Retire it** when it has served its value (§6).

**Budget:** a phase has typically cost 90k–230k tokens and 10–60 tool calls. A full two-arm
benchmark has cost 0.5M–1.3M tokens per run, and protocol 2 needs three. Spend that on questions
whose answer changes the plan. Cut phases rather than runs.

### 4.1 Protocol 2

Every benchmark with an agent in its scored path follows these rules
(D01M83RDWQ5EVKY5SFHDWAD5HFH). Benchmarks run before them are protocol 1. Their results stay on
the record as single samples: evidence of friction, not of a claim.

| Rule | Requirement | Catalog field | Why |
|---|---|---|---|
| **P1. Committed prompts** | Every rendered prompt, for every arm and phase, is committed under the experiment's `prompts/` before the first arm starts. A launch whose prompt differs is void | `prompts` | deploy-ab's prompts had to be recovered from agent records |
| **P2. Repeated runs** | At least three fresh runs per arm per phase. Publish every run and the minimum, median and maximum. A cost hypothesis holds or fails only when the medians differ by more than the wider arm's range; otherwise it is inconclusive. A run is dropped only for a harness fault named before the rerun, never for its result | `runsPerArm`, `spread`, `deterministic` | L10, L12 |
| **P3. Frozen rounds** | One commit fixes Toolbox, the harness, and every arm's scaffold, prompts and docs for the round. A fix starts a new round, and every arm reruns every scored phase | `frozenAt` | connector-ab replaced a failing phase with a rerun after an A-only tip |
| **P4. Symmetric arms** | Every arm's scaffold gives the same level of help: a recipe for all, or for none. Rival friction is fixed before the next round as carefully as Toolbox's. Deferring it means the round is not run. Probes treat the arms alike: if one gets an identity, all do | `fitAudit` | L11, L13 |
| **P5. No harness-shaped product** | A product change made after a benchmark names a use outside the harness and no harness file, schema or convention. The harness adapts to the product, not the reverse | `fitAudit` | L11 |
| **P6. A reported measure** | The bar is written in a measure that a dry run showed the runtime reports for every arm, and it is never swapped after the run. If it goes missing, its cost hypotheses are void | `measure` | L14 |
| **P7. Independent adjudication** | The adjudicator applies the bar and writes the outcome. They did not design the experiment, build an arm, or change the product between rounds. Each hypothesis gets one of four labels: holds, fails, inconclusive or void | `designer`, `adjudicator` | Both corrected results were judged by their designers |

`protocol: 2` in the catalog entry declares all of this. `maintainer-benchmark` checks the fields:
a benchmark not yet run must be registered (silver), and a run one must have recorded them
(gold). Benchmarks with no agent in the scored path, like ratchet-replay, set
`deterministic: true` and skip P2.

## 5. Prior art

What the industry does, what we take from it, and where DX Benchmarks differ:

| Source | What it is | What we take | How we differ |
|---|---|---|---|
| **SWE-bench Verified / Pro** (OpenAI, 2024; "Why we no longer evaluate SWE-bench Verified", 2026) | Real GitHub issues with tests; Verified is 500 problems filtered by human reviewers. OpenAI stopped reporting Verified for contamination and flawed tests (at least 59.4% of an audited hard subset), then retracted its Pro recommendation after estimating ~30% of tasks broken | Public repositories leak into training data; tests can reject correct work. We use our own tasks on pinned forks, and adjudicate rather than trust a test | We compare platforms on lifecycle work, not models on bug fixes |
| **Terminal-Bench 2.0 and Harbor** (Laude Institute, 2025) | 89 tasks, each with an instruction, tests and a reference solution, manually verified, run in containers by a harness that works with any installed agent | The task triple (instruction, check, reference), containers, and verifying each task by hand | Our tasks are lifecycle phases (add, change, upgrade, reuse, read), not single terminal tasks |
| **METR's RCT on experienced open-source developers** (2025, arXiv 2507.09089) | 16 developers, 246 tasks on their own mature repositories, randomised to allow or disallow AI. They were 19% slower with AI, while believing they were 20% faster | **Measure; don't ask.** Self-reported speed-ups are unreliable, so we measure tokens, calls and time, and ask for feedback separately | We randomise the platform, not the use of AI, and our subjects are agents |
| **AI Agents That Matter; HAL** (Kapoor, Narayanan et al., 2024; ICLR 2026) | Agent evaluations must be cost-controlled: simple baselines often beat complex agents at a fraction of the cost. HAL runs cost-aware evaluations at scale | **Cost is a first-class result.** Our bars compare tokens at equal correctness | — |
| **ABC: Agentic Benchmark Checklist** (Zhu, Kang et al., NeurIPS 2025) | A checklist for *outcome validity* (does the grader reflect success?) and *task validity* (does the task measure the capability?). Flaws in 7 of 10 popular benchmarks could shift scores by up to 100% relative | The two validities. Every DX Benchmark states how its truth is computed and what each task measures | — |
| **BetterBench** (Reuel et al., Stanford, NeurIPS 2024) | 46 criteria over a benchmark's life cycle (design, implementation, documentation, maintenance). Most benchmarks don't report statistical significance or support replication | Documenting and maintaining benchmarks, and retiring them. Our catalog and its scorecard enforce the documentation | Protocol 2 reports the spread of at least three runs (P2). We still don't claim significance from that |
| **"Demystifying evals for AI agents"** (Anthropic Engineering, 2026) | Code, model and human graders; grade outcomes, not paths. Capability evals start low and graduate into regression suites. pass@k against pass^k. Read the transcripts. Isolate each trial. Start with 20–50 tasks from real failures | Grade the final state, isolate trials, read transcripts, and graduate a passing benchmark into a regression check | — |
| **Inspect** (UK AI Security Institute, MIT, 2024) | Task = dataset + solver + scorer, with sandboxed tool execution | A vocabulary for harness parts: our phases are the dataset, the arms are solvers, and the truth scripts are scorers | — |
| **Next.js Evals and `@vercel/agent-eval`** (Vercel, 2025–2026) | A framework vendor benchmarks coding agents on its own framework: ~50 scenarios with hidden assertions, success, time and cost per agent. It found a bundled AGENTS.md docs index passing 100% against 79% for skills | **The closest precedent:** a platform measuring its agent experience, and changing its product on the result (AGENTS.md now ships in `create-next-app`) | They compare agents on one framework. We compare our platform with the alternatives a team would use instead |
| **Convex LLM leaderboard and convex-evals** (2025) | Models writing Convex code, with and without Convex's guidelines; the guidelines raise success by about 20% (Convex's figure) | The with/without comparison isolates what the platform's own guidance adds | The same difference as above: rival platforms, not rival models |
| **Agent Experience (AX)** (Biilmann, Netlify, 2025) | The discipline of designing products for the agents that use them, as DX is for developers | The term and the framing: our users include agents | AX is a design discipline; DX Benchmarks are how we measure it |
| **The DevEx framework; DX Core 4; DORA** (Noda, Storey, Forsgren, Greiler, ACM Queue 2023; DX, 2024; DORA) | Human DX in three dimensions (feedback loops, cognitive load, flow), plus delivery metrics | Our measures map onto them: tool calls and tokens stand in for cognitive load, phase time for feedback loops, friction lists for what breaks flow | They survey people and telemetry; we run controlled tasks with agents |
| **TechEmpower Framework Benchmarks** | Open, community-contributed implementations per framework, run under published rules, in rounds | Open rules, implementations anyone can inspect, and repeated rounds | Performance, not developer experience |
| **Benchmark saturation** (Epoch AI; "When AI Benchmarks Plateau", 2026) | Nearly half of 60 LLM benchmarks have saturated; retire or revise them when they stop discriminating | Retirement criteria (§6) | — |

**The gap.** Vendors' agent evals (Next.js, Convex) measure models on one product. Research
benchmarks measure models on tasks. Developer-experience research measures people. We found
nothing that measures a developer platform's own lifecycle against the alternatives, with agents,
on open-source repositories, with pre-registered bars and published losses. That gap is what we
fill (our survey, 2026-11-23).

## 6. Lifecycle and the catalog

Every benchmark has an entry in [portal/benchmarks/](../../portal/benchmarks), one YAML file per
benchmark, scored by `maintainer-benchmark` in Toolbox for Toolbox.

| Status | Meaning |
|---|---|
| `designed` | Pre-registered, not yet run |
| `blocked` | Designed, but waiting on a named prerequisite |
| `complete` | Run and published; rerun it when its subject changes (a release, the friction fixes) |
| `archived` | It has served its value and won't be rerun: its question is closed, it was superseded, or it graduated into a regression check |

**Archive a benchmark when:**
- its hypothesis is decided and the decision is recorded;
- a later benchmark asks a better question (L4);
- or it now passes reliably, and is kept as a deterministic regression check instead.

The entry keeps the reason (`servedValue`). An archived benchmark's files stay in `experiments/`,
because its harness and its truth are reused.

**What the scorecard checks:**

| Level | Rules |
|---|---|
| Bronze | The question, the SDLC layers and the status are declared, and every declared path exists |
| Silver | It names a rival arm, its hypotheses and open-source repositories at pinned commits; an archived benchmark says why; a benchmark not yet run is registered under protocol 2 (prompts, runs, measure, an independent adjudicator) |
| Gold | A run benchmark records its outcome and results, and what it changed in Toolbox, and it ran under protocol 2: the frozen commit, the spread, and the fit audit. A protocol-1 run stops at silver |

## 7. Next

| # | Item |
|---|---|
| B1 | A shared harness (`experiments/harness/`) built from the box and security A/B harnesses: setup, tool pins, isolation, cost capture, take-it-out, export before delete |
| B2 | Three runs per arm is now a rule (P2). Still to do: pass^k for anything we claim is reliable |
| B3 | Independent adjudication is now a rule (P7). Still to do: blinding, where a second agent judges pooled, arm-anonymised claims and the two judgements are compared |
| B4 | Benchmarks for the user and package-author layers. Package-author `build-connector` is measured by [connector-ab](../../experiments/connector-ab/README.md): v1 failed C1 and C2, and the reruns are inconclusive. Diagnose→repair feedback is measured by [connector-repair-ab](../../experiments/connector-repair-ab/README.md) (D1–D3), a protocol-1 run. The user layer still has none of its own |
| B5 | Rerun the box A/B after the rt-buddy friction fixes (L3), then the A + S arm of the security A/B, and native Scorecard against the kit |
| B6 | Publish the catalog on the docs site, with each benchmark's bar and result side by side |
| B7 | `deploy-ab`: a laptop deploy of Toolbox against Backstage, scored by catalog probes rather than `/readyz` ([local-simulation.md](./local-simulation.md), Q19). Run twice, inconclusive. Rerun under protocol 2 after the fixes in its [ADJUDICATION.md](../../experiments/deploy-ab/ADJUDICATION.md) |
| B8 | Review `connector drive --format catalog` under P5: keep it if it has a use outside connector-ab's harness, or drop it (D01M83RDWR52WVNER0MV5T84NSX) |
| B9 | Rerun connector-ab under protocol 2, with every arm's recipe written to the same level and Ocean's and Backstage's deferred friction fixed first |
| B10 | gate-repair-ab showed that the ratchet's gate output is cheaper to read (G8, and again G16 in round 4). It did not show that Toolbox beats reviewdog: round 4 lost on calls (G13), and round 5 tied. By L19, the question changed: [pin-repair-ab](../../experiments/pin-repair-ab/README.md) asked it on a task whose fixes need pins from other files. There, in round 2, Toolbox beat reviewdog on read (N7: median 5,794 vs 18,300 characters), with calls and gate read inconclusive, after D01M84X4JDHXS98K89KN4AB0RCD and a replay measure (L20). It is narrow: N7 holds by 1,073 characters over *W*, and it depends on the replay. Still to do: the same question on a second repository and on a real pull request, and a hint naming the jobs a mistyped `needs:` could mean ([gate-repair-ab README](../../experiments/gate-repair-ab/README.md) §0.6) |

## Sources

- OpenAI, "Why we no longer evaluate SWE-bench Verified" (2026-02-23) and "Separating signal from noise in coding evaluations"; openai.com/index/why-we-no-longer-evaluate-swe-bench-verified
- Terminal-Bench 2.0 announcement (2025-11-07), tbench.ai/news/announcement-2-0; `harbor-framework/harbor`; arXiv 2601.11868
- METR, "Measuring the Impact of Early-2025 AI on Experienced Open-Source Developer Productivity" (2025-07-10), arXiv 2507.09089
- Kapoor, Stroebl, Siegel, Nadgir, Narayanan, "AI Agents That Matter" (2024), arXiv 2407.01502; "Holistic Agent Leaderboard" (ICLR 2026), arXiv 2510.11977
- Zhu, Kang et al., "Establishing Best Practices for Building Rigorous Agentic Benchmarks" (NeurIPS 2025), arXiv 2507.02825
- Reuel et al., "BetterBench" (NeurIPS 2024), arXiv 2411.12990; betterbench.stanford.edu
- Grace, Hadfield, Olivares, De Jonghe, "Demystifying evals for AI agents" (Anthropic Engineering, 2026-01-09), read in full
- Inspect, inspect.aisi.org.uk (UK AI Security Institute, MIT, 2024)
- Next.js Agent Evals, nextjs.org/evals; `vercel-labs/agent-eval`; Vercel, "AGENTS.md outperforms skills in our agent evals"; Next.js 16.2 release post
- Convex, convex.dev/llm-leaderboard; `get-convex/convex-evals`; stack.convex.dev/convex-evals (the 20% figure is Convex's)
- Biilmann, "AX and why agent experience matters" (Netlify, 2025-01); netlify.com/agent-experience
- Noda, Storey, Forsgren, Greiler, "DevEx: What Actually Drives Productivity" (ACM Queue, 2023); Forsgren et al., "DevEx in Action" (CACM)
- TechEmpower Framework Benchmarks, `TechEmpower/FrameworkBenchmarks` (secondary: read through forks and round posts)
- "When AI Benchmarks Plateau: A Systematic Study of Benchmark Saturation" (2026), arXiv 2602.16763; Epoch AI benchmarking hub (secondary)
- This repository: [repo-value.md](./repo-value.md), [experiments/](../../experiments/README.md), [research-log.md](../research-log.md)

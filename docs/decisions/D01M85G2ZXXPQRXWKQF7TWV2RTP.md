---
id: D01M85G2ZXXPQRXWKQF7TWV2RTP
decided: 2026-11-24T00:42:57Z
---

# Graphify's value is not shown, not absent: the no-adoption verdict is restated, and a cost result can reopen it

**Toolbox for Toolbox still doesn't carry Graphify, and Toolbox still builds no code knowledge
graph, but for narrower reasons than D01M84ZW6K2V9CV2SD644DE5M0S gave. For a strong agent with no
turn cap, answering questions, Graphify didn't make answers more complete on any of three
repositories. Whether it lowers cost is not settled: on Toolbox, every Graphify run made fewer
calls and read less than every baseline run, which the registered rule called inconclusive and a
post-hoc rank test doesn't (p = 0.008). That saving is in characters, not billed tokens, and the
cost of keeping a document graph current was never measured. A round that measures both reopens
the decision.** This replaces the reasons in D01M84ZW6K2V9CV2SD644DE5M0S items 1–3; its items 4
and 5 stand. The red team and its triage are [graphify-red-team.md](../graphify-red-team.md) and
[graphify-red-team-response.md](../graphify-red-team-response.md).

1. **What the evidence shows.**

   | Question | Answer | Evidence |
   |---|---|---|
   | More complete answers? | **No**, for a strong model with no turn cap; coverage tied everywhere and had no headroom | graphify-ab (Toolbox, OpenFGA), graphify-monorepo-ab (Codex) |
   | Fewer calls and less read? | **Likely on Toolbox, not on OpenFGA or Codex.** Toolbox: 25% fewer calls and 29% less read by median, complete separation (post hoc, exact p = 0.008 each). OpenFGA: fewer calls (p = 0.14), more read. Codex: no difference (p ≥ 0.22) | Both `results/r1.tsv` |
   | Cheaper in billed tokens? | **Not measured.** The runtime reports no billed tokens per run ([Q21](../questions.md)), and each call re-sends context, so the character count understates the saving from fewer calls | L14; red team R4 |
   | Cheap to keep current? | **Not measured.** The first document build took 17 agents and 4.27M characters for this repository. Graphify re-extracts only changed files, and a commit here touches about 1.4% of the documents; the per-commit cost is unknown | graphify-ab §0; red team R5 |
   | Its impact command (`affected`)? | **Never exercised.** The vendor's Cursor rule doesn't name it, and none of 35 Graphify runs used it | red team R3 |
   | Its headline token ratios (71.5x)? | **Not reproduced** by this or any independent study | knowledge-graphs §3 |

2. **Why the outcome holds for now.** Adopting means an always-on rule in every session and a
   document graph that 141 of 200 commits make stale, and neither cost was measured. CAFE(S)
   puts fidelity before brevity, and independent billed-token studies under Graphify's hooks
   found higher cost. A post-hoc result in a proxy measure on one repository is not enough to
   adopt, as an inconclusive one was not enough to reject. The other options keep their verdicts:
   no other code-graph tool has an independent result of agent gains, and an extracted graph is
   open-world with ids that change between builds, so a scorecard can't be computed over it.
3. **Reopen when** any of these happens. Each reruns graphify-ab's harness on Toolbox under
   D01M85G2YXV2VRTZWQJ33MHXCKN.
   - **Cost.** A round with billed tokens per run (or, if Q21 answers no, calls and read registered
     together), `affected` named in Graphify's rule (and `graph --relation` in a Toolbox arm's, if the round
     has one), and the
     `--update` cost measured over ten real commits, shows at least 20% fewer tokens per session,
     including the amortized update cost, at equal coverage. That adopts Graphify in Toolbox for
     Toolbox.
   - **Headroom.** A cheaper model, or a question set that leaves the baseline headroom, shows a
     coverage gain under no turn cap.
   - **Maintenance.** Graphify, or a tool like it, keeps the document layer current without an
     LLM pass over each changed document.
4. **What is retracted.** "Showed neither helps nor hurts" and "didn't clear the noise" overstate
   an inconclusive label under a rule that detects a two-standard-deviation effect about a quarter
   of the time. "Its questions (where is X, what calls it) are answered by `rg` in a call or two"
   is contradicted by graphify-monorepo-ab, where the baseline took a median of 56 calls for four
   such questions.

**Status:** ✅

**Where:** [docs/design/knowledge-graphs.md](../design/knowledge-graphs.md) §1, §3, §5, §6, §7;
`portal/benchmarks/graphify-ab.yaml`, `portal/benchmarks/graphify-monorepo-ab.yaml`

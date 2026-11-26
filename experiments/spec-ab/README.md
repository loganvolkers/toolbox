# The Spec A/B: Results by Hand, with Seikika, or on Backstage

**Question:** with a well-specified results format ([results-spec.md](../../docs/design/results-spec.md)),
can agents produce and consume a repository's standards results reliably *without* Seikika? What
does Seikika add on top of the spec, and what does building on Backstage cost for the same work?

**Status: designed, not run.** It needs three things first (§6):
- the spec's v0.1 draft, with schemas and a conformance suite;
- a decision on Q17;
- a Backstage feasibility spike. The session's permission policy refused running
  `npx @backstage/create-app`, so the spike needs the user's approval.

This tests the box's contract, not a tool ([seikika-and-tools.md](../../docs/design/seikika-and-tools.md)).

## 1. Why three arms

The box A/B ([../box-ab](../box-ab/README.md) §0) found that both arms were fully correct, the box
used 11% more tokens, and most of that went to learning Seikika. Two follow-ups:
- **If the spec is the thing that matters, test the spec without Seikika.** Arm A gets only the spec.
  If it reaches conformance cheaply, the spec stands on its own. Seikika's value is then measured
  as the difference between arms A and B.
- **Backstage is what most teams would reach for.** An arm that builds on it, with the catalog,
  Tech Insights fact retrievers and checks, tells us whether a platform helps repository-level work
  or gets in its way. Tech Insights is an in-process TypeScript API, not a file format
  ([results-spec.md](../../docs/design/results-spec.md) §3), so this arm also tests the
  "no-code platform" hazard: the contract is the platform's code.

## 2. Arms

| | A: the spec by hand | B: Seikika | C: building on Backstage |
|---|---|---|---|
| Given | The spec, its JSON Schemas, examples, and `spec-validate` | The same, plus `seikika`, its docs and kits | The same as A, plus a Backstage app created at pinned versions with the Tech Insights backend and its JSON-rules check module |
| Builds | Producers and consumers in any language | A portal; Seikika writes the documents | Fact retrievers and checks in Backstage, and whatever turns their results into spec documents |
| Tools | semgrep, ruff, actionlint and zizmor at the box A/B's pins, on the same PATH | The same | The same |

**Pinned for arm C:**
- `@backstage/create-app` 0.9.4 and `@backstage-community/plugin-tech-insights-backend` 3.1.0, both
  resolved from npm on 2026-11-22.
- Node 22.23.5 and Yarn 1.22.22, as installed in this environment.
- The app is created and installed in setup, so arm C's agent doesn't pay for the install. The
  install is measured separately (time, disk), because a team would pay it.

## 3. Phases

| # | Task | Scored by |
|---|---|---|
| 1 | **Produce.** For the PostHog fork, write facts, findings (SARIF) and evaluation documents for the four tools, against a given standard document | Conformance suite (Producer class); evaluations against the box A/B's truth (`harness/ab.py truth`) |
| 2 | **Ratchet.** Given a baseline document and a planted pull request, write the verdict, and wire it into CI as a gate | Conformance (Ratchet class); the box A/B's planted cases |
| 3 | **Consume.** From evaluation documents of three repositories (the PostHog, Codex and gh-aw forks), write an organisation rollup: each repository's level, regressions and campaign progress | Conformance (Consumer class); a rollup key |
| 4 | **Read.** Answer the box A/B's phase 6 questions from spec documents alone | The box A/B's read key |

**Measured per phase:** tokens, tool calls and time; lines of glue; conformance results per
requirement; correctness.

## 4. The bar, set before running

| Hypothesis | Holds if | Fails if |
|---|---|---|
| **S1. The spec stands alone** | Arm A passes the conformance suite in phases 1 and 2 within the budget | Arm A can't reach conformance: the spec is too big or unclear, and v0.1 shrinks before anything else is built |
| **S2. Seikika adds value over the spec** | Arm B uses fewer tokens than arm A over phases 1–4, with equal conformance | Arm B uses as many or more: Seikika's value is the reference implementation, not saved work |
| **S3. A platform costs more at the repository** | Arm C uses more tokens and leaves more glue than arm B for phases 1–2 | Arm C is cheaper: repository-level work should build on Backstage, and the plan says so |

## 5. What is reused from the box A/B

- **Setup:** `../box-ab/harness/setup.sh`, the tool pins, the semgrep wrapper, and the PostHog fork
  at `153f1b8d`.
- **Truth and planted cases:** `ab.py truth`, `CASES`, and the read key.
- **The lessons:**
  - Each phase starts a fresh agent, and the prompts are rendered from one template.
  - Results are written to scratch and moved in when complete.
  - The harness kills a timed-out tool's process group.
  - A tool error is never read as zero findings.
  - Each arm's diff is exported before its working copy is deleted.

## 6. Before running

| # | Prerequisite | State |
|---|---|---|
| P1 | Q17 decided: write the spec, its name, and whether it is a Gemara profile | Open |
| P2 | Spec v0.1: schemas for the seven document types, numbered requirements, a conformance suite with cases per class, and a validator | Not started |
| P3 | Backstage spike: create the app at the pins, install Tech Insights, start the backend without network access, run one fact retriever and one check; record time, disk and what broke | **Blocked**: the permission policy refused `npx @backstage/create-app` (it downloads and runs code). Needs the user's approval, or a session where it is allowed |
| P4 | A read allowlist for arm C: Backstage's docs from `backstage/backstage` and `backstage/community-plugins` (backstage.io may be blocked) | Not started |
| P5 | Remove answer leaks, as in the box A/B (§3 there) | Not started |

## Sources

- [docs/design/results-spec.md](../../docs/design/results-spec.md): the spec and its prior art
- [experiments/box-ab](../box-ab/README.md): the harness, truth and lessons reused here
- npm registry, 2026-11-22: `@backstage/create-app` 0.9.4, `@backstage-community/plugin-tech-insights-backend` 3.1.0

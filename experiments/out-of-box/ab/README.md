# A/B: Do Agents Raise Standards Further With Toolbox?

**Question** ([repo-value.md](../../../docs/design/repo-value.md) R3): does a coding agent raise a
repository's standards further with `toolbox report` and its ratchet than with the same standards
written down?

**Answer (2026-11-21, two repositories, one run per arm): not on this sample.** The checklist arm
fixed as much or more: 86% against 83% on OpenFGA, and 100% against 82% on Codex. R3's bar was 15
points more with Toolbox. The Toolbox arm introduced no new failures and suppressed nothing. The
checklist arm introduced one new failure and, on Codex, reached 100% partly through a config file
that ignores 31 zizmor findings, with reasons.

## Method

| | Arm A: checklist | Arm B: Toolbox |
|---|---|---|
| Prompt | [prompt-checklist.md](prompt-checklist.md): the rules and fix texts, written out | [prompt-toolbox.md](prompt-toolbox.md): `toolbox report`, with evidence, and the ratchet |
| Tools | zizmor 1.32.0 and actionlint 1.7.12 (the checklist names both) | The same, plus `toolbox` |
| Budget | 40 tool calls; no network, commits or deleted functionality | The same |
| Agent | The same general-purpose agent and model | The same |

Each arm worked on its own copy of a fork: OpenFGA with [../portal](../portal/toolbox.yaml), and
Codex with [../portal-pnpm](../portal-pnpm/toolbox.yaml). Afterwards every copy was scored with
`toolbox report`, leaving out the six run-history rules, which can't be met offline. The diffs are
here as patches.

## Results

| Arm | Failing before | After | Fixed | New failures | Ignores added | Tool calls | Tokens | Wall time |
|---|---|---|---|---|---|---|---|---|
| OpenFGA A | 42 | 6 | 36 (86%) | 0 | 3 inline | 15 | 114k | 180 s |
| OpenFGA B | 42 | 7 | 35 (83%) | 0 | 2 inline | 11 | 120k | 124 s |
| Codex A | 67 | 1 | 67 (100%) | 1 | 3 inline, `.github/zizmor.yml` (31 findings), an actionlint runner-label config | 24 | 127k | 292 s |
| Codex B | 67 | 12 | 55 (82%) | 0 | 0 | 26 | 150k | 361 s |

No arm deleted a workflow or job. Codex A's new failure: moving `cla.yml`'s write permissions from
the workflow into its job exposed a write token on a risky trigger, which a ratchet in CI would
have caught. The two OpenFGA arms made nearly the same changes. Arm A also restricted the
`issue_comment`-triggered Claude review to repository members, a behaviour change it reported.

## Reading it

- **Most of these rules wrap two linters, and the checklist named them.** An agent that can run
  zizmor and actionlint gets much the same evidence that Toolbox shows. That comparison is fair,
  and it is where Toolbox adds the least. Its own facts (CODEOWNERS, docs links, pnpm, action
  pinning across workflows) were mostly passing already: 2 of OpenFGA's 42 failures and 1 of Codex's 67. Both arms
  added Codex's CODEOWNERS catch-all, and neither could pin OpenFGA's `@main` action offline. So
  this test barely exercised them.
- **What Toolbox did add here was a guardrail, not speed:** no regressions and no suppressions.
  Whether that matters is R4's question, the ratchet on real pull requests, not R3's.
- **A second A/B tests the rest of the lifecycle:** writing, sharing, changing and reading a
  standard, not moving one linter's count ([../../standards-ab](../../standards-ab/README.md)).
- **The sample is small:** two repositories and one run each, scored by the tool one arm used.

## Sources

- [prompt-checklist.md](prompt-checklist.md), [prompt-toolbox.md](prompt-toolbox.md), and the
  four patches here
- [../README.md](../README.md): the out-of-box portals

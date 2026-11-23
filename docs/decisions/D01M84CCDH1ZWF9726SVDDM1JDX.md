---
id: D01M84CCDH1ZWF9726SVDDM1JDX
decided: 2026-11-23T15:20:00Z
---

# A gate lists each new finding once, under its entity, and a lint rule's finding says what to change to

**When `toolbox report --baseline` (or `--against`) fails, its summary lists each new finding on
one line, grouped by the entity it is on. That line names every rule that reports the finding. The
summary no longer prints one block per rule, with that rule's evidence and the record the entity
came from. The `github-actions` package also gives the same finding the same text across its
rules. `zizmor-clean` no longer counts zizmor's `unpinned-uses`, because `pinned-to-sha` already
reports it. `pinned-everywhere` names the SHA that the repository already pins an action to.** It
amends the summary that [D01M841J2SN4V3AVZZ5D91YAR51](./D01M841J2SN4V3AVZZ5D91YAR51.md) and
[root-causes](../design/root-causes.md) describe. Keys, the `--verdict` document and the
annotations don't change.

1. **Why.** In round 3 of [gate-repair-ab](../../experiments/gate-repair-ab/README.md), the
   Toolbox gate's first output was 2,892 characters long for 7 regressions. The reviewdog gate
   needed about 1,100 characters for the same defects. Three things made up the difference:
   - one zizmor finding appeared under two or three rules, each with its own block;
   - every block ended with a `from: connector …` line naming the entity's first source record,
     which was not always the record that the rule read;
   - the zizmor items gave no severity or rule, only the line and message.

   Every run of both arms then searched the repository for the SHA that each action was already
   pinned to elsewhere. That search was the largest part of what the agents read after the gate.
2. **The summary.**

   ```
   ✗ worse   workflow:.github/workflows/pull_request.yaml · maintainer-ci-security
       no-high-findings, zizmor-clean: + high template-injection at line 40: code injection via template expansion: …
       pinned-to-sha: + actions/checkout@v6
   ✗ worse   action:actions/checkout · maintainer-action-deps
       pinned-everywhere: + actions/checkout@v6, pinned elsewhere as actions/checkout@de0fac2e…
   and 12 finding(s) already in the baseline
   ```

   - **Which regressions are listed this way:** those whose evidence is a list of new items, that
     is, a rule that now fails or a grown list whose baseline has keys. Every other regression
     keeps its `✗ worse … now fails` block and its cause chain. That covers an aggregate rule, a
     measure, a grown list without keys and a new entity below the floor.
   - **Folding:** a finding with the same text under several rules is one line naming them all. A
     rule on another entity is named `rule (entity)`, and that entity gets no header of its own when
     all of its findings are already listed.
   - **What stays:** the `reproduce:` command stays under its entity. The source record moves to
     the `--verdict` document, which keeps every regression's full `detail`.
   - **The output:** on the gate-repair-ab planted pull request the summary is 1,541 characters, down
     from 2,892. Two of the removed blocks repeated a finding, and the source lines named the wrong
     record.
3. **The package's text.** The `github-actions` rules that list zizmor findings (`no-high-findings`,
   `no-medium-findings` and `zizmor-clean`) write each one as
   `<severity> <rule> at line <n>: <message>`, so the same finding folds into one line. `zizmor-clean`
   skips `unpinned-uses`, so one defect fails one rule. `pinned-everywhere` names the SHA the
   repository already pins an action to, as `<ref>, pinned elsewhere as <action>@<sha>`, through a new
   computed field, `shaRefs`. That is the fix nearly every agent searched for.
4. **Uses outside the benchmark (P5).**
   - Any repository's bare `--baseline` gate, `--against` comparisons, and the stderr summary of the
     `actions/report` CI log. This repository's own ratchet is an example: a long new function now
     reads `short-functions: + gate at src/causes.rs:209: 82 lines` under its crate.
   - Every portal that includes `packages/github-actions`, including Toolbox for Toolbox and the
     pilot repositories that use the starter.

   No harness file changed with this decision.
5. **Not decided here.** Folding is by exact text, so findings that two tools report differently
   (actionlint's and zizmor's template injection) stay separate lines. A finding's identity across
   tools needs fingerprints that agree across tools, which
   [results-spec.md](../design/results-spec.md) leaves open.

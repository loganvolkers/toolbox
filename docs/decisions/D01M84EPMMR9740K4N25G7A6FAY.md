---
id: D01M84EPMMR9740K4N25G7A6FAY
decided: 2026-11-23T15:10:00Z
---

# A failing gate quotes the lines its new findings point at, and a pin hint carries the pin as written

**When `seikika report --baseline` (or `--against`) fails, each entity with a `path` field ends its
new findings with the lines of that file the findings name (`at line N`), read from the working
tree. The `github-actions` package writes each unpinned `uses:` as `<ref> at line <n>`, and
`pinned-everywhere` names the SHA pin the repository already writes, with its version comment
(`actions/checkout@<sha> # v6.0.2`).** It amends the summary that
[D01M84CCDH1ZWF9726SVDDM1JDX](./D01M84CCDH1ZWF9726SVDDM1JDX.md) describes. The `--verdict`
document and the annotations don't change.

1. **Why.** In round 4 of [gate-repair-ab](../../experiments/gate-repair-ab/README.md), the
   repair took arm A a median of 7 calls, against reviewdog's 3. Half the gap was exploration
   after the gate:
   - four of five Seikika runs read `pull_request.yaml` before editing it;
   - three made a separate lookup for the version comment the existing pins carry, because the
     SHA hint left it out.

   Every run in both arms printed parts of the changed files, and every diff copies the comments.
2. **The quote.**

   ```
   ✗ worse   workflow:.github/workflows/release.yaml · maintainer-ci-security
       pinned-to-sha: + actions/upload-artifact@v7 at line 88
       .github/workflows/release.yaml:
         88 |         uses: actions/upload-artifact@v7
   ```

   - **Which lines:** every `at line N` in the entity's new findings, as zizmor's, actionlint's and
     the pin rule's items write it. Gaps of up to five lines between them are filled, and longer
     gaps are shown as `⋮`. At most 16 lines are quoted per entity, each cut at 120 characters.
   - **Where from:** the entity's `path`, relative to the working directory. That is where the
     workspace connector read it (`root: .`). An entity without a `path`, or whose file isn't
     there, quotes nothing.
   - **Only on a failing gate:** the quote comes after the `reproduce:` line, and only new findings
     are quoted.
   - **The output:** on the gate-repair-ab planted pull request the summary is 2,091 characters,
     up from 1,529. It quotes 11 lines.
3. **The package's text.** The workspace connector now reads the text of each workflow as well as
   its YAML. It records two new fields:
   - `unpinnedAt`, each unpinned `uses:` as `<ref> at line <n>`;
   - `shaPins`, each action's SHA refs as written, with the comment after them, commented ones
     first.

   The `pinned-to-sha` evidence reads `unpinnedAt`. `pinned-everywhere` reads `shaPins`, which
   replaces the computed `shaRefs`. The baseline keys of `pinned-to-sha` and `pinned-everywhere`
   items change once, because their text changed, so a baseline written before this decision
   counts each of those findings as new until it is updated with `--update`.
4. **Uses outside the benchmark (P5).**
   - Any repository's bare `--baseline` gate and `--against` comparisons. This repository's own
     ratchet is an example: an unpinned `actions/checkout@v6` planted in `ci.yml` is quoted at its
     line, and its hint reads `pinned elsewhere as actions/checkout@11d5960a… # v4`.
   - Every portal that includes `packages/github-actions`, including Seikika for Seikika and the
     pilot repositories that use the starter.

   No harness file changed with this decision.
5. **The rival gets the same.** reviewdog's local reporter prints each linter's own output, and
   zizmor's and actionlint's default formats already quote the source. Round 5's reviewdog gate
   reads those formats instead of the one-line ones (P4), so both arms see the code.
6. **Not decided here.** The quote shows what the line says now, not what to change it to. A gate
   that proposes the edit (zizmor's `--fix`, scoped to the new findings) is left open: `--fix`
   rewrites every finding in a file, old debt included.

---
id: D01M84X4JDHXS98K89KN4AB0RCD
decided: 2026-11-23T19:11:46Z
---

# A pin hint is written on the unpinned ref's own path, at the version its tag names

**`pinned-everywhere` names, for each ref that isn't a SHA, the pin to write instead: one of the
action's SHA pins, on that ref's own path, with its version comment. Among the SHA pins it prefers
one whose comment is within the ref's tag (`v5` → `v5.0.5`), then one on the same path, then one
with a comment.** It amends point 3 of
[D01M84EPMMR9740K4N25G7A6FAY](./D01M84EPMMR9740K4N25G7A6FAY.md), which took the first of the
action's SHA pins as written.

1. **Why.** An action entity covers a whole repository, so `actions/cache`, `actions/cache/restore`
   and `actions/cache/save` share one list of SHA pins. In openfga, the first of them was
   `actions/cache/restore@27d5… # v5.0.5`, so `actions/cache@v5` got a hint naming a different
   action. Copied as it stands, it replaces a cache that restores and saves with one that only
   restores. The same list can hold pins of other majors: a hint for `actions/checkout@v4` named
   whichever SHA sorted first, not the v4 one. Found while preparing
   [pin-repair-ab](../../experiments/pin-repair-ab/README.md), before any run.
2. **What changes.**
   - The workspace connector's `action-ref` records gain `pinHints`: each ref that isn't a SHA,
     as `<ref>, pinned elsewhere as <pin>` with the pin chosen as above, or the ref alone when the
     repository pins the action nowhere. A SHA belongs to the repository, not the path, so writing
     it on the ref's own path keeps the action and stays a real commit. The connector writes the
     text, as it does for `unpinnedAt`: a map from ref to pin would be an `object` field, which
     has no declared shape (`meta-field`'s `has-shape`).
   - `pinned-everywhere`'s evidence is `pinHints`, where it was built from `shaPins[0]`.
   - `shaPins` stays.
   - An item's text changes only where the old hint named another path or version. A baseline
     written before this decision counts each such finding as new until it is updated with
     `--update`.
3. **Uses outside the benchmark (P5).** Any repository that pins sub-actions (`actions/cache/*`,
   `github/codeql-action/*`, `anchore/sbom-action/*`) or several majors of one action, through
   every portal that includes `packages/github-actions`. No harness file changed with this
   decision.

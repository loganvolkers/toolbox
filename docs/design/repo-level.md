# Repository Level: The Lowest Constituent Scorecard

*Status: scorecard built, 2026-11-20 (D86). `maintainer-repo` reads the other scorecards on a second
derive pass and takes the lowest level. A weighted 0–10 is not the gate. Related:
[scorecards-for-scorecards.md](./scorecards-for-scorecards.md),
[supply-chain-standards.md](./supply-chain-standards.md),
[federation.md](./federation.md).*

---

## 1. Conclusion

**A repository is as high as its lowest counted scorecard.**

The maintainer SDLC already has one scorecard per kind of file. A single number that averages them
hides the file that failed. The OpenSSF scorecard publishes a 0–10; this scorecard publishes the
level a reader already knows: none, bronze, silver or gold.

| Question | What answers it | In this repo |
|---|---|---|
| Does every counted scorecard reach bronze? | The minimum rank among the constituents | bronze |
| Does every counted scorecard reach silver? | The same minimum | silver |
| Does every counted scorecard reach gold? | The same minimum | gold |

Ranks use the position in each declared ladder. A constituent that stops at
bronze or silver caps the repository at that level even when all its rules
pass. For example, `maintainer-action-deps` has only bronze and
`maintainer-docs` stops at silver. Reaching a higher repository level with
those constituents would require a change to their ladders or to which
scorecards the repository declares in `policy.repoRollup`.

Unknown rules are left out of a constituent's level before the minimum, so a missing runner does
not force the repository to none. Fixture entities whose id contains `/demo/` are left out, so the
ladder fixtures do not drag the live repository down. Runner-backed cards stay off the list for
the same reason: with the runner skipped they have no level.
If a watched name does not identify a declared constituent, the repository
falls below bronze and the evidence names the unresolved entry.

## 2. What the field says

| Source | What it decides | Adopted |
|---|---|---|
| **OpenSSF Scorecard** | One number per repository, from weighted checks | The aggregation is adopted. The weights are not. The level is the minimum |
| **Federation** | An org's level is the lowest level any repo reaches | This is the one-repository case of that rule, computed in the same derive rather than across portals |
| **Scorecards for scorecards** | Recursion stops at one meta level | A constituent that itself reads `cards` is not a constituent |

## 3. How it is computed

Scorecards whose check, evidence or `appliesTo` contains the token `cards` wait for a second pass.
The first pass scores everything else. The second pass binds `cards` to the constituents named in
the entity's `watch` list, or in `policy.repoRollup` when that list is empty.

`policy.repoRollup` is `maintainer-docs`, `maintainer-ci-security`, `maintainer-action-deps`,
`maintainer-skill`, `maintainer-connector`, `maintainer-dependency`, `maintainer-agent-doc`,
`maintainer-package`, `maintainer-schema`, `maintainer-example`, `maintainer-plugin` and
`maintainer-supply-chain`. `maintainer-crate`, `maintainer-ci-reliability` and `user-mcp` are
runner-backed and excluded.

A fraction of constituents at each level can be read from the evidence. The ratchet compares the
level.

## 4. Where this repository lands

The live supply-chain entity is none (D85), and `maintainer-action-deps` stops at bronze. The
repository level is therefore none until those constituents rise. Fixtures watch one card each:
`repo:acme/focused` watches the gold plugin and is gold, `repo:acme/partial` watches a bronze
connector and is bronze, `repo:acme/unsafe` watches supply-chain and is none (D71).

## Sources

- OpenSSF Scorecard, which publishes one score per repository.
- [federation.md](./federation.md) §5, the lowest-level rule this is the one-repository case of.
- [scorecards-for-scorecards.md](./scorecards-for-scorecards.md), which stops recursion at one meta
  level.

# Repo Portals: Toolbox Without a Server

*Status: built 2026-11-20 (D66); the static-site extensions in §5 are proposals. A **repo portal**
is a Toolbox manifest in one repository, run by `toolbox report` locally and in CI. Toolbox for
Toolbox ([portal/](../../portal/README.md)) is the first one. It is the leaf of
[federation](./federation.md) and the first step of the go-to-market path.*

---

## 1. Conclusion

A team can adopt Toolbox in one repository with no infrastructure:
- a `toolbox.yaml`;
- in-image connectors that read the checkout;
- a committed baseline;
- one CI step.

They get the same scorecards a fleet portal would run, with evidence an agent can act on, a
Betterer-style ratchet, and a single HTML page to publish. When the company later runs a central
portal, these repos are already its children ([federation.md](./federation.md) §4, option A).

| Capability | Command | State |
|---|---|---|
| Score the repo | `toolbox report [--skip <connector>] [-o md\|json\|html] [--out FILE]` | ✅ |
| Ratchet | `--baseline FILE` (+ `--update`, `--ci`) | ✅ ([ratchets.md](./ratchets.md)) |
| Evidence for failing rules | rule `evidence:` CEL, shown by `report`, `scorecard_status`, `explain`, `scorecard_report` | ✅ |
| Work lists | `toolbox scorecard <name> --rule R` / MCP `scorecard_report` (needs `serve`) | ✅ |
| Live, agent-driven loop | `toolbox serve` + `toolbox mcp`, with refreshes that report what changed | ✅ |
| Static publishing | `report -o html`: one self-contained page | ✅ (§4); a multi-page site is a proposal (§5) |
| Readers for existing ratchets | Betterer results files; ESLint, PHPStan and basedpyright baselines | Betterer ✅; others proposed (ratchets.md R2) |

## 2. What runs where

```
toolbox report portal --baseline portal/results.json --ci
  ├─ for each in-image connector (in parallel, except --skip): spawn, send one resync, collect records
  ├─ full recompute (the same pipeline as serve)
  ├─ compare with the baseline: worse → exit 1; stale under --ci → exit 1; --update → write
  └─ print md / json / html
```

- **Remote connectors are skipped** with a note, since they push to a server.
- **Native values** (fields edited in a portal's UI) don't exist without a server. A repo portal's
  facts come only from its files and commands.
- **There is no authorization.** `report` runs as whoever runs it and shows everything (§4).

## 3. Making it cheap in CI

| Concern | Approach |
|---|---|
| Slow connectors (tests, builds) | Split them into their own connector and `--skip` them in the ratchet step. Toolbox for Toolbox runs `workspace` + `surfaces` (about 1 s), and its `runner` (tests, clippy, MCP trace) only in full runs |
| Deterministic baselines | Results are pure functions of the files. Keep time-dependent data (`lastChange`, durations) out of rules that the baseline records |
| Merge conflicts in the baseline | Rerun `--update` on the merged tree; never hand-merge |
| Pre-commit | A hook running `toolbox report --skip runner --baseline … --update` (documented; packaging is ratchets.md R4) |

## 4. One page to publish: `report -o html`

**What it is.** A single self-contained HTML file. It embeds:
- the report as JSON;
- an inline script (about 60 lines) that renders scorecard tabs, level bars, a filter, and the
  failing rules per entity with their fixes and evidence;
- light and dark themes;
- a layout that works at phone width.

There are no external requests. Toolbox for Toolbox's page is about 38 KB.

**Why one file.** Storybook's `storybook build` emits a static directory that teams host on GitHub
Pages or Chromatic (secondary). A repo portal wants the same "build once, host anywhere" property
with even less: one file can be:
- a CI artifact;
- a GitHub Pages page;
- an attachment on a PR comment;
- a shared Claude artifact;

and it never goes stale relative to the commit it describes.

**Publishing recipes**

| Where | How |
|---|---|
| CI artifact | `toolbox report … -o html --out report.html` and upload it |
| GitHub Pages | The same, in a Pages workflow on the default branch |
| PR | Upload it as an artifact and link it from the ratchet's summary comment |

**Visibility caveat.** The page shows *everything*, because `report` has no caller and applies no
facets. That is right for a repo portal, whose readers can read the repo anyway. It is wrong for a
central portal's data. A central portal must not publish `report` output for hidden facets. If static
publishing of central data is ever needed, it would render *as* a principal
(`--as group:everyone`), applying the same shaping as the API.

## 5. Proposed: a static site, and static federation

| # | Proposal | Why |
|---|---|---|
| S1 | `toolbox report -o site --out DIR`: an index plus one page per entity (fields, provenance, relations, scores, evidence) | Browse a repo portal like the minimal UI, offline |
| S2 | **Static federation:** an index page that loads several children's `report -o json` files (by URL or path) and shows them together, like Storybook composition (secondary) | A "parent" with no server, the minimal answer to [federation.md](./federation.md) FQ5 |
| S3 | Embed the snapshot id and git commit in the page; link entities to files at that commit | Every claim on the page is checkable |

## Sources

- Implementation: `crates/toolbox-cli/src/report.rs`, `report.html`;
  [toolbox-for-toolbox.md](./toolbox-for-toolbox.md); [ratchets.md](./ratchets.md).
- Storybook static builds and composition: Storybook documentation (secondary; not re-read in this
  session).

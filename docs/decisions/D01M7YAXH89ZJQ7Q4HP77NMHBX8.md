---
id: D01M7YAXH89ZJQ7Q4HP77NMHBX8
decided: 2026-11-21T05:58:19Z
---

# The site's portal report is generated at build time, not committed

**The pages workflow generates `site/public/report` for each commit; the repository no longer commits it.** This amends D92, which committed the report and had CI compare it with a fresh run byte for byte. That comparison failed `main` three times in two days: twice when an agent regenerated the report without the pinned zizmor and actionlint, which turned their rules `unknown`, and once when scores moved and nobody regenerated it. It also made the files conflict on every rebase that crossed another score change.

The pages build now builds Seikika, installs actionlint (SHA-256-pinned, `scripts/ci-tools.sh`) and zizmor 1.32.0, runs `seikika doctor portal`, which fails the build if a connector or pinned tool is missing, and then `seikika report portal --skip runner,github-actions -o web --out site/public/report`. Pull requests run the same build, so a report that can't be generated still fails review. `portal/results.json` stays committed: it is the ratchet's baseline, not a published artifact.

**Status:** ✅

**Where:** [.github/workflows/pages.yml](../../.github/workflows/pages.yml), [portal/README.md](../../portal/README.md#layout)

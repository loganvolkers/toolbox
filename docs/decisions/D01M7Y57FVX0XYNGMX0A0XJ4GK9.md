---
id: D01M7Y57FVX0XYNGMX0A0XJ4GK9
decided: 2026-11-21T04:18:29Z
---

# First-customer scorecard fixes: personal CODEOWNERS and pnpm workspaces

**Repo-portal silver and npm package standards must work for a personal GitHub repository and a pnpm workspace, not only for org teams and per-package npm lockfiles.**

1. **CODEOWNERS silver.** `owners-are-teams` passes when every owner is a GitHub team (`@org/team`) **or** every owner is a personal handle (`@user`). Mixing teams with individuals still fails (the pilot ratchet regression). Emails still fail. A single-maintainer OSS repo can reach silver without inventing a GitHub team.
2. **Workspace lockfiles.** The workspace connector's `lockfile()` fact walks from the `package.json` directory up to the checkout root and accepts `pnpm-lock.yaml` (and the other lockfile names) at an ancestor. Nested workspace members share the root lock.
3. **Lockfile-faithful CI.** `ciUsesInstall` is true for `npm install` and for `pnpm install` / `pnpm i` without `--frozen-lockfile`. `pnpm install --frozen-lockfile` (and `pnpm i --frozen-lockfile`) count as lockfile-faithful, like `npm ci`.

**Status:** ✅

**Where:** [examples/repo-portal-starter/seikika.yaml](../../examples/repo-portal-starter/seikika.yaml), [design/package-standards.md](../design/package-standards.md), [crates/seikika-connector-workspace/src/npm.rs](../../crates/seikika-connector-workspace/src/npm.rs)

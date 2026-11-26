---
id: D01M7YAHMGFZ8YH561KPF9CYY9T
decided: 2026-11-21T05:51:24Z
---

# Packages and connectors make no assumptions about the repository they score

**A package's connectors run on any repository, and a fact means what the tool it describes means.** The first out-of-box run on four repositories that Seikika's authors didn't write (OpenFGA, Codex, gh-aw, PostHog; [experiments/out-of-box](../../experiments/out-of-box/README.md)) failed on every one, because of assumptions carried over from this repository:

1. **The runner asks Cargo for crates only when a job runs per crate.** Lint and NDJSON jobs run in any checkout.
2. **Lint jobs take `{files}`:** the files their own globs match. `packages/github-actions` passes zizmor and actionlint exactly those, instead of `.github/workflows actions`. A repository needn't have an `actions/` directory, and one with no workflows gets no lint records rather than an error.
3. **actionlint's "unknown name" findings are ignored by the package:** unknown permission scopes, unknown `concurrency` keys and unknown runner labels. GitHub ships scopes and keys before actionlint knows them (`copilot-requests` and `concurrency.queue` are missing from actionlint's latest release, 1.7.12, and from its `main`), and larger or self-hosted runner labels are per organisation. A typo in one of those three places is no longer reported locally.
4. **pnpm facts follow pnpm.** This amends D01M7Y57FVX0XYNGMX0A0XJ4GK9: in CI a plain `pnpm install` is lockfile-faithful, since pnpm's `install` docs say it fails when the lockfile needs an update; `--no-frozen-lockfile` still fails. Corepack's hash pin (`pnpm@x.y.z+sha512.…`) counts as pinned. CI's Node may come from `node-version-file`. With no pnpm pin in mise, there is nothing to align with mise.
5. **Doc links resolve as a reader follows them.** `/x` is relative to the repository root, or to a docs site's content root above the document, possibly behind a base path. An extensionless link may name `x.md`, `x.mdx`, `x/index.md(x)` or `x/README.md`. Links inside code are examples and are skipped.

Seikika for Seikika's own results are unchanged by all five.

**Status:** ✅

**Where:** [design/repo-value.md](../design/repo-value.md) R2, [research-log.md](../research-log.md) (*Out-of-box R2*)

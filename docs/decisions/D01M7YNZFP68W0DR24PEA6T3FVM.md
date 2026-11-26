---
id: D01M7YNZFP68W0DR24PEA6T3FVM
decided: 2026-11-21T09:15:00Z
---

# Storybook catalog package and workspace facts

**Ship a git-first package `packages/storybook` with scorecard `storybook-catalog` (kind `Storybook`), backed by a new `builtin:workspace` record kind `storybook` discovered via `storybooks` globs — deterministic CEL for config/deps/a11y/CI; assessments deferred for qualitative coverage.**

1. **Connector facts.** `seikika-connector-workspace` gains opt-in `storybooks` globs (`storybook` records): Storybook `main.*` config, `storybook` / `@storybook/addon-a11y` deps, story file count, telemetry disable, preview a11y fail-on-violation, and CI build/test signals from workflows.
2. **Package.** `packages/storybook@0.1.0` exports the kind and scorecard (`bronze` → `gold`); consumers `include` it and configure `storybooks` on the workspace connector. Optional `requireA11yAddon` (default true) tightens silver.
3. **Not assessments for the ratchet.** Qualitative “critical screens have stories” stays assessment-backed later; v0 levels are all deterministic so `actions/report` can ratchet them.

**Status:** ✅

**Where:** [packages/storybook/](../../packages/storybook/), [crates/seikika-connector-workspace/src/storybook.rs](../../crates/seikika-connector-workspace/src/storybook.rs)

---
id: D01M7Y6X38AERPBT0YH1AG5TBRD
decided: 2026-11-21T04:50:00Z
---

# pnpm-expo package and workspace facts for Expo monorepos

**Ship a git-first package `packages/pnpm-expo` with scorecards `pnpm-workspace` (kind `JsWorkspace`) and `expo-app` (kind `ExpoApp`), backed by new `builtin:workspace` record kinds — do not reuse `maintainer-package` unchanged for pnpm/Expo repos.**

1. **Connector facts.** `toolbox-connector-workspace` gains opt-in `jsWorkspace: true` (`js-workspace` record) and `expoApps` globs (`expo-app` records): `packageManager` pin, root lockfile, linker (`.npmrc` / `pnpm-workspace.yaml`), mise pins, frozen CI install, Expo SDK / EAS / Metro / doctor-in-CI signals.
2. **Package.** `packages/pnpm-expo@0.1.0` exports the kinds and scorecards; consumers `include` it and configure the workspace connector. `requireHoisted` tightens linker policy for RN/Expo monorepos.
3. **Not `maintainer-package`.** That scorecard remains for npm-oriented private packages; pnpm workspace and Expo app norms live in this package so nested lockfile walk-up and `pnpm install --frozen-lockfile` are first-class without mis-scoring Expo apps.

**Status:** ✅

**Where:** [packages/pnpm-expo/](../../packages/pnpm-expo/), [crates/toolbox-connector-workspace/src/js_workspace.rs](../../crates/toolbox-connector-workspace/src/js_workspace.rs), [crates/toolbox-connector-workspace/src/expo.rs](../../crates/toolbox-connector-workspace/src/expo.rs)

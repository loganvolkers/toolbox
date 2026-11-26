---
title: pnpm and Expo
description: Check a pnpm workspace and its Expo apps for reproducible toolchain and build settings.
---

Use `pnpm-expo` to check a monorepo's package manager pin, lockfile, linker, and Expo configuration. It reads workspace files and continuous integration (CI) workflows to check how the project is set up.

## Add the workspace checks

You need Seikika 0.2 and the [Workspace connector](/connectors/workspace/). If you already have a connector named `workspace`, merge these settings and keep its existing records. Change the Expo app glob to match your layout. [Pin and install a package](/packages/#pin-and-install)

```yaml
include:
  - package: github:seikika/seikika/packages/pnpm-expo@5818e032cac3cefec1fe9f6a2a4559bd410174cf
connectors:
  - name: workspace
    package: builtin:workspace
    runtime: in-image
    config:
      crates: false
      jsWorkspace: true
      expoApps: ["apps/*/app.json"]
    records: [js-workspace, expo-app]
```

Set `requireHoisted: true` under the include's `with` settings if your repository requires a hoisted setup. In that setup, pnpm places dependencies in shared `node_modules` locations.

## Verify the workspace and apps

From the checkout root, run `seikika report .seikika --all`. Look for `pnpm-workspace` on the workspace and `expo-app` on each matching app. If an app is missing, compare its config path with the `expoApps` glob. The package checks whether CI pins Expo Doctor. Run Expo Doctor in your app's CI for the live checks. [Scorecard and parameter reference](https://github.com/seikika/seikika/blob/5818e032cac3cefec1fe9f6a2a4559bd410174cf/packages/pnpm-expo/README.md) · [Test portal](https://github.com/seikika/seikika/tree/5818e032cac3cefec1fe9f6a2a4559bd410174cf/packages/pnpm-expo/tests)

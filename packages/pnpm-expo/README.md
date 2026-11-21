# pnpm-expo: a Toolbox package

pnpm workspace and Expo app hygiene for monorepos (Expo SDK 53+ era). Built on
`builtin:workspace` facts (`js-workspace`, `expo-app`) — not on T4T's
`maintainer-package`, which still assumes npm-centric CI signals for some rules.

| Brings | What |
|---|---|
| Kinds | `JsWorkspace`, `ExpoApp` |
| Scorecards | `pnpm-workspace`, `expo-app` |
| Connector config | Consumer must enable `jsWorkspace: true` and `expoApps` globs on `builtin:workspace` |

## Consumer include

After this package is on `main` (and ideally tagged), pin it like `github-actions`:

```yaml
include:
  - package: github:loganvolkers/toolbox/packages/pnpm-expo@0.1.0
    with: {requireHoisted: true}   # RN/Expo monorepos that document hoisted node_modules
connectors:
  - name: workspace
    package: builtin:workspace
    runtime: in-image
    config:
      crates: false
      jsWorkspace: true
      expoApps: ["apps/*/app.json"]   # or ${param:expoAppGlobs} once wired by the operator
      workflows: [.github/workflows/*.y*ml]
    records: [js-workspace, expo-app, workflow]
```

Then:

```bash
toolbox install
toolbox doctor
toolbox report . --baseline .toolbox/results.json
```

### Publishing path

| Step | Required? |
|---|---|
| Merge this package + connector facts to `loganvolkers/toolbox` `main` | **Yes** — consumers that `include` a git path need the commit |
| Tag / GitHub Release that includes a Toolbox build with the new workspace facts | **Yes for external CI** — pin `toolbox-version: "0.1.1"` (or later). An older `0.1.0` binary will not emit `js-workspace` / `expo-app` |
| Separate npm/OCI publish of the package YAML | **No** — packages are git-first (`github:…/packages/pnpm-expo@<tag-or-semver>`) per D89 |

Same pattern as `packages/github-actions`: `toolbox install` pins the package in `toolbox.lock`.

Parameters (see `toolbox-package.yaml`):
- `requireHoisted` defaults to `false`; set `true` when the repo documents hoisted deps (Gradle/RN).
- `expoAppGlobs` documents the usual glob; the consumer's connector `expoApps` list is the live config.

Test the package with `toolbox test packages/pnpm-expo/tests`.

## Scorecards

### `pnpm-workspace` (kind `JsWorkspace`)

| Level | Rules |
|---|---|
| bronze | `has-workspace-file`, `package-manager-pinned`, `root-lockfile` |
| silver | `ci-frozen-install`, `linker-policy-declared` |
| gold | `toolchain-aligned`, `no-unbounded-root-deps` |

### `expo-app` (kind `ExpoApp`)

| Level | Rules |
|---|---|
| bronze | `has-expo-dep`, `has-app-config`, `single-canonical-eas` |
| silver | `sdk-packages-present`, `metro-uses-expo-default`, `doctor-pinned-in-ci` |
| gold | `eas-profiles` |

`expo-doctor` as a live runner job is deferred; CI pin detection is static from workflows.

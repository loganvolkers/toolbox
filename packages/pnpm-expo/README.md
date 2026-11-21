# pnpm-expo: a Toolbox package

pnpm workspace and Expo app hygiene for monorepos (Expo SDK 53+ era). Built on
`builtin:workspace` facts (`js-workspace`, `expo-app`) — not on T4T's
`maintainer-package`, which scores each `package.json` as an application and is
the wrong shape for workspace / Expo standards
([package-standards.md](../../docs/design/package-standards.md) §6).

| Brings | What |
|---|---|
| Kinds | `JsWorkspace`, `ExpoApp` |
| Scorecards | `pnpm-workspace`, `expo-app` |
| Connector config | Consumer must enable `jsWorkspace: true` and `expoApps` globs on `builtin:workspace` |
| Minimum Toolbox binary | **≥ 0.1.1** (first release that emits `js-workspace` / `expo-app`) |

## Consumer diff

Adopting this package is not “add one include line”. Diff against a starter-style portal
(`repo` + `docs` only):

### 1. Include (pin a git tag that **contains** this directory)

```yaml
include:
  - package: github:loganvolkers/toolbox/packages/pnpm-expo@0.1.1
    with: {requireHoisted: true}   # RN/Expo monorepos that document hoisted node_modules
```

| Pin | Safe value (as of Toolbox `v0.1.1`) |
|---|---|
| Package include | `github:loganvolkers/toolbox/packages/pnpm-expo@0.1.1` (or `@v0.1.1`) |
| `actions/report` | `loganvolkers/toolbox/actions/report@v0.1.1` |
| Binary | `toolbox-version: "0.1.1"` |

**Version vs git tag.** This package’s `toolbox-package.yaml` may still say `version: 0.1.0`
while consumers must pin **`@0.1.1`** because `toolbox install` clones `--branch v<ver>`. The
YAML `version:` and the Toolbox release tag that first *contains* `packages/pnpm-expo` can
differ; **pins follow the git tag**, not only the YAML field. Tag `v0.1.0` does **not** contain
this package — do not pin `@0.1.0`.

An older `toolbox-version: "0.1.0"` binary will not emit `js-workspace` / `expo-app`; scorecards
look empty.

### 2. Workspace connector config delta

```yaml
connectors:
  - name: workspace
    package: builtin:workspace
    runtime: in-image
    config:
      root: .
      crates: false
      repo: true                          # keep if you still score repo-hygiene
      docs: ["*.md", docs/**/*.md]        # keep if you still score docs
      jsWorkspace: true
      expoApps: ["apps/*/app.json"]       # adjust to your app layout
      workflows: [.github/workflows/*.y*ml]
    records: [repo, doc, js-workspace, expo-app, workflow]
```

### 3. Install, then report

```bash
toolbox install
toolbox doctor
toolbox report . --baseline .toolbox/results.json
```

In CI, run `toolbox install` (or otherwise populate the package cache for the pinned tag) **before**
`actions/report` / `toolbox report` when `toolbox.lock` lists packages.

### Publishing path

| Step | Required? |
|---|---|
| Merge this package + connector facts to `loganvolkers/toolbox` `main` | **Yes** — consumers that `include` a git path need the commit |
| Tag / GitHub Release that includes this directory **and** a Toolbox build with the new workspace facts | **Yes for external CI** — pin package tag + `toolbox-version` together (see handoff) |
| Separate npm/OCI publish of the package YAML | **No** — packages are git-first (`github:…/packages/pnpm-expo@<tag>`) per D89 |

Same pattern as `packages/github-actions`: `toolbox install` pins the package in `toolbox.lock`.

Parameters (see `toolbox-package.yaml`):
- `requireHoisted` (boolean, default `false`); set `true` when the repo documents hoisted deps (Gradle/RN).
- `expoAppGlobs` documents the usual glob; the consumer's connector `expoApps` list is the live config.

Test the package with `toolbox test packages/pnpm-expo/tests`.

## Operator notes (toolchain gold)

Gold `toolchain-aligned` reads every workflow `node-version:` token. **Every `node-version:` must
be a concrete number**, including matrix jobs — do not use `node-version: ${{ matrix.node }}`.
The connector captures the first non-space token; `${{` is not a semver major and fails
`ciNodeSatisfiesEngines`. Prefer literal majors in each job (or `node-version-file`) so mise /
`packageManager` / CI stay aligned.

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

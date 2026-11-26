# pnpm-expo: a Seikika package

pnpm workspace and Expo app hygiene for monorepos (Expo SDK 53+ era). Built on
`builtin:workspace` facts (`js-workspace`, `expo-app`) — not on S4S's
`maintainer-package`, which scores each `package.json` as an application and is
the wrong shape for workspace / Expo standards
([package-standards.md](../../docs/design/package-standards.md) §6).

| Brings | What |
|---|---|
| Kinds | `JsWorkspace`, `ExpoApp` |
| Scorecards | `pnpm-workspace`, `expo-app` |
| Connector config | Consumer must enable `jsWorkspace: true` and `expoApps` globs on `builtin:workspace` |
| Minimum Seikika binary | **≥ 0.2.0** (first release under the Seikika name) |

## Consumer diff

Adopting this package is not “add one include line”. Diff against a starter-style portal
(`repo` + `docs` only):

### 1. Include (pin a git tag that **contains** this directory)

```yaml
include:
  - package: github:seikika/seikika/packages/pnpm-expo@0.2.0
    with: {requireHoisted: true}   # RN/Expo monorepos that document hoisted node_modules
```

| Pin | Safe value (as of Seikika `v0.2.0`) |
|---|---|
| Package include | `github:seikika/seikika/packages/pnpm-expo@0.2.0` (or `@v0.2.0`) |
| `actions/report` | `seikika/seikika/actions/report@v0.2.0` |
| Binary | `seikika-version: "0.2.0"` |

**Version vs git tag.** This package’s `seikika-package.yaml` may still say `version: 0.1.0`
while consumers must pin **`@0.2.0`** because `seikika install` clones `--branch v<ver>`. The
YAML `version:` and the Seikika release tag that first *contains* `packages/pnpm-expo` can
differ. **Pins follow the git tag.** The workspace connector in Seikika 0.2.0 emits
`js-workspace` and `expo-app` entities for these scorecards.

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
seikika install
seikika doctor
seikika report . --baseline .seikika/results.json
```

In CI, run `seikika install` (or otherwise populate the package cache for the pinned tag) **before**
`actions/report` / `seikika report` when `seikika.lock` lists packages.

### Publishing path

| Step | Required? |
|---|---|
| Merge this package + connector facts to `seikika/seikika` `main` | **Yes** — consumers that `include` a git path need the commit |
| Tag / GitHub Release that includes this directory **and** a Seikika build with the new workspace facts | **Yes for external CI** — pin package tag + `seikika-version` together (see handoff) |
| Separate npm/OCI publish of the package YAML | **No** — packages are git-first (`github:…/packages/pnpm-expo@<tag>`) per D89 |

Same pattern as `packages/github-actions`: `seikika install` pins the package in `seikika.lock`.

Parameters (see `seikika-package.yaml`):
- `requireHoisted` (boolean, default `false`); set `true` when the repo documents hoisted deps (Gradle/RN).
- `expoAppGlobs` documents the usual glob; the consumer's connector `expoApps` list is the live config.

Test the package with `seikika test packages/pnpm-expo/tests`.

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

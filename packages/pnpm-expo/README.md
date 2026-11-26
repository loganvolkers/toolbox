# pnpm-expo

Check pnpm workspace and Expo app setup in a monorepo. The package compares toolchain settings and checks the workspace and Expo project configuration.
The Seikika package adds checks to your portal using records from the workspace connector.

## Add the package

[Build the CLI and workspace connector](https://seikika.com/start/install/#build-from-source)
using the commit in the include below as your checkout revision. This build emits the js-workspace and expo-app records
these checks need. Add the include to your portal manifest.

```yaml
include:
  - package: github:seikika/seikika/packages/pnpm-expo@25a208a3bc1a97f184588c512afcb12d636ce7be
    with: {requireHoisted: true}
```

Keep existing connector kinds and settings, and add these workspace connector
settings. The glob paths are relative to the checkout root. Adjust them to
match your repository.

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

From the portal directory, install the package, validate the manifest,
and run the checks. Keep both binaries on `PATH`.

```sh
seikika install .
seikika validate .
seikika report .
```

Look for the scorecards below. If they have no entities, check that the
workspace connector is available and its globs match your files.
Commit `seikika.lock` so other checkouts can reproduce the package selection.
Follow [Gate pull requests](https://seikika.com/operators/ci/) to add a baseline in CI.

Set `requireHoisted` to `true` when your project requires hoisted dependencies. Set `expoApps` in the connector to match your actual app layout. The package fixture is in [tests/](tests/).

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

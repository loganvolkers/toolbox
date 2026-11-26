# storybook: a Seikika package

Storybook project hygiene for React / React Native Web catalogs. Built
on `builtin:workspace` facts (`storybook` records via `storybooks`
globs) — deterministic CEL for config, deps, a11y addon, telemetry and
CI; qualitative story coverage stays assessment-backed later
([D01M7YNZFP68W0DR24PEA6T3FVM](../../docs/decisions/D01M7YNZFP68W0DR24PEA6T3FVM.md)).

| Brings                 | What                                                                                                                                     |
| ---------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| Kinds                  | `Storybook`                                                                                                                              |
| Scorecards             | `storybook-catalog`                                                                                                                      |
| Connector config       | Consumer must enable `storybooks` globs on `builtin:workspace`                                                                           |
| Minimum Seikika binary | **After the release that ships `storybook` workspace facts** (post-merge of this package + connector). Until then use a workspace build. |

## Consumer diff

### 1. Include (pin a git tag that **contains** this directory)

```yaml
include:
  - package: github:seikika/seikika/packages/storybook@0.1.2
    with: {requireA11yAddon: true}
```

Replace `@0.1.2` with the Seikika release tag that first contains
`packages/storybook` **and** emits `storybook` records. Pins follow the
**git tag**, not only `seikika-package.yaml` `version:` (same trap as
`pnpm-expo`).

An older `seikika-version` that lacks `storybook` facts will leave the
scorecard empty.

### 2. Workspace connector config delta

```yaml
connectors:
  - name: workspace
    package: builtin:workspace
    runtime: in-image
    config:
      root: .
      storybooks: ["**/.storybook/main.*"] # or apps/*/.storybook/main.*
      workflows: [.github/workflows/*.y*ml] # needed for CI signals
    records: [storybook, workflow] # keep any existing kinds too
```

### 3. Install, then report

```bash
seikika install
seikika doctor
seikika report . --baseline .seikika/results.json
```

### Publishing path

| Step                                                                                             | Required?                |
| ------------------------------------------------------------------------------------------------ | ------------------------ |
| Merge this package + connector facts to `seikika/seikika` `main`                            | **Yes**                  |
| Tag / GitHub Release that includes this directory **and** a Seikika build with `storybook` facts | **Yes for external CI**  |
| Separate npm/OCI publish of the package YAML                                                     | **No** — git-first (D89) |

Parameters: `requireA11yAddon` (boolean, default `true`).

Test the package with `seikika test packages/storybook/tests`.

## Scorecard: `storybook-catalog` (kind `Storybook`)

| Level  | Rules                                                     |
| ------ | --------------------------------------------------------- |
| bronze | `has-config`, `has-storybook-dep`, `has-stories`          |
| silver | `a11y-addon`, `telemetry-disabled`, `ci-builds-storybook` |
| gold   | `ci-tests-stories`, `a11y-fails-on-violation`             |

Live Storybook / Playwright execution is left to app CI; this scorecard
only reads static config and workflow text.

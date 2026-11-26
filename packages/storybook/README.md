# storybook

Check Storybook setup in React and React Native Web projects. The package scores configuration, dependencies, accessibility tooling, telemetry settings, and CI.
The Seikika package adds checks to your portal using records from the workspace connector.

## Add the package

[Build the CLI and workspace connector](https://seikika.com/start/install/#build-from-source)
using the commit in the include below as your checkout revision. This build emits the storybook records
these checks need. Add the include to your portal manifest.

```yaml
include:
  - package: github:seikika/seikika/packages/storybook@25a208a3bc1a97f184588c512afcb12d636ce7be
    with: {requireA11yAddon: true}
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
      storybooks: ["**/.storybook/main.*"] # or apps/*/.storybook/main.*
      workflows: [.github/workflows/*.y*ml] # needed for CI signals
    records: [storybook, workflow] # keep any existing kinds too
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

Set `requireA11yAddon` to control whether the accessibility addon is required. It defaults to `true`. The package fixture is in [tests/](tests/).

## Scorecard: `storybook-catalog` (kind `Storybook`)

| Level  | Rules                                                     |
| ------ | --------------------------------------------------------- |
| bronze | `has-config`, `has-storybook-dep`, `has-stories`          |
| silver | `a11y-addon`, `telemetry-disabled`, `ci-builds-storybook` |
| gold   | `ci-tests-stories`, `a11y-fails-on-violation`             |

Live Storybook / Playwright execution is left to app CI; this scorecard
only reads static config and workflow text.

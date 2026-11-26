---
title: Storybook standards
description: Check Storybook configuration, stories, accessibility settings, and CI wiring.
---

Use `storybook` to check Storybook projects for stories, configuration, accessibility tooling, and build or test steps in continuous integration (CI).

## Add the Storybook checks

You need Seikika 0.2 and the [Workspace connector](/connectors/workspace/). Add this snippet to your portal's manifest. If you already have a connector named `workspace`, add the glob and record kind to it. Narrow the glob if you want to score only some projects. [Pin and install a package](/packages/#pin-and-install)

```yaml
include:
  - package: github:seikika/seikika/packages/storybook@5818e032cac3cefec1fe9f6a2a4559bd410174cf
connectors:
  - name: workspace
    package: builtin:workspace
    runtime: in-image
    config:
      crates: false
      storybooks: ["**/.storybook/main.*"]
    records: [storybook]
```

The scorecard checks for the accessibility addon by default. Set `requireA11yAddon: false` under `with` if your project applies an accessibility check another way.

## Verify the setup

From the checkout root, run `seikika report .seikika --all` and look for the `storybook-catalog` scorecard. Open one project and confirm that its config path and story count match the checkout. If the project is absent, check the `storybooks` glob. These checks read source files and CI definitions. Run your Storybook build and story tests in CI to verify the project itself. [Scorecard reference](https://github.com/seikika/seikika/blob/5818e032cac3cefec1fe9f6a2a4559bd410174cf/packages/storybook/README.md) · [Test portal](https://github.com/seikika/seikika/tree/5818e032cac3cefec1fe9f6a2a4559bd410174cf/packages/storybook/tests)

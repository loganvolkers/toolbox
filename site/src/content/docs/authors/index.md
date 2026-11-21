---
title: Publish a standard
description: A package author ships scorecards, connectors and the tools they need, for other portals to include.
sidebar:
  order: 1
---

A package author publishes something other portals include. A package is standards as data: kinds, scorecards, fixtures and tests, and the parameters a consumer may set. It can also name a connector and the tools a rule needs. The consumer runs the portal. Changing Toolbox's crates is the [maintainer](/toolbox/maintainers/) loop.

The package author SDLC follows the maintainer SDLC through testing, then forks. A maintainer tests the whole system at one commit. A package author tests one package, then distributes it for other repositories to pin.

**Shipping a second package?** There is no scaffold. Use the checklist in
[`docs/design/package-author-sdlc.md`](https://github.com/loganvolkers/toolbox/blob/main/docs/design/package-author-sdlc.md) §6
(layout, when to extend `toolbox-connector-workspace`, meta bronze, CEL without `join()`, consumer
pins, release handoff). Copy `packages/github-actions/` or `packages/pnpm-expo/` and trim.

## What a package contains

```
toolbox-package.yaml     # name, version, parameters, fragment globs
fragments/*.yaml         # kinds, fields, scorecards (manifest YAML)
fixtures/                # records that exercise every exported rule both ways
tests/*.test.yaml        # expectations over those fixtures
```

A parameter can declare a direction, `tighten: lower` or `tighten: higher`. `toolbox validate` rejects a consumer value that moves the other way. A team can make `failRateBronze` stricter than the package default. It cannot make the bar easier.

## Hand a team a scorecard

`packages/github-actions` is the first package in this repository. It scores GitHub Actions workflows and the third-party actions they use. The rules need zizmor 1.32.0 and actionlint 1.7.12, and a `workspace` connector that reads workflow files. `packages/pnpm-expo` is the second: pnpm workspace and Expo app hygiene on `js-workspace` / `expo-app` workspace facts (enable `jsWorkspace: true` and `expoApps` globs).

Inside one checkout, include it by path:

```yaml
include:
  - package: ../packages/github-actions
    with: {githubOrg: acme, githubRepo: payments, failRateBronze: 15}
```

`githubOrg` and `githubRepo` are required. `branch` defaults to `main`. The fail-rate and duration parameters may only be lowered.

A package that lives in another repository is pinned by git, not by a floating branch:

```yaml
include:
  - package: github:owner/repo/packages/github-actions@1.2.0
    with: {githubOrg: acme, githubRepo: payments, failRateBronze: 15}
```

The version is a git tag: `@1.2.0` fetches `v1.2.0`. That tag must **contain** the package directory — package `toolbox-package.yaml` `version:` can lag the Toolbox release tag that first ships the package; consumer pins follow the **git tag**. Toolbox itself ships installable binaries from [v0.1.0](https://github.com/loganvolkers/toolbox/releases/tag/v0.1.0) (use ≥ `0.1.1` when the package needs `js-workspace` / `expo-app`); packages still pin by git tag or local path until an OCI mirror exists.

`toolbox install` clones that tag into the cache and records the package's content hash in `toolbox.lock`. Later loads read the lock and do not touch the network. Commit the lock file.

```bash
toolbox install path/to/portal
toolbox doctor path/to/portal
toolbox doctor path/to/portal --mise
```

`doctor` reports whether each connector resolves and whether each declared tool prints its pinned version. `--mise` prints the `[tools]` section that installs those pins. A rule whose tool is missing comes out `unknown`. Unknown is not a pass, and the ratchet does not treat it as a regression.

## Prove a rule both ways

```bash
toolbox test packages/github-actions/tests
```

The fixtures include a workflow that passes the scorecard and a workflow that fails it. An exported rule that only has one of those outcomes is not done. `toolbox validate` on a portal that includes the package checks the fragments against the manifest schema and checks that each `with:` value tightens rather than weakens.

## Where the loop stands

| Step | What you run | State |
|---|---|---|
| Author the package | `toolbox-package.yaml` and fragments | Built for a package in a directory. There is no `toolbox package new` scaffold |
| Build a connector | `toolbox connector test` against the conformance kit | Built for a binary you already have. The Rust SDK is not published as a release, and there is no scaffold |
| Test | `toolbox test` on the package's tests | Built. A matrix of released Toolbox versions waits on a Toolbox release |
| Install and pin | `toolbox install`, `toolbox.lock` | Built for a git tag or a local path |
| Publish an archive, attest it, roll it across a fleet | | Specified. No release archive is produced, and the package registry is still an open question |

Operators adopt a package by [including it in a portal](/toolbox/operators/). Users never see the package. They see the levels it computes, on [Use a portal](/toolbox/users/).

---
title: Publish a standard
description: A package author ships scorecards, connectors and the tools they need, for other portals to include.
sidebar:
  order: 1
---

A package author publishes something other portals include. A package is standards as data: kinds, scorecards, fixtures and tests, and the parameters a consumer may set. It can also name a connector and the tools a rule needs. The consumer runs the portal. Changing Seikika's crates is the [maintainer](/maintainers/) loop.

The package author SDLC follows the maintainer SDLC through testing, then forks. A maintainer tests the whole system at one commit. A package author tests one package, then distributes it for other repositories to pin.

**Shipping a second package?** There is no scaffold. Use the checklist in
[`docs/design/package-author-sdlc.md`](https://github.com/seikika/seikika/blob/main/docs/design/package-author-sdlc.md) §6
(layout, when to extend `seikika-connector-workspace`, meta bronze, CEL without `join()`, consumer
pins, release handoff). Copy `packages/github-actions/` or `packages/pnpm-expo/` and trim.

## What a package contains

```
seikika-package.yaml     # name, version, parameters, fragment globs
fragments/*.yaml         # kinds, fields, scorecards (manifest YAML)
fixtures/                # records that exercise every exported rule both ways
tests/*.test.yaml        # expectations over those fixtures
```

A parameter can declare a direction, `tighten: lower` or `tighten: higher`. `seikika validate` rejects a consumer value that moves the other way. A team can make `failRateBronze` stricter than the package default. It cannot make the bar easier.

## Hand a team a scorecard

`packages/github-actions` is the first package in this repository. It scores GitHub Actions workflows and the third-party actions they use. The rules need zizmor 1.32.0 and actionlint 1.7.12, and a `workspace` connector that reads workflow files. `packages/pnpm-expo` is the second: pnpm workspace and Expo app hygiene on `js-workspace` / `expo-app` workspace facts (enable `jsWorkspace: true` and `expoApps` globs).

Inside one checkout, include it by path:

```yaml
include:
  - package: ../packages/github-actions
    with: {githubOrg: acme, githubRepo: payments, failRateBronze: 15}
```

`githubOrg` and `githubRepo` are required. `branch` defaults to `main`. The fail-rate and duration parameters may only be lowered.

A package that lives in another repository is pinned by git:

```yaml
include:
  - package: github:owner/repo/packages/github-actions@1.2.0
    with: {githubOrg: acme, githubRepo: payments, failRateBronze: 15}
```

The `@ref` may be a semver tag (`@1.2.0` / `@v1.2.0` → clone `v1.2.0`), a branch (`@main`), or a commit SHA. Prefer a release tag for consumers; use a SHA or `main` only while dogfooding before a tag exists. That tag must **contain** the package directory — package `seikika-package.yaml` `version:` can lag the Seikika release tag that first ships the package; consumer pins follow the **git tag**. Check [GitHub Releases](https://github.com/seikika/seikika/releases) for tags. Packages pin by git ref or local path until an OCI mirror exists.

`seikika install` clones that ref into the cache and records the package's content hash in `seikika.lock`. Later loads read the lock and do not touch the network. Commit the lock file. The [report action](https://github.com/seikika/seikika/tree/main/actions/report) runs `seikika install --frozen` whenever the lock is present so clean CI runners do not see an empty package hash.

```bash
seikika install path/to/portal
seikika doctor path/to/portal
seikika doctor path/to/portal --mise
```

`doctor` reports whether each connector resolves and whether each declared tool prints its pinned version. `--mise` prints the `[tools]` section that installs those pins. A rule whose tool is missing comes out `unknown`. Unknown is not a pass, and the ratchet does not treat it as a regression.

## Prove a rule both ways

```bash
seikika test packages/github-actions/tests
```

The fixtures include a workflow that passes the scorecard and a workflow that fails it. An exported rule that only has one of those outcomes is not done. `seikika validate` on a portal that includes the package checks the fragments against the manifest schema and checks that each `with:` value tightens rather than weakens.

## Where the loop stands

| Step | What you run | State |
|---|---|---|
| Author the package | `seikika-package.yaml` and fragments | Built for a package in a directory. There is no `seikika package new` scaffold |
| Build a connector | Copy [`examples/connector-scaffold/`](https://github.com/seikika/seikika/tree/main/examples/connector-scaffold), then `seikika connector test` and `seikika connector drive` | Scaffold + drive landed (D01M83FFM2K6XS4EAVHVP92TBDZ). The scaffold uses a path dependency. See [Write a connector](/authors/connectors/) |
| Test | `seikika test` on the package's tests | Built. A matrix of released Seikika versions waits on a Seikika release |
| Install and pin | `seikika install`, `seikika.lock` | Built for a git tag, branch, commit SHA, or a local path |
| Publish an archive, attest it, roll it across a fleet | | Specified. No release archive is produced, and the package registry is still an open question |

Operators adopt a package by [including it in a portal](/operators/). Users never see the package. They see the levels it computes, on [Use a portal](/users/).

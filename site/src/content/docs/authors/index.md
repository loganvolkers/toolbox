---
title: Publish a standard
description: Package scorecards, test them with fixtures, and pin them in another portal.
sidebar:
  order: 1
---

Package a standard so another portal can reuse the same rules and tests. As a package author, you publish the configuration and its fixtures. An operator chooses where to apply it.

## Start from an existing package

Use [the file-size package](https://github.com/seikika/seikika/tree/main/packages/file-size) for a numeric rule, or [the SARIF package](https://github.com/seikika/seikika/tree/main/packages/sarif) to use findings from an existing tool. To author a new data source, follow [Write a connector](/authors/connectors/).

A package typically includes these files:

| File | Purpose |
|---|---|
| `seikika-package.yaml` | Package name, version, parameters, and the fragments to include |
| `fragments/*.yaml` | Kinds, fields, connectors, or scorecards shared with a portal |
| `fixtures/` | Example records that exercise rules |
| `tests/*.test.yaml` | Expected outcomes over those fixtures |

If a threshold should move only in one direction, set its parameter's `tighten` value to `lower` or `higher`. For example, a lower failure-rate limit makes a rule stricter. Validation rejects a value that would loosen that limit.

## Test the rules and the package

From the [Seikika source checkout](/start/install/#build-from-source), run:

```bash
seikika test packages/github-actions/tests
```

Look for passing expectations that cover both passing and failing rule outcomes. Run `seikika validate` on a portal that includes your package to check its fragments and parameter values. If the rules need a tool, declare its version and run `seikika doctor` in the consuming portal.

## Share the package and pin it

Publish the package directory in a Git repository. In the consuming portal's manifest, add an entry like this one. Set the GitHub organization and repository to the repository you score.

```yaml
include:
  - package: github:seikika/seikika/packages/github-actions@c14c9fbb60405f9fea12d86de3d0b32b717755d9
    with: {githubOrg: acme, githubRepo: payments, failRateBronze: 15}
```

This package needs a workspace connector that reports `workflow` and `action-ref` records. Copy the connector configuration from the [package test portal](https://github.com/seikika/seikika/blob/c14c9fbb60405f9fea12d86de3d0b32b717755d9/packages/github-actions/tests/seikika.yaml) and install the binaries and tools named in the [package README](https://github.com/seikika/seikika/tree/main/packages/github-actions).

The include pins the source to a commit. If you use a release tag, Seikika resolves a version such as `@1.2.0` to `v1.2.0`. That tag must contain the package directory. Use a relative path such as `../packages/github-actions` to work on a package in the same checkout.

From the consuming portal directory, run:

```bash
seikika install .
seikika validate .
seikika doctor .
```

Commit `seikika.lock` with the manifest. The lock records the package content hash. On a clean CI runner, use `seikika install --frozen` to fill the cache from the committed lock. [Gate pull requests](/operators/ci/) describes how the report action uses the lock.

For the publishing checklist and release handoff, see [Package author development lifecycle](https://github.com/seikika/seikika/blob/main/docs/design/package-author-sdlc.md). Package distribution uses Git or local paths. The OCI registry mirror remains planned.

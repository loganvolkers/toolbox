---
title: GitHub Actions standards
description: Check workflow security, pin third-party actions, and track GitHub Actions reliability.
sidebar:
  order: 2
---

Use `github-actions` to check workflow syntax, permissions, and third-party action pins. It can also score recent run failures and durations on one branch.

## Configure the workflow checks

You need Seikika 0.2 and the Workspace, Runner, and GitHub connectors. Install the versions of zizmor and actionlint reported by `seikika doctor`. For run history, provide `GITHUB_TOKEN` with Actions read access to the repository. [Connector directory](/connectors/) · [Pin and install a package](/packages/#pin-and-install)

Replace `acme` and `payments` with your GitHub organization and repository. Set `branch` if the branch differs from `main`. If you already have a connector named `workspace`, merge the workflow settings and record kinds into it.

```yaml
include:
  - package: github:seikika/seikika/packages/github-actions@5818e032cac3cefec1fe9f6a2a4559bd410174cf
    with: {githubOrg: acme, githubRepo: payments}
connectors:
  - name: workspace
    package: builtin:workspace
    runtime: in-image
    config:
      crates: false
      workflows: [.github/workflows/*.y*ml, actions/*/action.y*ml]
    records: [workflow, action-ref]
```

## Verify the checks

From the checkout root, run `seikika report .seikika --skip github-actions` to check workflows without fetching live run history. Look for `ci-security` and `action-deps` and open the score for a known workflow or action. Run `seikika report .seikika` with the GitHub credential to see run-based checks in `ci-reliability` and `ci-speed`.

If workflow entities are missing, check the workspace globs. If a lint check is unknown, run `seikika doctor .seikika` and install the pinned tools. [Parameter reference](https://github.com/seikika/seikika/blob/5818e032cac3cefec1fe9f6a2a4559bd410174cf/packages/github-actions/README.md) · [Test portal](https://github.com/seikika/seikika/tree/5818e032cac3cefec1fe9f6a2a4559bd410174cf/packages/github-actions/tests)

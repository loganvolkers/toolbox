---
title: Releases and upgrades
description: Match the docs to your binary and verify a portal upgrade before rolling it out.
sidebar:
  order: 6
---

These docs describe the current Seikika source. Check what your build can do with `seikika --version` and the command's `--help`. Use [GitHub Releases](https://github.com/seikika/seikika/releases) for downloadable artifacts and the [changelog](https://github.com/seikika/seikika/blob/main/CHANGELOG.md) for changes in source.

| Version or status | What to use |
|---|---|
| 0.1 releases | The release tag's README and `toolbox` assets. These releases predate the Seikika command and manifest rename. |
| 0.2 documentation | The revision in [Install Seikika](/start/install/#build-from-source) or a matching published release. Check the release assets before relying on installer or container examples. |
| Planned | Design documents describe work such as live remote planning and a generated portal client. Check [what Seikika supports](/concepts/capabilities/) for available workflows and proposed features. |

## Change a release pin with mise

In your portal repository's mise config, set `version` for the CLI and each Seikika
connector to the same published release. Keep each entry's `matching` value so mise
selects the right archive. Review the config, then install and verify the candidate.

```sh
mise trust
mise install
mise exec -- seikika --version
mise exec -- seikika doctor .seikika
```

Use your portal directory in place of `.seikika` if needed. Confirm the version and
connector checks before verifying the upgrade. To roll back the tools, restore the
previous version pins and repeat these commands. For a global CLI install, repeat the
global install command with the chosen version. [Global CLI setup](/start/install/#install-the-cli-globally)

## Verify an upgrade

1. Read the release notes for changes to manifests, connectors, and outputs that your scripts consume. Pin the CLI, connector binaries, and any CI action to known revisions. Record your current image digest or source revision and package lock.
2. Back up the portal database and keep the current manifest revision. For a rename from Toolbox, check the manifest header, executable names, environment variables, and action inputs against the new release documentation.
3. Use the candidate CLI to run `seikika validate`, `seikika test`, and `seikika doctor` against the portal directory. If you use packages, run `seikika install --frozen` against the committed lock. Review a [local plan](/operators/portal/) when the manifest changes.
4. Deploy the candidate in an isolated environment. Check readiness, one representative catalog read as a permitted user, and one denied or redacted read.
5. If the candidate fails, use the prior image and manifest. Check database migration compatibility first. Keep a restorable backup and test the restore path before upgrading a shared portal.

See [deploy and operate a portal](/operators/deploy/) for readiness and security checks.

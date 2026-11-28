---
title: Contracts and schemas
description: Look up schemas for portal configuration, assessments, connector and executor messages, and tool settings.
---

Use the schema from your installed release when validating data or configuring an editor. The schemas linked here follow the main branch. On GitHub, choose the matching release tag to see an earlier version. [Seikika releases](https://github.com/seikika/seikika/releases)

## JSON Schemas

| Contract | Use it to check | Source |
|---|---|---|
| Portal manifest | Portal settings, kinds, connectors, rules, and actions in `seikika.yaml`. | [Manifest JSON Schema](https://github.com/seikika/seikika/blob/main/crates/seikika-model/schema/manifest.schema.json) |
| Connector messages | Registrations, commands, records, and completion messages exchanged over stdio or HTTP. | [Connector message JSON Schema](https://github.com/seikika/seikika/blob/main/crates/seikika-model/schema/connector-messages.schema.json) |
| Executor messages | Requests and responses on the action executor protocol. | [Executor message JSON Schema](https://github.com/seikika/seikika/blob/main/crates/seikika-model/schema/executor-messages.schema.json) |
| Assessment document | Verdicts and evidence submitted against a rubric. | [Assessment JSON Schema](https://github.com/seikika/seikika/blob/main/crates/seikika-model/schema/assessment.schema.json) |

Use the [portal manifest reference](/operators/manifest/) for an overview of fields. Run `seikika validate` to check the manifest and its cross-references. Use the [Rust SDK reference](/reference/sdks/) for the connector types and functions that implement the protocol.

## Connector and integration settings

| Interface | What you can look up | Reference |
|---|---|---|
| `builtin:workspace` | File globs, record kinds, and fields emitted from a checkout. | [Workspace connector reference](https://github.com/seikika/seikika/blob/main/docs/reference/workspace-connector.md) |
| `builtin:runner` | Job configuration, result parsers, and record shapes. | [Runner connector reference](https://github.com/seikika/seikika/blob/main/docs/reference/runner-connector.md) |
| Other first-party connectors | Configuration keys and record kinds for each connector. | [Connector directory](/connectors/) |
| Standards packages | Parameters and scorecards for each package. | [Standards package directory](/packages/) |

## GitHub Actions

| Action | Use it to | Inputs and outputs |
|---|---|---|
| `seikika/seikika/actions/setup` | Install a released CLI and repo-portal connectors, or register existing binaries. | [Setup reference](https://github.com/seikika/seikika/tree/main/actions/setup) |
| `seikika/seikika/actions/ratchet` | Enforce a committed scorecard baseline and emit a verdict. | [Ratchet reference](https://github.com/seikika/seikika/tree/main/actions/ratchet) |
| `seikika/seikika/actions/build-portal` | Generate the static repo portal for an artifact, GitHub Pages, or Astro. | [Build portal reference](https://github.com/seikika/seikika/tree/main/actions/build-portal) |
| `seikika/seikika/actions/report` | Score, manage a baseline, and upload a report in one step. | [Report reference](https://github.com/seikika/seikika/blob/main/actions/report/README.md#inputs) |

For workflow examples, see [Run scorecards in CI](/operators/ci/) and
[Generate and host a static report](/operators/publish/).

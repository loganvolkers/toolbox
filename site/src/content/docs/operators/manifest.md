---
title: Portal manifest reference
description: Look up the configuration that defines a portal, its data, and its standards.
sidebar:
  order: 4
---

The portal reads `seikika.yaml` from the directory you pass to `seikika validate`, `seikika report`, or `seikika serve`. Paths to portal files belong in the manifest alongside the configuration that uses them.

## Minimum manifest

Save this as `seikika.yaml` in an empty practice directory. Run `seikika validate .` there. The manifest declares the `Service` kind. The catalog starts empty because this example supplies no records.

```yaml
apiVersion: seikika.com/v1alpha1
portal:
  name: practice
  owner: group:platform
  seikika: "0.2"
kinds:
  - name: Service
    identityFrom: [native]
    fields:
      title: {type: string}
    sources:
      default: [native]
```

## Main fields

| Field | Purpose | Where to see it |
|---|---|---|
| `apiVersion`, `portal` | Set the manifest format and identify the portal and its owner. `portal.seikika` records the intended major and minor version. | Minimum manifest above |
| `include`, `tools` | Reuse fragments or packages and pin the tools that supply facts. | [Publish a standard](/authors/) |
| `connectors` | Name the records to collect and how the connector runs. | [Connector directory](/connectors/) |
| `kinds` | Define entity fields and map connector records to catalog identities and relationships. | [The catalog](/concepts/catalog/) |
| `scorecards`, `tests` | Declare levels and rules, then check their outcomes with fixtures. | [The repo portal starter](https://github.com/seikika/seikika/tree/main/examples/repo-portal-starter) |
| `authz` | Load an authorization model and provide relationship data. | [Authentication and authorization](/operators/access/) |
| `actions` | Define inputs, approvals, the executor, and how to confirm the result. | [Define a self-service action](/operators/actions/) |
| `federation`, `forges`, `ci` | Give a parent access to child actions and configure where the actions run. | [Federate repository actions](/operators/federate-actions/) |
| `campaigns`, `tracks`, `waivers` | Track improvement and record exceptions. | [Track progress and change a standard](/operators/standards/) |
| `rubrics` | Define criteria for human or agent judgments. | [Assess a catalog entity](/operators/assessments/) |

Consult the [manifest JSON Schema](https://github.com/seikika/seikika/blob/main/crates/seikika-model/schema/manifest.schema.json) for required fields, allowed values, and nested properties. Run `seikika validate` for schema checks, cross-references, and Common Expression Language (CEL) expressions. Use `seikika describe portal PATH_TO_PORTAL` to inspect the loaded kinds, fields, and rules as records.

Each guide in the [connector directory](/connectors/) lists its record kinds and setup steps. Find protocol and assessment schemas in [Contracts and schemas](/reference/contracts/). For a complete manifest with connectors and entity mappings, see the [reference portal](https://github.com/seikika/seikika/blob/main/examples/reference-portal/seikika.yaml).

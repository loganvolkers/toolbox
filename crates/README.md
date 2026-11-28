# Seikika Rust crates

These crates build the Seikika CLI, server, and first-party connectors. To work
on them, follow [Build and check Seikika](../npm/site/src/content/docs/start/run.md)
from the repository root.

| Crate | Responsibility |
| --- | --- |
| [seikika-api](seikika-api/README.md) | Serves a configured portal over HTTP and the Model Context Protocol (MCP). |
| [seikika-authz](seikika-authz/README.md) | Identifies callers and checks what they can access in a portal. |
| [seikika-cli](seikika-cli/README.md) | Builds the `seikika` command. |
| [seikika-connect](seikika-connect/README.md) | Defines how Seikika and connectors exchange records and commands. |
| [seikika-connector-git-files](seikika-connector-git-files/README.md) | Reads catalog files from local checkouts or through the GitHub contents API. |
| [seikika-connector-github](seikika-connector-github/README.md) | Reads GitHub repositories, CODEOWNERS, apps, and installations. |
| [seikika-connector-cloudflare](seikika-connector-cloudflare/README.md) | Reads Workers deployments and version traffic splits. |
| [seikika-connector-grafana](seikika-connector-grafana/README.md) | Reads Prometheus metrics and dashboard links through Grafana. |
| [seikika-connector-gitlab](seikika-connector-gitlab/README.md) | Reads projects and group membership from GitLab.com or a self-managed GitLab instance. |
| [seikika-connector-runner](seikika-connector-runner/README.md) | Runs configured local tools and feeds their results into Seikika. |
| [seikika-connector-scim](seikika-connector-scim/README.md) | Receives user and group changes from an identity provider over System for Cross-domain Identity Management (SCIM) 2.0 and sends them to Seikika. |
| [seikika-connector-workspace](seikika-connector-workspace/README.md) | Reads a local checkout and reports facts such as Rust workspace members, Markdown links, and GitHub Actions workflows. |
| [seikika-derive](seikika-derive/README.md) | Turns a consistent snapshot of raw inputs into catalog entities, relations, and scores. |
| [seikika-exec](seikika-exec/README.md) | Runs Seikika self-service actions. |
| [seikika-expr](seikika-expr/README.md) | Checks and evaluates Common Expression Language (CEL) expressions for Seikika manifests and catalog queries. |
| [seikika-model](seikika-model/README.md) | Defines and validates the portal manifest, which is the configuration that tells Seikika what to collect, map, and check. |
| [seikika-secrets](seikika-secrets/README.md) | Resolves connector secret references when a portal runs. |
| [seikika-store](seikika-store/README.md) | Stores raw connector records and values that Seikika manages. |
| [seikika-telemetry](seikika-telemetry/README.md) | Sets up telemetry and exposes metrics. |

---
id: D01M8DDGT9KEVE4PSJ3WV7C44NE
decided: 2026-11-27T02:32:00Z
---

# Give interface lookups a Reference home

Group task pages under Guides and interface lookups under Reference in the Starlight sidebar.
Link the Reference landing page from the docs home. Keep the existing CLI, HTTP, manifest,
and npm directory routes when placing them in the new group.

Use Starlight to render Seikika's CLI and MCP definitions from `seikika describe`. Use the
published docs.rs Rustdoc and npm READMEs for the SDK contracts. Link to schemas and connector
reference sources from the same hub, and point readers to the release or running portal that
matches the interface they are using.

See the [reference documentation audit](../audits/reference-docs-2026-11-26.md) for the comparison
and [the writing guide](../../site/src/content/docs/maintainers/writing.md) for the publishing rules.

---
id: D01M8CK6XW8PR5AXSA1FQEZ5YD8
decided: 2026-11-26T18:52:13Z
---

# Ratchet documentation for the public SDKs

Keep Rust API comments with their declarations and publish the generated rustdoc on docs.rs.
Keep npm API comments with the types and components consumers import. Ship a package README
with configuration, examples, component slots, and other guidance that needs more room.

Catalog the connector SDK and public npm packages in Seikika for Seikika. The `surfaces` runner
uses rustdoc and the npm exports to find reference coverage gaps. The `maintainer-sdk-docs`
scorecard requires gold from new SDKs and ratchets the measured gaps against `portal/results.json`.
Review behavior, failure modes, and the example output against implementation and tests.

See the [SDK documentation comments guide](../../site/src/content/docs/maintainers/sdk-docs.md)
and the [coverage audit](../audits/sdk-documentation-2026-11-26.md).

---
id: D01M8BGQBN0PZ6PE5389FT1W400
decided: 2026-11-26T08:49:31Z
---

# Add the durable workflow crate with its implementation

Remove the private `seikika-durable` workspace placeholder. It has no executable code,
public API, or dependents, and the crates.io packaging jobs have to exclude it.
This updates the placeholder provision of [the crates.io decision](D01M8BEC7ZX7TFBTSC47W1XQVV4.md).

Add the crate when it has a caller and an implementation of the `Durable` contract in
[v1-spec.md](../v1-spec.md#81-durable-workflow-tables). The durable workflow design and the
Postgres schema experiment remain the implementation references.

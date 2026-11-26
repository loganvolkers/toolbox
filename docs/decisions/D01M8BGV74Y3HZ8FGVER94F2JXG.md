---
id: D01M8BGV74Y3HZ8FGVER94F2JXG
decided: 2026-11-26T08:51:37Z
---

# Run implementation CI on pull requests before merging

Use short-lived branches and focused pull requests against `main`. Run the repository's CI
and pass the applicable tests and scorecard ratchet before merging. Start each branch from
current `main` and update it if the base changes enough to invalidate its checks.

This supersedes [D32](D01M7TCJ32Q0000000000000000.md), which used direct commits to share the
design phase's state. The implementation workflow already runs branch tests and opens pull
requests for release preparation. Contributors and automation now use the same merge path.

See [the maintainer SDLC](../design/maintainer-sdlc.md).

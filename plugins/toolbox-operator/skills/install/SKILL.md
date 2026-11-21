---
name: install
description: >-
  Clones git packages and downloads released executable connectors, verifies them against toolbox.lock, and unpacks them into the cache. Use when a portal declares github: includes or connectors, before doctor or report, and in CI with --frozen.
---

# toolbox install

1. Run the command on the portal directory. The first run pins packages and connectors that the lock does not list yet.
2. Pass --frozen in CI so an unpinned package or connector fails instead of being pinned. The report GitHub Action does this automatically when toolbox.lock is present.
3. Re-run doctor afterwards. Do not unpack a connector archive or seed the package cache by hand unless you are debugging.

Git package refs (`github:owner/repo[/subdir]@ref`):

- Semver (`@0.1.1` or `@v0.1.1`) clones tag `v0.1.1`.
- A branch name (`@main`) clones that branch.
- A commit SHA (full or abbreviated hex) fetches that commit.

```
toolbox install portal
toolbox install portal --frozen
```

Do not pass --frozen until toolbox.lock pins the packages and connectors. If it fails, the content hash or archive digest does not match the lock.

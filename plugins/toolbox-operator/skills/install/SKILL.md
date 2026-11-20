---
name: install
description: >-
  Downloads the portal's released executable connectors for this platform, verifies them against toolbox.lock, and unpacks them into the cache. Use when a portal declares github: connectors, and before doctor.
---

# toolbox install

1. Run the command on the portal directory. The first run pins connectors that the lock does not list yet.
2. Pass --frozen in CI so an unpinned connector fails instead of being pinned.
3. Re-run doctor afterwards. Do not unpack a connector archive by hand.

```
toolbox install portal
toolbox install portal --frozen
```

Do not pass --frozen until toolbox.lock pins the connectors. If it fails, the archive digest does not match the lock.

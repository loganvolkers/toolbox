---
id: D01M80Y0XEK069GD08VCCNF0WB8
decided: 2026-11-22T07:00:00Z
---

# Prove value from both ends: bottom-up at the repository, top-down at the portal, then meet

**Refines D01M7Y46VDDBK2SW6H96ZSG89MP. Federation is unpaused. Value is now proven from both ends
at once:**
- **Bottom-up:** Seikika installed at a leaf node, a single repository (repo portals, the box,
  the ratchet, `actions/report`, packages). This continues as before, with the exit in
  [repo-value.md](../design/repo-value.md) §6.
- **Top-down:** the central portal, inspired by Backstage, DX, Port and other IDPs, proceeds in its
  own sessions.

How the two meet, federation being the likely middle, is decided once both use cases are proven.

1. **Why both:** the bottom-up evidence (repo portals on forks, the A/B tests) can't show what a
   platform team gets from a portal, and the top-down track can't show adoption one repository at a
   time. Each needs its own proof.
2. **What stays paused:** self-service actions (M5), by D64, until its self-service review
   lands. This decision doesn't change it.
3. **Working in parallel:** sessions say which end they work on. Shared contracts (the manifest,
   packages, the results format) change only through a decision, so both ends can rely on them.

**Status:** ✅

**Where:** [docs/design/repo-value.md](../design/repo-value.md), [docs/design/federation.md](../design/federation.md)

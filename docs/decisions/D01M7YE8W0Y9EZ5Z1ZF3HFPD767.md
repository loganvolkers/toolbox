---
id: D01M7YE8W0Y9EZ5Z1ZF3HFPD767
decided: 2026-11-21T06:56:31Z
---

# Forks are the repo-value testbed; the exit no longer waits on outside maintainers

**This amends the exit of D01M7Y46VDDBK2SW6H96ZSG89MP.** The installed pilot is `loganvolkers/rt-buddy`. The account's forks of OpenFGA, Codex, gh-aw and PostHog are the testbed: out-of-box runs for R2, the agent A/B for R3, and replays of their upstream history through the ratchet for R4. Replaying shows how the ratchet treats real changes written by other people, without asking those people to install anything.

Repo-level value is proven when R1, R2, R4 and R5 continue on rt-buddy and on at least three of the four forks, and R3 is either met or replaced by evidence that the ratchet keeps the gains agents make. A maintainer other than us choosing to keep Toolbox on can't be shown by a fork. It stays open, and it becomes the first step of federation instead of a condition for starting it.

**Status:** ✅

**Where:** [design/repo-value.md](../design/repo-value.md) §4, §6, §8

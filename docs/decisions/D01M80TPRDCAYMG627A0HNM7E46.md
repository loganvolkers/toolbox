---
id: D01M80TPRDCAYMG627A0HNM7E46
decided: 2026-11-22T05:12:18Z
---

# Unpause federation with an operator-first, API-first charter

**Federation (D68) is unpaused.** Design and engineering may resume on F1+ with an **operator-first** bias: org operators roll Toolbox out at federal/org level via **GitHub and GCP APIs** (and similar) as the primary way to **score and load**, without requiring every leaf repo to install Toolbox. Absorbing team/leaf toolboxes is **not** the primary path. Fleet-wide checks (for example zizmor) run from a central automation repo (CentralRepoOps-shaped), not only from per-repo Actions. An **org-operator starter** must work without sub-team portals. Local workspace files and org Contents/Files API loads share a documented precedence model. IDP migration from Backstage, DX, Port and peers is **light** (docs, prompts, skills, thin CLIs) — do not over-engineer importers.

**Amends** [D01M7Y46VDDBK2SW6H96ZSG89MP](./D01M7Y46VDDBK2SW6H96ZSG89MP.md) for **federation only**. M5 (D64) and M6 remain paused. Repo-value work may continue in parallel; leaf install stays valuable but is optional depth.

**Status:** ✅

**Where:** [design/federation.md](../design/federation.md), Project charter `docs/federation-operator-charter.md` (Toolbox for RTBuddy store)

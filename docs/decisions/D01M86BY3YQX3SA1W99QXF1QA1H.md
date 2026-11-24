---
id: D01M86BY3YQX3SA1W99QXF1QA1H
decided: 2026-11-24T09:00:00Z
---

# Self-service actions are federated: defined where the effect is owned, run where the credentials are

**Lifts D64's pause on M5, and the M5 pause that D01M7Y46VDDBK2SW6H96ZSG89MP and
D01M80Y0XEK069GD08VCCNF0WB8 kept. Revises D68.** An action is declared in the portal of whoever
owns its effect: a repo portal, or a package that repos include. A parent portal offers its
children's actions by reading their declarations, and a run goes down to the child, which
previews, runs and verifies it with its own executor and credentials. A run succeeds only when
its effect is observed.

1. **Why now:** the owner asked for self-service actions and federation together, from user
   stories grounded in market research. D64's condition is met: an independent adversarial review
   of self-service coordination reported ([self-service-review.md](../self-service-review.md),
   F1–F24). Every critical and high finding is fixed, and the rest are fixed or deferred with
   reasons. The fixes are folded into [action-runs.md](../design/action-runs.md),
   [federated-actions.md](../design/federated-actions.md) and v1-spec §6.6 and §7.5.
2. **What federation gains:** actions go up as data, through the workspace connector's `portal`
   records, and runs come down as a dispatch. No portal queries another at runtime; a run is the
   one call that goes down. A child is identified by where it was found (`portal:<key>`), never by
   the name it gives itself.
3. **The parent owns its runners; the child owns its effect.** A child names how it can be
   reached (`federation.runner`), and the parent decides whether to reach it, with its own
   allowlists and credentials (`federation.runners`). A child can't choose where a parent's token
   goes. Every executor starts in a cleared environment, which holds only the action's declared
   `secrets`.
4. **Approvals are a person's, signed and pinned.** A repo portal holds no approvals. A parent
   signs the approvals it collects (Ed25519, bound to the run, action version, inputs, effects
   and requester), and a child accepts only the keys it pins (`acceptApprovalsFrom`). Approvals
   need an interactive sign-in, never a token or MCP (D57). Nobody approves their own run, and
   each person approves once. A served run changes atomically, so two approvals or an approval
   and a cancel can't both act.
5. **One lifecycle at every level, and done means observed:** `toolbox run` (a checkout, no
   server), `toolbox serve` (REST and MCP), and a parent dispatching to a child share one
   preparation step, so a preview means the same everywhere. A change counts only once it is on
   its base. A postcondition that held before the run ends `unverified`, and a workflow that is
   never seen ends `timed_out`.
6. **Proof:** `scripts/m5-acceptance.sh` (in CI) covers the built stories and the review's races,
   and the DX benchmark [actions-ab](../../experiments/actions-ab/README.md) compares the result
   with GitHub Actions, Backstage's scaffolder and a vanilla app, against a bar set before the
   run.

**Status:** ✅

**Where:** [docs/design/federated-actions.md](../design/federated-actions.md), [docs/self-service-review.md](../self-service-review.md), [docs/design/action-runs.md](../design/action-runs.md), [docs/design/federation.md](../design/federation.md), [docs/v1-spec.md](../v1-spec.md) §3, §6.6, §7.5

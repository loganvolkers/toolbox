---
id: D01M8727W7FEED6VVRY6MB3JFR3
decided: 2026-11-24T23:00:00Z
---

# Who serverless approvals are for, and groups in a checkout

**Extends D01M86WAVQQXYF9DFHCP3G035MA.** That decision was motivated only by the actions DX
Benchmark, which makes it a fit risk. The benchmark's fit audit
([FIT-AUDIT.md](../../experiments/actions-ab/results/FIT-AUDIT.md)) asked for the outside use to
be named, and for the trust it rests on to be stated where operators read it. It also found that a
checkout could not resolve a group approver, so a rule as common as "someone in release approves"
needed a server.

1. **The outside use.** A parent portal's checkout holds approvals for setups where one machine,
   or one person, already holds the parent:
   - one operator running the org's portal from a laptop, before there is a team to sign in;
   - a trial of an action across repositories before anyone deploys a server;
   - a CI job or bastion host that keeps the parent's state directory, where the job's own
     access control decides who may run `--approve`.
   It is not the way for a team to approve: that remains `seikika serve` behind a proxy or OIDC
   (D57), where an approver is proven and the key stays on the server's host.
2. **The trust is stated where operators read it.** The `add-action` skill and
   [examples/federated-actions](../../examples/federated-actions/README.md) say that a checkout
   takes `--as` on trust, and when to serve instead.
3. **A checkout resolves group approvers from the portal's own model and tuples.** An approval
   entry `group:<name>` counts `who` when the portal's authorization model, evaluated over the
   tuples it derives (`authz.tuples`, static or emitted), makes `who` a `member` of the group:
   directly, through a nested group (`group:leads#member`), or through a wildcard. The evaluator
   (`seikika_authz::local`) is exact for direct types, `or`, computed relations and `from`, which
   follow from the tuples alone. It refuses `and` and `but not`, rather than grant more than
   OpenFGA would; a group defined that way needs a served portal. Without a model, a group is
   still met by nobody (T6). Groups are defined in the portal that decides approvals, which in a
   federation is the org's, so a group is declared once for every child.
4. **`seikika test` cases can say who is in a group.** A child's cases run without the org's
   portal, so a case's `members` (`group:release: [user:dave]`) stands in for it; groups the
   child's own model resolves count as well.

Rejected: OpenFGA as a co-process for every `--approve` in a checkout (it adds a binary to
the no-server path for a check the tuples settle exactly in the common case), and group members
listed under `federation` (a second way to declare membership beside the authorization model).

Owning docs: [federated-actions.md](../design/federated-actions.md) §5.3 (T6), the `add-action`
and `test` skills, [examples/federated-actions](../../examples/federated-actions/README.md),
`scripts/m5-acceptance.sh` step 11.

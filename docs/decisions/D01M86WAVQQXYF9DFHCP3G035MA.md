---
id: D01M86WAVQQXYF9DFHCP3G035MA
decided: 2026-11-24T18:00:00Z
---

# A parent portal holds approvals without a server

**Extends D01M86J769VGE6K5FJJQ76EP4AA.** Until now only a served portal kept approvals: a
request with approvers in any checkout stopped at exit 3, and approving needed `toolbox serve`,
a port, `TOOLBOX_URL` and a server that stayed up. In rounds 5 and 6 of the actions DX Benchmark
([experiments/actions-ab](../../experiments/actions-ab/README.md)) every Toolbox run spent calls
on that server rather than on the action: starting and stopping it, learning the CLI against it,
testing a restart, writing an `env.sh` to export `TOOLBOX_URL` after `toolbox run` without it
failed, and putting the server in the handoff. Round 6's toolbox-1 used 33 tool calls against
vanilla-1's 12, and the vanilla arm has no server to keep.

1. **A parent's checkout records and decides runs that wait for approval.** In a portal that
   federates children (`federation.children` or `federation.portals`), `toolbox run … --yes` on
   an action with approvers saves the run under `$TOOLBOX_STATE_DIR/runs`, exits 3 and names the
   command that approves it. `toolbox run --as <approver> --approve <id>` (or `--reject`) in that
   checkout applies the rules a server applies (T6): a pending approver, never the requester,
   each person once; a group can't be met without an authorization model. The last approval
   prepares the run again against today's facts, fails it `preview_stale` if the action or its
   effects changed, and runs it with a grant signed by the parent's key. A lock file keeps two
   approvals from both starting it. A repository's own checkout still holds no approvals (T4).
2. **One key, served or not.** The checkout signs with the seed a served parent keeps at
   `<state>/served/<portal>/signing-key` (or `federation.signingKey`), so children pin one key
   whichever way the parent runs. `toolbox run` in a parent's checkout prints the key after the
   listing, and as `approval_key` with `-o json`, so getting it needs no server.
3. **`--as` names a principal as a server does.** `--as ann` and `--as ann@acme.dev` are
   `user:ann` in a checkout too, so a requester written either way can't approve their own run.
4. **A server stays the way to approve as a team.** It adds sign-in (D57: no token or MCP
   approves), the UI, REST and MCP, and keeps its runs apart from the checkout's; a run is
   approved where it was requested.

**Trust.** A checkout takes `--as` on trust, as a dev server on a loopback address does. That
grants nothing new: whoever can use the state directory can read the signing key, and could sign
a grant directly. An operator who needs approvers proven puts the parent behind `toolbox serve`
with a proxy or OIDC and keeps the key on that host only.

Rejected: finding a running server from the state directory when `TOOLBOX_URL` is unset (it keeps
the server as the only way, and a stale pid or address would pick the wrong one), and an approval
file committed to the parent's repository (git history is not a lock, and two approvers could
both start a run).

Owning docs: [federated-actions.md](../design/federated-actions.md) §5.2, §5.3 (T4, T6), §6.4,
[examples/federated-actions](../../examples/federated-actions/README.md), the `add-action` and
`run` skills.

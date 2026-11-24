---
name: run
description: >-
  Previews and runs a self-service action in a repo portal, a parent portal that dispatches to its children, or a served portal (--url). Use when an agent should change something through a declared action, and when a proposed or dispatched run needs checking again.
---

# toolbox run

1. Run it with no action to list what this portal offers, with each action's inputs.
2. Preview first. Read the effects, blockers and approvers, and keep the confirm token.
3. Run with `--confirm <token>` within ten minutes. A stale token exits 1: preview again. A served
   portal spends each token once (`409 token-used`), so preview again for a second run.
4. Read the state. Only `succeeded` means the effect was observed: the postcondition held, or a
   `change`'s commit is on its base. `unverified` means no postcondition, or one that already
   held before the run, so the run proved nothing. For a `change`, Toolbox fetches the base
   and compares every file the commit changed (`verification.files`, with each blob); a change
   that would differ from the confirmed preview fails before it commits. No extra check script
   is needed. Its commit carries `Toolbox-Run`, `Requested-By` and `Approved-By` trailers. A
   `change` its base already has commits nothing and succeeds (`outputs.changed: false`).
5. For a `proposed` change or a `dispatched` workflow, check again later with `--verify <run id>`,
   or add `--wait` to follow the run until it ends or waits for a person.

On a served portal, add `--url <server>` (and `--token`, or `--as` on a server listening on a
loopback address) to every command: the same flags, output and exit codes, with the server's
authorization and approvals. `--runs` lists runs, and `--verify <run id>` prints one's record.
`TOOLBOX_URL` and `TOOLBOX_AS` set them for every command, so a script needs no wrapper.

To offer several repositories' actions from one place, list them in an org portal
(`federation.children: [{git: <url>}]`), serve it with `toolbox serve <org> --detach`, and pin
the key it prints in each repository's `federation.acceptApprovalsFrom`. The README of
`examples/federated-actions` in the Toolbox repository has the two manifests, every field and
every command.

```
toolbox run -o json
toolbox run set-owners --entity repository:alpha -i team=@acme/alpha -o json
toolbox run set-owners --entity repository:alpha -i team=@acme/alpha --confirm ct_… -o json
toolbox run --verify run_… --wait
toolbox run --url https://portal.example add-security-policy --entity portal:gamma -i contact=security@acme.dev --yes --wait -o json
```

Exit codes: 0 succeeded, proposed or dispatched; 1 failed, stale or timed out; 2 invalid inputs or refused; 3 awaiting approval; 4 unverified (the executor finished, but nothing observed its effect). A repo portal holds no approvals. An action with approvers needs a served parent portal: request it there with `--url`, and a person approves it there, signed in (the UI, or `toolbox run --url <server> --approve <run id>`). No token and no MCP tool can approve, so tell the approver the run id and that command, and stop. A `change` with `land: push` lands on the base once approved and needs no review; any other `change` proposes a branch, and merging it is the approval. The parent signs the approvals and the child checks the signature against the key in its `federation.acceptApprovalsFrom`. A preview blocked by `child-refuses-approvals` means that key is missing. Never pass `--grant` or `--parent-run` yourself. In a parent portal, a child's action takes `--entity portal:<child>` and the child previews, runs and verifies it. A served parent runs a `local` child only if `federation.runners.local.portals` lists it; otherwise it returns `runner-not-allowed`.

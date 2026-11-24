---
name: run
description: >-
  Previews and runs a self-service action without a server, in a repo portal or a parent portal that dispatches to its children. Use when an agent should change something through a declared action, and when a proposed or dispatched run needs checking again.
---

# toolbox run

1. Run it with no action to list what this portal offers, with each action's inputs.
2. Preview first. Read the effects, blockers and approvers, and keep the confirm token.
3. Run with `--confirm <token>` within ten minutes. A stale token exits 1: preview again.
4. Read the state. Only `succeeded` means the postcondition was observed.
5. For a `proposed` change or a `dispatched` workflow, check again later with `--verify <run id>`.

```
toolbox run -o json
toolbox run set-owners --entity repository:alpha -i team=@acme/alpha -o json
toolbox run set-owners --entity repository:alpha -i team=@acme/alpha --confirm ct_… -o json
toolbox run --verify run_… --wait
```

Exit codes: 0 succeeded, proposed or dispatched; 1 failed or stale; 2 invalid inputs or refused; 3 awaiting approval. A repo portal holds no approvals. An action with approvers needs a served parent portal: request it there, and a person approves it in that portal's UI or REST API. The parent signs the approvals and the child checks the signature against the key in its `federation.acceptApprovalsFrom`. A preview blocked by `child-refuses-approvals` means that key is missing. Never pass `--grant` or `--parent-run` yourself. In a parent portal, a child's action takes `--entity portal:<child>` and the child previews, runs and verifies it. A served parent runs a `local` child only if `federation.runners.local.portals` lists it; otherwise it returns `runner-not-allowed`.

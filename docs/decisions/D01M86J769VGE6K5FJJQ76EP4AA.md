---
id: D01M86J769VGE6K5FJJQ76EP4AA
decided: 2026-11-24T14:00:00Z
---

# An org portal lists its children by git URL, and the shortest federated action is two short manifests

**Extends D01M86BY3YQX3SA1W99QXF1QA1H.** Round 1 of the actions DX Benchmark
([experiments/actions-ab](../../experiments/actions-ab/README.md)) showed agents in the Toolbox arm
spending most of their calls on glue: a workspace connector and a `Portal` kind to declare, a
script to keep each child's checkout current, a signing key to generate and keep, and an auth mode
to find. None of it was the action. Each of these is now Toolbox's default, so an org portal is
a list of repositories and a repo's action is its inputs, its approvers and its command.

1. **`federation.children: [{git, branch?, key?}]`.** `git` takes anything `git clone` does, a
   path relative to the manifest included. The key defaults to the last segment of the URL,
   without `.git`, and the child is offered as `portal:<key>`. The model expands the list into
   the connector, the `Portal` kind, `federation.portals` and `runners.local.portals`, unless
   the manifest declares them itself. Toolbox keeps one clone per child under the state
   directory, at `branch` (default `main`).
2. **Children are read again before they are used.** A served parent refreshes its children
   before it lists, previews or runs an action, and the executor brings the child's clone to its
   origin branch before the child previews or runs. A key pinned and pushed by a child counts on
   the next request, not after a refresh interval.
3. **The parent keeps its own signing key.** With `federation.portals` set and no `signingKey`,
   the served portal generates an Ed25519 seed once, keeps it at
   `<state>/served/<portal>/signing-key` (mode 0600), and returns its public key as
   `approval_key` from `GET /v1/actions`. `toolbox run --url <server>` prints the line a child
   pins. A request whose child doesn't pin it is refused with that line (exit 2).
4. **Actions on `kind: Portal` in a repo portal need no kind.** The portal declares a `self`
   entity of kind `Portal` for itself.
5. **An action's `scope` defaults to `day2` and its `permission` to `can_run_actions`.**
6. **`toolbox serve --help` says how people sign in**: with `--as` on a loopback address (the
   existing default), and otherwise through the `TOOLBOX_AUTH_MODE` it lists.
7. **Children that can't be read are listed**, as `unreadable` with their error, instead of
   silently offering nothing.

Rejected: a hosted registry of children (a second source of truth beside git), and cloning on
every request (slow, and a network failure would hide every action).

Owning docs: [federated-actions.md](../design/federated-actions.md) §5,
[examples/federated-actions](../../examples/federated-actions/README.md).

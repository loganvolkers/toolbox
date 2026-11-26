---
title: Test and run an action
description: Try an action in a disposable checkout, review its preview, and verify its result.
sidebar:
  order: 3
---

A self-service action lets you request a change defined by a portal, such as proposing a file update. The portal validates the inputs and shows the effects before execution. Repo portals and parent portals can run actions through the CLI. A served portal also exposes action endpoints and MCP tools.

## Try action tests without changing a real repository

From a [Seikika source checkout](/start/install/#build-from-source), use Bash and a CLI on `PATH`. Copy the example out of the checkout so the test has a disposable Git repository.

```bash
practice_dir=$(mktemp -d)
cp -R examples/federated-actions/minimal-repo/. "$practice_dir/"
git -C "$practice_dir" init -b main
git -C "$practice_dir" add .
git -C "$practice_dir" -c user.name='Seikika example' \
  -c user.email='example@example.com' commit -m 'Add action example'
(
  cd "$practice_dir"
  seikika validate .seikika
  seikika test .seikika
)
```

The test runs approved and refused requests in scratch copies. Look for passing cases for an approved change, invalid input, refused approvers, and a rejected Git push. The example leaves the practice repository's commit unchanged.

## Request an action on a portal

1. Ask the portal operator which actions are available and which identities can request them. Use `seikika run` from a portal checkout to list its actions. For a served portal, add `--url` and authenticate on each call.
2. Preview the selected action with `seikika run ACTION --entity ENTITY -i KEY=VALUE`. Replace the uppercase names with the action name, target entity ID, and required input. Read what the action would change, any blockers, and who must approve.
3. Use the same command with `--confirm` and the preview's confirm token when you are ready to request the change. Preview again if the token expires. A served portal accepts each token once.
4. Save the run ID and check it with `seikika run --verify RUN_ID`. Use the same portal and authentication as the request.

| State | What to check |
|---|---|
| `awaiting_approval` | Send the run ID to an authorized approver. The requester cannot approve their own run. |
| `proposed` or `dispatched` | Follow the review branch or workflow, then check the run again. |
| `succeeded` | Seikika observed the required result, such as a commit on the target branch. |
| `failed` or `unverified` | Read the run record and inspect the target before retrying. An effect may already have occurred. |

For the operator's setup and approval flow, see the [federated actions example](https://github.com/seikika/seikika/tree/main/examples/federated-actions). For exact flags and exit codes, use [CLI reference](/start/cli/) or `seikika run --help`.

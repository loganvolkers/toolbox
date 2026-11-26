---
title: Define a self-service action
description: Configure action inputs, approvals, execution, and tests before offering it to users.
sidebar:
  order: 8
---

Define an action in the repository that owns the change. The action says what a user can request, who must approve, and how Seikika can tell that it worked. Follow [Federate repository actions](/operators/federate-actions/) when people should request actions from a shared parent.

## Define the change and its inputs

Use the [practice repository](/users/actions/#try-action-tests-without-changing-a-real-repository) to try the example with a source-built 0.2 CLI. Its manifest includes the files under `.seikika/actions/`. The example adds a code of conduct with an address provided by the requester.

This excerpt shows the action's fields. The practice file also configures its federation runner and test file.

```yaml
actions:
  - name: add-code-of-conduct
    kind: Portal
    description: Add a code of conduct with a contact address.
    approval: "[['user:ann', 'user:bob']]"
    inputs:
      type: object
      required: [contact]
      properties:
        contact: {type: string, pattern: '^[^@\s]+@[^@\s]+$'}
    preconditions: ["!inputs.contact.startsWith(entity.name + '@')"]
    executor:
      type: change
      land: push
      base: main
      run: >-
        printf '# Code of conduct\n\nReport conduct issues to %s.\n'
        "$SEIKIKA_INPUT_CONTACT" > CODE_OF_CONDUCT.md
```

`kind: Portal` targets the repository portal. The JSON Schema checks the input before preview. The command receives it as `SEIKIKA_INPUT_CONTACT`. The precondition uses Common Expression Language (CEL) to block a contact that starts with the repository's own name before the `@`.

Each approval entry needs an approval. A user entry names one person. A group entry needs an eligible member of that group. A nested list such as the one above needs one member of that list. The requester cannot approve. A shared parent resolves group membership using its authorization model.

## Choose an executor and a verification method

| Goal | Configuration |
|---|---|
| Propose a repository change for review | Use `executor.type: change` with `land: propose`, the default. Seikika pushes a review branch and checks whether the commit reaches the base. |
| Land an approved change directly | Use `land: push` when the destination allows the push. The preview shows the proposed file edits. The run checks the commit and its changed files on the base. |
| Run a command or dispatch a workflow | Use the executor that owns the effect. Where the executor supports it, set `expect.postcondition` to a check that demonstrates the effect. |

Keep a repository change's effects inside its worktree. Seikika previews a `change` by running its command in a scratch worktree, then showing the diff. A side effect outside that worktree can happen during preview. Put access credentials in the executor's environment through the configured secret mechanism. Use the least access needed for the target.

`succeeded` means Seikika observed the expected result. A completed command with no postcondition, or with a condition that was already true, can finish as `unverified`. See the [executor and action schema](https://github.com/seikika/seikika/blob/main/crates/seikika-model/schema/manifest.schema.json) for all executor types and options.

## Test allowed and refused requests

From the practice checkout, run:

```bash
seikika validate .seikika
seikika test .seikika
```

Expect five passing cases. They check who may approve, what happens with invalid input, what happens when a precondition blocks a run, and how a rejected push surfaces. The approved case checks the exact file on the scratch base. `seikika test` uses scratch repositories and leaves the real origin alone.

When you add an action, include a case that checks the expected effect and cases for its most likely refusals. Start from the [example test file](https://github.com/seikika/seikika/blob/main/examples/federated-actions/minimal-repo/.seikika/tests/add-code-of-conduct.test.yaml). For a group approval case, set `members` in the test so the fixture can resolve who belongs to the group.

See [Test and run an action](/users/actions/) for request, approval, and recovery steps.

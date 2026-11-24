---
name: add-action
description: >-
  Adds a self-service action that a repository owns, tests it, and offers it with those of other repositories from one org portal, where people request, approve and check runs. Use when asked to let people or agents change repositories through a declared, approved and audited action.
---

# Add an action

This skill is the whole path, for one repository or a fleet: you need no other skill, example or
`--help` to follow it, and no server.

1. Lay out one standard action file and one test file, the same in every repository.
2. Declare the action: inputs, approvers, preconditions and the change it makes.
3. Test it in each repository with `toolbox test .toolbox`.
4. Write an org portal over the repositories, and pin its key in the standard file.
5. Request, approve and check runs from the org portal's directory.
6. Hand it off.

Each step is one shell command: write every file with heredocs at once, then copy the standard
files to each repository, then validate, test, commit and push all of them in one loop.

## 1. Lay it out

An action that several repositories offer is a standard: two files that are the same, byte for
byte, in every repository. Write them once and copy them. Only `toolbox.yaml` is each
repository's own.

```
<repo>/.toolbox/toolbox.yaml                     # this repository: its name, owner and include
<repo>/.toolbox/actions/add-code-of-conduct.yaml # the action: the same file in every repository
<repo>/.toolbox/tests/add-code-of-conduct.test.yaml # its cases: the same file too
org/toolbox.yaml                                 # the org portal: lists the repositories
```

```yaml
# <repo>/.toolbox/toolbox.yaml
apiVersion: toolbox.dev/v1alpha1
portal: {name: alpha, owner: group:alpha-maintainers, toolbox: "0.1"}
include: [actions/*.yaml]
```

## 2. Declare the action

```yaml
# <repo>/.toolbox/actions/add-code-of-conduct.yaml
federation:
  runner: {type: local} # the org portal runs it in its clone of this repository
  acceptApprovalsFrom: [{portal: org, key: "ed25519:<the key the org portal prints>"}]
tests: [tests/add-code-of-conduct.test.yaml]
actions:
  - name: add-code-of-conduct
    kind: Portal # the repository itself
    description: Publish a CODE_OF_CONDUCT.md with the team's contact on main.
    approval: "[['user:ann', 'user:bob']]" # either one; the requester never counts
    inputs:
      type: object
      required: [contact]
      additionalProperties: false
      properties: {contact: {type: string, pattern: '^[^@\s]+@[^@\s]+$'}}
    preconditions: ["!inputs.contact.startsWith(entity.name + '@')"]
    executor:
      type: change
      land: push # commit to main once approved; succeeded only once main has it
      base: main
      message: "add-code-of-conduct for {entity} (run {run})"
      run: >-
        printf '# Code of conduct\n\nReport conduct issues to %s.\n'
        "$TOOLBOX_INPUT_CONTACT" > CODE_OF_CONDUCT.md
```

Inputs reach the command as `TOOLBOX_INPUT_<NAME>`. Toolbox checks them against the schema
(`pattern`, `minLength`, `maxLength`, `enum`, `required`), previews the diff, holds the approval,
commits with `Toolbox-Run`, `Requested-By` and `Approved-By` trailers, and verifies the files on
the fetched base. Write no script for any of that. A rule the schema can't state goes in
`preconditions`, CEL over `inputs` and `entity`; a false one blocks the run. `entity.name` is the
name of the repository that holds the file, so a standard never names its repository.

## 3. Test it

```yaml
# <repo>/.toolbox/tests/add-code-of-conduct.test.yaml
name: add-code-of-conduct
defaults: {action: add-code-of-conduct, as: user:ann, inputs: {contact: conduct@acme.dev}, approve: [user:bob]}
runs:
  - name: ann requests, bob approves, and CODE_OF_CONDUCT.md lands on main
    expect: {state: succeeded, files: {CODE_OF_CONDUCT.md: "# Code of conduct\n\nReport conduct issues to conduct@acme.dev.\n"}}
  - name: an invalid contact is refused
    inputs: {contact: nobody}
    expect: {state: invalid, files: {CODE_OF_CONDUCT.md: null}}
  - name: the repository's own address is blocked
    inputs: {contact: '{portal}@acme.dev'}
    expect: {state: blocked}
  - name: ann cannot approve her own run, and carol is not an approver
    approve: [user:ann, user:carol]
    expect: {state: awaiting_approval, refused: [user:ann, user:carol]}
  - name: a push the origin rejects fails the run
    rejectPush: main is protected
    expect: {state: failed, error: main is protected, files: {CODE_OF_CONDUCT.md: null}}
```

`toolbox validate .toolbox && toolbox test .toolbox` in each repository runs every case through
the code `toolbox serve` runs, in a scratch copy of the repository and its origin's `main`, before
you commit. Nothing reaches the real origin. `{portal}` is the repository under test. Inputs
merge into `defaults` key by key (`null` leaves one out); quote a value with a comma inside
`{…}`. A passing case needs no live rehearsal: cite it in your handoff.

## 4. Offer it from one place

```yaml
# org/toolbox.yaml
apiVersion: toolbox.dev/v1alpha1
portal: {name: org, owner: group:platform, toolbox: "0.1"}
federation:
  children: [{git: ../origins/alpha.git}, {git: https://github.com/acme/beta.git}]
```

Each child is offered as `portal:<last part of its URL>`; `git` takes anything `git clone` does,
including a path relative to this file. `toolbox run` in `org/` lists every child's actions and
ends with the key the org portal signs approvals with:

```sh
(cd org && toolbox run) | grep -o 'ed25519:[0-9a-f]*' | head -1   # made once, then reused
```

Toolbox keeps the key, the children's clones and the runs under `$TOOLBOX_STATE_DIR` (default
`~/.local/state/toolbox`); use the same one for every command, and name it in your handoff if you
set it. Paste the key into the standard file, then copy it to every repository, test, commit and
push. `toolbox serve org --detach` serves the same portal (UI, REST, MCP) with the same key, for
approvers on other machines; nothing below needs it.

## 5. Request, approve, check

From `org/`; `-o json` works on every command.

```sh
toolbox run --as ann                                                                        # every repository's actions
toolbox run --as ann add-code-of-conduct --entity portal:alpha -i contact=conduct@acme.dev        # preview: the diff
toolbox run --as ann add-code-of-conduct --entity portal:alpha -i contact=conduct@acme.dev --yes  # exit 3; prints the run id
toolbox run --as bob --approve <run-id> --wait                                              # exit 0 once main has it
toolbox run --as ann --verify <run-id>                                                      # the record
toolbox run --as ann --runs                                                                 # every run
```

Exit codes: 0 succeeded, 1 failed, 2 invalid or refused, 3 waiting for approval, 4 unverified.
With `-o json`, `--yes`, `--approve` and `--verify` print the run's record: `id`, `state`,
`requested_by`, `inputs`, `approval.approved_by`, `effects` (the diff), `events` (each with `at`)
and `verification`. The org portal applies the approval rule: the requester and anyone not listed
are refused (exit 2), and the last approval pushes, then verifies on the repository's `main`. A
rejected push fails the run with the origin's message. A run whose content the base already has
succeeds without a commit. The test cases already cover refusals and a rejected push; one real
run is enough.

## 6. Hand it off

Say where each file is, the `TOOLBOX_STATE_DIR` if you set one, and the commands in step 5; name
the test cases that cover refusals and a rejected push.

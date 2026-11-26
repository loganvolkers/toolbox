---
name: add-action
description: >-
  Adds a self-service action that a repository owns, tests it, and offers it with those of other repositories from one org portal, where people request, approve and check runs. Use when asked to let people or agents change repositories through a declared, approved and audited action.
---

# Add an action

An action a repository owns is a file in that repository's `.seikika/`. An org portal lists the
repositories and offers all their actions in one place, where people request, approve and check
runs. This skill covers the whole path, for one repository or many:

1. Write the org portal over the repositories, and take the key it signs approvals with.
2. Declare the action, pinned to that key: inputs, approvers, preconditions and the change it makes.
3. Write its test cases.
4. Copy the files to each repository, then validate, test, commit and push them.
5. Request, approve and check runs from the org portal's directory.
6. Tell the people who will request and approve runs how to do it.

## 1. Lay it out, starting with the org portal

An action that several repositories offer is a standard: two files that are the same, byte for
byte, in every repository. Write them once and copy them. Only `seikika.yaml` is each
repository's own.

```
<repo>/.seikika/seikika.yaml                        # this repository: its name, owner and include
<repo>/.seikika/actions/add-code-of-conduct.yaml    # the action: the same file in every repository
<repo>/.seikika/tests/add-code-of-conduct.test.yaml # its cases: the same file too
<org>/seikika.yaml                                  # the org portal: lists the repositories
```

```yaml
# <repo>/.seikika/seikika.yaml
apiVersion: seikika.com/v1alpha1
portal: {name: web, owner: group:web-maintainers, seikika: "0.1"}
include: [actions/*.yaml]
```

```yaml
# <org>/seikika.yaml
apiVersion: seikika.com/v1alpha1
portal: {name: org, owner: group:platform, seikika: "0.1"}
federation:
  children:
    - git: https://github.com/acme/web.git          # offered as portal:web
    - {git: git@github.com:acme/api.git, branch: trunk}
```

Each child is offered as `portal:<last part of its URL>`. `git` takes anything `git clone` does,
including a path relative to this file. `seikika run` in the org portal's directory lists every
child's actions and ends with the key the org portal signs approvals with. It prints the key
before any child has an action, so take it before you write the action file:

```sh
KEY=$(cd <org> && seikika run | grep -o 'ed25519:[0-9a-f]*' | head -1)   # made once, then reused
```

Seikika keeps the key, the children's clones and the runs under `$SEIKIKA_STATE_DIR` (default
`~/.local/state/seikika`). Use the same one for every command, and tell the people who run the
action if you set it.

## 2. Declare the action

```yaml
# <repo>/.seikika/actions/add-code-of-conduct.yaml
federation:
  runner: {type: local} # the org portal runs it in its clone of this repository
  acceptApprovalsFrom: [{portal: org, key: "ed25519:<key>"}] # the key from step 1
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
        "$SEIKIKA_INPUT_CONTACT" > CODE_OF_CONDUCT.md
```

Replace `ed25519:<key>` with the key from step 1. If you write the file with a shell heredoc, quote
it (`<<'EOF'`), so `$SEIKIKA_INPUT_CONTACT` stays as written.

Inputs reach the command as `SEIKIKA_INPUT_<NAME>`. Seikika checks them against the schema
(`pattern`, `minLength`, `maxLength`, `enum`, `required`), previews the diff, holds the approval,
commits with `Seikika-Run`, `Requested-By` and `Approved-By` trailers, and verifies the files on
the fetched base. Write no script for any of that. A rule the schema can't state goes in
`preconditions`, CEL over `inputs` and `entity`; a false one blocks the run. `entity.name` is the
name of the repository that holds the file, so a standard never names its repository.

`approval` is a list of entries, each of which must approve: a user (`user:ann`), a group
(`group:security`), or a list meaning any one of them. The requester never counts, and each
person approves once.

A group's members come from the authorization model of the portal that decides approvals, the
org portal here, so define each group once there rather than in every repository:

```yaml
# <org>/seikika.yaml, beside federation
authz:
  model: groups.fga
  tuples:
    - static:
        - {user: user:ann, relation: member, object: group:security}
        - {user: group:sre#member, relation: member, object: group:security} # everyone in sre
```

```
# <org>/groups.fga
model
  schema 1.1
type user
type group
  relations
    define member: [user, group#member]
```

The org portal's checkout and its server read the same model. A group nobody can be shown to be
in can't approve: without a model, no group is ever met.

## 3. Write its test cases

```yaml
# <repo>/.seikika/tests/add-code-of-conduct.test.yaml
name: add-code-of-conduct
defaults: {action: add-code-of-conduct, as: user:ann, inputs: {contact: conduct@acme.dev}, approve: [user:bob]}
runs:
  - name: an approved run lands CODE_OF_CONDUCT.md on main
    expect: {state: succeeded, files: {CODE_OF_CONDUCT.md: "# Code of conduct\n\nReport conduct issues to conduct@acme.dev.\n"}}
  - name: an invalid contact is refused
    inputs: {contact: nobody}
    expect: {state: invalid, files: {CODE_OF_CONDUCT.md: null}}
  - name: the repository's own address is blocked
    inputs: {contact: '{portal}@acme.dev'}
    expect: {state: blocked}
  - name: neither the requester nor someone unlisted can approve
    approve: [user:ann, user:dana]
    expect: {state: awaiting_approval, refused: [user:ann, user:dana]}
  - name: a push the origin rejects fails the run
    rejectPush: main is protected
    expect: {state: failed, error: main is protected, files: {CODE_OF_CONDUCT.md: null}}
```

`seikika validate .seikika && seikika test .seikika` in each repository runs every case through
the code `seikika serve` runs, in a scratch copy of the repository and its origin's `main`, before
you commit. Nothing reaches the real origin. `{portal}` is the repository under test. Inputs
merge into `defaults` key by key (`null` leaves one out); quote a value with a comma inside
`{…}`. The last two cases are the refusals every approved action should have: the requester,
someone the rule doesn't list, and a protected branch. A repository's cases run without the org
portal, so a case that approves as a group member says who is in it:
`members: {'group:security': [user:ann]}`.

## 4. Copy, test, commit and push

Copy the standard files from the first repository to the others, then test, commit and push each,
stopping at the first failure:

```sh
REPOS="web api docs"; first=${REPOS%% *}
(set -e; for r in $REPOS; do
  [ "$r" = "$first" ] || cp -r "$first/.seikika/actions" "$first/.seikika/tests" "$r/.seikika/"
  (cd "$r" && seikika validate .seikika && seikika test .seikika && git add .seikika &&
    git commit -qm "Add the add-code-of-conduct action" && git push -q origin HEAD)
done)
```

## 5. Request, approve, check

From the org portal's directory; `-o json` works on every command.

```sh
seikika run --as ann                                                                       # every repository's actions
seikika run --as ann add-code-of-conduct --entity portal:web -i contact=conduct@acme.dev        # preview: the diff
seikika run --as ann add-code-of-conduct --entity portal:web -i contact=conduct@acme.dev --yes  # exit 3; prints the run id
seikika run --as bob --approve <run-id> --wait                                             # exit 0 once main has it
seikika run --as ann --verify <run-id>                                                     # the record
seikika run --as ann --runs                                                                # every run
```

Exit codes: 0 succeeded, 1 failed, 2 invalid, blocked or refused, 3 waiting for approval, 4 unverified.
With `-o json`, `--yes`, `--approve` and `--verify` print the run's record: `id`, `state`,
`requested_by`, `inputs`, `approval.approved_by`, `effects` (the diff), `events` (each with `at`)
and `verification`. The org portal applies the approval rule: the requester and anyone not listed
are refused (exit 2), and the last approval pushes, then verifies on the repository's `main`. A
rejected push fails the run with the origin's message. A run whose content the base already has
succeeds without a commit.

**Who may say `--as`.** A checkout takes `--as` on trust: whoever can use its state directory
holds the signing key anyway. That suits one operator, a trial, or a CI job or bastion host that
alone holds the state directory. When everyone has a token on the forge, mark that forge
`identity: true` in `forges:` instead: then each person runs `seikika run --identity-token-file
<their token file>` (no `--as`), Seikika asks the forge who owns the token, and `--as` alone is
refused (a `read_user` token is enough on GitLab). For a team, serve the portal behind sign-in instead
(`seikika serve <org>` with a proxy or OIDC, see the `serve` skill): there an approval needs an
interactive sign-in, and `seikika run --url <server>` requests and checks runs.

## 6. Tell people how to use it

Give requesters and approvers the commands in step 5, the `SEIKIKA_STATE_DIR` if you set one, and
where the action and its test cases live, so they can change the action by changing those files.

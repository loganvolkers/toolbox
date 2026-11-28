---
name: test
description: >-
  Recomputes a portal from its fixtures and runs its test files: CEL expectations, and action cases that request, approve and run an action in a scratch copy of the repository. Use when scorecard rules, fixtures, expectations or an action changed, and after validate has passed.
---

# seikika test

1. Validate the portal first, so a schema error is not reported as a failed expectation.
2. Run the command on the portal directory. Each expectation and case prints pass or fail.
3. Fix the rule, the action, the fixture or the expectation that failed, then run it again.

```
seikika test .seikika
seikika test .seikika
```

An action case goes under `runs:` in a file the manifest lists in `tests`. It runs the real
preview, approvals, change and check in a scratch clone whose origin holds the base with your
working tree on top, so you can test before you commit. Nothing reaches the real origin and no
run is recorded, so test refusals and a rejected push here, not against a live portal.

```yaml
name: add-security-policy
defaults: {action: add-security-policy, as: user:ann, inputs: {contact: security@acme.dev}, approve: [user:bob]}
runs:
  - name: bob approves ann's request, and SECURITY.md lands on main
    expect: {state: succeeded, files: {SECURITY.md: "# Security\n\nReport a vulnerability to security@acme.dev.\n"}}
  - name: a contact that is not an email address is refused
    inputs: {contact: nobody}
    expect: {state: invalid, files: {SECURITY.md: null}}
  - name: ann and carol cannot approve
    approve: [user:ann, user:carol]
    expect: {state: awaiting_approval, refused: [user:ann, user:carol]}
  - name: a rejected push fails the run
    rejectPush: main is protected
    expect: {state: failed, error: main is protected, files: {SECURITY.md: null}}
```

Every case starts from `defaults`. Its own fields replace them, except `inputs`, which merge
key by key; an input set to `null` is left out. `expect.state` is a run state, or `invalid`
(inputs refused), `blocked` (a precondition failed) or `refused`. `files` gives each file's
exact content on the base afterwards; `null` means absent. Quote any value with a comma inside
`{…}`: YAML ends the value at the comma. `{portal}` in any value is the name of the portal under
test, so one test file serves every repository that includes the same action
(`inputs: {contact: '{portal}@acme.dev'}`). A group approver counts a member of the group
in the portal's own authorization model, or one the case lists in `members`
(`members: {'group:security': [user:bob]}`), which stands in for an org portal's groups.

Do not weaken an expectation to make it pass. If it fails, the check, the action or the fixture
is wrong.

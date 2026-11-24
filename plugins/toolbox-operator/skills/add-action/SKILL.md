---
name: add-action
description: >-
  Adds a self-service action that a repository owns, tests it, and offers it with those of other repositories from one org portal, where people request, approve and check runs. Use when asked to let people or agents change repositories through a declared, approved and audited action.
---

# Add an action

1. **Declare it in the repository**, in `.toolbox/toolbox.yaml`, and commit it there. This is
   `examples/federated-actions/minimal-repo`; its README's field table lists every other field.

   ```yaml
   apiVersion: toolbox.dev/v1alpha1
   portal: {name: alpha, owner: group:alpha-maintainers, toolbox: "0.1"}
   federation:
     runner: {type: local} # a served org portal may run it in its clone of this repository
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
       executor:
         type: change
         land: push # commit to main once approved; succeeded only once main has it
         base: main
         message: "add-code-of-conduct for {entity} (run {run})"
         run: >-
           printf '# Code of conduct\n\nReport conduct issues to %s.\n'
           "$TOOLBOX_INPUT_CONTACT" > CODE_OF_CONDUCT.md
   ```

   Inputs reach the command as `TOOLBOX_INPUT_<NAME>`. Toolbox checks them, previews the diff,
   holds the approval, commits with `Toolbox-Run`, `Requested-By` and `Approved-By` trailers,
   and verifies the files on the fetched base. Write no script for any of that.
2. **Test it** before anyone runs it: `toolbox validate .toolbox && toolbox test .toolbox`. The
   `test` skill has the case format: the approval, invalid inputs, self-approval, someone who
   may not approve, and a push the origin rejects, each run in a scratch copy of the repository.
3. **Offer it from one place.** An org portal lists the repositories; each is offered as
   `portal:<last part of its URL>`. `git` takes anything `git clone` does, including a path
   relative to the org's `toolbox.yaml`.

   ```yaml
   apiVersion: toolbox.dev/v1alpha1
   portal: {name: org, owner: group:platform, toolbox: "0.1"}
   federation:
     children: [{git: https://github.com/acme/alpha.git}, {git: https://github.com/acme/beta.git}]
   ```

   `toolbox serve org --listen 127.0.0.1:7100 --detach` returns once ready and prints
   `approval key: ed25519:…`. The key is kept with the runs under
   `$TOOLBOX_STATE_DIR/served/org/` and reused on restart, so you can serve first and paste it
   into each repository's `acceptApprovalsFrom` as you write it. `toolbox serve org --stop`
   stops it.
4. **Request, approve, check.** Set `TOOLBOX_URL=http://127.0.0.1:7100`, then add `-o json` to
   any command for machine-readable output:

   ```sh
   toolbox run --as ann                                                       # list every repository's actions
   toolbox run --as ann add-code-of-conduct --entity portal:alpha -i contact=conduct@acme.dev        # preview
   toolbox run --as ann add-code-of-conduct --entity portal:alpha -i contact=conduct@acme.dev --yes  # exit 3: the run id
   toolbox run --as bob --approve <run-id> --wait                             # exit 0 once main has it
   toolbox run --as ann --verify <run-id>                                     # the record: who, when, what was seen
   toolbox run --as ann --runs                                                # every run
   ```

Exit codes: 0 succeeded, 1 failed, 2 invalid or refused, 3 waiting for approval, 4 unverified.
A rejected push fails the run with the origin's message; a run whose content the base already
has succeeds without a commit. The `run` skill covers the rest of the run lifecycle.

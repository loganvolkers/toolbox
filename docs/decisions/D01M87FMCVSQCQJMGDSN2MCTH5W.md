---
id: D01M87FMCVSQCQJMGDSN2MCTH5W
decided: 2026-11-24T19:30:00Z
---

# A checkout can take who you are from your own forge token

**Extends D01M8727W7FEED6VVRY6MB3JFR3 (T6) and D01M87AXG095Q0Z0DGGTKYYAM7Y.** A parent's checkout
took `--as` on trust, so a team on GitLab with no `toolbox serve` had no way to know who asked or
who approved. The round 0 dry run of the GitLab DX Benchmark
([experiments/actions-gitlab](../../experiments/actions-gitlab/README.md)) found it: the Toolbox
arm wrote a wrapper that read each person's GitLab token, asked GitLab who owned it, and passed
the answer as `--as`. Every organisation that approves in a shared checkout or a CI job, where
each person already holds a forge token, needs the same thing.

1. **`toolbox run --identity-token-file <file>`, or `TOOLBOX_IDENTITY_TOKEN`, says who you are.**
   Toolbox asks the forge who owns the token (GitLab `GET /user`, an active account; GitHub
   `GET /user`) and acts as `user:<username>`. A `--as` that names someone else is refused (exit
   2). The token goes to that forge's API and nowhere else: not to an action's process (it is not
   in `BASE_ENV`), not into a run record.
2. **`forges[].identity: true` makes it required.** In a checkout of that portal, a request, an
   approval or a rejection without an identity token is refused, naming the flag. At most one
   forge sets it (`toolbox validate`). Without a marked forge, a token is checked against the
   portal's only forge. A child run by its parent (`--parent-run`, with a signed grant) is exempt:
   the parent vouches for the requester.
3. **This narrows T6; it doesn't replace sign-in.** Whoever can use the checkout's state directory
   still holds its signing key. The check binds the name on a request or approval to a person the
   forge knows, which is what a shared checkout behind a wrapper, or a CI job whose state
   directory is guarded, needs. A team that needs more approves through `toolbox serve`, whose
   sign-in already is the identity (D57), and `--url` refuses the flag.

Rejected: reading `GITLAB_USER_LOGIN` or `GITHUB_ACTOR` (anyone can set an environment variable;
in a job they name whoever started the pipeline, which the forge already records as the forge's
actor); a token per forge (one person, one sign-in: a portal says which forge knows its people,
and mapping people across forges stays with identity resolution); and verifying `--as` against
the portal's own service token (it names the bot, not the person).

Owning docs: [forges.md](../design/forges.md) §4.6, [federated-actions.md](../design/federated-actions.md)
T6, the `run` skill.

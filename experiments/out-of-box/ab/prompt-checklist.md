# Arm A prompt (written checklist)

The same rules, goal, SHA note and reply format as [prompt-toolbox.md](prompt-toolbox.md), with
`<dir>` = `/tmp/claude-0/ab/<repo>-A`, no `toolbox` command, and the standards written out. The
checklist is every rule of the portal's scorecards, with its level and fix text, except the six
run-history rules. Three lines were made self-contained, since the agent couldn't see the
evidence: `links-resolve` says "every relative link must point at a file that exists",
`no-high-findings` names `zizmor --offline .github/workflows`, and `cancels-superseded-runs` says it
applies to workflows triggered by `pull_request`. Codex's prompt adds the `pnpm-workspace` rules,
and its `toolchain-aligned` reads "if mise.toml exists, pin its node and pnpm to concrete versions
matching packageManager; CI's node-version (or node-version-file) must satisfy engines.node".

> ## repo-hygiene (each Repository)
> - [ ] **has-readme** (bronze): Add a README of at least 50 words: what this is, how to run it, who owns it.
> - [ ] **has-codeowners** (bronze): Add .github/CODEOWNERS so changes are routed to an owning team.
> - [ ] **codeowners-catch-all** (silver): Start CODEOWNERS with a `*` rule so every path has an owner.
> - [ ] **owners-are-teams** (silver): Name teams (@org/team), or for a single-maintainer repository only personal handles (@user). Do not mix teams with individuals, and do not use emails.
> - [ ] **has-license** (silver): Add a LICENSE file (or a note in the README if the code is internal-only).
>
> ## docs (each Doc)
> - [ ] **links-resolve** (bronze): Every relative link must point at a file that exists. Fix or remove broken relative links.
>
> ## maintainer-ci-security (each Workflow)
> - [ ] **actionlint-clean** (bronze): Fix what actionlint reports (`actionlint` in the repository root).
> - [ ] **no-high-findings** (bronze): Fix zizmor's high-severity findings (`zizmor --offline .github/workflows`); see https://docs.zizmor.sh/audits/.
> - [ ] **least-privilege-token** (bronze): Declare `permissions:` at the top of the workflow (read-only, e.g. `contents: read`) and grant writes per job.
> - [ ] **pinned-to-sha** (bronze): Pin every third-party `uses:` to a full commit SHA (with the tag in a comment), or adopt a workflow lockfile; let Dependabot update them.
> - [ ] **no-medium-findings** (silver): Fix zizmor's medium findings (e.g. `artipacked`: set `persist-credentials: false` on checkout).
> - [ ] **no-write-tokens-on-risky-triggers** (silver): Don't combine pull_request_target/workflow_run/issue_comment with write permissions; split privileged steps into a separate workflow.
> - [ ] **zizmor-clean** (gold): Resolve or explicitly ignore (with a reason) zizmor's low and informational findings.
>
> ## maintainer-ci-reliability (each Workflow)
> - [ ] **every-job-has-timeout** (bronze): Set `timeout-minutes` on every job; the default is 360 minutes of a hung runner.
> - [ ] **cancels-superseded-runs** (silver): Add `concurrency: {group: ${{ github.workflow }}-${{ github.ref }}, cancel-in-progress: true}` to workflows triggered by pull_request, so a new push cancels the old run.
>
> ## maintainer-action-deps (each ActionDependency)
> - [ ] **pinned-everywhere** (bronze): Pin this action to a full commit SHA in every workflow that uses it.

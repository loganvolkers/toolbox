# Notifications and GitHub Integration for Service Teams

*Status: design. Addresses red-team gap A3 (nobody hears about scorecards). Scope: **GitHub-native
channels, email and webhooks. Slack and Teams are out of scope for now.** The webhook channel lets
customers bridge to anything themselves.
Related: [sdlc.md](./sdlc.md) (Actions for the portal repo), [dashboards.md](./dashboards.md) (reports),
[meta-catalog.md](./meta-catalog.md) (routing data problems to connector owners).*

There are two halves:
1. **The portal reaches teams where they work.** The Toolbox GitHub App posts issues, check runs and
   fix PRs.
2. **Teams' own CI talks to the portal.** Toolbox ships GitHub Actions for service repos.

---

## 1. Principles

1. **Route by ownership, through the graph.** Recipients are resolved from the catalog:
   entity → owner group → the group's channels. Fallback: parent group, then portal owner.
2. **Route to the right owner for the problem.** A stale fact is the *connector owner's* problem, not
   the service team's. A confusing rule is the *rule owner's* problem. The meta-catalog makes this
   possible ([meta-catalog.md](./meta-catalog.md) §6).
3. **Living documents over streams.** Following Renovate's Dependency Dashboard, keep one issue per
   repo, updated in place and closed when green, instead of a new issue per event.
4. **Actionable or silent.** Every notification carries the fix text, a remediation action and an
   `explain` link. If there is nothing to do, nothing is sent.
5. **Budgets and previews.** Each team has a daily notification budget, deduplication keys and digest
   windows. `toolbox plan` estimates notification volume before a change ships, e.g. "this rule would
   open 214 issues", with a gate (§5).
6. **Defined in code, opt-in rollout.** Policies are code. Each channel starts in `shadow` mode (log
   what would be sent), then `pilot` (allowlisted teams), then `on`.

## 2. Events that can notify

| Event | Default recipient | Default channel |
|---|---|---|
| Entity drops a scorecard level / rule newly failing | Entity owner | Repo dashboard issue (living) |
| Initiative assigned / deadline approaching / overdue | Entity owners in scope | Initiative tracking issue + email digest |
| Fix PR available (remediation action) | Entity owner | PR in the repo (Dependabot-style) |
| Action needs approval | Approvers (`ListUsers(run, can_approve)`) | Email + portal inbox; GitHub Environments when the executor is a workflow |
| Action run finished/failed | Requester (human or agent) | Portal inbox / MCP inbox; email if requested |
| Unmatched / ambiguous identity proposal | Candidate entity owners | Repo dashboard issue section |
| Connector stale / failing / permission probe red | **Connector owner** | Issue in the portal repo + email |
| Rule outcomes `unknown` because of stale data | **Connector owner** (not the service team) | Same as above |
| Entity TTL expiring | Entity owner / requester | Email + dashboard issue |
| Ownership gap (owner team dissolved, unverified ownership) | Parent group / portal owner | Portal-repo issue + email digest |

## 3. GitHub channel (via the Toolbox GitHub App)

The App is installed org-wide with least-privilege permissions per feature: issues, checks, pull
requests, contents for fix PRs only, and members (read).

### 3.1 Living "Toolbox Dashboard" issue per repo
- One issue per repo with a deterministic title and a hidden marker. It is created on first actionable
  finding and **edited in place** after that.
- Sections:
  - the scorecard status of the repo's entities
  - failing rules with fix text
  - open initiatives with deadlines
  - identity proposals
  - expiring TTLs
- **Checkboxes trigger actions** (as in Renovate): `- [ ] Open a fix PR for "has runbook"` runs the
  remediation action, and `- [ ] Snooze 30 days` opens a waiver PR.
- Closed automatically when everything is green; reopened on regression.
- Mentions only the owner team (`@acme/payments`) and only on *transitions*, never on every edit.

### 3.2 Check runs on pull requests (shift-left)
- "Toolbox / scorecards" shows the **score delta caused by this PR**: entity fragments and facts are
  computed for the PR head (via the `toolbox/scorecard-check` Action, §4) and evaluated by the portal
  read-only.
- Annotations point at the offending lines in `catalog-info.yaml`, `CODEOWNERS`, OpenAPI files and so
  on.
- **Optional enforcement** in code, e.g. `require: { tier: ["0","1"], minLevel: "bronze" }`. The check
  then fails, and branch protection can require it.

### 3.3 Fix PRs
- Remediation actions can open PRs in the style of Dependabot and Renovate: grouped, rate-limited,
  labelled `toolbox`, with the rule's explanation in the body, and auto-closed when obsolete.

### 3.4 Initiative tracking
- One tracking issue per initiative per owner team, with a GitHub **sub-issue** per affected entity,
  deadline in the title, and progress updated in place.

### 3.5 Approvals through GitHub
- When an action's executor is a GitHub workflow, approval can be delegated to **GitHub Environments
  required reviewers**, so approvals happen where engineers already are. The portal mirrors the state
  into the run.

## 4. GitHub Actions for service repos (customers' SDLC)

These are thin wrappers over the `toolbox` CLI, so GitLab CI and others get the same features. They
authenticate with **GitHub OIDC**, with no long-lived secrets. The portal trusts GitHub's OIDC issuer
and maps the `repository` claim to a *repo principal* that can only act on entities linked to that
repo.

| Action | What it does |
|---|---|
| `toolbox-dev/validate-catalog` | Validates `catalog-info.yaml` against the portal's generated JSON Schemas (fetched once and cached); annotates errors |
| `toolbox-dev/scorecard-check` | Sends the PR head's fragments and facts to the portal for a **read-only what-if evaluation**; posts the check run and a sticky PR comment with the delta |
| `toolbox-dev/push` | Pushes facts, docs, OpenAPI/AsyncAPI specs, SBOMs, test and coverage results and deploy events to the portal via the connector protocol's push transport |
| `toolbox-dev/context` | Fetches the **context pack** for the PR's entity (owners, dependencies, failing rules, standards), writes it to the job summary, and exposes it as an output for AI review agents in the same workflow |
| `toolbox-dev/run-action` | Triggers a portal action (with dry-run) from a workflow, e.g. "register deployment", "request prod access" |
| `toolbox-dev/publish-docs` | Publishes rendered docs and the entity link to the knowledge pipeline on merge |

Example service-repo workflow:

```yaml
name: toolbox
on: [pull_request, push]
permissions: { id-token: write, contents: read, checks: write, pull-requests: write }
jobs:
  toolbox:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
      - uses: toolbox-dev/validate-catalog@v1
        with: { portal: https://portal.acme.dev }
      - uses: toolbox-dev/push@v1
        if: github.event_name == 'push'
        with: { portal: https://portal.acme.dev, facts: coverage.json, openapi: api/openapi.yaml }
      - uses: toolbox-dev/scorecard-check@v1
        if: github.event_name == 'pull_request'
        with: { portal: https://portal.acme.dev }
```

## 5. Notification policy in code

```ts
new tb.NotificationPolicy("default", {
  owner: "group:default/platform",
  mode: "pilot", pilotTeams: ["group:default/payments", "group:default/search"],   // shadow | pilot | on
  channels: {
    github: { app: tb.env("TOOLBOX_GITHUB_APP_ID"), dashboardIssue: true, prChecks: true, fixPRs: { maxOpenPerRepo: 3 } },
    email:  { smtp: tb.env("SMTP_URL"), digest: "weekly" },
    webhook:{ url: tb.env("NOTIFY_WEBHOOK"), format: "cloudevents" },
  },
  budgets: { perTeamPerDay: 5, dedupWindow: "7d" },
  gates:   { maxNewIssuesPerPlan: 50 },   // plan fails unless acknowledged by the policy owner
});

// Groups declare where they want to be reached
// catalog-info.yaml (Group): spec.notify: { github: "@acme/payments", repo: "acme/payments-api", email: "payments@acme.dev" }
```

**Channel contract:** every notification is a typed event (CloudEvents envelope) with a dedup key,
recipients, severity, actions and an `explain` URL. Channels are adapters. This keeps Slack or Teams
a later adapter rather than a redesign.

## 6. Agents

- Every principal, human or agent, has an **inbox**, exposed as the MCP resource `toolbox://inbox` and
  the tool `inbox_list`. An agent working on a repo sees the same actionable items the team sees.
- Agent-requested runs report completion to the agent's inbox, and to the human it acts for if that
  human opted in.

## 7. Phasing

| Phase | Scope |
|---|---|
| 1 | Event model, routing by ownership, inbox, email channel, `shadow` mode, `validate-catalog` + `push` Actions with OIDC |
| 2 | GitHub App: living dashboard issue, PR check runs, `scorecard-check`, notification-volume estimate + gate in `plan` |
| 3 | Fix PRs, initiative tracking issues with sub-issues, approvals via GitHub Environments, `run-action` |
| 4 | `context` Action, digests rendered from report templates, webhook (CloudEvents) channel GA |

## Sources

- [Renovate Dependency Dashboard](https://docs.renovatebot.com/key-concepts/dashboard)
- [Datadog: service ownership best practices](https://datadoghq.com/blog/service-ownership-best-practices-datadog)
- [OpsLevel check ownership](https://docs.opslevel.com/docs/checks-and-filters)

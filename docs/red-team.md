# Red Team: How a Port Sales Engineer Would Attack Toolbox

*Written 2026-11-18 against [plan.md](./plan.md), [authorization.md](./authorization.md) and
[ocean.md](./ocean.md). The point of view is deliberately adversarial. Claims about Port are limited
to what this research verified:*
- *Port's Terraform provider schemas (approvals; automations including `timer_property_expired_event`
  TTLs; `upsert_entity_method`; webhook and Kafka changelog destinations).*
- *Its public repos (`port-sender` scorecard/initiative reminders to Slack, `port-agent`, Ocean's
  centralized rate limiter).*
- *Its public positioning (context lake, AI agents, AI Builder).*

*Everything else is phrased as what a competitor **would** pitch.*

---

## How to read this

Each gap has the same structure:
- **The pitch:** what the Port salesperson says.
- **Reality check:** what our plan says today.
- **Verdict:** 🔴 real gap, 🟠 partly covered but under-designed, 🟢 covered (rebut it).
- **Fix:** the smallest credible fix.

The ranking at the end is by *deal risk*: how likely each gap is to lose an evaluation.

---

## A. Buyer-facing gaps (these lose deals)

### A1. "Who builds and runs this? You've reinvented the Backstage staffing problem." 🔴
**Pitch:** "Port is live in an afternoon: connect GitHub and you have a catalog. Toolbox needs a repo,
an SDK, a build pipeline, Postgres, OIDC, connector hosting and on-call. That's the 3–5 FTE Backstage
story with a Rust logo."

**Reality check:** we claim "agents author the portal", but nothing in the plan *measures* time to
value. There's no quickstart target, no starter templates, and no hosted demo. Upgrades are a version
bump, but *initial* setup is still a platform project.

**Fix:**
- Make **time to first useful portal** a tested KPI: `toolbox init --from github-org acme` →
  deployed, populated portal in **< 30 minutes**, measured in the agent eval suite.
- Ship **starter portals** (opinionated programs for small, mid-size and regulated orgs), a public demo
  portal, and a one-command docker-compose with SQLite.

### A2. "Your managers can't see anything: no dashboards, no reports, no trends." 🔴
**Pitch:** "VPs buy portals to see standards adoption by team, initiative burndown and trends over
quarters. Port has dashboards and reports. Toolbox has entity pages and Grafana dashboards *about the
portal's own health*."

**Reality check:** the plan's non-goals exclude "drag-and-drop dashboard builders", and the UI is
"mostly read". Score history appears only as one word in a table. There are no reports, no trend
storage design, no exports and no leadership views. **The economic buyer is not served.**

**Fix:**
- **Reports-as-code:** chart, table and scorecard-rollup views declared in the portal program over
  catalog and score history. Agents generate them well.
- A designed **time-series store for scores and facts** (daily snapshots per entity, rule and team;
  retention in code).
- **Initiative burndown** views, CSV export, and scheduled digests (see A3).

### A3. "Scorecards nobody hears about don't improve anything." 🔴
**Pitch:** "Port sends scorecard and initiative reminders to Slack (we even open-sourced the GitHub
Action). Engagement drives adoption. How does a team learn its security score went red in Toolbox?"

**Reality check:** automations *could* call a webhook. There is no notification subsystem: no
Slack/Teams/email channels, digests, routing to owners, quiet hours or dedup. There is also no chat
interface for humans.

**Fix:**
- A first-class **notifications** capability, defined in code:
  - Channels: Slack, Teams, email, webhook.
  - Routing by ownership: notify the owning team's channel.
  - Digests, thresholds and dedup.
  - Per-deadline initiative reminders.
- A **Slack/Teams app**: humans ask the portal questions (MCP-backed), approve actions, and receive
  digests.

### A4. "Your humans don't get AI. Only people who already run Claude Code do." 🟠
**Pitch:** "Port has built-in AI agents, an AI assistant and an AI Builder, plus agent governance.
Toolbox is 'bring your own agent': fine for staff engineers, useless for an EM or a security analyst."

**Reality check:** a deliberate non-goal (we don't run agents). But we offer no *assistant* in the UI
or Slack. Agent governance is also thin: there are agent principals and ceilings, but no registry of
which agents exist, what they did, or what they cost.

**Fix:**
- An optional **"Ask" panel** in the UI and Slack. It is a thin client over our own MCP server and the
  customer's model provider: we orchestrate one chat, not an agent platform.
- An **agent registry view** built from audit and analytics data: agents, owners, scopes, activity,
  failure rate.

### A5. "Integrations: 16 planned vs. our 50+ maintained." 🟠
**Pitch:** "Who fixes your GitHub connector when the API changes at 3am? We have a team for it."

**Reality check:** the connector protocol is solid, and the Ocean compatibility runner is speculative.
There is **no maintenance model**: no ownership tiers, no live-sandbox testing, no deprecation policy.
There is also no **central rate-limit budget**: ten connectors and the actions engine can all exhaust
the same GitHub token. Ocean ships a centralized rate limiter for exactly this reason.

**Fix:**
- Connector tiers (*core*, maintained by the project with nightly tests against vendor sandboxes;
  *community*; *verified*).
- An engine-level **shared quota manager** per credential and host.
- The Ocean spike moved earlier, to Phase 1.

### A6. "Enterprise readiness: who signs the SLA, where's SOC 2, who answers the pager?" 🔴 (not a technical gap; still decisive)
**Pitch:** "Port is SOC 2 and ISO certified, has an SLA and a support team. Toolbox has GitHub issues."

**Reality check:** the plan is silent on support, governance, security response, LTS commitments and
who funds the project.

**Fix:**
- Publish a governance and sustainability plan: a foundation path and/or a commercial support entity.
- A security policy with disclosure process and CVE handling.
- LTS lines with support windows.
- A hardening guide and a reference architecture that customers' own SOC 2 controls can map to.

---

## B. Product-design gaps (these hurt in a proof of concept)

### B1. Identity resolution across sources is hand-waved 🔴
The same service is `payments-api` in GitHub, `Payments API` in PagerDuty, `payments` in Datadog and
`svc-payments` in Snyk. The plan says fragments are "keyed by entity identity" but never says **how
identity is decided**. In real deployments this is the hardest, longest-running problem.

**Fix:**
- An **identity resolution** design: declared matchers per connector (exact, normalized, annotation,
  CEL), alias sets, and confidence scores.
- An **unmatched queue** (UI + MCP) where agents propose links as PRs (alias files in code).
- Metrics on match coverage.

### B2. Renames, merges and schema evolution of *data* 🔴
Kinds are code but entities are state. The plan versions the config IR, yet says nothing about:
- renaming a property or kind
- splitting a kind
- an entity rename (all refs break)
- two entities found to be the same
- score history when a rule is renamed

**Fix:**
- `toolbox plan` detects destructive model changes and requires declared migrations (`rename`,
  `split`, `retype`).
- Entity **aliases and redirects** keep old refs resolving.
- Rule ids are stable, with an explicit `renamed_from`.

### B3. Data that has no system of record 🟠
Business criticality, data classification, PCI scope, and vendor SaaS with no repo: in Port, someone
edits a field in the UI. Our answer, "edit opens a PR", needs specifics:
- *which* repo
- bulk edits (re-tier 200 services = 200 PRs?)
- review routing
- latency until the change is visible

**Fix:**
- Define a **catalog data repo** pattern for curated data (not config), with bulk-edit PRs (one PR,
  many entities) and CODEOWNERS-based review routing.
- Show a visible "pending change" state in the UI until merge.

### B4. Things actions create should exist immediately 🔴
An ephemeral environment created by an action should be in the catalog *now*, with a TTL, and be
destroyed on expiry. Port has `upsert_entity_method` and timer-expiry automations. Our actions can only
register entities through PRs or connectors.

**Fix:**
- An **action-sourced fragment** source (state, with provenance and an expiry).
- **TTL/timer properties** with expiry triggers.
- **Entity-scoped "day-2" actions** bound to a kind: the action appears on the entity page with the
  entity pre-filled. The plan never says how actions attach to entities.

### B5. Multi-step workflows 🟠
"Provision DB → create repo → wait for approval → deploy → notify" is a workflow, not one executor.
Delegating everything to GitHub Actions loses portal-level visibility, approvals mid-flow,
compensation on failure, and resumability.

**Fix:**
- Either a small **durable step engine** (steps, waits, approvals, retries, compensation) on the
  existing run state machine, or an explicit stance with deep integration into Temporal or Argo, with
  step-level status mirrored into runs.
- Decide this before Phase 3.

### B6. Third-party knowledge has its own permissions 🔴
The "context lake" pitch requires Confluence, Notion, Google Docs and Slack, each with its *own* ACLs.
Our ReBAC design derives permissions from the *catalog* only. Ingest Confluence without its ACLs and
the context lake becomes a data-leak engine for agents.

**Fix:**
- Connectors can emit **permission relations** (e.g. `doc:x#viewer@group:y`) as derived tuples, with
  a mapping from external principals to Toolbox users.
- Without ACL sync, a source is ingested as **owner-only by default**.
- Add RAG-leak tests to the Level 3 surface suite.

### B7. Live data on entity pages 🟠
Backstage users expect to see deployments, pods, CI runs and on-call *now*, and Port shows these via
integrations. Our `resolve` capability is a one-line stub: no caching, no auth propagation, no UI
contract.

**Fix:** a "live panels" spec covering a `resolve` request/response schema, caching and TTLs, per-user
identity for the upstream call, timeouts and degradation, and view bindings.

### B8. Portal governance at scale (who owns the portal repo?) 🟠
Fifty teams wanting their own scorecards and actions means one portal repo becomes the new
bottleneck. Packages help, but the *process* isn't designed: module ownership, review routing,
per-module CI, and blast-radius limits.

**Fix:**
- A documented multi-team pattern: CODEOWNERS per module; `toolbox plan` scoped per module with
  impact gates.
- A **tenancy story** for business units: one portal with authz partitions, or federated portals with
  cross-portal refs. Decide which.

---

## C. Security and compliance gaps

### C1. `toolbox plan --against prod` leaks production data into CI 🔴
The plan's flagship feature pulls a production snapshot into CI to preview score and authorization
diffs. The PR author, and the CI runner, may not be allowed to see that data (vulnerability lists,
costs, incident details). This **bypasses the ReBAC model entirely.**

**Fix:**
- **Remote plans**, Terraform-Cloud style: CI uploads the manifest, the portal computes the diff
  server-side, and returns output **redacted to the PR author's permissions**. Only aggregates go to
  untrusted forks.
- Never export raw snapshots by default.

### C2. Locking yourself out with a bad authorization change 🔴
A model bug merged to main can remove everyone's access, including the operators who would fix it.

**Fix:**
- An authz diff **threshold gate** in `plan`, like Ocean's deletion threshold: e.g. block if > 20% of
  users lose `can_view`, unless the change is explicitly acknowledged.
- A **break-glass principal** defined at deploy time (not in the model), audited loudly.
- Automatic rollback to the previous image if the operator-access probe fails after deploy.

### C3. Personal data, retention and erasure (GDPR and friends) 🔴
HR and IdP connectors import people. Raw-record stores, search indexes, embeddings, analytics and
audit all copy that data. The plan has retention windows but no **erasure** path, no data inventory,
and no stance on sending documents to third-party embedding or LLM providers.

**Fix:**
- A personal-data inventory per kind and fact.
- `toolbox admin erase <subject>` propagating to raw records, indexes, embeddings and analytics sinks,
  with audit kept but pseudonymized.
- An explicit **AI data-egress policy** in code: which facets may be embedded or sent to which
  provider.

### C4. Connector supply chain 🟠
A community connector container receives a GitHub admin token. The plan pins digests but has no
signature verification, egress allowlisting, or least-privilege token scoping per connector.

**Fix:**
- Sigstore/cosign verification of connector images.
- A declared egress allowlist per connector, enforced when Toolbox runs it.
- Permission probes that *fail* on over-privileged tokens, not only under-privileged ones.

### C5. Prompt injection from catalog data 🟠
PR titles, README text and incident notes flow into context packs. "Wrapping as untrusted" is
necessary, not sufficient.

**Fix:**
- Provenance-tagged sections in context packs.
- An optional instruction-stripping filter.
- No tool-invocation hints sourced from data.
- Red-team evals in the agent eval suite.

---

## D. "-ilities" nobody has written down yet

| Area | Status | Gap |
|---|---|---|
| Accessibility (WCAG 2.2 AA) | 🔴 | Not mentioned; a procurement blocker in the public sector and large enterprises |
| Internationalization | 🔴 | Not mentioned (UI strings, dates, docs) |
| UI design quality | 🟠 | "Generated pages" invites the "Backstage looks like an admin panel" objection. Needs a design system and a UX owner |
| SCIM / identity lifecycle | 🔴 | OIDC only. No SCIM provisioning, no deprovisioning flow, no **orphaned-ownership** workflow when a team dissolves |
| Search relevance | 🟠 | Hybrid search is designed; relevance *evaluation* (golden queries, zero-result tracking → tuning) is not |
| Reliability of ingest | 🟠 | Webhook dead-lettering exists; end-to-end delivery guarantees and backfill-time budgets for 10k-repo orgs are undefined |
| Cost / TCO | 🔴 | No published TCO model vs. SaaS (infrastructure + people), which every buyer asks for |
| Upgrade testing | 🟠 | SemVer + codemods, but no upgrade-compatibility test matrix (N-2 → N) in CI |
| Measuring portal ROI | 🟠 | Usage analytics exist; there is no adoption/ROI report for the buyer (overlaps A2) |

---

## E. Ranked by deal risk

| Rank | Gap | Why it loses deals | Suggested phase |
|---|---|---|---|
| 1 | A2 Dashboards, reports, trends | The economic buyer sees nothing for them | 2 (with scorecards) |
| 2 | A1 Time to value | "Backstage again" ends the evaluation early | 0–1 (KPI + starter portals) |
| 3 | B1 Identity resolution | The PoC catalog looks wrong on day one | 1 |
| 4 | A3 Notifications + Slack | Scorecards without nudges don't move | 2 |
| 5 | C1 Remote plan / data leak | Security review blocker | 2 (before `plan --against` ships) |
| 6 | B4 Action-created entities, TTL, day-2 actions | Self-service demos fall flat | 3 |
| 7 | A6 Support, governance, compliance story | Procurement blocker | before 1.0 |
| 8 | B6 Third-party knowledge ACLs | The context-lake pitch is unsafe without it | 4 |
| 9 | C3 Erasure + AI data egress | Legal/DPO blocker in the EU | 4 |
| 10 | B2 Renames/migrations | Shows up in month 2, erodes trust | 1–2 |

**Rebuttals we can already make** (🟢):
- Config-as-code drift: Port's IaC is a client of its database; ours can't drift by construction.
- No vendor lock-in or data residency issues when self-hosted.
- Testable authorization (`fga model test`) and testable scorecards (`plan`) that no SaaS offers.
- Agent-first surface designed around tasks, not CRUD.
- Ownership-derived permissions with no sync lag.

## F. Recommended next step

Fold these into the plan as follows:
- **New design sections:** A2, A3, B1, B4 and C1. These are product-defining.
- **Constraints on existing sections:** B2, B3, B5, B6, C2, C3 and C4.
- **A tracked "-ilities" checklist** gating 1.0: section D plus A6.

# Operator SDLC (Layer 3): Local → CI → PR → Ephemeral → Production

> **Layer 3 of [the four SDLC layers](../sdlc.md) (D63, D87):** how operators build and change a portal.
> Building Seikika itself is the [maintainer SDLC](./maintainer-sdlc.md); building apps on a portal
> is the [user SDLC](./user-sdlc.md). Section numbers are stable because other documents cite them.
>
> **Implementation status (M0–M4 done):**
>
> | Command / capability | State |
> |---|---|
> | `seikika validate`, `seikika test` (expectations + `.fga.yaml`), `plan --local --base`, `derive` | ✅ |
> | `seikika serve --fixtures` / `--database-url`, `push`, `token create`, `connector test` | ✅ |
> | `seikika dev` (watch mode) | Not yet: rerun `test` / restart `serve` |
> | `plan --remote` (§4), preview portals (§5), gates (§6), `build`, deploy, `verify`, rollback (§7) | M6 |
> | GitHub Actions for the portal repo (§8) | M6 |

> **Status 2026-11-20 (D34, G2):** the operator SDLC (plan, build, test, deploy, monitor) is now a **charter goal** and the main counterpoint to Port and Backstage. The default deploy target is one image on Cloud Run or equivalent (D33); `plan` is two full recomputes over one snapshot and a diff (D37).

*Status: design. Addresses red-team gaps C1 (`plan --against prod` leaks data) and C2 (authz
lock-out), and the request to ship GitHub Actions so customers can plan, build and test their portal
locally, in CI, on PRs and in ephemeral staging environments.
Related: [plan.md](../plan.md) §2 and §6, [authorization.md](../authorization.md),
[dashboards.md](./dashboards.md), [notifications.md](./notifications.md).*

## 1. Prior art

| Tool | Pattern we borrow | Pitfall we avoid |
|---|---|---|
| **Terraform Cloud** speculative plans | A plan runs **server-side** on every PR; the PR gets a status and a *link* to the run. Fork PRs can't trigger plans by default | The status reflects the plan only, not the later apply |
| **Atlantis** | Plan output as a PR comment; apply from the PR | Plan output in comments can leak values |
| **Pulumi** (`pulumi/actions`) | `comment-on-pr` + `edit-pr-comment` (one sticky comment); **Review Stacks**: an ephemeral environment per PR, destroyed on close | Self-managed per-PR stacks leave orphans when cleanup fails |
| **Tilt** | Fast local inner loop with live reload | A Kubernetes-shaped loop for many services. The reference local deploy is one container ([local-simulation.md](./local-simulation.md)) |

## 2. Environments

| Env | Purpose | Data | Identity |
|---|---|---|---|
| **local** (`seikika dev`) | Author and iterate | Fixtures and/or a synthetic org (in-memory store) | Dev user, or the developer's real IdP login |
| **CI** | Test everything without a cluster | Fixtures + synthetic | Fixture principals |
| **PR preview** (ephemeral) | Humans and agents click around the change | Fixtures / synthetic / **redacted prod sample** (§5) | Real IdP login; ReBAC applies |
| **staging** (long-lived, optional) | Integration with real connectors in read-only mode | Real sources, read-only credentials | Real IdP |
| **production** | — | Real | Real |

**One artifact flows through all of them.** `seikika build` produces an image (digest-pinned, signed,
with SBOM and provenance), and promotion is by digest.

## 3. Local

The inner loop is designed for speed; see [developer-velocity.md](./developer-velocity.md).

```bash
seikika dev                      # watch → TS builder (if any) → validate → full recompute over fixtures → tests; API/MCP/UI at :7000
seikika check                    # changed definitions only: manifest, CEL types, authz model (≤ 300 ms target)
seikika test --affected          # tests whose inputs changed, in parallel, with a content-addressed result cache
seikika test --all               # everything, no cache (what CI runs on merge)
seikika plan --local --base ./seikika.base.yaml  # compare with the base manifest over local fixtures
seikika plan --remote prod       # remote plan (§4) using *your* identity; details redacted to your permissions
seikika preview up               # docker compose: the built image + Postgres + fixtures
```

For a local plan, use a copy of the base revision's manifest and its referenced files.
For a standalone manifest, `git show main:seikika.yaml > seikika.base.yaml` from the portal root
creates the base file shown above. The plan prints a diff. Score drops in that diff do
not change the command's exit status.

That block is the authoring loop: fixtures in, a recompute, a local API. The deploy loop, which
runs the image against a GitHub API and a SCIM feed instead of `--fixtures`, is
[local-simulation.md](./local-simulation.md) (Q19). `seikika dev` stays the inner loop. Tilt is an
optional adapter for customers who already use it, not the simulator.

## 4. Remote plan: Git is the access boundary

**Never pull production data into CI.** CI uploads the **manifest bundle** (config only: no data and
no secrets) to the production portal. The portal computes the plan **server-side**:
- It recomputes derived state under the candidate manifest over the same snapshot as the current one (two full recomputes and a diff, D37), using the same engine
  ([incremental-computation.md](./incremental-computation.md)).
- It evaluates against current state.
- Plan jobs run as durable workflows with resource limits.

**Plan sections:**

| Section | Example |
|---|---|
| Model changes + required migrations | `Service.spec.tier` retyped → migration `retype` required |
| Mapping changes | 1,204 PagerDuty records re-map; 3 fields change on 87 entities |
| Identity coverage | +212 linked, 3 new ambiguities |
| Scorecards | 214 services drop silver→bronze; rule `has-runbook` new |
| **Authorization diff** | +12 users gain `can_view_score_details` on 3 services; 1,240 lose `can_run_actions` on `system:checkout` |
| **Declassifications** | New: `security` rule outcomes declassified to summary |
| Unverified claims | 2 `spec.owner` claims not backed by CODEOWNERS |
| Notifications | Estimated 41 new dashboard-issue updates, 0 new fix PRs |
| Agent surface | MCP tool `scorecard_status` gains param `history`; no removals |
| Views | `security-overview` reads `facts.snyk.critical` (renamed: **broken**) |
| Meta catalog | 2 new kinds without owners (**blocks**: ownership required) |

**Who sees what: the portal repo is the boundary (the Terraform model).**
- Plan output is posted in full to the PR, as a sticky comment and check run, like `terraform plan`.
- Anyone who can read the portal repo can read plan output. **The portal repo is therefore a
  privileged repository**, with access managed in Git like an infrastructure repo. `seikika init`
  documents this, and `seikika verify` warns when the repo is public.
- **Fork PRs never trigger plans.**
- *Considered and rejected:* per-viewer redaction, aggregate-only output and k-anonymity floors for
  plans. They are complex, and they duplicate controls Git already provides.

**Authentication:** CI uses GitHub OIDC. The portal accepts plan submissions only from the configured
portal repository's workflows.

## 5. Ephemeral PR environments (preview portals)

- `seikika/preview` creates the environment on PR open or update and destroys it on close. It
  registers a **GitHub Deployment** (environment `pr-<n>` with a URL), so the link shows on the PR.
- Adapters:
  - **Kubernetes:** a namespace per PR, via the Helm chart, with TTL labels.
  - **docker-compose on the runner:** for e2e tests without a cluster.
  - **Fly / Cloud Run:** later.
- **Orphan protection:** previews carry an expiry. A scheduled `seikika preview gc` workflow deletes
  any preview whose PR is closed or whose TTL passed, avoiding Pulumi's documented orphan problem.
- **Data options**, chosen per portal in code:
  1. Fixtures, the default.
  2. Synthetic organisation via `seikika-synth`.
  3. **Redacted production sample:** production exports a sample containing only kinds and facets marked
     `preview: allowed` in code. PII is pseudonymised and secrets are never included. The export runs
     server-side and goes directly to the preview's database. Previews still enforce ReBAC using the
     sample's derived tuples, so viewers see only what they could see in production.
- **Connectors in previews are off by default.** They can be enabled read-only with dedicated
  read-only credentials, never production write credentials.

## 6. Gates (policy as code)

```ts
new seikika.PlanGates("default", {
  owner: "group:default/platform",
  gates: {
    deletionRatio:       { max: 0.10 },             // entities deleted by mapping changes
    authzLoss:           { maxUsersLosingView: 0.05, maxUsersLosingAdmin: 0 },
    operatorAccess:      { mustRetain: ["group:default/platform"] },   // C2: can't lock out operators
    scoreDrop:           { maxEntitiesDroppingLevel: 0.20 },
    notificationVolume:  { maxNewIssues: 50 },
    agentSurface:        { allowToolRemoval: false },
    ownership:           { requireOwnerOnNewMetaEntities: true },
  },
  acknowledge: "label:seikika-ack/<gate>",   // must be applied by an owner of the affected module
});
```

**Ownership-aware review:**
- The plan computes which **meta-catalog entities** a change touches (kinds, scorecards, authz
  modules, connectors…) and their owners ([meta-catalog.md](./meta-catalog.md)).
- The plan check stays failing until each affected owner team has approved. This catches cross-module
  impact that path-based CODEOWNERS misses, such as a security-module change that alters a kind owned
  by the platform team.

## 7. Build, deploy, verify, roll back

- `seikika build`:
  - Produces one OCI image with the canonical `manifest.json`, UI bundles, in-image executors and connectors, and the generated OpenAPI (D33, D53).
  - Adds an SBOM, a **build provenance attestation** (`actions/attest-build-provenance`) and a
    cosign/sigstore signature.
- **Deploy** uses the customer's tooling (Helm, Argo CD, Terraform, ECS…). Seikika provides the chart
  and modules rather than a deploy service.
- `seikika verify --against <url>` runs post-deploy checks:
  - an operator-access probe (C2)
  - MCP tool list = the plan's expected list
  - connectors healthy
  - migrations complete
  - view queries valid
  A failure exits non-zero, and the workflow **rolls back to the previous digest**.
- **Break-glass** (C2): a deploy-time break-glass principal (environment-bound, not in the model) has
  read access and admin-ops rights for recovery. Every use is audited and triggers a notification to
  the portal owners.

## 8. GitHub Actions for the portal repo

All logic lives in the `seikika` CLI, and the Actions are thin wrappers. A **reusable workflow** wires
them together, so adoption is one file.

| Action | Inputs | Outputs |
|---|---|---|
| `seikika/setup-seikika` | version (default from `seikika.lock`) | CLI on PATH; caches language hosts and connector images |
| `seikika/test` | test globs, shards | JUnit + job summary; view snapshot artifacts |
| `seikika/plan` | portal URL, gates | Sticky PR comment (aggregates), check run with link to detailed plan, gate status |
| `seikika/build` | registry, tags | Image digest, SBOM, provenance attestation, signature |
| `seikika/preview` | adapter (`k8s`/`compose`), data mode | GitHub Deployment + URL; teardown on close |
| `seikika/verify` | URL, expected plan | Pass/fail; triggers rollback in the calling workflow |

```yaml
# .github/workflows/portal.yml in the customer's portal repo
name: portal
on:
  pull_request:
  push: { branches: [main] }
permissions: { id-token: write, contents: read, pull-requests: write, checks: write, deployments: write, packages: write, attestations: write }
jobs:
  portal:
    uses: seikika/workflows/.github/workflows/portal.yml@v1
    with:
      portal-url: https://portal.acme.dev
      preview: k8s                 # or: compose | none
      preview-data: synthetic      # fixtures | synthetic | redacted-sample
      registry: ghcr.io/acme/portal
```

What the reusable workflow does:

| Trigger | Steps |
|---|---|
| **PR** | test → remote plan (gates) → build (unpublished tag) → preview up → surface/e2e tests against the preview (Playwright, MCP smoke) |
| **PR closed** | preview down |
| **main** | test → build → attest + sign → deploy hook (customer's) → verify → rollback on failure |
| **schedule** | `preview gc`, `seikika upgrade --check` (opens a PR for new Seikika versions with codemods applied) |

## 9. Agents in the loop

- The authoring MCP server (`seikika mcp --dev`) exposes `test`, `plan` (remote, as the developer),
  `preview_url` and `explain_plan`. An agent can therefore iterate on a portal change exactly like a
  human: edit, test, plan, open PR, read the plan check, fix.
- Plan output is machine-readable (`--format json`), with stable section ids for agents.

## 10. Phasing

| Phase | Scope |
|---|---|
| 1 | `seikika dev/test/build`; `setup-seikika`, `test`, `build` Actions; signing + provenance; `verify` + rollback |
| 2 | Remote plan as a durable workflow (portal repo is the access boundary); `plan` Action with sticky comment and gates (deletion, authz, operator access, notifications); detailed plan pages |
| 3 | Preview environments (compose, k8s), GitHub Deployments, GC; fixture/synthetic data modes; ownership-aware review |
| 4 | Redacted production sample mode; `upgrade --check` PRs; agent authoring tools for plan and preview |

## Sources

- [Terraform Cloud speculative plans on PRs (UI/VCS runs)](https://developer.hashicorp.com/terraform/enterprise/v202402-1/run/ui)
- [pulumi/actions README](https://github.com/pulumi/actions/blob/main/README.md) ·
  [Pulumi Review Stacks](https://dev.to/pulumi/review-stacks-collaborate-in-the-cloud-1nbe) ·
  [PR previews with Pulumi components](https://www.pulumi.com/blog/building-kubernetes-pr-previews-with-shared-pulumi-components)

# Open questions

Questions that are not decided yet, or that a decision has since closed. A question keeps its `Q` number. Decisions are one file each in [decisions/](./decisions/README.md).

## Open questions (consolidated)

| # | Question | Recommendation | Where |
|---|---|---|---|
| Q1 | Licence | **Resolved by D88: MIT** | [plan.md](./plan.md) §10 |
| Q2 | TCP governance: separate neutral spec repo now, or later? | **Resolved by D42: not now** | [plan.md](./plan.md) §10 |
| Q3 | First-release connectors: GitHub only, or GitHub + GitLab? | GitHub first | [plan.md](./plan.md) §10 |
| Q4 | Durable runtime behind the `Durable` contract (D39): adopt the DBOS Rust SDK, or own a schema-compatible minimal runtime? | v1 implements the DBOS-compatible minimal schema ([experiments/durable-schema](../experiments/durable-schema/README.md)); decide on the SDK after the Phase 0 spike | [design/incremental-computation.md](./design/incremental-computation.md) |
| Q5 | Temporary/just-in-time grants as approved, time-bound state? | Open | [authorization.md](./authorization.md) §8 |
| Q6 | Redaction markers on or off by default | On (helps agents ask for access) | [authorization.md](./authorization.md) §8 |
| Q7 | Agent ceilings in the authorization model, or a simpler allow-list? | Open | [authorization.md](./authorization.md) §8 |
| Q8 | GitHub spike: ~~the exact endpoint path~~ (**resolved 2026-11-20:** `PUT /user/installations/{installation_id}/repositories/{repository_id}`, usable by GitHub Apps, per the REST OpenAPI description); still open: which token types the 2026-03-10 behaviour accepts (the description doesn't yet mention `installation_repositories` or `offline_access`), and whether the owner-only install restriction applies to Apps | Verify in the sandbox spike before M5 | [examples/approved-github-apps.md](./examples/approved-github-apps.md) §2.6 |
| Q9 | How much DBSP state survives a circuit rebuild (sub-circuit reuse)? | **Deferred with DBSP (D37)** | [design/developer-velocity.md](./design/developer-velocity.md) §5 |
| Q10 | cel-rust conformance level (688 of ~2,771 conformance tests skipped; no type checker) | Measure in Phase 0; validate rule outputs against JSON Schemas on fixtures | [design/extensibility.md](./design/extensibility.md) |
| Q11 | How OpenFGA is packaged inside the one container (D38): **a supervised co-process** (the unmodified upstream binary, talking over localhost, reading the tuple view) or **linked in-process** via the c-archive (78 MB, 45 s build, Go runtime inside the Rust process) | **Co-process**: one container either way, no FFI, upstream unmodified. Phase 0 measures cold start, memory and crash recovery on Cloud Run for both | [authorization.md](./authorization.md) §4.3 |
| Q12 | Isolation for in-image executors (D40): separate Unix user, scrubbed environment, per-action secrets, resource limits; is that enough, or is a sandbox (gVisor, WASM) needed? | Unix user + scoped secrets for v1; revisit in Phase 3 | [design/action-entities.md](./design/action-entities.md) |
| Q13 | Cloud Run fit (D33): background recompute and workflows need instance-based billing and ≥ 1 minimum instance; autoscaling can stop an instance mid-workflow, so another instance must recover it from Postgres. Split worker role on Cloud Run worker pools (GA 2026, secondary source) or keep one service? | One service with min instances ≥ 1 by default; verify recovery in the Phase 0 deployment spike | [plan.md](./plan.md) §6.2 |
| Q14 | How can Toolbox observe which repositories another app's installation covers? Installation webhooks go only to the installed app; `GET /orgs/{org}/installations` (org owner) lacks repository lists; `GET /user/installations/{id}/repositories` covers what one user can see | Verify in the sandbox spike: candidate credentials are an org-owner user token (with `offline_access`) for the connector and the acting user's token for `verify`; otherwise runs end `unverified` | [examples/approved-github-apps.md](./examples/approved-github-apps.md) §2.6 |
| Q15 | Where do federation's shared standard packages live? | **Resolved by D89:** git-first with an OCI mirror on the same digest | [design/packages.md](./design/packages.md) §4 |
| Q16 | How do Toolbox, connectors and upstream tools reach consumers without rebuilding from source? | **Resolved by D90:** `dist` releases, executable and WASM connectors, declared tools via mise/aqua | [design/package-author-sdlc.md](./design/package-author-sdlc.md) §3 |

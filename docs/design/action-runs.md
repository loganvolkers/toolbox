# Action Runs: Preview, Confirm, and "Done Means Observed"

*Status: design, 2026-11-20 (decision D49). Builds on [plan.md](../plan.md) §4.3,
[frontend.md](./frontend.md) (action UIs), [action-entities.md](./action-entities.md) and the
[approved GitHub Apps example](../examples/approved-github-apps.md). Uses
[connector-control.md](./connector-control.md) for verification.*

> **Implementation status (D01M86BY3YQX3SA1W99QXF1QA1H):** built, with federation, as
> [federated-actions.md](./federated-actions.md) describes: run states, confirm tokens, the
> `command`, `change`, `github-workflow`, `in-image` and federated executors, approvals and
> postcondition verification, in `toolbox run` and `toolbox serve`. The independent review that
> D64 asked for is [self-service-review.md](../self-service-review.md); its fixes are folded in
> here and in v1-spec §6.6 and §7.5. Built differently from this design: confirm tokens also bind
> the requester and entity and are single-use in a server; approvals are refused over MCP and from
> personal access tokens (D57); `timed_out` is emitted, `compensated` is not yet; a `change` can land on its base once approved
> (`land: push`) and is verified by the base. Runs in `serve` are written through to Postgres or
> the state directory and survive a restart; durable steps across instances are not built.

---

## 1. Conclusion

**An action run succeeds only when Toolbox has observed its effect.** Handing a message to an executor,
a queue or a workflow is *dispatch*, not completion. The maintainer's Port experience was the
opposite: a Kafka message was posted and the run counted as complete, even though the work hadn't
succeeded and might still fail.

Every run follows one lifecycle, the same in API, MCP, CLI and UI:

```
draft → previewed → [awaiting_approval] → confirmed → dispatched → running
      → executor_succeeded → verifying → succeeded
                                       ↘ failed | timed_out | unverified | cancelled | compensated
```

Three mechanisms make this work:

1. **Preview, then confirm, bound together.** Users and agents see exactly what will happen, and the
   confirmation is valid only for that preview (§3).
2. **A real executor contract:** `preview`, `run` with progress, `cancel` and an optional `verify`
   (§5), so executors report outcomes instead of accepting messages.
3. **Verification against the catalog:** each action declares postconditions in CEL. After the
   executor reports success, Toolbox asks the relevant connector for a scoped refresh, recomputes, and
   checks the postcondition (§4).

## 2. Run states

| State | Meaning | Shown as |
|---|---|---|
| `previewed` | Inputs validated, policy and approvals evaluated, effects listed | The preview panel |
| `awaiting_approval` | Durable wait for approvers | Who must approve, and why |
| `confirmed` | The requester confirmed this exact preview | — |
| `dispatched` | The executor accepted the run (it has an id) | "Started" |
| `running` | Progress events, steps and logs arriving | A live timeline |
| `executor_succeeded` | The executor says it finished | "Checking the result…" |
| `verifying` | Scoped refresh + postcondition evaluation | "Checking the result…" |
| **`succeeded`** | **Postconditions hold at a snapshot after the run** | Success, with the snapshot "as of" time |
| `failed` | The executor failed, or a postcondition is false | The error or the unmet postcondition, with links |
| `timed_out` | No terminal executor status or verification within the deadline | Explicit, never shown as success |
| `unverified` | The executor reported success but Toolbox could not observe the effect (no postcondition, no refresh capability) | A distinct warning state; never rendered as plain success |
| `cancelled` / `compensated` | Stopped by a user, or undone by a declared compensation | — |

Runs are durable workflows (D39): every transition is checkpointed in Postgres, with idempotency keys,
deadlines and recovery after restarts.

## 3. Preview and confirm

- **Two preview layers:**
  - **Toolbox preview (always, fast):** schema validation, `enumFrom` eligibility, policy, approval
    path, blast radius from the catalog. Target p95 ≤ 150 ms ([frontend.md](./frontend.md) §6).
  - **Executor preview (when supported):** the executor computes the concrete effects with read-only
    calls, for example "Renovate will get `contents:write` on 1 repository". It streams into the panel
    and is cached by `(inputs hash, snapshot id)`.
- **Structured effects:** a preview is a list of intended changes (resource, operation, before → after,
  permissions granted), plus blockers and warnings. UI, MCP and CLI render the same structure.
- **Confirmation token:** `hash(action version, inputs, preview effects, snapshot id)`, short-lived. Run
  creation requires it. If the catalog or the executor preview changed meanwhile, Toolbox returns
  `preview_stale` with the new preview instead of running.
- **Agents:** the MCP tool returns the preview and a token on the first call. The client shows it to the
  user (MCP elicitation where the client supports it) and calls again with the token. Destructive
  actions always need this two-step flow.

## 4. Verification ("done means observed")

Actions declare what must be true afterwards:

```ts
tb.action("install-approved-app", {
  scope: "day2", on: "repository",
  inputs: { app: tb.schema.enumFrom("approvedApps(entity)") },
  executor: tb.executor.inImage("./executors/github-apps"),
  expect: {
    // CEL over the catalog after the run
    postcondition: "entity.installedApps.exists(a, a.slug == inputs.app)",
    refresh: [{ connector: "github", kind: "installation", ids: "[entity.externalId('github')]" }],
    within: "2m",
  },
});
```

After `executor_succeeded`:

1. Toolbox issues the declared `refresh` commands with `verification` priority
   ([connector-control.md](./connector-control.md) §5).
2. It waits for `complete`, then for the snapshot that contains the results.
3. It evaluates the postcondition at that snapshot: true → `succeeded`; false → `failed` with the
   observed values; refresh unavailable → the executor's `verify` hook; nothing available →
   `unverified`.

Verification also fixes the stale-page problem: when the run ends, the catalog already shows the
effect, because the run waited for the snapshot that contains it.

`toolbox plan` flags actions with no postcondition, and the meta catalog's action scorecard penalises
them, so `unverified` is a visible, deliberate choice.

## 5. Executor contract and fidelity tiers

| Operation | Purpose |
|---|---|
| `preview(inputs, entity) → effects` | Read-only; optional but strongly encouraged |
| `run(inputs, idempotency_key) → run_id` | Starts work and returns an executor run id |
| progress events | Steps, logs, percentage; via stdio, callback or polling |
| terminal status | `succeeded | failed` with outputs or error |
| `cancel(run_id)` | Best effort |
| `verify(inputs, outputs) → bool` | Optional read-back when no connector refresh fits |

Executors differ in how much of this they can do, and the UI shows the difference honestly:

| Executor | Preview | Progress / status | Cancel | Verification | UX tier |
|---|---|---|---|---|---|
| **In-image executor** (child process over stdio, D40) | Native | Native, streamed | Yes | Refresh + postcondition, or `verify` | **Best** (default for rich actions) |
| **HTTP executor** (same contract over HTTP) | Native | Callbacks + polling | Yes | Same | Best |
| **GitHub Actions** (`workflow_dispatch`) | Toolbox preview only, or a separate dry-run workflow (slow) | Run id from dispatch (`return_run_details`, GitHub changelog 2026-02-19 [S]); status via the Runs API and `workflow_run` webhooks | Yes (cancel run API) | Refresh + postcondition | Good; previews are coarser |
| **Webhook / queue (fire-and-forget)** | Toolbox preview only | None until a callback | No | **Required:** refresh + postcondition, or the action must declare `unverified: true` | Acceptable only with verification |

A webhook or queue publish is never treated as completion: without a postcondition, its runs end as
`unverified`.

## 6. Interfaces

- **API:** `POST /v1/actions/{name}/previews` → preview + token; `POST /v1/actions/{name}/runs` with the
  token; `GET /v1/runs/{id}` with SSE events.
- **MCP:** `preview_action` / `run_action` (or a per-action tool with `confirm_token`); long runs are MCP
  tasks; the final result includes the verification outcome and snapshot id.
- **CLI:** `toolbox run <action> --preview`, then `--confirm <token>`; `toolbox runs watch <id>`.
- **UI:** the live preview panel and run timeline from [frontend.md](./frontend.md), with
  "checking the result" as a visible step and the final page rendered at the verified snapshot.
- **Notifications** fire on terminal states only ([notifications.md](./notifications.md)).

## 7. Testing (G2)

- **Preview snapshot tests:** the structured effects for fixture inputs.
- **Postcondition tests:** for each action, a simulated executor plus a simulated connector refresh;
  assert `succeeded`.
- **Failure injection:** the executor reports success but the effect is missing → `failed`; the
  connector times out → `timed_out`; no refresh capability → `unverified`. A run must never reach
  `succeeded` without the postcondition holding (to be added as a named property in
  [verification.md](./verification.md)).
- **Stale preview:** state changes between preview and confirm → `preview_stale`.

## Sources

- GitHub: [Workflow dispatch API now returns run IDs](https://github.blog/changelog/2026-02-19-workflow-dispatch-api-now-returns-run-ids) (2026-02-19; github.blog is blocked here, so this is from search excerpts; verify the response schema before relying on it).
- [plan.md](../plan.md) §4.3 (the earlier run states and dry-run requirement, which this replaces),
  [frontend.md](./frontend.md), [ocean.md](../ocean.md) (Ocean's async completion via webhooks).

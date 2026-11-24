# Connector Control: Telling Connectors What to Refresh

*Status: design, 2026-11-20 (decision D48). Amends D42: ingestion stays an OpenAPI endpoint and is
still not a governed standard, but Toolbox now also owns a **control channel** to connectors. Works
with [action-runs.md](./action-runs.md), which uses scoped refreshes to verify action outcomes.*

> **Implementation status (M4 done):** the hub (queues, coalescing, priorities, deadlines,
> `ack`/`complete`), the stdio supervisor for in-image connectors, long-poll for remote ones, initial
> resync on register, the freshness loop, resync reconciliation with a deletion threshold, and
> entity refresh routed by the records that produced the entity are built, with the GitHub, git-files
> and SCIM connectors and a conformance kit (`toolbox connector test`). **Not yet:** commands are held
> in memory and lost on restart (durable with M5); the simulated connector for `toolbox dev`. Details:
> [v1-spec.md](../v1-spec.md) §9 (M4 notes).

---

## 1. Conclusion

Toolbox needs to **ask** connectors for data, not only receive it. The connector API therefore has two
halves:

| Half | Direction | Purpose |
|---|---|---|
| **Ingest** (unchanged, D42) | Connector → Toolbox | `POST /v1/ingest`: raw records, fragments, relations, deletes, snapshot markers |
| **Control** (new) | Toolbox → connector | Registration and capabilities, **commands** (scoped refresh, resync, cancel), acknowledgements and completion |

The point of control is **closing loops**: a user clicks "refresh" on an entity page, an agent needs
current data before answering, or an action has to prove its effect happened (§5). Each of those needs a
refresh of *specific* records, and a way to know **when the fresh data is visible**.

## 2. Requirements

1. **Scoped refresh:** by external ids, by kind plus selector, or "changed since" a cursor, not only a
   full resync.
2. **Completion you can wait on:** a command ends in a terminal status, and Toolbox can tell the
   requester which snapshot contains the result (read-your-writes).
3. **Works in the single-container deployment (G1):** in-image connectors need no network setup, and
   remote connectors need no inbound ports.
4. **Durable and bounded:** commands survive restarts, coalesce, respect connector concurrency and rate
   limits, and time out visibly.
5. **Safe:** a scoped refresh never causes implicit deletes; only explicit `DELETE` or `not_found`
   results remove data.
6. **Replaceable (G3):** the control messages are transport-independent, so a connector can move from
   in-image to remote without changing code.

## 3. Transports

| Connector runs… | Commands arrive by | Why |
|---|---|---|
| **In the Toolbox image** (child process, D40) | NDJSON on **stdin**; results on stdout | No network, no credentials; same model as MCP stdio servers |
| **Remotely, long-running** (default) | **Long-poll / SSE pull**: `GET /v1/connectors/{id}/commands?wait=30s` | No inbound ports or public endpoint on the connector; works behind NAT, on Cloud Run, Kubernetes or a laptop. The connector authenticates to Toolbox with a per-connector token, so credentials flow one way |
| **Remotely, serving HTTP** (optional) | Toolbox POSTs to the connector's `/control` endpoint | For connectors that already expose an endpoint and want push latency |
| **On a schedule or in CI** (`once` mode) | On start, the connector drains its pending commands, runs them, reports completion, and exits | Cron, GitHub Actions or Cloud Run jobs; Toolbox can trigger such a run (e.g. `workflow_dispatch`) when a command is waiting |

An in-image connector does not inherit the server's environment. It gets the process basics
(`PATH`, `HOME`, locale, proxies, CA bundles, `RUST_LOG`, `GITHUB_API_URL`) and
`TOOLBOX_CONNECTOR_CONFIG`, never `DATABASE_URL`, PATs or `TOOLBOX_SECRET_*`. Its own credentials
reach it through its config (`token: ${env:GITHUB_TOKEN}`, `${secret:…}`).

It still runs as the server's user, and a same-user process can read `/proc/<pid>/environ` of any
dumpable process. So on Linux `toolbox serve` makes itself non-dumpable at start-up, and the
OpenFGA co-process gets the same kind of allowlist plus its datastore URI in a `0600`
`config.yaml` that is deleted once OpenFGA is up, not in its argv (`/proc/<pid>/cmdline` is
readable by every local user). The same file gives OpenFGA a preshared key minted at start-up,
so a local process that finds its loopback port cannot read or rewrite the authorization model.
What remains is OpenFGA's memory: with
`kernel.yama.ptrace_scope=0` a same-user process may attach to it. A connector that must not
reach the database at all runs remotely, holding only its own token.

## 4. Messages

All messages are JSON, versioned with the connector API (`v1`), and identical across transports.

**Registration and heartbeat** (connector → Toolbox):

```json
{ "type": "register", "connector": "github", "version": "1.4.0", "instance": "github-7f3c",
  "capabilities": {
    "refresh": { "kinds": ["repository", "installation", "team"],
                 "scopes": ["external_ids", "selector", "since"] },
    "resync": { "kinds": ["repository", "installation", "team"] },
    "max_concurrency": 4,
    "rate_limit": { "remaining": 4800, "resets_at": "2026-11-20T10:00:00Z" } } }
```

Capabilities are recorded in the meta catalog ([meta-catalog.md](./meta-catalog.md)), so `toolbox plan`
can warn when an action's verification needs a refresh scope that its connector doesn't offer. An
[Ocean compatibility runner](../ocean.md) declares `resync` per kind only, because Ocean integrations
expose `on_resync(kind)` but no scoped refresh.

**Commands** (Toolbox → connector):

| Command | Fields | Meaning |
|---|---|---|
| `refresh` | `command_id`, `kind`, one of `external_ids[]` / `selector` / `since`, `reason`, `priority`, `deadline` | Re-read exactly this scope and ingest it |
| `resync` | `command_id`, `kinds[]` | Full read with snapshot markers; the only command that can cause implicit deletes (with the existing safety valves) |
| `cancel` | `command_id` | Stop a running command |

**Results** (connector → Toolbox):

- `ack {command_id}` when the connector accepts the command.
- Ingest batches carry the `command_id`, so Toolbox knows which records answer which command.
- For each requested external id: a record, an explicit `DELETE`, or `not_found`. Only explicit
  results change existence; missing ids are reported as `unresolved`.
- `complete {command_id, status: succeeded | partial | failed, counts, errors}`.

## 5. Semantics

- **Durable:** commands are workflow state in Postgres (D39). Delivery is at-least-once; commands are
  idempotent by `command_id`.
- **Coalescing:** a pending command with an overlapping scope absorbs a new one; both requesters wait on
  the same completion.
- **Priority:** `interactive` (a person or agent waiting) > `verification` (an action run, see
  [action-runs.md](./action-runs.md)) > `freshness` (scheduled) > `background`.
- **Backpressure:** Toolbox never exceeds the connector's declared `max_concurrency` and pauses
  low-priority commands as the declared rate-limit budget runs low.
- **Deadlines:** a command past its deadline becomes `timed_out`, and its requesters see that state.
  Nothing waits forever.
- **Read-your-writes:** after `complete`, the next recompute includes the results. `GET
  /v1/commands/{id}?wait=…` returns the terminal status and the **snapshot id** that contains the
  results, so API, MCP, CLI and UI can show fresh data deterministically.
- **Freshness policies replace connector-side cron.** A kind can declare `maxStaleness` in the
  manifest, and Toolbox issues `refresh … since` commands to meet it. Schedules then live in reviewed
  config, not inside each connector.

## 6. Who requests refreshes

| Requester | Interface | Experience |
|---|---|---|
| A person on an entity page | UI "Refresh" | Inline progress, then the page re-renders at the new snapshot with an "as of" time |
| An agent | MCP `refresh_entity` (an MCP task when it runs long) | Returns once fresh data is visible, or a status if it times out |
| An action run | Its verification step | See [action-runs.md](./action-runs.md) §4 |
| Freshness policy | Manifest `maxStaleness` | Background `since` refreshes |
| Operators | CLI `toolbox connector refresh|resync`, ops MCP | Same commands, with audit |

**Authorization:** requesting a refresh needs a `can_refresh` permission on the entity in the OpenFGA
model, plus per-principal rate limits. Connectors authenticate with tokens scoped to the kinds they
serve. Every command is audited with its requester.

## 7. Prior art

| System | Mechanism | Lesson |
|---|---|---|
| **Backstage** | `POST /refresh {entityRef}` schedules reprocessing of an entity [P: `plugins/catalog-backend/src/service/createRouter.ts` at `184acad`] | Entity-level refresh is expected. The handler returns once the refresh is scheduled, without saying when fresh data is visible [I, from the handler], which is the gap §5 closes |
| **Flux** | `reconcile.fluxcd.io/requestedAt` annotation asks a controller to reconcile now [S] | A request token, plus the controller recording which request it last handled, lets callers know when their request was served |
| **Argo CD** | `argocd.argoproj.io/refresh` annotation (normal or hard) [S] | Two refresh depths: from cache vs. re-reading the source |
| **Port Ocean** | Per-kind `on_resync` and per-integration resync [P, [ocean.md](../ocean.md)] | Full resyncs only; no scoped refresh to wait on |

## 8. Testing (G2)

- The connector conformance kit (`toolbox connector test`) adds command scenarios: scoped refresh,
  coalescing, cancel, deadline, `not_found` vs. missing, and crash between `ack` and `complete`.
- A simulated connector in `toolbox dev` and preview environments answers commands from fixtures, so
  refresh and action-verification flows run in tests without real SaaS calls.

## Sources

- Backstage catalog router and API docs: `plugins/catalog-backend/src/service/createRouter.ts`,
  `docs/features/software-catalog/api.md` (`backstage/backstage` at `184acad`, read from a local clone).
- Flux: [Receivers and the reconcile annotation](https://fluxcd.io/flux/components/notification/receivers/) (secondary; not re-read in this session).
- Argo CD: [Application refresh annotation](https://argo-cd.readthedocs.io/en/stable/user-guide/commands/argocd_app_get/) (secondary; not re-read in this session).
- [ocean.md](../ocean.md) for Ocean's resync model.

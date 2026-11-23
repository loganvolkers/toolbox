# Local simulation: a customer deploy without a cloud account

*Status: proposal, 2026-11-23 (Q19). Nothing in this document is built. It is about the box
([toolbox-and-tools.md](./toolbox-and-tools.md)): how an operator SDLC deploy is stood up and how a
user SDLC session proves the portal is usable. Maintainer SDLC owns the harness.*

---

## 1. Conclusion

**The integration environment runs the same artifact a customer runs — one container plus Postgres
(D33) — against a real git server and a live directory, and it passes only when the catalog answers
a question that was not known when the process started.**

`toolbox serve --fixtures` and `toolbox push` load records straight into the store. That checks the
derivation pipeline. It does not check the image, the listen address, the connector HTTP client, the
supervisor, or a person opening an entity page. Those are the gaps this environment closes.

| Choice | What we use | Limit that decided it |
|---|---|---|
| Workload contract | A Score file (`score.dev/v1b1`) for the portal container and its Postgres | The spec is still `v1b1`. `score-compose` validates HTTP probes and then ignores them, so our harness probes `/readyz` itself |
| Daily runtime | Docker Compose, generated from that file | Compose is the runtime an agent and CI already have. It is the target [plan.md](../plan.md) §6.2 already lists beside Cloud Run |
| Kubernetes check | A small Helm chart, installed on kind, scored by the same probes | Customers install a chart (D9). `score-k8s` emits manifests and a local state directory, which is the wrong operator interface |
| GitHub | Forgejo for the git objects, and a façade that speaks the GitHub REST paths `GITHUB_API_URL` already selects | Forgejo's maintainers say its API will never be the GitHub API. The façade is the impedance mismatch, not a second catalog |
| Directory | The reference portal's SCIM connector, against a live feed | Backstage's production guide treats checked-in `examples/org.yaml` as demo data |
| Cloud, first | The Cloud Run container contract: listen on `0.0.0.0:$PORT`, `/readyz`, `SIGTERM` | Google's own local test is `docker run -e PORT`. `toolbox serve` today binds `127.0.0.1:8080` via `TOOLBOX_LISTEN` and does not read `PORT` |
| Cloud, later | floci-gcp's Cloud Run Admin v2, Docker-backed, after we run it | It does not speak the v1 Knative API `gcloud run` uses. It is a Toolbox-only path, so it is not the Backstage comparison |
| Inner loop | `toolbox dev` for the manifest. Compose `develop.watch` only when the image itself must rebuild | Tilt's job is many services on Kubernetes, configured in Starlark. D3 left Starlark. One container does not need that control plane |
| Pass | A harness probe, including a catalog entity pushed after the agent has stopped | A 200 from `/readyz`, or the agent's own "it works", is recorded and is not a pass |

The reference portal's `expected/` outputs and `tools/mock_github.py` stay. They are the fast
differential check. This environment is the layer below them.

## 2. The two loops

| Loop | Data enters through | What it proves | Where it lives |
|---|---|---|---|
| **Authoring** | Fixture NDJSON, or `toolbox push` of those records | The manifest derives the oracle byte for byte | `examples/reference-portal`, `toolbox test` |
| **Deploy** | Connectors calling a GitHub API and a SCIM feed, writing Postgres, serving HTTP | A customer can host the image and use the portal | This design. `toolbox dev` and `serve --fixtures` do not cover it |

The operator SDLC already draws this line as environments ([sdlc.md](./sdlc.md) §2): local and CI
use fixtures, staging uses real sources. We have no cloud project, so staging's "real sources" are
processes on the same machine that speak the same protocols. The image, the database, and the
request path are the ones a customer uses.

Today the server listens on `TOOLBOX_LISTEN`, default `127.0.0.1:8080`
(`crates/toolbox-cli/src/main.rs`). Cloud Run injects `PORT` and sends traffic to `0.0.0.0` on that
port. A container started the way Google documents local testing will not receive a request until
`serve` binds `0.0.0.0:$PORT` when `PORT` is set. That bind is a prerequisite, not a mock.

The reference `examples/reference-portal/Dockerfile` is a sketch of the one image (D33): it does not
copy a `toolbox` binary. The stack test uses a locally built binary in that shape until `toolbox
build` exists (operator stage `operator/deploy` is still designed, not built).

## 3. How low the fakes sit

A fixture is a record that skipped the connector. A mock at the lowest level still executes the
connector, the HTTP stack, and the git object store. Only the vendor's multi-tenant service is
absent.

| Step a customer deploy takes | Authoring loop | This environment |
|---|---|---|
| Git blob for `catalog-info.yaml` | Text inside `fixtures/git-files.ndjson` | A commit in Forgejo |
| `GET /repos/{org}/{repo}/contents/catalog-info.yaml` with `Accept: application/vnd.github+json` | `tools/mock_github.py`, which reshapes the NDJSON | The façade, reading the blob from Forgejo and base64-encoding it the way GitHub does |
| Connector process | Not started under `--fixtures` | In-image stdio child (D40), `GITHUB_API_URL` pointed at the façade |
| Postgres rows | In-memory store, or rows inserted by `push` | Written by ingest |
| `GET /v1/entities/...` and the entity page | Served, but from fixture input | Served from that ingest |
| Someone pushes a new file | `POST /_mock/repos/{name}` mutates the mock's dict | `git push` to Forgejo, then a refresh |

`mock_github.py` stays the contract test: if the connector is right, those responses become
`fixtures/github.ndjson`. The façade gets the same test for the routes both speak. The integration
oracle is computed by reading the git seed, not by loading `expected/entities.json` into the server.

`git-files` can also read a directory (`root` in the connector). That path skips HTTP. The stack
test uses the API.

## 4. The deployment standards

### 4.1 Score

Score is a CNCF Sandbox workload spec (accepted 2024-07-08; still `score.dev/v1b1` two years later).
One file names the container, the service port, and resource dependencies. An implementation renders
a platform file. `score-compose` renders Compose and provisions Postgres with outputs `host`, `port`,
`name`, `username`, `password`. `score-k8s` renders Kubernetes manifests.

That split matches plan §6.1: portal behaviour is in the image, environment binding is outside it.
The Score file is the binding that is stable across Compose, a chart, and Cloud Run. It is not a
second manifest language for kinds and scorecards.

```yaml
apiVersion: score.dev/v1b1
metadata:
  name: portal
containers:
  main:
    image: .
    variables:
      PORT: "8080"
      DATABASE_URL: postgresql://${resources.db.username}:${resources.db.password}@${resources.db.host}:${resources.db.port}/${resources.db.name}
    readinessProbe:
      httpGet:
        path: /readyz
        port: 8080
service:
  ports:
    http:
      port: 8080
      targetPort: 8080
resources:
  db:
    type: postgres
```

Two limits, both from the `score-compose` README:

- HTTP readiness probes are validated and **ignored**. Compose has no HTTP probe. The harness curls
  `/readyz`.
- CPU and memory limits are validated and ignored.

We do not adopt Humanitec's orchestrator, and we do not treat `.score-compose/` state as the
operator interface. `score-compose generate` is a build step. The committed inputs are `score.yaml`
and the chart. The generated Compose file can be committed or produced in CI; either way the probes
are the drift check.

### 4.2 Compose

The Compose Specification's `develop.watch` (rebuild, restart, sync) is the live-reload Tilt would
otherwise be for. The portal image changes rarely relative to the manifest. `toolbox dev` remains
the authoring loop (D44): watch the manifest, full recompute, no image rebuild. Watch belongs on
the deploy loop only.

### 4.3 Helm

The chart is what a customer installs (D9, plan §6.2). It is small:

| Object | Contents |
|---|---|
| Deployment | One replica of the portal image. `DATABASE_URL` from a value. `PORT=8080`. Listen `0.0.0.0:8080` |
| Probes | HTTP GET `/readyz` (startup and readiness), `/healthz` (liveness) |
| Service | Port 8080 |
| Database | A required `database.url`. A Postgres pod is a CI values file for kind, not a production subchart |

Bitnami and CloudNativePG are not requirements of the chart. The kind job's extra Postgres is test
scaffolding.

`score-k8s` output is a useful diff against the chart (does the chart still match the Score file?)
and a poor thing to hand an operator. The chart is the artifact.

### 4.4 Tilt

Tilt's README is "Kubernetes for Prod, Tilt for Dev": file watch, image build, and a dashboard, with
the Tiltfile in Starlark. `docker_compose()` is supported and is how a team that already runs Tilt
would attach. `tilt get` reads an API, and `UIResource` is documented as a legacy status object.

That solves a problem we do not have on the default path. G1 is one container, not a set of services
that must reload together. The authoring loop is already specified as `toolbox dev`. A Tiltfile that
calls `docker_compose` on the generated file is an optional adapter for customers who have Tilt. It
is not the simulator, and Starlark is not added to the box (D3).

### 4.5 kind

kind runs Kubernetes nodes as containers, bootstrapped with kubeadm, and is a CNCF-certified
installer. The current install line in its README is `kind` v0.33.0. The chart's CI job is
`kind create cluster`, `helm install`, then the same probes as Compose. k3d and minikube run a
cluster too; kind is the one whose nodes are upstream kubeadm, which is what we want the chart
tested against. The daily path stays Compose, because requiring a cluster to see the portal
contradicts G1.

### 4.6 The others

| Tool | What it is good at | Why it lost this job |
|---|---|---|
| **Skaffold Cloud Run deployer** | Google's inner loop onto Cloud Run | The deployer talks to a real GCP project. `gcloud beta code dev` builds locally and still needs the container contract; it is not a fake control plane |
| **Knative** | The model Cloud Run came from | A local install is a cluster plus serving plus networking. The contract we must pass is the container contract |
| **LocalStack, Moto** | AWS APIs | The reference target is Cloud Run (D33), not ECS. An ECS Terraform module, if it is tested later, can use them |
| **fake-gcs-server** | GCS | Object storage is optional (plan §6.2). Add it when a test writes an object |
| **floci-gcp** | Cloud Run Admin API v2, and by default it runs the image in Docker, injecting `PORT`, `K_SERVICE`, `K_REVISION`, `K_CONFIGURATION` | MIT, first commit 2026-05-17. Its own docs say `gcloud` is not supported, because `gcloud` uses the v1 API. Not run in this session. Optional, after a hands-on check, for a Toolbox-only "call the admin API" test |
| **kauche/cloud-run-api-emulator** | A gRPC Cloud Run API | The project calls itself super-experimental. It does not replace the container |
| **Testcontainers** | Starting Postgres from a Rust test | The artifact under test is the Compose file and the chart, which an agent must also be able to start. The acceptance script is that file |
| **WireMock, MockServer, MSW** | Stubbed HTTP | Same layer as `mock_github.py`. Right for the connector unit test |
| **Garden, DevSpace, Telepresence, Radius** | Kubernetes platforms | Another control plane in front of one container |
| **Dagger** | CI pipelines as code | No runtime contract |

## 5. The fake GitHub

The GitHub connector calls these paths (`crates/toolbox-connector-github`), and `git-files` calls
the contents path. Both send `Accept: application/vnd.github+json`, `X-GitHub-Api-Version`, and a
bearer token, and both page with `per_page=100` until a page returns fewer than 100 items.

| Path | Used for |
|---|---|
| `GET /orgs/{org}/repos` | Repository list |
| `GET /repos/{org}/{repo}/branches/{branch}` | `protected` |
| `GET /repos/{org}/{repo}/collaborators?permission=admin&affiliation=direct` | Admins |
| `GET /repos/{org}/{repo}/contents/{path}` | CODEOWNERS, `catalog-info.yaml` |
| `GET /orgs/{org}/installations` and `GET /user/installations/{id}/repositories` | GitHub App installations |
| `GET /repos/{org}/{repo}/actions/...` | Workflow runs |

Forgejo serves a GitHub-like API under `/api/v1`. In forgejo/forgejo#9285 the project states that
the API will never be the GitHub API, and that an `Accept: application/vnd.github+json` response is
a selective compatibility shim. Installations and Actions runs are GitHub-shaped. Pointing the
connector at Forgejo's API would test a different integration from the one a customer configures.

The façade is a small HTTP server:

1. It requires the GitHub Accept header and the bearer token the seed created.
2. It answers the six rows above. List routes honour `page` and `per_page` and return a short last
   page, or the connector will request up to 1000 pages.
3. Contents and CODEOWNERS are read from Forgejo (its API or its git), then returned as GitHub's
   base64 contents object. A push is visible on the next GET.
4. `404` is a 404, because the connector treats 404 as "absent" and any other status as an error.
5. Installations and Actions are a projection, off by default. The standard portal does not need
   GitHub Apps. The reference portal's approved-apps example keeps `mock_github.py` for that. When
   the façade grows those routes, they are generated from Forgejo state (a config file naming the
   app, workflow runs read from Forgejo Actions if they actually ran), not from `github.ndjson`.

`GITHUB_API_URL` is already the switch. No connector change is required to aim at the façade.

### 5.1 The org the façade serves

Two seeds, one façade:

| Seed | Contents | Who it is for |
|---|---|---|
| **Acme** | A small org we author, in Backstage descriptor form, matching the reference portal's `acme` name | The deterministic stack test. The git history is committed (a bundle or a seed script). The oracle walks those files |
| **Pinned mirrors** | `backstage/backstage` at `feeadd4d4efcbda9f1573b95e356e0f3fc0b5d17` and `openfga/openfga` at `d67ba7b1d2eaf945cffab3a9129ca1a4c7ad5251`, cloned into the façade's Forgejo | The benchmark. OpenFGA has no `catalog-info.yaml`; the harness adds descriptors in a commit on top of the pin and records that commit. The agent can read the result. The agent cannot see the probe's expected values, because those are computed at score time |

Acme is the idiomatic portal. The mirrors exist so the benchmark is not only a world we wrote to be
easy to ingest.

## 6. The idiomatic portal

Backstage's system model (`docs/features/software-catalog/system-model.md` in `backstage/backstage`)
is the vocabulary: a **Component** is software, an **API** is a boundary a component provides or
consumes, a **Resource** is infrastructure the component needs, a **System** groups a handful of
those, a **Domain** groups systems, and **User** / **Group** are the org. A **Location** points at
more descriptors. Backstage's production app-config says the sample `examples/entities.yaml` and
`examples/org.yaml` are demo data to replace. Descriptors live in the repos they describe.
`catalog-info.yaml` at the root of `backstage/backstage` is itself a Component (`owner: CNCF`,
`github.com/project-slug`).

Acme follows that, and it follows the reference portal's kinds so the same manifest can be pointed
at the façade:

| Entity | Lives in | Relations |
|---|---|---|
| `group:platform`, `group:payments`, four users | SCIM feed (`idp` connector on the reference portal) | Group membership |
| Domain `payments`, domain `platform` | `acme/systems` repo, `catalog-info.yaml` | Systems `partOf` a domain |
| System `checkout` | `acme/systems` | Owned by `group:payments` |
| Component `payments-api` | its own repo | `owner` group:payments, `system` checkout, `providesApi` payments |
| API `payments` | the payments-api repo | `definition` a small OpenAPI file in the repo |
| Component `checkout-web` | its own repo | `consumesApi` payments |
| Component `ledger` | its own repo | `system` checkout, `dependsOn` resource payments-db |
| Resource `payments-db` | the ledger repo | A described database. It is not a second cloud account |
| CODEOWNERS | each repo | The github connector's ownership input |

One repo has no README, so a "has a README" scorecard fails because the file is absent, not because
a fixture said so. Users and groups are not YAML in git. The SCIM feed
(`examples/reference-portal/tools/scim_feed.py` is the seed format) runs as a server the `idp`
connector polls. Adding a group member there, then refreshing, is the directory's mutation probe.

The manifest for this run is the reference portal's manifest with connector URLs aimed at the
façade and the feed, and with `--fixtures` off. Where the reference portal's expectations only hold
on the NDJSON, the stack test's expectations are a separate file derived from the seed, so we do
not fork the byte oracle.

## 7. Probes

The harness owns the probes. The agent's last message is ignored
([dx-benchmarks.md](./dx-benchmarks.md) §2, independent truth). `/readyz` and `/healthz` are
recorded. A pass requires the catalog.

**Shared probes** (Toolbox and Backstage, each through a harness adapter):

| # | Probe | Fail |
|---|---|---|
| S1 | Every Component, API, System, Domain and Resource in the seed's `catalog-info.yaml` files is returned by the catalog API, with the same `spec.owner` and system relation | A missing name, an extra name, or a wrong owner |
| S2 | The HTML page for `payments-api` contains its name and its owner | 200 with an empty shell, or a page that never mentions the entity |
| S3 | After the agent process has exited, the harness pushes a new `catalog-info.yaml` whose component name is a random id the prompt did not contain, using a git credential the agent was not given. A refresh (or the connector's own schedule, bounded) makes that component appear | The name never appears. A static server, a fixture load, and a one-shot import all fail here |

The Backstage adapter reads `/api/catalog/entities`. The Toolbox adapter reads the entity API. The
adapter is harness code.

**Toolbox probes** (the stack test and the Toolbox arm; not required of Backstage):

| # | Probe |
|---|---|
| T1 | `toolbox get` shows the component's fields came from the github or git-files connector, not from a native or fixture source |
| T2 | A principal in `group:payments` can read `payments-api`, and a principal in no group cannot, through the API. OpenFGA is the one in the image (Q11) |
| T3 | The README scorecard's failing set matches a walk of the repos |
| T4 | MCP `initialize` and the entity read return the same component S1 saw |
| T5 | `GET /metrics` includes the catalog, and a trace for the entity read is exported when a collector is attached (the OTel testbed already does this for `--fixtures`) |

S3 is what stops the benchmark being gamed. The agent is stopped first, so it cannot watch the push
and update a stub. The name is chosen at probe time. Health-only success scores zero. An empty
catalog scores zero.

A hand-rolled process that polls the façade and serves a catalog would pass S1–S3. That process is
a portal. The benchmark does not offer that arm. Both scored arms have to leave a running product
that tracks the git server without the agent still being in the loop.

## 8. The ladder

| | Name | Run | Pass |
|---|---|---|---|
| L0 | Reference portal | `toolbox validate`, `toolbox test`, `tools/reference.py --check` | `expected/` byte for byte. Exists |
| L1 | Connector contract | `mock_github.py` plus `toolbox connector test` | Records match the NDJSON. Exists in part |
| L2 | Stack | Compose: Postgres, portal image, Forgejo, façade, SCIM. No `--fixtures` | S1–S3 and T1–T4 against Acme |
| L3 | Deploy contract | `docker run -e PORT` as in Google's local Cloud Run doc; then `helm install` on kind | The same probes. A `SIGTERM` test: the process exits without a second signal inside the 10s Cloud Run window |
| L4 | Usability script | A fixed sequence, no agent: open the entity page, read the owner, read the score, push a file, see it | S2, T3, S3 |
| L5 | DX Benchmark | Fresh agents, Toolbox against Backstage, on the pinned mirrors | §7 shared probes, plus cost. Catalogued, blocked |

L3's optional last step, once floci-gcp has been run and its limits confirmed: create the service
through the Cloud Run Admin v2 API and hit the URL it returns. That step is not in L5. Backstage's
golden path deploys a stateless Node container plus external Postgres (image, env, health check,
public URL), on Compose for a small install. Giving only Toolbox a fake `gcloud` would make the
cheaper arm the one we wrote the fake for.

## 9. The benchmark

Pre-registered in [experiments/deploy-ab/README.md](../../experiments/deploy-ab/README.md), catalogued
as `deploy-ab`. The first run is recorded in
[experiments/deploy-ab/RESULTS.md](../../experiments/deploy-ab/RESULTS.md). The bar in that README
is the bar.

| | |
|---|---|
| Question | Can a fresh agent deploy a usable portal onto a laptop, with Toolbox and with Backstage, when the only services are a local GitHub and Postgres? |
| Layers | Operator SDLC (`operator/deploy`, `operator/verify`) and user SDLC (`user/discover`) |
| Rival | Backstage, created from `@backstage/create-app` and run as one container plus Postgres, catalog pointed at the same façade. That is the deploy Backstage's own golden path describes for a small install |
| Repos | `backstage/backstage@feeadd4d4efcbda9f1573b95e356e0f3fc0b5d17`, `openfga/openfga@d67ba7b1d2eaf945cffab3a9129ca1a4c7ad5251` |
| Bar | **Holds** if the Toolbox arm passes S1–S3 and T1–T4, the Backstage arm passes S1–S3, and Toolbox's token count is lower at that equal shared-probe result. **Fails** if Toolbox misses a shared probe Backstage hits, or if Toolbox passes them with at least as many tokens. **Void** if either arm's only success is `/readyz` |
| First run | 2026-11-23. Inconclusive: Toolbox passed S1–S3 and T1–T4, Backstage failed S2, and the agent runtime returned no token counts. create-app 0.9.4 ran; it rejected the Node 22.23.5 pin |
| Rerun | 2026-11-23. The portal image pushes the directory. Toolbox passed again. Backstage failed S2 again, and there were still no token counts. Inconclusive, not the defeat [RESULTS-v2.md](../../experiments/deploy-ab/RESULTS-v2.md) first recorded. The bar above is unchanged |
| Adjudication | 2026-11-23, by an agent that designed neither arm. The arms were unequal. S2 authenticated only Toolbox. The Toolbox image and container existed before the run, although the prompt said the portal was not running. The Backstage prompt suggested a provider that needs GraphQL, which the facade lacks. Two Backstage launches overlapped in the rerun. Rerun under protocol 2 after those are fixed ([ADJUDICATION.md](../../experiments/deploy-ab/ADJUDICATION.md), D01M83RDWR52WVNER0MV5T84NSX) |

The harness starts Forgejo, the façade, the registry and Postgres before the agent. It does not
start the portal. The agent is told the façade URL, the org, and the Postgres URL, and must leave a
base URL. The harness then runs the probes against that URL. Cost is tokens, tool calls, wall time
and lines of glue, as in the other benchmarks.

Known limits, written before any run: one agent per arm; floci-gcp has not been executed; the
Backstage image build was refused here before (Q18); OpenFGA's descriptors are a harness commit on
top of the pin.

## 10. Work, in dependency order

| # | Work | Done when |
|---|---|---|
| 1 | `serve` binds `0.0.0.0:$PORT` when `PORT` is set, and still honours `TOOLBOX_LISTEN` | `docker run -e PORT=8080 -p 8080:8080` answers `/readyz` from another network namespace, not only from `127.0.0.1` inside the container |
| 2 | `score.yaml` for the reference portal and `score-compose generate` | Compose comes up with Postgres and the image |
| 3 | Acme seed in Forgejo and the façade | The github and git-files connectors, pointed at the façade, emit the seed's repos, CODEOWNERS and `catalog-info.yaml` |
| 4 | SCIM feed as a process | The `idp` connector ingests the groups |
| 5 | L2 script | S1–S3 and T1–T4, no `--fixtures` |
| 6 | Helm chart and a kind job | L3 probes match L2 |
| 7 | L4 script | A non-agent run of the usability sequence |
| 8 | Unblock `deploy-ab` | The README's bar is unchanged except for its status line; then run it |

## 11. What this leaves alone

- D33: one image plus Postgres. Modular deployment stays optional.
- D9: the customer deploys with their own tooling. We give them the chart, the Score file and Compose.
- The reference portal's `expected/` files. This environment has its own oracle, derived from the seed.
- `mock_github.py` and `serve --fixtures`. They remain the fast path.
- M5 actions, which stay paused (D64). The façade does not implement write endpoints for Actions.
- A fake GCP as the benchmark's target. It can be a later Toolbox-only L3 step.

## Sources

- Score specification README and `score-v1b1.json`, `score-spec/spec` (read from GitHub). `score-compose` README: Postgres outputs, and HTTP probes validated then ignored (read from GitHub). CNCF project page: accepted 2024-07-08 at Sandbox (search excerpt, cncf.io/projects/score). Two-year sandbox note, score.dev blog 2026-11-04 (search excerpt).
- Compose Develop Specification, `compose-spec/compose-spec` `develop.md` (`develop.watch` actions, read from GitHub).
- Tilt README, `tilt-dev/tilt` (read from GitHub): "Kubernetes for Prod, Tilt for Dev". `docker_compose` and `tilt get` / `UIResource`: docs.tilt.dev, which this environment blocks; read as search excerpts. `internal/store/store.go` is an in-process Redux-style store (read from GitHub); it is not the public API.
- kind README, `kubernetes-sigs/kind` v0.33.0 (read from GitHub): nodes are containers, kubeadm, CNCF-certified installer.
- Cloud Run container runtime contract (listen on `0.0.0.0` and `PORT`, `SIGTERM` then about 10s, startup) and "Test a Cloud Run service locally" (`docker run -e PORT`, `gcloud beta code dev`). Read as fetched documentation pages.
- Skaffold Cloud Run deployer (skaffold.dev, search excerpt): deploys to a GCP project id and region.
- floci-gcp README and `docs/services/cloud-run.md` (read from GitHub): Cloud Run Admin v2, Docker execution injecting `PORT` and `K_*`, `gcloud` unsupported because it uses the v1 API. MIT, repository created 2026-05-17. Not run.
- Forgejo API usage (forgejo.org docs, search excerpt) and forgejo/forgejo#9285 (search excerpt of the PR body): the API will never be the GitHub API; `Accept: application/vnd.github+json` is a selective shim.
- Backstage system model, `backstage/backstage` `docs/features/software-catalog/system-model.md`, and root `catalog-info.yaml`, at `feeadd4d4efcbda9f1573b95e356e0f3fc0b5d17` (read from GitHub). Golden-path deploying doc and `app-config.production.yaml` (GitHub; the deploying page via search excerpt): stateless Node container, external Postgres, health checks, and the sample catalog locations marked as demo. Getting started: `npx @backstage/create-app`, local SQLite, not a production install (search excerpt).
- This repository: [plan.md](../plan.md) §6.1–6.2, [v1-spec.md](../v1-spec.md) §10, [sdlc.md](./sdlc.md), [dx-benchmarks.md](./dx-benchmarks.md), D33, D9, D40, D44, Q11, Q18. Connector paths in `crates/toolbox-connector-github` and `crates/toolbox-connector-git-files`. `examples/reference-portal/tools/mock_github.py`.

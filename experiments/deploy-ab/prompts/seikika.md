Deploy the Acme developer portal so it is running and usable on this machine. You are a fresh agent. Work only in /tmp/deploy-ab/arm-seikika. Do not read /agent/repos/seikika, /agent/repos/seikika/experiments, docs/design/local-simulation.md, or docs/research-log.md, and do not search for scoring scripts, probes, or expected answers.

What is already running, in Docker Compose project acme (network acme_default):
- Postgres. From the host: postgres://seikika:seikika@127.0.0.1:54329/seikika. From a container on acme_default: postgres://seikika:seikika@postgres:5432/seikika. The database is empty.
- A GitHub-shaped HTTP API for org acme. From the host: http://127.0.0.1:8091. From a container on that network: http://facade:8091. Send Authorization: Bearer acme-reader and Accept: application/vnd.github+json. This is the only catalog source. Do not load fixtures.
- A local registry on port 5000.
- The portal is not running. Do not recreate or wipe Postgres, Forgejo, the facade, or the registry. Docker is already running; do not start another dockerd.

You are given this workspace, which contains the seikika CLI at bin/seikika, the connector binaries beside it, the Score file examples/acme-live/deploy/score.yaml, the Helm chart examples/acme-live/deploy/chart, and the operator docs examples/acme-live/README.md. Put bin on PATH. Deploy with the portal image (seikika serve). Compose or the chart are both acceptable. The image tag seikika-acme:local may already exist; rebuild it from this workspace if you change anything it needs.

The running portal must:
- be the portal process, reachable on a base URL from this machine
- read org acme from that GitHub API, including repositories and catalog-info.yaml
- serve the directory the operator docs describe
- keep running after you exit and keep reflecting catalog-info.yaml files in that org, including files that appear later, without you still watching

When that is true, write the base URL and nothing else to /tmp/deploy-ab/arm-seikika-url. Then stop. Do not leave a watcher, a poll loop, or a follow-up agent running.

In your final message, report the base URL you wrote and a short list of what was hard. Do not claim a score.
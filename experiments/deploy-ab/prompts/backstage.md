Deploy a usable Backstage portal on this machine. You are a fresh agent. Work only in /tmp/deploy-ab/arm-backstage. Do not read /agent/repos/seikika, /tmp/deploy-ab/arm-seikika, experiments/deploy-ab, docs/design/local-simulation.md, or docs/research-log.md, and do not search for scoring scripts, probes, or expected answers.

Create the app with @backstage/create-app at version 0.9.4. Node is 22.23.5 at /opt/node-v22.23.5-linux-x64/bin (put that directory first on PATH). Yarn is 1.22.22. There is no GCP, AWS, or Azure account. Docker is already running; do not start another dockerd.

What is already running, in Docker Compose project acme (network acme_default):
- Postgres. From the host: postgres://seikika:seikika@127.0.0.1:54329/seikika. From a container on acme_default: host postgres, port 5432, user seikika, password seikika, database seikika. You may CREATE another database on that server for Backstage. Do not drop the seikika database, and do not recreate Postgres, Forgejo, the facade, or the registry.
- A GitHub-shaped HTTP API for org acme. From the host: http://127.0.0.1:8091. From a container on that network: http://facade:8091. Send Authorization: Bearer acme-reader and Accept: application/vnd.github+json. The org's repositories contain catalog-info.yaml files. Point the catalog at this API (the GitHub catalog provider, or locations that read this API). The sample examples/entities.yaml does not satisfy the task.
- A local registry on port 5000.

The running portal must:
- be the Backstage backend image, or the Backstage backend process, reachable on a base URL from this machine
- show the catalog entities from that org, including each one's name and owner
- keep running after you exit and keep reflecting catalog-info.yaml files in that org, including files that appear later, without you still watching

When that is true, write the base URL and nothing else to /tmp/deploy-ab/arm-backstage-url. Then stop. Do not leave a watcher, a poll loop, or a follow-up agent running.

In your final message, report the base URL you wrote and a short list of what was hard. Do not claim a score.
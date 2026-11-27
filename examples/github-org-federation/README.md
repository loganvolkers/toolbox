# GitHub organization score federation

Run Seikika locally to score the GitHub repositories and workflows your credentials can read. The
organization portal shows its own repository hygiene and GitHub Actions security standards alongside
the catalog, scorecards, tracks, campaigns, and actions published by child portals. Each definition
identifies where it came from.

## Create a local snapshot

From the Seikika repository root with the pinned Rust toolchain, build the CLI and Workspace
connector from this source. Run `gh auth status` to check your existing GitHub login. The connector
uses `GH_TOKEN`, `GITHUB_TOKEN`, or the current GitHub CLI login, in that order. The credential's
repository access determines the inventory. Install the pinned `zizmor` from the maintainer portal
for the workflow audit and ensure it is on `PATH`.

```bash
pnpm install --frozen-lockfile
pnpm build
cargo build --locked -p seikika-cli -p seikika-connector-workspace
target/debug/seikika tools install portal --via pipx
target/debug/seikika validate examples/github-org-federation
target/debug/seikika test examples/github-org-federation
snapshot=$(mktemp -d)
target/debug/seikika report examples/github-org-federation -o web --out "$snapshot"
python3 -m http.server 8765 --bind 127.0.0.1 --directory "$snapshot"
```

Open [the loopback report](http://127.0.0.1:8765/) and select **Scorecards** for the per-repository
results. In **Catalog → GitHub Actions workflows**, switch between one list and groups by source
repository. Open any workflow to see its score, findings, and links to the **Workflow security**
track and **Resolve zizmor findings** campaign. The output includes an offline `index.html` and
`report.json`. Keep snapshots made with private repository access private. Refresh by running
`report` again. The build and report run without a GitHub App registration or Postgres.

To browse the same report from a running portal, start `target/debug/seikika serve
examples/github-org-federation --listen 127.0.0.1:8080` and open
[the live report](http://127.0.0.1:8080/report/). Local development identity can be selected with
`?as=<username>` when a portal has no `auth` configuration. The static report shows the action list
and form and previews the selected target and inputs in the browser. A live report can request a
preview from the action executor and can dispatch actions the portal is configured to run.

For the docs site, build the public variant and pass its snapshot to Astro.

```bash
target/debug/seikika report examples/github-org-federation/public.yaml -o web \
  --out site/.generated/organization
SEIKIKA_BIN="$PWD/target/debug/seikika" \
  SEIKIKA_ORG_REPORT_DATA="$PWD/site/.generated/organization/report.json" pnpm docs:build
```

The Pages workflow rebuilds this snapshot from public inventory. The organization view lives at
`/demo/organization/` and the framework repository's report lives at `/report/`.

## Publish a child portal snapshot

Build a shareable snapshot from the child repository and commit its report data at the published
path. Review the static data before making the commit public.

```bash
snapshot=$(mktemp -d)
seikika report .seikika -o web --out "$snapshot"
cp "$snapshot/report.json" .seikika/report.json
git add .seikika/report.json
```

The parent reads this file at the same default-branch commit as the repository facts. A missing
publication leaves the repository and its organization scores in view. An unreadable publication
shows an error in the federation source list. A configured Git child uses the same publication
path at its configured revision. A custom `Portal` kind can read `record.report` and
`record.reportError` through the Workspace connector's `portal` mapping, as shown in the
[federated-actions example](../federated-actions/org/seikika.yaml).

## Verify the trust boundary

The parent reads GitHub's inventory and uses the default branch's commit and tree IDs to read
README, LICENSE, and CODEOWNERS facts. It checks the presence of a child manifest using Git tree
metadata. The parent uses its own policy to score repository facts and runs `zizmor` offline on
bounded workflow files from the pinned tree, with repository configuration and ignores disabled.
Symbolic links and submodules are excluded. Child report data appears under its source and cannot
change the parent standard or its scores. Each scan reads bounded Git objects and skips oversized
README and CODEOWNERS blobs. A GitHub error for one repository appears as **Unavailable** alongside
healthy repositories. An inventory error fails the snapshot.

| Demonstration | Expected organization result |
|---|---|
| [Oversized repository](https://github.com/seikika/example-unperformant-repo) | Seikika skips the oversized README and large unrelated asset. The repository is scanned and falls below bronze. |
| [Malformed child](https://github.com/seikika/example-incompetent-repo) | The invalid manifest and report leave all repository scores available. The demo falls below bronze. |
| [Untrusted child](https://github.com/seikika/example-malicious-repo) | The parent ignores the child's silver claims and rule replacement. Its short README and missing license fail. The report displays the literal HTML marker and apostrophe in the filename as text. |

See the [federation design](../../docs/design/federation.md) and [Workspace connector reference](../../docs/reference/workspace-connector.md).

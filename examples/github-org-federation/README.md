# GitHub organization score federation

Run Seikika locally to score the GitHub repositories your credentials can read. The organization
portal evaluates one repository hygiene standard for every default branch. Its static snapshot
shows each score, the commit that supplied the facts, and whether a repo portal manifest is
present. This example uses the box to collect facts, apply the standard, and write the report.

## Create a local snapshot

From the Seikika repository root with the pinned Rust toolchain, build the CLI and Workspace
connector from this source. Run `gh auth status` to check your existing GitHub login. The connector
uses `GH_TOKEN`, `GITHUB_TOKEN`, or the current GitHub CLI login, in that order. The credential's
repository access determines the inventory.

```bash
cargo build --locked -p seikika-cli -p seikika-connector-workspace
target/debug/seikika validate examples/github-org-federation
target/debug/seikika test examples/github-org-federation
snapshot=$(mktemp -d)
target/debug/seikika report examples/github-org-federation -o web --out "$snapshot"
python3 -m http.server 8765 --bind 127.0.0.1 --directory "$snapshot"
```

Open [the loopback report](http://127.0.0.1:8765/) and select **Scorecards** to see the per-repository
results. The output includes an offline `index.html` and `report.json`. Any snapshot made with
private repository access belongs in a private location. Refresh it by running `report` again.
The build and report run without a GitHub App registration, a Seikika server, or Postgres.

For the docs site, build the public variant and pass its snapshot to Astro.

```bash
target/debug/seikika report examples/github-org-federation/public.yaml -o web \
  --out site/.generated/organization
SEIKIKA_BIN="$PWD/target/debug/seikika" \
  SEIKIKA_ORG_REPORT_DATA="$PWD/site/.generated/organization/report.json" pnpm docs:build
```

The Pages workflow rebuilds this snapshot from public inventory. The organization view lives at
`/demo/organization/` and the framework repository's report lives at `/report/`.

## Verify the trust boundary

The parent reads GitHub's inventory and uses the default branch's commit and tree IDs to read
README, LICENSE, and CODEOWNERS facts. It checks the presence of a child manifest using Git tree
metadata. It treats child manifests, report baselines, workflows, symbolic links, and submodules
as outside the parent policy. Each scan reads bounded Git objects and skips oversized README and
CODEOWNERS blobs. A GitHub error for one repository appears as **Unavailable** alongside healthy
repositories. An inventory error fails the snapshot.

| Demonstration | Expected organization result |
|---|---|
| [Oversized repository](https://github.com/seikika/example-unperformant-repo) | Seikika skips the oversized README and large unrelated asset. The repository is scanned and falls below bronze. |
| [Malformed child](https://github.com/seikika/example-incompetent-repo) | The invalid manifest and report leave all repository scores available. The demo falls below bronze. |
| [Untrusted child](https://github.com/seikika/example-malicious-repo) | The parent ignores the child's silver claims and rule replacement. Its short README and missing license fail. The report displays the literal HTML marker and apostrophe in the filename as text. |

See the [federation design](../../docs/design/federation.md) and [Workspace connector reference](../../docs/reference/workspace-connector.md).

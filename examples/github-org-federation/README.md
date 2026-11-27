# GitHub organization score federation

Run Seikika locally to score the GitHub repositories and workflows your credentials can read. The
organization portal shows its own repository hygiene and GitHub Actions security standards alongside
the catalog, scorecards, tracks, campaigns, and actions published by child portals. Each definition
identifies where it came from.

## Create a local snapshot

From the Seikika repository root with the pinned Rust toolchain, build the CLI and the Workspace
and Runner connectors. Run `gh auth status` to check your existing GitHub login. The connector
uses `GH_TOKEN`, `GITHUB_TOKEN`, or the current GitHub CLI login, in that order. The credential's
repository access determines the inventory. Install the pinned `zizmor` from the maintainer portal
for the workflow audit and ensure it is on `PATH`.

```bash
pnpm install --frozen-lockfile
pnpm build
cargo build --locked -p seikika-cli -p seikika-connector-workspace -p seikika-connector-runner
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

## Verify a child implementation

The child examples use checks from the project’s release process. They use the
[meta-scoring package](../../packages/meta-scoring/README.md) to check the release
standard itself and show its review evidence.

| Fork checkout | Check | Verified commit |
|---|---|---|
| [Plannotator](children/plannotator/seikika.yaml) | Run the project's release version check across npm and agent integration manifests. | `e145a52fc468a20d5a37a4443a4f1398673f479e` |
| [Impeccable](children/impeccable/seikika.yaml) | Check matching engine and launcher pins and well-formed, nonzero checksum entries for five platforms. | `9b1ebc34c415ebc8b4c1bacb699539179087f271` |
| [Kubernetes](children/kubernetes/seikika.yaml) | Compare generated Go workspace membership and declared module names with the staging manifests. | `2795fb9e20e05ea10c884e5d15aede6d4aa3b69d` |
| [OpenCodeReview](children/open-code-review/seikika.yaml) | Check the launcher and platform package templates against the release workflow’s platform list. | `03c2708b1239b76b2d2ae9e29e05cc0369789f45` |
| [Astryx](children/astryx/seikika.yaml) | Run Astryx’s published-package alignment logic and report the manifest containing each version mismatch. | `0348252d8a1baffaa3e9ef5959bd0c3ea0797490` |
| [Magpie](children/magpie/seikika.yaml) | Run the native release asset packer and check the embedded GUI paths and bytes. | `e612932fbe305eb4fd3522824d5e51beeb473ab8` |
| [Security Audit Skill](children/security-audit-skill/seikika.yaml) | Compare the documented installation inventory with the distributed files and entrypoint name. | `d4a347c707bd996703763fcb30141b37085e48fa` |

Use a disposable checkout of the named fork and a source build of Seikika. From this repository's
root, set `child` to a directory name from the table and set `checkout` to the absolute path of that
checkout. The commands replace that checkout's demo portal manifest. Put a compatible Go toolchain
on `PATH` for Kubernetes and Magpie. The Node and Python checks use their standard libraries.

```bash
export PATH="$PWD/target/debug:$PATH"
source="examples/github-org-federation/children/$child"
target="$checkout/.seikika"
mkdir -p "$target"
sed 's|package: ../../../../packages/meta-scoring|package: packages/meta-scoring|' \
  "$source/seikika.yaml" > "$target/seikika.yaml"
cp -R "$source/fixtures" "$source/tests" "$source/assessments" "$target/"
for script in "$source"/check-*; do
  if [ -f "$script" ]; then cp "$script" "$target/"; fi
done
seikika vendor "$PWD/packages/meta-scoring" --portal "$target"
(
  cd "$checkout"
  seikika validate .seikika
  seikika test .seikika
  seikika assess check .seikika
  seikika report .seikika -o web --out /tmp/"$child"-report
)
```

The seven pinned checkouts reached `ready` in the local replay. Each release rule and scorecard
earned gold on its fixtures and current independent review. The parent replay used those child
reports alongside its own repository and workflow standards. The published organization snapshot
follows each fork’s default branch and reads its report when the fork publishes it.

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
README, license, and CODEOWNERS facts. It checks the presence of a child manifest using Git tree
metadata. The parent uses its own policy to score repository facts and runs `zizmor` offline on
bounded workflow files from the pinned tree, with repository configuration and ignores disabled.
Symbolic links and submodules are excluded. Child report data appears under its source and cannot
change the parent standard or its scores. Each scan reads bounded Git objects and skips oversized
README, license, and CODEOWNERS blobs. A GitHub error for one repository appears as **Unavailable** alongside
healthy repositories. An inventory error fails the snapshot.

| Demonstration | Expected organization result |
|---|---|
| [Oversized repository](https://github.com/seikika/example-unperformant-repo) | Seikika skips the oversized README and large unrelated asset. The repository is scanned and falls below bronze. |
| [Malformed child](https://github.com/seikika/example-incompetent-repo) | The invalid manifest and report leave all repository scores available. The demo falls below bronze. |
| [Untrusted child](https://github.com/seikika/example-malicious-repo) | The parent ignores the child's silver claims and rule replacement. Its auxiliary README and missing license fail. The report displays the literal HTML marker and apostrophe in the filename as text. |

See the [federation design](../../docs/design/federation.md) and [Workspace connector reference](../../docs/reference/workspace-connector.md).

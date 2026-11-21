# Out-of-box: Toolbox on Repositories We Didn't Write

**Question** ([repo-value.md](../../docs/design/repo-value.md) R2): with no tuning, are the
starter's and the packages' findings true and worth fixing on real repositories?

**Answer (2026-11-21, four forks):** yes, once three kinds of false finding are fixed. Five
connector and package bugs, found on the first run, are fixed. Missing run data is still scored
as a failure until PR #24 scores it `unknown`. After that, 939 of 940 failing (entity, rule)
pairs are true, and 70% of the true ones are worth fixing. The triage is an agent's, checked by
sampling the evidence, not by the repositories' maintainers. Full results:
[research-log.md](../../docs/research-log.md), *Out-of-box R2*.

## What it runs

Two portals, the setup a pilot copies:

| Portal | Includes | For |
|---|---|---|
| [portal/](portal/toolbox.yaml) | The starter's `repo-hygiene` and `docs`, and [packages/github-actions](../../packages/github-actions/README.md) | Any repository |
| [portal-pnpm/](portal-pnpm/toolbox.yaml) | The same, plus [packages/pnpm-expo](../../packages/pnpm-expo/README.md) | pnpm workspaces |

Run history (`github-actions`) is skipped: it needs a token and changes over time.

## Run it

```bash
cargo build
git clone --depth 1 https://github.com/openfga/openfga /tmp/openfga
cd /tmp/openfga
PATH=/path/to/zizmor-and-actionlint:$PATH \
  /path/to/toolbox/target/debug/toolbox report /path/to/toolbox/experiments/out-of-box/portal \
  --skip github-actions -o md
```

`toolbox validate experiments/out-of-box/portal` and `toolbox validate
experiments/out-of-box/portal-pnpm` check the portals themselves. Both need zizmor 1.32.0 and
actionlint 1.7.12 on the `PATH` (`toolbox doctor` lists them), or their rules are `unknown`.

## Repositories

| Repository | Language | Entities scored | Run time |
|---|---|---|---|
| OpenFGA | Go | 66 | 0.4 s |
| Codex | Rust and TypeScript, pnpm | 86 | 11.9 s |
| gh-aw | Go, 518 workflow files (most generated `.lock.yml`) | 965 | 11.8 s |
| PostHog | Python and TypeScript, pnpm | 271 | 36.4 s |

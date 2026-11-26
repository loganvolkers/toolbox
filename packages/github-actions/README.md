# github-actions: a Seikika package

GitHub Actions security, reliability and supply-chain standards (docs/design/github-actions-standards.md,
D70), packaged as data (D89). It is the first package extracted from Seikika for Seikika, which includes
it like any consumer.

| Brings | What |
|---|---|
| Connectors | `ci-lint` (zizmor and actionlint through the runner), `github-actions` (run history) |
| Tools | zizmor 1.32.0, actionlint 1.7.12 (`seikika doctor --mise` prints the mise config) |
| Kinds | `Workflow`, `ActionDependency` |
| Scorecards | `ci-security`, `ci-reliability`, `ci-speed`, `action-deps`, each after the `prefix` parameter (Seikika for Seikika passes `maintainer-`) |

It needs a `workspace` connector with `workflows` globs.

**Box or tool:** a kit carrying two existing tools
([seikika-and-tools.md](../../docs/design/seikika-and-tools.md)). zizmor fits the box's SARIF slot.
actionlint has no SARIF output, so it comes in through a fitting (`parse: actionlint`). The box
passes neither tool a setting the tool can't read itself. Each record's `command` field reruns
that file's check with the tool alone.

**From 0.3.0, scorecard names carry no prefix by default** (`ci-security`, not
`maintainer-ci-security`), and `owner` sets who owns them. Pass `prefix: maintainer-` to keep
0.2.0's names, so ratchet baselines and links still match.

**From 0.2.0, actionlint's ignores live in your repository's `.github/actionlint.yaml`**, not in
the package. [actionlint.yaml](./actionlint.yaml) is a starting point: unknown permission scopes
and `concurrency` keys that GitHub shipped before actionlint knew them, and your runner labels.
Without it, 0.1.0's three ignored findings can appear as new failures on upgrade.

```yaml
include:
  - package: github:<owner>/seikika/packages/github-actions@<version>   # or a path in your repo
    with: {githubOrg: acme, githubRepo: payments, failRateBronze: 15}   # thresholds may only tighten
```

Then run `seikika install`, which pins the package in `seikika.lock`, and `seikika doctor`.

Parameters (see `seikika-package.yaml`):
- `githubOrg` and `githubRepo` are required;
- `branch` defaults to `main`;
- `failRateBronze`, `failRateSilver`, `failRateGold` and `p90Seconds` may only be lowered
  (integers with `tighten: lower`), and `minDecidedRuns` only raised (`tighten: higher`). Boolean parameters are also supported — see
  `requireHoisted` in [`packages/pnpm-expo`](../pnpm-expo/).

Authoring a further package: [package-author-sdlc.md](../../docs/design/package-author-sdlc.md) §6
(CEL: no `join()`, use `.map`; consumer pin = git tag that contains the package).

Test the package with `seikika test packages/github-actions/tests`.

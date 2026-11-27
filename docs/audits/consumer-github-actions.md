# Consumer GitHub Actions

Use separate actions to set up Seikika, enforce a baseline, and build a static portal in the
operator SDLC. Keep a single-step integration for existing pilot workflows.

## Comparison

I inspected public action metadata and upstream usage examples. I compared names, installation,
which step can fail the job, which step publishes or needs credentials, and what later steps
can consume. The upstream comparison is a source review. The executable checks use Seikika.

| Project | Public action surface and defaults | Implication for Seikika |
|---|---|---|
| HashiCorp Terraform | `setup-terraform` installs a selected CLI version and makes it available to later shell steps. An optional wrapper, enabled by default, exposes stdout, stderr, and exit code. The workflow calls `validate`, `plan`, and `apply` itself. [Setup metadata](https://github.com/hashicorp/setup-terraform/blob/main/action.yml) · [Usage](https://github.com/hashicorp/setup-terraform#usage) | One setup step gives access to operator commands without one action per command. |
| SonarQube | `sonarqube-scan-action` submits analysis. `sonarqube-quality-gate-action` is an additional step that reads scanner metadata, waits for the result, exposes a status, and can fail the workflow. [Scan](https://github.com/SonarSource/sonarqube-scan-action) · [Gate metadata](https://github.com/SonarSource/sonarqube-quality-gate-action/blob/master/action.yml) | Make enforcement easy to discover and explicitly selectable. Keep a structured verdict for callers. |
| Trivy | `trivy-action` offers bundled setup with `skip-setup-trivy` for callers who install once. Inputs select what gets scanned, the output format, and the exit code for findings. A separate upload step publishes SARIF, including when the scan fails if the workflow requests it. [Action metadata](https://github.com/aquasecurity/trivy-action/blob/master/action.yaml) · [Upload recipe](https://github.com/aquasecurity/trivy-action#using-trivy-to-scan-your-git-repository) | Keep the existing bundled path for easy adoption. Expose local files before applying a gate. Leave artifact retention up to the caller in the granular steps. |
| golangci-lint | `golangci-lint-action` uses `only-new-issues` to limit findings to a change. The action gets a PR patch or the appropriate Git diff based on the event. The input defaults to `false`. [Metadata](https://github.com/golangci/golangci-lint-action/blob/main/action.yml) · [Diff behavior](https://github.com/golangci/golangci-lint-action#how-to-use) | Keep variants of one policy as inputs. Seikika's `ci` and `check` both compare the same saved scorecard baseline. |
| Docker Buildx | `setup-buildx-action` creates a builder, `login-action` handles registry authentication, and `build-push-action` builds the image. `push` defaults to false, so pushing takes an explicit choice. [Setup](https://github.com/docker/setup-buildx-action) · [Build and push inputs](https://github.com/docker/build-push-action#inputs) | Expose the build artifact independently of the credential-bearing operation. Keep output format as an input. |
| GitHub CodeQL | One repository ships `init`, `analyze`, and `upload-sarif` as separately addressable paths on a shared release ref. Analysis can control upload, and the standalone uploader can take SARIF from other tools. [Action family](https://github.com/github/codeql-action) · [Analyze metadata](https://github.com/github/codeql-action/blob/main/analyze/action.yml) · [Upload metadata](https://github.com/github/codeql-action/blob/main/upload-sarif/action.yml) | Use `owner/repo/actions/<operation>@ref` so Seikika can ship a family on a single tag. |
| GitHub Pages | `configure-pages`, `upload-pages-artifact`, and `deploy-pages` have separate contracts. The deploy step consumes an artifact and can run in a separate job with `pages: write` and `id-token: write`. [Configure](https://github.com/actions/configure-pages) · [Upload](https://github.com/actions/upload-pages-artifact) · [Deploy](https://github.com/actions/deploy-pages#security-considerations) | Return a host-ready directory and use GitHub's existing upload and deploy steps. |
| Betterer | The comparable ratchet is exposed through the CLI. The results file records the accepted baseline, and `betterer ci` gives CI a distinct enforcement command. [Repository and results example](https://github.com/phenomnomnominal/betterer) · [CI implementation](https://github.com/phenomnomnominal/betterer/blob/master/packages/cli/src/ci.ts) | Keep the CLI as the authority for enforcement and baseline changes across CI systems. |

The compared projects use recognizable operation names. GitHub supplies the repository and
ref in `uses:`, so `setup`, `ratchet`, and `build-portal` identify the operation without a
repeated project name or `-action` suffix. `build-portal` also distinguishes a static artifact
from a running service. Prefer composite actions here because callers need steps inside their
own jobs. A reusable workflow fits an opinionated sequence of jobs with runners and permissions
of its own. [GitHub's reuse comparison](https://docs.github.com/en/actions/concepts/workflows-and-actions/reusing-workflow-configurations)

## Gaps and coverage

| Operator need | Before | Result |
|---|---|---|
| Install once, then validate or test | The report action installed a CLI as an implementation detail. Operators had to script installation to use other commands. | `setup` sets PATH and returns `bin-dir`. The CLI handles validate, test, and any workflow-specific commands. |
| Block regressions without distributing a report | `report` defaulted to strict CI and always uploaded artifacts, even when HTML was disabled. | `ratchet` returns a verdict and Markdown in the current job. Callers decide whether to retain either file. |
| Build a portal without a baseline | `mode: report` scored without a baseline. It also emitted a Markdown report and uploaded an artifact. The Pages recipe copied the HTML into an `index.html` staging directory. | `build-portal` returns an `index.html` directory directly, with optional web files for Astro. |
| Publish with restricted credentials | The old Pages recipe already separated the deployment job. | The build keeps that boundary and consumes read-only checkout permissions. The caller chooses the publication destination and grants its write permissions. |
| Create or accept a baseline change | `report` in update mode wrote a candidate baseline. | Continue to update through the CLI or the one-step action. The independent gate reads the baseline, so its PR job cannot silently approve changes. |
| Operate a hosted portal | A running service uses a separate deployment and authentication model. | Use `setup` for applicable CLI commands and keep deployment in the chosen platform's workflow. The static build serves an offline snapshot. |

The platform operations stay in the CLI. The actions supply GitHub-specific setup, outputs,
and annotations. Use a release introducing the new paths, pin all action paths to the same
commit, and choose the CLI version that matches the manifest.

## Verification

The consumer pilot exercises the same composite metadata callers use. It creates a baseline
with the bundled action and runs validation and fixture tests after setup. With the baseline
removed, the HTML build succeeds and the gate fails. It restores the baseline, passes a clean
ratchet, and adds a CODEOWNERS regression. The gate rejects it, the web build succeeds, and the
baseline checksum stays the same. The job is part of merge-readiness.
New actions are discoverable through the release registry and the existing workflow inventory.
The maintainer ratchet also checks the workflow timeout rule and the new documentation.

[Consumer pilot](../../.github/workflows/ci.yml) · [Action references](../../site/src/content/docs/reference/contracts.md#github-actions) · [Decision](../decisions/D01M8DGVXNG05GF6W3996FEFT6C.md)

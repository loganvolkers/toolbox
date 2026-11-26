---
title: Quickstart
description: Score a practice repository, record a baseline, and see the check catch a regression.
sidebar:
  order: 1
---

Score a practice repository and watch Seikika catch a broken Markdown link. You need Git, Bash, curl, the Seikika CLI, and the workspace connector. If you build from source, keep the terminal where you added the binaries to `PATH`. If you install a release with mise, set it up in the practice repository in step 2. [Install Seikika](/start/install/)

## 1. Create a practice repository

From a directory where you can create a new folder, run these commands. The manifest defines the checks that Seikika runs.

```bash
mkdir seikika-practice
cd seikika-practice
git init -b main
mkdir -p .seikika/tests .seikika/fixtures

starter=https://raw.githubusercontent.com/seikika/seikika/c14c9fbb60405f9fea12d86de3d0b32b717755d9/examples/repo-portal-starter
curl -fsSL "$starter/seikika.yaml" -o .seikika/seikika.yaml
curl -fsSL "$starter/tests/standards.test.yaml" -o .seikika/tests/standards.test.yaml
curl -fsSL "$starter/fixtures/workspace.ndjson" -o .seikika/fixtures/workspace.ndjson

cat > README.md <<'README'
# Practice repository

This repository is a small example for learning how Seikika checks Markdown links and repository
standards. The readme describes the project, the owner in CODEOWNERS reviews changes, and the
license explains the terms for reuse. The [portal notes](portal.md) contain a short description
of what the example repository is for.
README
printf '# Portal notes\n' > portal.md
printf '* @example-maintainer\n' > CODEOWNERS
printf 'Practice file for the Seikika tutorial.\n' > LICENSE
```

The demo ownership and license files are for this exercise. Use your own team or GitHub handle and your project's license when you [adopt the starter](/operators/ci/).

## 2. Check the setup and record a baseline

If you are using mise to install the release, follow the repo portal setup from `seikika-practice`.
Activate mise in your shell to run the commands as shown. You can also prefix each Seikika
command with `mise exec --`, as in `mise exec -- seikika doctor .seikika`.
[Repo portal setup](/start/install/#install-for-a-repo-portal) · [Mise shell setup](https://mise.jdx.dev/shell-setup.html)

From `seikika-practice`, run:

```bash
seikika doctor .seikika
seikika validate .seikika
seikika report .seikika --baseline .seikika/results.json --update
```

`doctor` checks the connector executable and any tools the portal declares. `validate` checks the manifest. The final command creates `.seikika/results.json`. This file is your **baseline**, a record of the scores and findings you compare against later.

## 3. Break a link and catch the regression

Move the file that `README.md` links to and rerun the check.

```bash
mv portal.md moved-portal.md
seikika report .seikika --baseline .seikika/results.json --ci
```

The command exits with status 1. Look for the `docs` scorecard and the `links-resolve` rule for `README.md`. The report names `portal.md` in the evidence.

## 4. Fix the link and verify the result

```bash
mv moved-portal.md portal.md
seikika report .seikika --baseline .seikika/results.json --ci
```

The check now exits with status 0. You restored the link and kept the original baseline. Seikika calls this comparison a **ratchet** because the baseline can record improvements and catch regressions.

## Next steps

[Gate pull requests](/operators/ci/) to adopt the same checks in a real repository, or [explore a running catalog](/start/portal/). If a command fails before the link exercise, see [troubleshooting](/start/troubleshooting/).

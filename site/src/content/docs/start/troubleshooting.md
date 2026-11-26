---
title: Troubleshoot Seikika
description: Diagnose install, baseline, portal startup, authentication, and connector failures.
sidebar:
  order: 7
---

Find the symptom you see, run the check, and confirm that the original command works after the fix.

## Installation and scoring

| Symptom | Check | Recover |
|---|---|---|
| Installer URL returns 404, or `seikika` is missing after installing | Check the release's assets and your shell's `PATH`. The 0.1 release assets use `toolbox`. | Choose a release with Seikika assets or [build from source](/start/install/#build-from-source). |
| `doctor` reports a missing connector or tool | Run `seikika doctor .seikika` from the repository root. | Install the named connector or the pinned tool version and rerun `doctor`. |
| `validate` reports a missing test file | Compare the manifest's `tests` entries with the files in the portal directory. | Copy the starter's `tests/` and `fixtures/` alongside the manifest. |
| A package cannot load on a clean runner | Check that `seikika.lock` is committed and available in the portal directory. | Run `seikika install --frozen` with Git access before reporting. |
| CI reports a missing or stale baseline | Check the job's portal directory and `results.json` path. | Follow [Gate pull requests](/operators/ci/) to create the baseline or record a reviewed improvement. |
| The result shows a new failure | Read the reported scorecard, evidence, and fix. | Correct the underlying issue and rerun the same command. |

## Running a portal

| Symptom | Check | Recover |
|---|---|---|
| Server reports an address already in use | Check which port you supplied with `--listen` or `PORT`. | Select an unused loopback port for the local tutorial and update the client URL. |
| Server needs `openfga` | Run the OpenFGA binary with `version` and check `SEIKIKA_OPENFGA_BIN`. | Install the server binary and set its absolute path. Keep authorization enabled for permission examples and shared deployments. |
| `/healthz` responds but `/readyz` returns 503 | Read the readiness problem code and server logs. | Check the database connection, initial connector runs, or federation child access. Retry readiness once the first snapshot is available. |
| A read returns 401 | Check the host, authentication mode, and credential. | In the loopback tutorial use `--as`. For a shared portal use a token or the identity provider your operator configured. |
| A field or scorecard rule shows a redaction marker | Check the required relation in the response. Ask the operator to confirm your identity and access. | Handle the hidden value in your client. Request access through the portal's usual process if you need the details. |
| `/readyz` reports unconfined connectors | Check the host's Landlock support and `SEIKIKA_CONNECTOR_CONFINEMENT`. | Use a supported host and `required` confinement when your deployment depends on that boundary. |

## Get help

Report reproducible problems in [GitHub Issues](https://github.com/seikika/seikika/issues). Include the version or source revision, platform, exact command, and a small sanitized example. Follow the [security policy](https://github.com/seikika/seikika/security/policy) for vulnerabilities.

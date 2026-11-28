---
title: GitHub connector
description: Import GitHub repositories, code owners, app installations, and workflow run statistics.
sidebar:
  order: 2
---

Use the GitHub connector to bring repository metadata and ownership into your portal. With the right credentials, you can also see which apps are installed and measure recent GitHub Actions runs. To import catalog descriptors such as `catalog-info.yaml`, use the [Git files connector](/connectors/git-files/).

## Connect an organization

You need the Seikika CLI and `seikika-connector-github` from the same 0.2 source revision, an organization name, and a credential with access to the repositories you want to read. [Install a connector](/start/install/#build-from-source)

The connector reads branch protection and direct administrators. For the collaborators endpoint, a user token needs an organization member with Write, Maintain, or Admin access to the repository. A fine-grained token needs repository Metadata read permission. Code owners also require repository Contents read permission. [GitHub collaborator permissions](https://docs.github.com/en/rest/collaborators/collaborators#list-repository-collaborators) · [GitHub contents permissions](https://docs.github.com/en/rest/repos/contents#get-repository-content)

Add this entry to the `connectors` list in your portal's `seikika.yaml`. Replace `acme` with your organization and provide `GITHUB_TOKEN` through the environment where you run Seikika.

```yaml
connectors:
  - name: github
    package: builtin:github
    runtime: in-image
    config:
      org: acme
      token: ${env:GITHUB_TOKEN}
    records: [repository, codeowners]
```

GitHub.com uses `https://api.github.com` by default. For GitHub Enterprise Server, set `apiUrl` to your instance's REST API root, such as `https://github.example.com/api/v3`. Set `apiVersion` if your API requires a version other than the default `2022-11-28`.

As an alternative to a supplied token, configure a GitHub App installation with `appId`, `installationId`, and `privateKey`. Set the first two as strings and supply the key through a [secret reference](/operators/secrets/). Remove the `token` entry and leave `GITHUB_TOKEN` unset in that environment so the connector uses the installation. The connector exchanges the app credentials for an installation access token. [GitHub App installation authentication](https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/authenticating-as-a-github-app-installation)

## Check the records

From the portal directory, run these commands. Keep the connector binary on your `PATH` or set `SEIKIKA_CONNECTORS_DIR` to the directory that contains it.

```bash
seikika validate .
seikika connector records --portal . --name github
```

Look for a `repository` record with a known repository name. For a repository with a `CODEOWNERS` file, also look for `codeowners` and check one rule. The command prints newline-delimited JSON. In a running portal, map the fields to entities before looking them up in the catalog. [Catalog mappings with sample records](https://github.com/seikika/seikika/tree/main/examples/reference-portal)

| Record kind | Data it contains |
|---|---|
| `repository` | Repository ID, name, archive status, default branch protection, and direct administrators |
| `codeowners` | Repository name, file patterns, and their owners |
| `installation` | Installation ID, app slug, repository selection, and visible repositories |
| `app` | App slug, owner, and permissions |
| `workflow-runs` | Failure rate, reruns, consecutive failures, and run times per workflow and branch |

Add `installation` and `app` to `records` when you want app data. Supply a GitHub App user access token for an organization owner. Listing organization installations requires read access to the organization's Administration permission. The connector also reads which repositories are accessible to the authenticated user for each installation. Check both endpoints with the token you plan to use. [Organization installations](https://docs.github.com/en/rest/orgs/orgs#list-app-installations-for-an-organization) · [Repositories accessible to a user access token](https://docs.github.com/en/rest/apps/installations#list-repositories-accessible-to-the-user-access-token)

To add run statistics, add `workflow-runs` to `records` and set `config.actions.repos` to a list of repository names within the organization. Set `branch` and `runs` inside `actions` to change the defaults of `main` and 50 recent runs per workflow. The connector accepts a window of 1 to 100 runs. Give the credential read access to Actions. [GitHub workflow run permissions](https://docs.github.com/en/rest/actions/workflow-runs#list-workflow-runs-for-a-workflow)

## Troubleshoot a read

- For a `401` or `403`, check that the service can read the credential, that GitHub accepts it, and that it can reach every endpoint needed for the requested kinds.
- If a private repository is missing, check its visibility to the token or the GitHub App installation's repository selection.
- If a repository has no code owners record, check the default branch for `.github/CODEOWNERS`, `CODEOWNERS`, or `docs/CODEOWNERS`. The connector uses the first file it finds in that order.

The connector supports full reads and refresh by external ID. It also supports selector refresh for installations. To check the protocol without GitHub credentials, run the mock GitHub acceptance test. The test declares all kinds advertised by the connector. [Acceptance test](https://github.com/seikika/seikika/blob/main/scripts/m4-acceptance.sh)

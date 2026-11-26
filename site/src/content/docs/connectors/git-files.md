---
title: Git files connector
description: Import Backstage catalog YAML or check for selected files in repositories.
sidebar:
  order: 4
---

Use Git files to read catalog descriptors such as `catalog-info.yaml`. It can also record whether selected paths exist. Choose a local directory for a dry run or GitHub for live reads. To read repository settings and code owners, use the [GitHub connector](/connectors/github/).

## Read a local catalog file

You need the Seikika CLI and `seikika-connector-git-files` from the same 0.2 source revision. From the directory that contains your portal's `seikika.yaml`, create a sample catalog file inside a disposable checkout. Use the minimum manifest if you are starting with an empty directory. [Install a connector](/start/install/#build-from-source) · [Minimum manifest](/operators/manifest/#minimum-manifest)

```bash
mkdir -p catalog/sample-service
cat > catalog/sample-service/catalog-info.yaml <<'YAML'
apiVersion: backstage.io/v1alpha1
kind: Component
metadata:
  name: sample-service
spec:
  type: service
  lifecycle: experimental
  owner: platform
YAML
```

Add this entry to the `connectors` list. The connector looks for each path inside each repository directory under `root`.

```yaml
connectors:
  - name: git-files
    package: builtin:git-files
    runtime: in-image
    config:
      org: acme
      root: ./catalog
      paths: [catalog-info.yaml]
    records: [catalog-info]
```

From the same directory, read the records. A `catalog-info` record with the external ID `git:acme/sample-service/catalog-info.yaml` should contain the file's parsed YAML under `data.content`.

```bash
seikika connector records --portal . --name git-files
```

## Read GitHub files or split descriptors

To read from GitHub, remove `root`, set `org` to your organization, and set `token: ${env:GITHUB_TOKEN}` under `config`. Use a token that can list the repositories and read their contents. Set `apiUrl` to the instance's REST API root for GitHub Enterprise Server. [GitHub contents permissions](https://docs.github.com/en/rest/repos/contents#get-repository-content)

Set `splitKinds: true` to emit one record for each supported Backstage document in a YAML file. Declare `component`, `api`, `system`, `domain`, and `resource` in `records`. Set `files` to a list such as `[README.md]` in split mode and also declare `file` to get a presence record for each path the connector finds. A `file` record contains the repository, path, and `present: true`. [Manifest with record mappings](https://github.com/seikika/seikika/blob/main/examples/acme-live/seikika.yaml)

The connector can read a full snapshot and refresh records by external ID. In split mode, each descriptor ID ends in `#<kind>:<name>`, such as `#component:sample-service`.

## Troubleshoot missing records

- For a local read, check that each descriptor is under `<root>/<repo>/<path>` and the connector can read it. Use an absolute `root` when a deployment starts the connector from a different working directory.
- For a GitHub read, check the credential's repository access and look for the exact path on the default branch.
- If a split record is missing, check the document's `kind` and `metadata.name`. Check that the portal declares the record kind and maps it to an entity.

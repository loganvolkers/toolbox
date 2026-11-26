---
title: GitLab connector
description: Import GitLab groups, direct members, and projects from GitLab.com or a self-managed instance.
sidebar:
  order: 3
---

Use the GitLab connector to discover projects and model team membership from a GitLab group. A full read includes the group, its subgroups, direct memberships, and projects that are not archived.

## Connect a group

You need the Seikika CLI and `seikika-connector-gitlab` from the same 0.2 source revision. Use a GitLab token with the `read_api` scope and access to the group and its projects. [Install a connector](/start/install/#build-from-source) · [GitLab token scopes](https://docs.gitlab.com/user/profile/personal_access_tokens/#personal-access-token-scopes)

Add this entry to the `connectors` list in your portal's `seikika.yaml`. Replace `acme` with the full group path, such as `acme/platform`. Make `GITLAB_TOKEN` available in the environment where you run Seikika.

```yaml
connectors:
  - name: gitlab
    package: builtin:gitlab
    runtime: in-image
    config:
      group: acme
      token: ${env:GITLAB_TOKEN}
    records: [group, project]
```

The connector defaults to GitLab.com. For a self-managed instance, set `config.url` to its base URL. The API root defaults to that URL plus `/api/v4`. Override it with `config.apiUrl` when needed.

## Check the records

From the portal directory, run these commands with the connector on your `PATH`.

```bash
seikika validate .
seikika connector records --portal . --name gitlab
```

Look for a `group` record with an ID such as `gitlab:group/acme`. Check the `members` list for a direct member you expect to see. Look for a known project by its path in a `project` record.

| Record kind | Data it contains |
|---|---|
| `group` | Full group path, name, parent path for a subgroup, direct member usernames, and access levels |
| `project` | Project path, name, default branch, archive status, and web URL |

The connector includes direct members at Developer access or higher by default. Set `config.minAccess` to a different GitLab numeric access level if your portal needs it. To represent inherited membership, map the parent relationship in your portal. The connector can resync both kinds and refresh by external ID. [GitLab group member API](https://docs.gitlab.com/api/group_members/) · [Group mapping in a working portal](https://github.com/seikika/seikika/blob/main/examples/federated-actions/gitlab-org/seikika.yaml)

## Troubleshoot a read

- If the group lookup fails, verify the full group path, the instance URL, and the token's access to the group.
- If a user is missing from a subgroup record, check their direct membership and their access level. For inherited members, check your parent mapping.
- If the records appear but catalog entities are missing, check the `kinds` mappings for the `gitlab/group` and `gitlab/project` sources you intend to use.

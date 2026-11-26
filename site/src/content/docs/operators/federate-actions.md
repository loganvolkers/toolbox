---
title: Federate repository actions
description: Offer actions from several repositories through one parent portal and verify an approved run.
sidebar:
  order: 9
---

Use **action federation** when teams own their actions in Git and you want to offer those actions from one portal. The **parent portal** lists each **child repository**, shows its actions, and records approvals. The child keeps its action definition and decides which parent's signatures to trust.

This tutorial runs one approved action against a disposable local Git repository. You need Bash, Git, `jq`, and the Seikika CLI from the [0.2 source install](/start/install/#build-from-source). Run the commands from the root of that source checkout. The tutorial uses a local identity for each role.

## 1. Set up the parent and child

Create a local Git origin and a checkout named `service`. Copy the example action into the checkout and push it to the local origin.

```bash
practice_dir=$(mktemp -d)
git init --bare -b main "$practice_dir/service.git"
git init -b main "$practice_dir/service"
cp -R examples/federated-actions/minimal-repo/. "$practice_dir/service/"
git -C "$practice_dir/service" add .seikika
git -C "$practice_dir/service" -c user.name='Seikika example' \
  -c user.email='example@example.com' commit -m 'Add action example'
git -C "$practice_dir/service" remote add origin "$practice_dir/service.git"
git -C "$practice_dir/service" push origin main
```

Create the parent manifest. The child key `service` gives the action its target ID, `portal:service`.

```bash
mkdir "$practice_dir/org"
cat > "$practice_dir/org/seikika.yaml" <<'MANIFEST'
apiVersion: seikika.com/v1alpha1
portal: {name: org, owner: 'group:platform', seikika: "0.2"}
federation:
  children:
    - {git: ../service.git, branch: main, key: service}
MANIFEST

seikika validate "$practice_dir/org"
run_org() {
  SEIKIKA_STATE_DIR="$practice_dir/state" seikika run --portal "$practice_dir/org" "$@"
}
run_org -o json | jq '{actions: [.actions[].name], approval_key}'
```

Look for `add-code-of-conduct` and an `approval_key`. The key is public. Seikika keeps the corresponding signing key in the practice state directory.

## 2. Pin the parent's approval key

The copied example contains a placeholder key. Replace it with the public key, then commit it to the local origin. The parent reads the child from that origin.

```bash
approval_key=$(run_org -o json | jq -er '.approval_key')
action_file="$practice_dir/service/.seikika/actions/add-code-of-conduct.yaml"
sed "s/ed25519:0\{64\}/$approval_key/" "$action_file" > "$action_file.tmp"
mv "$action_file.tmp" "$action_file"
seikika validate "$practice_dir/service/.seikika"
git -C "$practice_dir/service" -c user.name='Seikika example' \
  -c user.email='example@example.com' commit -am 'Trust the practice parent'
git -C "$practice_dir/service" push origin main
```

## 3. Preview, approve, and verify

Preview the action as Ann. Check that the preview adds `CODE_OF_CONDUCT.md` before taking the next step.

```bash
run_org add-code-of-conduct --entity portal:service \
  -i contact=conduct@example.com --as ann -o json > "$practice_dir/preview.json"
jq '{effects, blockers, approval}' "$practice_dir/preview.json"
```

Use the confirm token from the preview with the same inputs. The request exits with status 3 and returns `awaiting_approval`. The file still has to land on the origin.

```bash
confirm_token=$(jq -er '.confirm_token' "$practice_dir/preview.json")
run_org add-code-of-conduct --entity portal:service -i contact=conduct@example.com \
  --as ann --confirm "$confirm_token" -o json > "$practice_dir/request.json"
```

Use the returned run ID to approve as Bob. Seikika refuses a requester who tries to approve their own run.

```bash
run_id=$(jq -er 'select(.state == "awaiting_approval") | .id' "$practice_dir/request.json")
run_org --approve "$run_id" --as bob --wait
run_org --verify "$run_id" --as ann -o json \
  | jq '{state, approved_by: .approval.approved_by, verification}'
git --git-dir="$practice_dir/service.git" show main:CODE_OF_CONDUCT.md
```

Look for `succeeded`, Bob's approval, and the example contact address on the origin. You can delete the temporary directory after reviewing the run and its files.

## Configure a shared parent

1. Add each child under `federation.children`. Set its `git` URL and the branch to read, which defaults to `main`. Use `key` to give it a stable target ID. Without `key`, the repository name supplies the ID.
2. [Deploy the parent with authentication](/operators/deploy/). Keep its signing key and state private and persistent. People with access to development identities or the checkout's signing state can act as approvers.
3. Obtain the public approval key from the served parent and pin it in each child's `federation.acceptApprovalsFrom` as `{portal: PARENT_NAME, key: PUBLIC_KEY}`. Commit the change to the child's configured branch. Use the served parent's URL to request and check its runs.
4. For execution in the parent's clone, set the child's `federation.runner` to `{type: local}`. Limit the parent to trusted children with `federation.runners.local.portals`. Grant its Git credentials access to the repositories and branches the actions need.
5. Give requesters and reviewers the [action workflow](/users/actions/). Use [the forge examples](https://github.com/seikika/seikika/tree/main/examples/federated-actions) for GitHub, GitLab, mixed hosts, and execution through a child's CI system.

Use a parent you trust to sign approvals and select source revisions. Review changes to action definitions and permissions as you would changes to CI workflows.

## Diagnose a federation problem

| Symptom | Check |
|---|---|
| The action is missing | Check the parent list's `unreadable` entries, Git access, and the branch that contains the child's `.seikika/seikika.yaml`. |
| `child-refuses-approvals` | Compare the child pin with the public key from this parent. Check the parent name and confirm the pin is on the branch the parent reads. |
| `no-runner` or `runner-not-allowed` | Check the child's runner and the parent's runner policy. |
| A push is rejected | Check the run record, credentials, and branch protection. Use a review branch if the destination requires a merge. |

Current federation covers action discovery, approvals, execution, and verification. Catalog, score, and report roll-up across children remain proposals. See [what Seikika supports](/concepts/capabilities/).

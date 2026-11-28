---
title: Change a portal
description: Validate the manifest, run fixture tests, and compare the change with its base revision.
sidebar:
  order: 2
---

Before you merge a portal change, check the manifest, run its tests, and see whether the change moves any scores or permissions.

## 1. Validate and test the candidate

From your portal repository, set `portal_dir` to the relative path to the directory that contains `seikika.yaml`. Replace `portal` in this example with that directory or use `.` if the manifest is at the root.

```bash
portal_dir=portal
seikika validate "$portal_dir"
seikika test "$portal_dir"
```

Look for zero validation errors and passing expectations. If validation fails, fix the file and location in the diagnostic. If an expectation fails, check the fixture and the rule against the behavior you intended.

## 2. Compare against the base revision

Use a worktree to preserve any includes and related files from the base revision. In this example, the base is the fetched `origin/main`. Choose the base of your pull request if it differs.

```bash
base_dir=$(mktemp -d)
git worktree add --detach "$base_dir" origin/main
seikika plan "$portal_dir" --local --base "$base_dir/$portal_dir/seikika.yaml"
git worktree remove "$base_dir"
```

Read the entity, score, and access changes. The local plan compares both manifests against the candidate's fixtures. Include examples of entities that gain or lose access when reviewing an authorization change. Plan against live data remains planned.

See the [manifest reference](/operators/manifest/) for configuration fields and the [deployment checks](/operators/deploy/#verify-the-rollout) for what to verify after rollout.

---
title: Scorecards
description: Understand how rules become levels and how a baseline catches regressions.
---

A scorecard groups rules for one kind of catalog entity, such as a repository or a document. If a repository passes every bronze rule and fails one silver rule, its level is bronze. A level requires every rule at that level and below to pass.

Each rule declares a check and can explain why the rule matters, what evidence to report, and what to fix. For example, the starter's `links-resolve` rule checks whether a Markdown file links to files that exist. If it fails, the report lists the missing paths.

| Result | Meaning |
|---|---|
| Pass | The available facts satisfy the rule. |
| Fail | The facts show a problem. Use the evidence to find it. |
| Unknown | The check could not finish, for example because a tool did not run. Treat the result as incomplete. |
| Skipped | The scorecard does not apply to this entity under its conditions. |

## The ratchet

A **baseline** is a saved results file. The **ratchet** compares a new report against that file. It catches newly failing rules, growing evidence or numeric measures, a drop in level, and new entities below the scorecard's minimum level.

In CI, use `--ci` to catch regressions and a stale or missing baseline. For a reviewed improvement, use `--update` and commit the new baseline. A rule can be unknown because you explicitly skipped its connector. The ratchet ignores those skipped inputs when checking regressions. An unknown result from a connector that ran can fail `--ci` or `--update`. See [Gate pull requests](/operators/ci/) and the [CLI reference](/start/cli/) for command options.

## This site

The `maintainer-site` scorecard checks each page in the [repository report](/demo/report/).

| Level | What the checks require |
|---|---|
| Bronze | A title, description, working local link targets, and no credential-shaped strings |
| Silver | A visible page in the sidebar |
| Gold | A title distinct from every other page |

The [documentation style guide](/maintainers/writing/) describes the checks authors do by hand, such as running examples and reviewing prose.

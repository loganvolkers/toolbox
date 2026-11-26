---
title: Documentation style guide
description: Write clear, accessible Seikika docs with examples readers can run and verify.
sidebar:
  order: 2
---

Write for a capable developer who is new to Seikika. Give them a result they can recognize, explain the terms they need, and show them how to check their work.

## Audience and reading level

- Assume readers of **Start** know Git, files, and a terminal. Introduce continuous integration (CI), the portal manifest, and scorecards before relying on those concepts. Assume no Rust, Kubernetes, or platform engineering experience unless the page needs it.
- Name the primary reader when the task requires a particular role. Operators run portals, package authors publish reusable standards and connectors, users read catalogs and request actions, and maintainers change Seikika.
- Aim for US grade **8–10** in introductory prose and grade **10–12** in operator and reference explanations. Use these ranges to flag difficult passages. Review short sentences and unfamiliar terms by hand. Exclude commands, code, API identifiers, and tables from automated grade estimates. Prerequisites describe technical skills, regardless of the reader's formal education.
- Define new terms in place and use the [glossary](/concepts/glossary/) for a deeper definition. Spell out an abbreviation on first use when its meaning matters. Keep exact names such as `min_snapshot` and `seikika.lock`.

## Voice and tone

| Context | How to write | Example |
|---|---|---|
| Tutorial | Warm, direct, and encouraging. Explain why the step matters. | Record a baseline so the next run can tell you what changed. |
| Procedure | Start with an action verb. Say where the reader works and what they need. | From your repository root, run `seikika doctor .seikika`. |
| Reference | Use present tense and name the behavior, inputs, and limits. | `--ci` exits with an error if the baseline is stale. |
| Troubleshooting | Stay calm. Name the symptom, give a check, and explain how to recover. | If the port is in use, set `--listen` to an unused loopback port. |
| Security and status | State the condition and its consequence. | In development mode, anyone with access to the port can claim any identity. |

Use active voice and address the reader as “you.” Name the program or person that takes each action. Prefer familiar words, one idea per sentence, short paragraphs, and US English. Use sentence case for headings. Avoid idioms, filler such as “simply” and “obviously,” unsupported superlatives, and claims about how long a task takes.

Use **portal** for a configured catalog, **repo portal** for the local reporting workflow, **manifest** for the portal's configuration, and **baseline** for saved results. Introduce **ratchet** as the comparison that catches regressions. Use “software development lifecycle” before “SDLC” where the acronym helps, and name the relevant lifecycle.

## Structure pages around a reader's question

| Page type | Question | Include |
|---|---|---|
| Tutorial | How do I learn this? | A small worked example, a starting state, checkpoints, a deliberate mistake when useful, and a result. |
| How-to guide | How do I finish this task? | Prerequisites, steps in order, expected results, and recovery for common failures. |
| Reference | What does this field, command, or endpoint do? | Accepted inputs, defaults, output, errors, and links to the schema or implementation. |
| Explanation | How does this work? | A concrete example, the underlying model, and the reason it matters. |

Start with the outcome or answer. Use descriptive link text and add links where readers need a prerequisite or a next step. Keep one canonical procedure per task. Preserve existing routes and heading links when moving content. Use root-relative links such as `/start/quickstart/` within the site.

## Make examples verifiable

1. Name the version or source revision, tools, operating system or shell, working directory, and permissions when they affect the outcome.
2. Use small, copyable, language-tagged code blocks. Separate commands from output. Name the terminals explicitly if a service needs to keep running. Label placeholders and supply safe example values.
3. Show what success looks like and what to do if the most likely step fails. Teach in fixtures or a disposable checkout. Explain data changes and add cleanup steps when needed.
4. Run new or changed examples from the stated starting point. Record the command, version, result, and any limits in the pull request or [research log](https://github.com/seikika/seikika/blob/main/docs/research-log.md).
5. Mark availability using the release, the source revision, or “planned.” Check the release assets and the current command before making availability claims. Link to the release, schema, source, or acceptance test that supports the claim. Use clearly fake secrets and keep real credentials out of commands, logs, screenshots, and commits.

## Write for accessibility

Use heading levels in order, meaningful headings, and links that make sense on their own. Give informative images useful alternative text. Describe the outcome in text when an image, color, or icon shows status. Put long commands in code blocks and keep tables narrow. Check the rendered page at a narrow width and make sure readers can reach links and copy examples with a keyboard.

## Review a change

- Can the intended reader find the page and tell whether it applies to their version?
- Can they follow the route without opening an internal design document or guessing a command?
- Do commands, sample output, and links match the current product?
- Does the page explain unusual terms, likely failures, and what success looks like?
- Did the docs build, link checks, and [site scorecard](/concepts/scorecards/#this-site) pass? Check prose and task completion by hand as well.

These standards draw on role and task organization from Backstage and Diátaxis, worked examples from React, and plain-language and accessibility guidance from Google, Kubernetes, and W3C. [Backstage style guide](https://backstage.io/docs/contribute/doc-style-guide/), [Diátaxis](https://diataxis.fr/), [React's documentation redesign](https://react.dev/blog/2023/03/16/introducing-react-dev), [Google voice and tone](https://developers.google.com/style/tone), [Google procedures](https://developers.google.com/style/procedures), [Kubernetes style guide](https://kubernetes.io/docs/contribute/style/style-guide/), [W3C writing for accessibility](https://www.w3.org/WAI/tips/writing/), [W3C reading level guidance](https://www.w3.org/WAI/WCAG22/Understanding/reading-level.html).

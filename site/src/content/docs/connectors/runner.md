---
title: Runner connector
description: Run tests or linters and turn their output into records a portal can score.
sidebar:
  order: 6
---

Use Runner when your portal needs facts that a tool computes, such as test results or lint findings. Runner executes programs with the connector's access to the checkout. Review job commands and use a trusted manifest. If you only need to read files or repository metadata, use the [Workspace connector](/connectors/workspace/).

## Read the output of a command

You need the Seikika CLI and `seikika-connector-runner` from the same 0.2 source revision. Use Bash and a disposable checkout that contains a portal manifest at its root. [Install a connector](/start/install/#build-from-source)

From the checkout root, create a file with one connector record as newline-delimited JSON (NDJSON). Here it stands in for a tool that writes records to standard output.

```bash
cat > sample-check.ndjson <<'NDJSON'
{"type":"record","kind":"check","externalId":"check:sample","data":{"passed":true}}
NDJSON
```

Add this entry to the `connectors` list in `seikika.yaml`. The job reads the example file with `cat`. Each item in `command` is a separate program argument.

```yaml
connectors:
  - name: runner
    package: builtin:runner
    runtime: in-image
    config:
      root: .
      jobs:
        - name: sample
          command: [cat, sample-check.ndjson]
          parse: ndjson
          kinds: [check]
    records: [check]
```

From the same directory, read the records. Look for kind `check`, external ID `check:sample`, and `"passed":true` in its data.

```bash
seikika connector records --portal . --name runner
```

## Run a test or a linter

Declare the kinds your jobs emit under `records`. For a test or lint parser, the job name usually becomes the record kind. For `ndjson`, the tool supplies the kinds and external IDs. Runner supports a full resync and refresh by external ID. With `per: crate`, a refresh can rerun just the matching Cargo member's job.

For a linter that produces Static Analysis Results Interchange Format (SARIF), use the `sarif` parser. Set `files` to scope the results and set `output` if the tool writes to a file. For Cargo tests or Clippy, choose the corresponding parser. [Runner jobs and parser reference](https://github.com/seikika/seikika/blob/main/docs/reference/runner-connector.md) · [Reusable SARIF package](https://github.com/seikika/seikika/tree/main/packages/sarif)

## Troubleshoot a job

- If the program cannot start, check that it is installed where the connector runs. An `optional: true` job can skip a missing executable and leave rules that depend on it unknown.
- If no lint records appear, check the `files` patterns. If an argument is exactly `{files}` and no paths match, the connector skips the job.
- If parsing fails, run the command from the configured `root` and compare its output with the selected parser's format.

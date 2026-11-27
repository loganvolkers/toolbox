---
title: Assess a catalog entity
description: Get a rubric and evidence, record a human or agent judgment, and check its freshness.
sidebar:
  order: 12
---

Use an **assessment** when a standard needs judgment, such as whether a rule's tests cover the behavior it promises. The portal supplies a **rubric**, a set of review criteria. A person or agent reads the evidence and supplies a verdict for each criterion. Seikika stores the verdicts as data a scorecard can use.

## Browse assessments

Open **Assessments** in the served portal or report. **Recent runs** lists the latest published result for each subject and rubric, with the recorded date and time when available. Open a result to read the criterion verdicts, evidence, assessment quality scorecards and any recorded review.

**Rubrics** lists the questions and guidance for assessing each kind. A rubric can include `links:` with a `title` and `url` for each supporting document.

## Get a brief

For a worked example, use the [0.2 source CLI](/start/install/#build-from-source). From the Seikika source checkout, run this against its own portal. Use the required tools from [Build Seikika](/maintainers/#check-the-repository-standards).

```bash
seikika assess brief rule:seikika/meta-rule/tested --portal portal --skip runner,github-actions
```

Look for the rule's facts, rubric criteria, the `subjectHash`, and the path and template for an assessment file. The subject hash identifies the version you review. If your portal has more than one rubric for the kind, add `--rubric NAME`.

## Write and check the assessment

1. Follow the facts to the source files and review each criterion. Record `pass`, `fail`, or `unsure`. Give a reason that cites the evidence. For a failure or uncertainty, suggest a concrete change or say what evidence you need.
2. Use the template from the brief. Keep its `subjectHash`, record the assessor and date, and write the file to the suggested path in your portal. Use your own judgments if the checkout already has an assessment.
3. From the portal repository, run `seikika assess check PATH_TO_PORTAL`, passing the directory that contains the manifest. Use the same fixture and connector settings you used for the brief.

The check reports `fresh` when the hash matches, `stale` when the subject changed, and `missing` when no assessment exists. For stale results, get a new brief and review the changed evidence. An invalid file fails validation. Add `--require-fresh` when your check should also fail on stale assessments.

For Git-based reviews, commit the assessment file with the change. A served portal can also receive an assessment from a configured remote connector. With its credential in `SEIKIKA_CONNECTOR_TOKEN`, use `seikika assess submit FILE --portal PATH_TO_PORTAL --url PORTAL_URL`. Replace the uppercase values with the reviewed file, local manifest directory, and server URL.

See the [assessment schema](https://github.com/seikika/seikika/blob/main/crates/seikika-model/schema/assessment.schema.json) for required fields and [the CLI reference](/start/cli/) for command options.

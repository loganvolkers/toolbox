# Meta scoring

Use these standards to check a portal’s own configuration. The package derives catalog entities
for the portal’s scorecards, rules, kinds, fields, connectors and rubrics. It reads assessment files
from the checkout and scores the review evidence. The package’s definitions are in scope too.
It configures Seikika’s Runner and Workspace connectors for these checks.

From a repository with its manifest in `.seikika/seikika.yaml`, vendor the package and include it.

```sh
seikika vendor /path/to/seikika/packages/meta-scoring --portal .seikika
```

```yaml
include:
  - package: packages/meta-scoring
    with:
      owner: group:platform
```

Run the CLI from the repository root with `seikika` and its runner and Workspace connectors
available. Use a build that supports `describe portal`.

```sh
seikika validate .seikika
seikika test .seikika
seikika report .seikika -o failures
seikika assess check .seikika
```

The `meta-*` cards check properties a tool can observe, such as runnable fixtures, useful field
descriptions and evidence attached to rule failures. Use `review-rule` and `review-scorecard` to
judge the substance of those standards using cited reviews of the current definitions. The
`review-assessment` card follows independent review evidence for those judgements. Use
`seikika assess brief <id> --portal .seikika` to prepare each review.

Gold on the review cards requires a complete assessment of the current definition and rubric,
with a passing review by a different named assessor citing the assessment or its sources.
Assessor names are declared in the assessment files. An older assessment can still show its
findings after the rubric changes, and one without a rubric hash can reach silver if its criteria
and subject still match.

For layouts other than `.seikika`, set `manifest` to the portal path,
`describeCommand` to the argument vector for `seikika describe portal <manifest>`,
and `assessments` to globs. All paths are relative to `root`. If your
portal has a quality track, set `track` to its name. Add a campaign with a concrete scope and target
for the gaps you intend to close. The [organization portal](../../examples/github-org-federation/meta.yaml)
tracks review coverage, and [Seikika for Seikika](../../portal/campaigns.yaml) tracks open findings.

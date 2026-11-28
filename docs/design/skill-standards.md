# Agent Skill Standards: Cataloguing and Scoring the Skills We Ship

*Status: deterministic catalog and scorecard built, 2026-11-20 (D72). Seikika for Seikika catalogues
every `SKILL.md` in this repository and scores it with `maintainer-skill`. Live agent trials are
designed and not built. Related: [agent-plugins.md](./agent-plugins.md),
[seikika-for-seikika.md](./seikika-for-seikika.md), [scorecards-for-scorecards.md](./scorecards-for-scorecards.md).*

---

## 1. Conclusion

**Catalogue every skill we ship, and gate the checks that do not need a model.**

An agent skill is a directory with a `SKILL.md`. The name and description are loaded for every skill
at startup, and the body is loaded only when the skill is selected. A skill that is invalid, vague,
or huge fails before it can help. A skill that is well formed can still fail to help. Those are
different questions, and the tools that answer them should not be mixed into one CI number.

| Question | What answers it | In this repo |
|---|---|---|
| Will a harness load it, and can a router tell when to use it? | Agent Skills spec + a description that states the condition | `maintainer-skill` bronze |
| Can an agent follow a short, concrete, portable procedure? | Anthropic's authoring checklist, the spec's size guidance | silver |
| Does it state limits, carry eval cases, and hide nothing hostile? | Spec progressive disclosure, Anthropic's "evals first", SkillBench / SkillSpector hard rules that are pure text | gold |
| Does the skill actually raise task success? | Paired with/without-skill runs (SkillsBench, SkillEvaluator Tier 3, skill-creator, SkillPortrait) | **Not gated.** The gold rule only requires the cases to exist |

The workspace connector emits one `skill` record per `SKILL.md`. It reports facts. `.seikika/seikika.yaml`
holds the thresholds. A skill added after this baseline must reach bronze (`ratchet.newEntitiesAtLeast`),
which is the spec plus a routable description. CLI commands are catalogued separately: each leaf has
one skill, and `cli-skill` scores the command (D94, [cli-skills.md](./cli-skills.md)).

**What we did not adopt, and why:**

| Proposal | Why it is not the gate |
|---|---|
| NVIDIA SkillEvaluator's 0–100 quality score | Weighted deductions, not a level ladder. Several checks match substrings (`for`, `use`, `>`, `fail`) and fire on correct skills (their issue #30). The token figure is `characters / 4`, which they now label an estimate (issue #461) |
| SkillEvaluator Tier 3, SkillsBench, skill-creator benchmark mode, SkillPortrait | The right measurement of whether a skill helps: the same tasks with and without it. They need a model and a sandbox, and the numbers move when the model moves. D71 already keeps LLM assessment and agent trials out of the gate |
| SkillSpector's full scanner (71 patterns, optional LLM) | Right tool for skills we did not write. Re-implementing it, or failing our own skills for mentioning a dangerous command in a "never do this" line, would not match what we need. The two hard rules we can check as text — hidden characters and high-precision secret shapes — are in the scorecard |
| SkillBench's provenance weight and letter grade | We are the author. Licence is Q1, so a missing `license` field is not a failure. NVIDIA's required `metadata.author` / `## Purpose` headings are not in the Agent Skills spec |

## 2. What the field measures

Read 2026-11-20. "Primary" means the repository or the spec text. The rest are the project's own
docs or papers, read from the pages cited in [Sources](#sources).

| Source | What it decides | What is deterministic | What needs a run |
|---|---|---|---|
| **Agent Skills spec** + `skills-ref validate` | A skill is a directory. `SKILL.md` has YAML frontmatter. `name` (≤ 64, lowercase, hyphens, matches the directory) and `description` (≤ 1024, what *and when*) are required. Optional: `license`, `compatibility` (≤ 500), `metadata` (string map), `allowed-tools`. Body should stay under 500 lines; instructions under about 5,000 tokens; references one level deep | Frontmatter, name, lengths, directory match | Nothing |
| **Anthropic skill authoring** and the skill-creator update | Write evals before a long body. Description is specific, third person, and says when. Body under 500 lines. Concrete examples. One-level file references. At least three evals. Trigger on relevant prompts and not on unrelated ones. Compare tokens, time, and pass rate with and without the skill | Shape of `SKILL.md` and the presence of eval cases | The runs, and description tuning against sample prompts |
| **Anthropic skill-reviewer** | Description is the critical field: trigger phrases, third person, not vague. Body lean. Progressive disclosure (`references/`, `examples/`, `scripts/`) | A reading of the file | Nothing, but the rubric is judgement |
| **NVIDIA SkillSpector** | "Is this skill safe to install?" Static patterns plus an optional LLM pass. Risk 0–100. In their studied subset, 26.1% of skills had a vulnerability and 5.2% looked malicious | The pattern scan | The LLM pass |
| **NVIDIA SkillEvaluator** | Tier 1: quality score and SkillSpector. Tier 2: overlap with other skills. Tier 3: the same tasks with and without the skill, in Harbor. Tier 3 dimensions are security, correctness, discoverability, effectiveness, efficiency. Skill Lift is the difference. Tier 1 and Tier 3 share names and nothing else | Tier 1 structure. Three of six Tier 3 signals are functions of the trace | Tier 3 accuracy and goal checks are an LLM judge |
| **SkillBench** (Arcade) | Six weights to a letter grade: safety 35% (and a hard F), tool boundary 25%, workflow 12%, provenance 10%, portability 10%, discoverability 8%. Hard safety rules include hidden Unicode and a credential combined with a network call | The static rules | The safety judgement of intent |
| **SkillsBench** (arXiv 2602.12670) | 87 tasks, paired no-skill and curated-skill runs, deterministic verifiers. Reported pass rate 33.9% → 50.5% (+16.6 points). Skills with at most three modules beat larger bundles | The verifier, once a trial exists | The trials |
| **SkillPortrait** (SkillAudit) | Utility (`pass_rate_gain`, `efficiency_score`) and security (five exploitability labels) from Harbor trajectories. The two utility numbers are not combined | The trajectory checks | The judge and the sandbox |

The same split shows up everywhere. Static checks decide whether the package is loadable, routable,
bounded, and free of a few obvious hostile shapes. A paired run decides whether it helps. Judgement
calls (vague wording, whether a check matches its name) are the same class of problem as D71's
proposed LLM assessment: useful, cached, and not a gate by themselves.

## 3. The catalog

`workspace` grows an opt-in `skills` glob (this portal: `plugins/**/SKILL.md` and `skills/**/SKILL.md`).
One `skill` record per file. The id is `skill:<directory>`.

| Fact | What it is |
|---|---|
| `name`, `description`, `plugin`, `path`, `dir` | The catalog entry. `plugin` is the directory under `plugins/` when the skill lives in one |
| `frontmatterErrors` | The Agent Skills checks `skills-ref` applies: required fields, name shape, directory match, allowed fields, `compatibility` length, `metadata` as a string map. Names are not NFKC-normalised (the spec normalises; our skills are ASCII) |
| `descriptionChars`, `triggerCues`, `pronouns` | Length, and the words `when` / `whenever` / `before` / `after`, or `you` / `your` / `i` / `we` / `my` / `our`, on word boundaries. Not substrings |
| `bodyLines`, `estimatedTokens` | Body after the frontmatter. Tokens are `characters / 4` of that body, the same estimate SkillEvaluator uses, named as an estimate |
| `numberedSteps`, `codeFences`, `boundaryCues` | Numbered steps, fenced blocks, and limit words (`never`, `don't`, `do not`, `limitation`, `prerequisite`, `troubleshooting`, `on error`, `error handling`, `if it fails`) |
| `missingFiles`, `linkDepth`, `supportFiles`, `windowsPaths` | Relative links that do not exist, how many markdown hops those links chain, files in the directory besides `SKILL.md`, Windows paths |
| `evalCases` | Length of `evals/evals.json` when it is an array, or of its `evals` array. Missing or unreadable is 0 |
| `hiddenChars`, `secretHits` | Zero-width, bidi overrides, and Unicode tags, after one leading BOM. Secret *shapes* only (`AKIA…`, `ghp_…`, `github_pat_…`, `sk-ant-` / `sk-proj-` / `sk-live-`, `xox[baprs]-`), reported as `file:line kind` with the secret left out |

`SKILL.md` is also a `doc`, because the docs glob already covers `plugins/**/*.md`. The doc scorecard
checks links and which SDLC layer a document names. The skill scorecard checks the skill. A file can
carry both.

## 4. The scorecard

`maintainer-skill`, kind `Skill`, levels bronze / silver / gold. Maintainer SDLC: we are the authors.
The skills themselves are what operators' agents load.

| Level | Rule | Passes when |
|---|---|---|
| bronze | `valid-frontmatter` | `frontmatterErrors` is empty |
| bronze | `description-says-when` | 20–1024 characters and at least one trigger cue. 20 is SkillEvaluator's hard short-description deduction. The spec's own ceiling is 1024. The cue list is the spec's "when", not SkillEvaluator's `use` / `for` / `helps`, which match almost every sentence |
| bronze | `third-person` | A non-empty description with none of those pronouns. The description is part of the prompt that routes skills |
| silver | `has-procedure` | At least three numbered steps |
| silver | `has-example` | At least one fenced block |
| silver | `links-resolve` | No missing relative target |
| silver | `within-body-budget` | Body ≤ 500 lines and ≤ 5000 estimated tokens |
| silver | `portable-paths` | No Windows path |
| gold | `states-boundaries` | The body names a limit, an error path, or a prerequisite. Incidental `fail` does not count |
| gold | `has-evals` | At least three cases in `evals/evals.json`. Anthropic's bar for sharing. The file is the contract a later runner would execute |
| gold | `no-hidden-instructions` | No hidden or bidi character. SkillBench SAFE-03 / SAFE-04 |
| gold | `no-hardcoded-secrets` | No secret shape. SkillBench SAFE-12, narrowed so a sentence about tokens is not a finding |
| gold | `one-level-deep` | Link chains stop at one hop, and the directory has at most three other files. The spec's "one level deep", and SkillsBench's finding that focused skills (at most three modules) beat larger ones |

A new skill below bronze is a ratchet regression. Existing skills were recorded as they scored on
the first run (section 5). Gold is deliberately not the floor: we do not yet have eval cases, and
inventing them to pass the rule would not be a measurement.

**Left for a later runner job, on the same kind:** a SkillSpector (or equivalent) report as an
optional field, `unknown` when the tool is absent (D70), and a paired-run record (`pass` with the
skill, `pass` without it, tokens, tool calls) that never gates alone (D71). The `evalCases` fact is
the hook for that job. Tier 2 deduplication, across a catalog large enough to have overlaps, is the
same kind of follow-on. Four skills do not need it.

## 5. This repository

Measured with `seikika report .seikika --skip runner,github-actions` after the connector change.
The four operator skills:

| Skill | Level | What fails above bronze |
|---|---|---|
| `add-permission` | bronze | No fenced example (silver). No `evals/evals.json` (gold). The description was updated in this change to say *when*, which the spec asks for and the previous text did not |
| `add-scorecard` | bronze | No fenced example. No boundary cue and no eval cases (gold) |
| `review-plan` | bronze | No fenced example. No eval cases |
| `seikika-authoring` | bronze | No fenced example. No eval cases |

`review-plan` and `seikika-authoring` already say *before* / *whenever* and *never*. They still stop
at bronze because none of the four skills includes an example block. That is the authoring checklist,
not a parser quirk.

D94 adds one skill per CLI command and scores the command, not the file, with `cli-skill`
([cli-skills.md](./cli-skills.md)). `maintainer-skill` still scores each of those files.

`seikika test .seikika` pins the three fixture skills: `plugins/demo/focused` at gold,
`plugins/demo/partial` at bronze, `plugins/demo/unsafe` at none. Every `maintainer-skill` rule is
named in those expectations and fails on one fixture and passes on another, so the new rules and the
new scorecard clear `meta-rule` / `meta-scorecard` from the first commit (D71).

## Sources

- Agent Skills specification, `agentskills/agentskills` `docs/specification.mdx`, read from
  `raw.githubusercontent.com` on 2026-11-20. `skills-ref` `validator.py` (frontmatter checks).
  agentskills.io itself is blocked here; see [research-log.md](../research-log.md).
- Anthropic, "Skill authoring best practices" (`platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices`)
  and "Improving skill-creator" (`claude.com/resources/articles/improving-skill-creator-test-measure-and-refine-agent-skills`),
  fetched 2026-11-20. skill-reviewer prompt: `anthropics/claude-plugins-official`
  `plugins/plugin-dev/agents/skill-reviewer.md`.
- NVIDIA SkillSpector README and `docs/DEVELOPMENT.md` (`NVIDIA/SkillSpector`). SkillEvaluator
  metrics reference (`docs.nvidia.com/skills/skillevaluator/metrics-reference`), Tier 3 results page,
  and `src/skillevaluator/validators/quality_score.py` plus `constants.py` (5000-token estimate is
  `characters / 4`; issue #461, issue #30 on substring matches).
- SkillBench methodology, `skillbench.arcade.dev/methodology`, and Arcade's write-up. Weights and the
  hard safety rules cited above are from that page.
- SkillsBench, arXiv:2602.12670. The pass-rate and "at most three modules" claims are the paper's
  reported results, not re-run here.
- SkillPortrait, `SkillAudit/skillaudit` README: `pass_rate_gain` and `efficiency_score` kept
  separate, plus the five security verdicts.

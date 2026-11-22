# A Results Spec: the Contract Above SARIF

*Status: proposal, 2026-11-22. Not decided: [questions.md](../questions.md) Q17 and Q18 hold the
choices. About the box ([toolbox-and-tools.md](./toolbox-and-tools.md)): what the box remembers and
reports, written as a contract any tool can produce and any tool can consume. It is not about a
tool. Bottom-up end (D01M80Y0XEK069GD08VCCNF0WB8); federation and hosted portals consume it.*

---

## 1. Conclusion

**Make the specification, not the implementation, the thing Toolbox is about. But compose existing
standards wherever one fits, and invent only the layer none of them covers.**

- **The gap is real.** SARIF standardises findings, and nothing standardises what sits above them:
  - which entity is being judged (a repository, a service, a package, not only a file);
  - the facts gathered about it;
  - the rules that judge those facts, and each rule's outcome;
  - the baseline a pull request is compared with, and the verdict;
  - the campaign that pays debt down, and the root cause of a regression.

  Today each portal (Toolbox's `results.json`, Backstage Tech Insights, Port, a team's
  `baseline.json`) invents its own format for these.
- **Inventing a fresh format has a cost, and the box A/B measured it.** The box arm spent 30k extra
  tokens in phase 1 learning Toolbox ([experiments/box-ab](../../experiments/box-ab/README.md) §0).
  A new spec has the same problem: agents are trained on SARIF and JSON Schema, not on a spec
  written last week. So the spec must be:
  - small;
  - built from formats agents already know;
  - self-describing (`$schema` on every document);
  - full of worked examples;
  - checked by a conformance suite rather than by trusting the prose.
- **The shape:** a short stack of document types, each producible alone by plain code in any
  language (§4). Findings stay SARIF 2.1.0. Envelopes for signing and federation are in-toto
  Statements. Rule catalogs and evaluations map onto Gemara where its model fits. The new part is
  subjects and facts, rule evaluations with honest outcomes, the baseline and ratchet verdict, and
  campaigns.
- **Toolbox becomes one implementation**: the reference producer, evaluator, ratchet and consumer,
  judged by the same conformance suite as anyone else's.
- **Test it before deciding** (§7): one A/B where agents produce and consume spec documents three
  ways (by hand from the spec alone, with Toolbox, and on Backstage), scored by the conformance
  suite.

**Why this doesn't contradict C5 and D42.** Those rejected a *governed ingestion protocol* (TCP),
because ingestion is internal plumbing and a generic OpenAPI endpoint covers it. Results are the
opposite case. They are exactly what crosses boundaries:
- a repository's report read by an organisation's portal (federation);
- a hosted page;
- another CLI;
- an agent in a different harness.

An interchange format earns a spec where a pipe between two of our own components doesn't.

## 2. Evidence from our own runs

| Observation | Source | What a spec would have changed |
|---|---|---|
| Both A/B arms invented a private baseline (`.github/quality-gate/baseline.json`, `.toolbox/results.json`), and neither is readable by the other's gate | box A/B, phase 1 | One baseline format: the arms differ in how they compute it, not in what they write |
| The harness imposed a small interface (`TOOLS.md`: gate, accept, progress, tools with commands) and could then score two unrelated implementations with one script | box A/B, harness | That *was* a spec. It made two implementations comparable and replaceable |
| A finding moved down two lines read as a new one, so the read key needed line arithmetic, and both arms wanted new findings marked | box A/B, phase 6 (both arms), root-causes C1 | Fingerprints per finding, normative in the baseline (SARIF already defines `partialFingerprints`) |
| One `unknown` covered "tool not installed", "data missing" and "skipped on purpose", and a missing tool passed the gate silently | box A/B, phase 1 (arm B) | Distinct outcomes, as Scorecard (5) and Gemara (6) already have |
| A restarted campaign lost its history, so an upgrade and a changed standard looked the same | box A/B, phases 2–4 | Campaign starts and restarts as normative records with a reason |
| The tool's version was pinned in three or four places | box A/B, phase 4 (both arms) | Provenance: every document names its producer and version, so drift is visible downstream |

## 3. Prior art, read from sources

Read from each project's repository on 2026-11-22 (commit dates in [Sources](#sources)).

| Standard | Standardises | Subject | Outcomes | Baseline / ratchet | Aggregates | Reproducing a result | How the spec is run |
|---|---|---|---|---|---|---|---|
| **SARIF 2.1.0** (OASIS) | Static-analysis findings | Artifacts (files, logical locations) | `kind`: pass, fail, review, open, notApplicable, informational; `level` | `baselineState`: new, unchanged, updated, absent; `partialFingerprints` | `runAggregates`, `policies`, `taxonomies` (little used) | `invocations[].commandLine` | OASIS standard, JSON Schema |
| **reviewdog rdjson** | Diagnostics for review comments | File and range | Severity only | None | None | `original_output` | Protobuf and a JSON Schema in the tool's repository |
| **Code Climate issue spec** | Engine output for code-quality hosts | File and range | Severity; `categories` | `fingerprint` | None | None | A markdown SPEC with a VERSION file (last changed 2020) |
| **OpenSSF Scorecard** | Repository security checks | A repository | Probes: True, False, NotAvailable, Error, NotSupported; checks 0–10 | None | A check is scored from probes; **conformance mode** judges a catalog (OSPS Baseline) | Probe definitions (`def.yml`) | One implementation; output JSON, and Gemara Evaluation Logs |
| **Gemara** (OpenSSF, v0.17-dev) | Governance, from guidance to controls, policy, evaluation, enforcement and audit, in layers | `target` (a resource) per log | Not Run, Passed, Failed, Needs Review, Not Applicable, Unknown; confidence: Undetermined, Low, Medium, High | None | `EvaluationLog.result` over control evaluations | Assessment steps and evidence with sources | CUE schemas with lifecycle tags; Go and Python SDKs; used by FINOS CCC and the OSPS Baseline |
| **in-toto attestation v1** | A signed claim about artifacts | `subject[]` by digest | Per predicate (test-result: passed, warned, failed tests; VSA: a verification result) | None | None | Per predicate | Layered: Envelope → Statement → Predicate; predicate types are URIs with their own versions |
| **Backstage Tech Insights** | Facts about catalog entities, and checks over them | A catalog entity reference | Boolean check results (json-rules-engine) | None | Scorecards in the frontend | Fact retriever code | **An in-process TypeScript plugin API**, not a file format: facts are typed by a `FactSchema` and versioned per retriever |
| **OpenAPM v0.1** (microsoft/apm) | Agent package manifests, lockfiles and policy | Packages | — | Lockfile drift | — | — | **The model for running a spec:** an editor's working draft with 0.x semver, numbered RFC 2119 requirements (`req-xx-NNN`), conformance classes (Producer, Consumer, Registry, Governance), a generated conformance statement binding each requirement to behavioural tests, and an "honesty contract" that tells real coverage from text-only |

**What the table says:**
- **SARIF already covers findings, passes and baselines for files.** Don't redefine them.
- **Gemara is the closest prior art above findings.** It is compliance-shaped (catalogs, policies,
  audit) and has no ratchet, campaigns, relations between entities or per-file measures. Its
  outcome set and evidence model are worth adopting.
- **in-toto solves envelopes and signing.** Reuse it for federation, where a report crosses
  organisations.
- **Backstage standardises an API, not a document.** A tool can't emit a Tech Insights fact without
  running inside Backstage. That's the "no-code platform" hazard in another form: the contract is
  the platform's code.
- **OpenAPM shows how to run a spec** so that it, not one implementation, is the contract.

## 4. The proposed spec, in document types

**Working name: the Toolbox Results spec, v0.1 (an editor's draft).** The name is open (Q17). Each
document is JSON with `$schema` and `type`, and validates against a JSON Schema 2020-12. Every type
can be written alone, by a script in any language. A consumer reads only the types it needs.

| # | Document | Holds | Composes | Produced today by |
|---|---|---|---|---|
| 1 | **facts** | A subject (an id with a kind: `repository`, `file`, `package`, `service`), its typed facts, and provenance (producer, version, the command run, input digests) | Subject ids reuse purl and URLs where they fit | Connector records (`builtin:workspace`, runner), Tech Insights fact retrievers |
| 2 | **findings** | **SARIF 2.1.0, unchanged**, with three normative conventions: `partialFingerprints` set, `suppressions` for inline ignores, and `invocations[].commandLine` set so the result can be reproduced with the tool alone | SARIF | ruff, semgrep, zizmor, ESLint formatters |
| 3 | **standard** | Rules over facts and findings: id, the level it belongs to, the check (CEL, normative), a measure, evidence, fix text, owner, and a mapping to an external control (Gemara, OSPS Baseline) | Gemara Layer 2 controls as optional mappings | Toolbox scorecards, `-o standard` |
| 4 | **evaluation** | Per subject and rule: an outcome, the measure, evidence (references to facts and to SARIF results by fingerprint), the assessor (tool, agent or person) and its confidence, and the subject's level | Gemara Layer 5 results and confidence | `toolbox report -o json`, agent assessments (D79) |
| 5 | **baseline** | Recorded evaluations (compact: only what fails), finding fingerprints, and campaign starts and restarts with a reason | — | `results.json` (`toolbox-report/v1`) |
| 6 | **verdict** | A comparison of an evaluation with a baseline: worse, better and changed, each regression's root-cause chain, and an exit code | — | `toolbox-verdict/v1` |
| 7 | **envelope** (optional) | Any of the above as an in-toto Statement (`predicateType` set to the document type's URI), so that federation can sign and verify it | in-toto attestation v1 | Not yet |

**Outcomes (normative, one enum for every assessor):**

| Outcome | Means | Ratchet |
|---|---|---|
| `pass` | The rule holds | Better if it failed before |
| `fail` | The rule doesn't hold; the evidence says why | Worse if it passed, or if its measure grew |
| `not-applicable` | The rule doesn't apply to this subject | Neither |
| `not-run` | Skipped on purpose (a connector excused) | Neither |
| `unavailable` | The data couldn't be collected (a tool not installed, the network down) | Fails `--fail-on-unavailable`; never silently better |
| `error` | The tool or check failed | Always fails the gate |
| `needs-review` | An assessor (an agent or a person) must decide | Neither, but it is listed |

This splits today's `unknown` four ways, as Scorecard and Gemara already do.

**Ratchet semantics are normative,** not left to implementations. Worse means one of:
- a rule moved from `pass` to `fail`;
- a failing rule's measure grew;
- a new subject arrived below the standard's floor;
- a fingerprint is in the evaluation but not in the baseline.

Better is the reverse. Everything else is a change.

**A minimal evaluation, for one subject and two rules** (informative):

```json
{
  "$schema": "https://toolbox.dev/spec/results/v0.1/evaluation.schema.json",
  "type": "evaluation",
  "producer": {"name": "toolbox", "version": "0.2.0", "command": "toolbox report .toolbox -o evaluation"},
  "standard": {"id": "github-actions", "version": "0.2.0", "digest": {"sha256": "…"}},
  "subject": {"kind": "file", "id": ".github/workflows/ci.yml", "in": "repository:github.com/acme/app"},
  "level": "bronze",
  "rules": [
    {"rule": "pinned-to-sha", "outcome": "fail", "measure": 1,
     "evidence": [{"sarif": "zizmor.sarif", "fingerprint": "unpinned-uses:3f9c…"}],
     "assessor": {"kind": "tool", "name": "zizmor", "version": "1.32.0",
                  "command": "zizmor --offline --format sarif .github/workflows/ci.yml"}},
    {"rule": "timeouts-set", "outcome": "unavailable", "reason": "semgrep is not installed"}
  ]
}
```

## 5. How the spec is run (borrowed from OpenAPM)

| Practice | Here |
|---|---|
| Editor's working draft, semver 0.x, breaking changes allowed with a migration note until 1.0 | `v0.1` |
| Numbered RFC 2119 requirements (`req-ev-001`), indexed in an appendix | Every MUST and SHOULD |
| Conformance classes | **Producer** (writes facts, findings and evaluations); **Evaluator** (standard plus facts gives evaluations); **Ratchet** (evaluation plus baseline gives a verdict); **Consumer** (reads: reports, portals, federation) |
| A conformance suite: inputs with byte-exact expected outputs, as `examples/reference-portal` already does | `spec/conformance/<class>/<case>/` |
| A generated conformance statement per implementation, with an honesty contract: each requirement is `active` only if a behavioural test exercises it | `CONFORMANCE.md` generated by Toolbox's CI |
| Normative JSON Schemas, inline, and a validator anyone can run | `spec/schemas/*.json`; `toolbox spec validate <file>` |

## 6. What changes in Toolbox, if adopted

No code moves until Q17 is decided. If it is adopted:
- `results.json` becomes a **baseline** document, and `--verdict` a **verdict** document.
- `-o json` writes **evaluation** documents.
- Connector records become **facts**. The SARIF slot keeps reading SARIF.
- The current formats get one release of compatibility: `toolbox-report/v1` keeps reading.
- `scripts/take-it-out.py` becomes the spec's check that a producer's `command` reproduces its
  findings.
- Federation (D68) consumes evaluation and baseline documents in in-toto envelopes, instead of
  Toolbox's own format.

## 7. Testing the spec before deciding

The [spec A/B](../../experiments/spec-ab/README.md) is designed, not run. It asks whether a
well-specified format lets agents produce and consume results reliably without Toolbox, and what
Toolbox and Backstage add. Arms:
- **A:** by hand, from the spec alone;
- **B:** with Toolbox;
- **C:** building on Backstage.

All three are scored by the spec's conformance suite.

## 8. Risks

| Risk | Mitigation |
|---|---|
| A new standard nobody adopts | Compose SARIF, in-toto and Gemara rather than compete; propose a Gemara mapping, or upstream as a Gemara profile, if the model fits (Q17) |
| The spec drifts from the implementation | OpenAPM's conformance statement with an honesty contract |
| Over-design before use | v0.1 covers only what this repository's portal and the box A/B already exercised; everything else is reserved |
| Agents don't know the spec | Measure it (the spec A/B, arm A). If arm A can't reach conformance cheaply, the spec is too big |

## Sources

- OASIS SARIF 2.1.0, `oasis-tcs/sarif-spec` `sarif-2.1/schema/sarif-schema-2.1.0.json` (2026-10-15): `result.kind`, `result.baselineState`, `run` properties
- `reviewdog/reviewdog` `proto/rdf/reviewdog.proto` (2026-11-21)
- `codeclimate/platform` `spec/analyzers/SPEC.md` (2020-09-04)
- `ossf/scorecard` `finding/finding.go`, `docs/probes.md`, `docs/conformance.md` (2026-11-19)
- `gemaraproj/gemara` `evaluationlog.cue`, `collections.cue`, `evidence.cue`, `VERSION` v0.17.0-dev (2026-11-19)
- `in-toto/attestation` `spec/v1/statement.md`, `spec/predicates/test-result.md`, `vsa.md` (2026-10-21)
- `backstage/community-plugins` `workspaces/tech-insights/plugins/tech-insights-common/src/types/{facts,checks}.ts`, `tech-insights-node/src/facts.ts` (2026-11-21)
- `microsoft/apm` `CONFORMANCE.md`, `docs/src/content/docs/specs/openapm-v0.1.md` (2026-11-22)
- [technology-critique.md](../technology-critique.md) C5 and [D42](../decisions/D01M7V8P2VS0000000000000000.md): why ingestion isn't a governed protocol
- [experiments/box-ab](../../experiments/box-ab/README.md) §0: the evidence in §2

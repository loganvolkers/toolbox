# Adversarial Critique: Should Seikika Be Built at All?

*Written 2026-11-19 from the point of view of a skeptical platform-engineering lead at a company with
500–5,000 engineers, who asks: "Why invent a framework instead of vibe-coding something on Backstage,
Next.js or Django, or just buying something?" The aim is to **disprove** that Seikika is a worthwhile
investment. Each core claim is stated as a falsifiable hypothesis and attacked with evidence.*

*Scope: whether to build at all, not design details. [red-team.md](./red-team.md) (competitive gaps)
and [principal-review.md](./principal-review.md) (architecture holes) are not repeated unless there is
new evidence. Where this document contradicts [research.md](./research.md), §12 lists the corrections.*

*Evidence labels:* **[P]** primary source read directly (a repository, or code I ran);
**[S]** secondary (search excerpt, vendor blog, press); **[I]** my inference. Most vendor sites and
metr.org, dora.dev and thenewstack.io were unreachable from this environment
([research-log.md](./research-log.md)), so many market figures are [S].

---

## 1. Verdict

**Do not build Seikika as currently scoped.** The need it targets is real: agents that answer "who owns
X, does it meet our standards, and why not", with permissions. But between November 2025 and November
2026 every incumbent shipped that agent surface. That includes the open-source default: Backstage now
has MCP over OAuth, audit events, published agent skills and an `AiResource` kind. Seikika puts its
novelty budget into internals buyers never see: a DBSP circuit, embedded Go OpenFGA via a C ABI,
Pulumi-style polyglot SDKs, a new connector protocol and a deterministic simulator. Meanwhile the
visible value is becoming a commodity, and the most common way platforms fail, low adoption for
organisational reasons, is left untouched.

The project currently has **one human contributor** (25 of 26 commits are by Claude) and 31 planned
Rust crates **[P]**. It depends on three pre-1.0 or pre-production libraries, and in one of them the
production features it needs are enterprise-gated in the reference product (§3, H5). A lean wedge
could still be worth building (§11). The full framework is not.

| # | Hypothesis (Seikika's claim) | Verdict | One-line reason |
|---|---|---|---|
| H1 | Agents need a **dedicated agent-first portal** to work well | **Unresolved, leaning weakened** | No evidence either way that catalog context improves agent outcomes. The best adjacent evidence (ETH 2026) found extra context files *lowered* success and raised cost by >20%. Seikika defers its own test of this to Phase 4 |
| H2 | Incumbents' agent surfaces are thin and "bolted on", leaving room for an agent-first entrant | **Weakened** | Backstage 1.50–1.57 added MCP OAuth (CIMD), audit, skills, `AiResource` and durable scaffolder recovery. Harness cut 130+ tools to 11. OpsLevel, Cortex, Roadie, Port and RHDH all ship MCP |
| H3 | Config as code in *real languages* (Pulumi model) is a buyer-valued differentiator | **Weakened** | Config as code is wanted (GitOps lock-outs, Terraform providers), but nothing shows that buyers pick a portal for TS/Python SDKs. The cost is Pulumi-scale, and Pulumi raised ~$100M–250M [S] |
| H4 | Incremental computation (DBSP) is **required** at the target scale | **Refuted at the stated scale** | My naive full recompute of 100 CEL rules × 100k entities took **6.3 s single-threaded, 2.0 s on 4 threads** [P], against a full-rescore SLO of < 5 min. The ownership closure took 7 ms |
| H5 | The technology bets are mature enough to build on | **Weakened, close to refuted for v1** | `dbsp` is at v0.385 (388 tags) and Feldera gates checkpoints and state-preserving pipeline changes behind Enterprise. DBOS Rust is 0.6 "pre-production". cel-rust skips 688 of ~2,771 conformance tests and has no type checker. No precedent found for an OpenFGA c-archive [P] |
| H6 | "Built with agents" makes the scope tractable (D25: no estimates) | **Weakened** | METR's RCT found experienced developers 19% *slower* while believing they were 20% faster. Its 2026 update is "very weak evidence" of a speed-up. Agents do best on well-trodden stacks, and Seikika is the opposite [S/I] |
| H7 | Operators will adopt a new framework over Backstage, Port and the rest | **Unresolved, no demand evidence** | Backstage has ~3,400 known adopters [S]. Port raised $100M at $800M [S]. 45.3% of platform teams cite adoption as their main struggle, and culture outweighs tech [S]. The docs name no design partner |
| H8 | An open-source project can sustain this scope | **Refuted as currently staffed** | One human. Rust + Go FFI + TS/Python SDKs + React + a protocol spec + a simulator. 60% of maintainers are unpaid [S]. Precedents: Earthly's open-source project went to maintenance mode (2025) and Lyft Clutch was archived (Feb 2026) [S/P] |
| H9 | No open-source project fits, so the gap is worth filling | **Survives literally; doesn't imply value** | True that nothing combines all the pillars. But the "Rust agent-first niche filling" evidence is projects with 2–3 stars [P], which shows no demand |
| H10 | Self-hosted open source avoids vendor risk | **Survives** | Atlassian ended Compass sales on 13 May 2026, and scorecards don't migrate to DX [S]. This is the strongest argument *for* Seikika, but Backstage already offers it |

**Strongest single argument against (§8):** Seikika's value to a buyer is permission-aware agent access to
ownership and standards. That is now a feature every incumbent ships, and its cheapest form costs
almost nothing (CODEOWNERS plus GitHub custom properties plus the GitHub MCP server). Seikika's
distinctive costs sit in internals that solve problems buyers don't report, at a scale where a full
recompute measures in seconds.

**Cheapest alternative that gets ~80% of the value (§9):** `catalog-info.yaml` + CODEOWNERS + GitHub
custom properties as the ownership record, standards as CI checks, and agents querying through
existing MCP servers plus one thin, read-only "ownership MCP" server. Organisations that already run
Backstage should switch on its MCP actions backend instead.

---

## 2. Method

1. I extracted Seikika's claims from [plan.md](./plan.md) §1, §6.6, §8 and §9, from
   [decisions.md](./decisions/README.md) and from [research.md](./research.md) §7.
2. For each claim I wrote down the evidence that would falsify it *before* searching.
3. Research:
   - Primary sources: shallow clones of `backstage/backstage`, `feldera/feldera`,
     `dbos-inc/dbos-transact-rust` and `cel-rust/cel-rust` (commits in §13), plus `gh api search/*`
     for repository statistics.
   - Everything else: web-search excerpts, labelled [S].
4. **One experiment.** A naive full-recompute benchmark with cel-rust (§7), to test H4 directly.
5. Verdict scale: *refuted* (the evidence contradicts the claim); *weakened* (the evidence cuts against
   it, but not decisively); *unresolved* (no decisive evidence); *survives*.

---

## 3. Hypotheses and evidence

### H1. Agents need a dedicated, agent-first portal

**Claim.** [plan.md](./plan.md) §1: "Agents are the primary user." The Phase 4 exit criterion is that
agents with `context` beat agents without it at an equal token budget.

**Would falsify it:** agents doing as well by querying source systems directly (GitHub, PagerDuty,
the IdP) or an incumbent's MCP server; or curated context failing to improve outcomes.

**Evidence:**

| Fact | Label |
|---|---|
| ETH Zurich, *Evaluating AGENTS.md* (arXiv 2602.11988, ICLR 2026): repository context files **reduced** task success against no context (the ICLR version says "no improvement") and raised inference cost by **>20%**. Agents followed unnecessary instructions and explored more | [S] |
| No benchmark was found that measures coding agents with vs. without catalog (ownership/dependency) context. CodeScaleBench (Sourcegraph) reports large gains from cross-repo code search, which is search, not catalog metadata | [S] |
| Tool overload is real: Anthropic reports tool definitions consuming 134k tokens in internal setups, and code-execution and tool-search patterns cutting this sharply. Harness cut its MCP server from 130+ tools to 11 | [S] |
| The GitHub MCP server has **34.6k stars**, about the same as Backstage (34.9k) | [P] (stars are a weak proxy) |
| GitHub gave every active repository a validated owner in under 45 days using **custom properties** and an internal service catalog; ~8,000 unused repositories were archived | [S] (GitHub blog via mirror; blog host blocked) |
| Commentary (Edmunds, Apr 2026): "AI skills are the new platform interface"; portals are passive websites | [S] (opinion) |

**Inference.**
- A *consolidating* layer has some support: tool overload hurts, and "which source do I trust" is a
  real problem. That is the best case for a portal.
- But the measured effect of adding curated context is unproven and can be negative.
- Seikika has put the one experiment that tests its thesis last (Phase 4), after building everything
  it depends on.

**Verdict: unresolved, leaning weakened.**

### H2. Incumbents are "bolted on", leaving room for an agent-first entrant

**Claim.** [research.md](./research.md) §1: every agent interface is "bolted onto a human-first,
DB-configured product". Backstage's MCP relies on static tokens, "a temporary workaround".

**Would falsify it:** incumbents shipping first-class auth, audit, permission-aware tools and lean tool
surfaces.

**Evidence:**

| Vendor | Agent surface shipped 2025–2026 | Label |
|---|---|---|
| **Backstage (OSS)** | **MCP actions over Streamable HTTP with OAuth via CIMD**; static tokens are now only a fallback for apps on the old frontend system (`docs/ai/mcp-actions.md`) | [P] |
| Backstage (OSS) | Audit events for MCP connections, tool discovery and execution (v1.54); OpenTelemetry spans for `tools/call` (v1.51); `refresh-catalog-entity` action for agents (v1.54) | [P] |
| Backstage (OSS) | **Published agent skills** at `backstage.io/.well-known/skills/` (`docs/ai/skills.md`); `AiResource` kind and `API` `type: mcp-server` (v1.51/1.54) | [P] |
| Backstage (OSS) | Durable **scaffolder task recovery** from the last completed step (v1.55); frontend system 1.0 (v1.57) | [P] |
| Spotify Portal | "Portal Studio" with Claude builds internal tools and plugins "in days" (QCon London 2026) | [S] (vendor talk) |
| Port | Remote MCP, AI Builder, agentic platform; $100M Series C at $800M (Dec 2025) | [S] |
| Cortex | Cortex MCP GA; Context Graph | [S] |
| OpsLevel | MCP since June 2025; July 2026 update: leaner responses, agents tracked as catalog entities | [S] |
| Roadie | Context Store with four MCP servers (May 2026) | [S] |
| Harness | MCP server v2: 11 consolidated tools, per-user OAuth, audit | [S] |
| Red Hat Developer Hub | MCP for catalog and TechDocs (1.8); 2.1 adds an AI catalog and MCP for scorecards and workflows | [S] |

**Inference.** The window [research.md](./research.md) describes has narrowed sharply. Some gaps
remain:
- Backstage's registry still leaves permission checks to each action.
- No incumbent offers `plan`-style previews of permission and score diffs.

Those are *features*, which can be contributed to Backstage. They are not reasons for a new framework.

**Verdict: weakened.**

### H3. Config as code in real languages is a buyer-valued differentiator

**Claim.** D2 and D3: config never lives in a database; Pulumi-style TS/Python programs emit a
manifest.

**Would falsify it:** buyers satisfied by YAML/Terraform plus GitOps, or the cost of polyglot SDKs
outweighing their benefit.

**Evidence:**
- **For:** Cortex locks UI editing under GitOps; Port, Cortex and Datadog have Terraform providers
  ([research.md](./research.md) §3) [S].
- **For Backstage specifically:** it already keeps catalog descriptors in repos, and v1.50 added
  `provideStaticCatalogModel` for static, code-provided catalog models [P].
- **Cost:** Pulumi has raised ~$100M–$252M and employs ~100–300 people (sources conflict) to sustain
  multi-language SDKs [S]. The principal review already called this "the biggest cost in the plan"
  ([principal-review.md](./principal-review.md) §4.1); it was kept anyway.
- **No evidence found** that portal buyers choose on authoring language. The survey pain points are
  adoption, measurement and culture (H7).

**Verdict: weakened.** The *config in Git, plan on the PR* idea survives as a feature. The
real-language SDK layer is a cost with no demonstrated demand.

### H4. Incremental computation is required at the target scale

**Claim.** D21: "no batch or nightly recomputation"; one DBSP circuit for all derived data. The scale
target is 100k entities, 2M relations, 50 × 100 rules, and "full rescore < 5 min"
([plan.md](./plan.md) §6.6).

**Would falsify it:** a naive full recompute at the target scale finishing well inside the freshness
SLOs (< 60 s webhook → visible).

**Evidence (my experiment, §7) [P]:**

| Workload | 1 thread | 4 threads |
|---|---|---|
| 100k entities × 100 CEL rules (10M evaluations, cel-rust interpreter, no caching) | **6.3 s** | **2.0 s** |
| 250k entities × 100 rules (25M evaluations) | 17.9 s | 4.4 s |
| Nested membership closure (10k users, 1k teams) + ownership roll-up to the root | 7 ms | — |

Supporting facts:
- Spotify's Soundcheck docs cite **~1 GB for an in-memory catalog of 250k entities** [S].
- Re-scoring one changed entity against 100 rules costs ~63 µs at the measured per-evaluation rate
  [I, arithmetic].

**Inference.**
- At the stated scale, "recompute the dirty entities on change, plus a periodic full recompute" fits
  the SLOs by more than an order of magnitude.
- What I did not measure: aggregate rules, the 2M-relation join and authz tuple derivation. They will
  cost more, but nothing suggests minutes.
- DBSP is therefore an engineering preference, not a requirement. It brings the risks in H5. The
  principal review's "full recomputation for v1" ([principal-review.md](./principal-review.md) §2.2)
  was right, and D21 overruled it without numbers.

**Verdict: refuted at the stated scale.**

### H5. The technology bets are mature enough

**Would falsify it:** pre-1.0 churn, missing production features, or no precedent for the integration.

| Dependency | Finding | Label |
|---|---|---|
| **DBSP (`dbsp` crate)** | Workspace version **0.385.0**; **388 release tags**. Every release is 0.x, so each minor bump may break. Recent changelog entries are full of "Incompatible change" | [P] |
| DBSP / Feldera | Feldera's docs: "Checkpoints, fault tolerance, and disaster recovery features are only available in Feldera Enterprise Edition." **"Modifying a pipeline while preserving its state" (bootstrapping) is Enterprise-only.** Object-store checkpoint pull/push is behind `#[cfg(feature = "feldera-enterprise")]` in `crates/connectors/src/controller.rs` | [P] |
| DBSP / Feldera | The MIT `dbsp` crate does contain `checkpointer.rs` and `bootstrap.rs`. The production orchestration around them is what Feldera monetises. Seikika's Q9 (sub-circuit state reuse) and its object-storage checkpoints are exactly these features | [P] / [I] |
| DBSP adoption | ~100 code-search hits for `dbsp` in `Cargo.toml`, mostly Feldera, experiments and theses | [P] |
| **DBOS Rust SDK** | README: "**Pre-production release.** DBOS Transact for Rust 0.6 … isn't yet supported for production use. Its API may still change in breaking ways." The repo has 50 stars and the main branch is at 0.7.0-dev. DBOS Inc. raised an $8.5M seed (2024); no later round found | [P] / [S] |
| **cel-rust** | `conformance/src/bin/ignored.txt` lists **688 skipped conformance tests** out of ~2,771 generated (~25%), including 66 `type_deduction` tests. **There is no type checker** in `cel/src`. Seikika's "typed CEL" (D24) means writing one | [P] |
| **OpenFGA as a c-archive in Rust** | No precedent found. OpenFGA's own datastore semantics are still being fixed (open issues #3346 "ReadChanges on Postgres skips changes under concurrent writes"; #3393 "preserve ReadChanges ordering"). A custom read-only datastore must match semantics the project itself is still stabilising | [P] / [I] |
| Rust UI | Not a risk: the UI is React (D31) | [P] |

**Verdict: weakened, close to refuted for v1.** Each bet has a fallback, but the fallbacks are boring
technology. That raises the question of why Seikika doesn't start there.

### H6. "Built with agents" makes the scope tractable

**Claim.** D25 removes estimates because "the project is built with agents".

**Would falsify it:** controlled evidence that AI speed-ups are small or negative on complex, novel
codebases, or that AI-written code accrues maintenance debt.

**Evidence:**

| Fact | Label |
|---|---|
| METR RCT (July 2025, 16 experienced OSS developers, 246 tasks): **19% slower** with AI. Developers *predicted* a 24% speed-up and *believed afterwards* they were 20% faster | [S] (arXiv 2507.09089) |
| METR update (Feb 2026): new estimates with wide confidence intervals; selection bias (30–50% refused to work without AI). METR calls it "only very weak evidence" for the size of the speed-up, though it believes speed-ups are now likely | [S] |
| DORA 2025 (~5,000 respondents): AI acts as an "amplifier" of existing practices; 90% have adopted platform engineering; platform quality correlates with AI value | [S] |
| GitClear (211M changed lines, 2020–2024): duplicated blocks up 8× in 2024; changes to old code down sharply | [S] |
| Veracode 2025: AI-generated code had security flaws in ~45% of tasks | [S] |

**Inference.** The evidence is symmetric, and it matters which side benefits.
- **Vibe-coding a portal** (Next.js/Django CRUD over Postgres) is the best case for agents: abundant
  training data and stable APIs. It suffers the debt and security costs above.
- **Seikika** is the worst case: novel APIs (`dbsp` 0.38x, cel-rust without a type checker, a
  hand-rolled C ABI to Go, a new protocol), where agents have little prior knowledge and subtle
  correctness matters.
- "Agents make it cheap" therefore applies *more* to the alternatives than to Seikika. Spotify's
  Portal Studio shows the incumbents capturing the same speed-up on their own platforms.

**Verdict: weakened.**

### H7. Operators will adopt a new framework over Backstage, Port and the rest

**Would falsify it:** absence of unmet demand among target buyers; consolidation towards incumbents;
adoption failures being organisational rather than technical.

**Evidence:**

| Fact | Label |
|---|---|
| Backstage: **~3,400 known adopters** (Sept 2025); 34.9k stars; 36 contributors to v1.54 alone | [S] / [P] |
| An "89% market share" for Backstage, and "~10% internal adoption outside Spotify". The latter is attributed to Spotify's Helen Greul in The New Stack (host blocked) | [S], low quality |
| Gartner Market Guide for IDPs (31 Mar 2025): 85% of organisations with platform teams will provide portals by 2028, up from 60% in 2025 | [S] (vendor summary) |
| platformengineering.org Vol. 4 (Jan 2026, n=518): **45.3% struggle with developer adoption**, 29.6% don't measure success, 18.3% delivered no measurable results; "organisational and cultural barriers consistently massively outweigh technical ones"; 55.9% run more than one platform | [S] |
| Perforce 2026 (n=820): platform maturity correlates with AI success (73% vs 44%) | [S] (vendor survey; correlational) |
| Funding: Port $100M Series C at $800M (Dec 2025); Cortex $60M Series C at ~$470M (Sept 2024); Atlassian bought DX for ~$1B (Sept 2025); Feldera $21.5M | [S] |
| Consolidation: Atlassian Compass end of sale **13 May 2026**, end of renewals 31 Dec 2026, folded into DX, and **scorecards don't migrate** | [S] |
| The Seikika docs name **no design partner or interviewed buyer** | [P] |

**Inference.**
- The market is consolidating around Backstage (open source) and a few funded SaaS products.
- The dominant failure mode is adoption and culture, which a new runtime architecture does not address.
- A new framework adds a migration and retraining cost that teams already struggling with adoption
  are least able to absorb.

**Verdict: unresolved, with no demand evidence.** I would treat it as weakened until a design partner
exists.

### H8. An open-source project can sustain this scope

**Would falsify it:** a scope far beyond plausible maintainer capacity; base rates of maintainer
burnout; precedents of stalled ambitious frameworks.

**Evidence:**
- **Scope** [P]:
  - 31 Rust crates ([plan.md](./plan.md) §3).
  - A Go OpenFGA c-archive.
  - TS and Python SDKs with language hosts.
  - A React design system with Storybook.
  - The TCP spec and conformance kit; two agent plugins.
  - A deterministic simulator, reference models, an MCP server, GraphQL, REST, a CLI, Helm charts and
    GitHub Actions.
  - Languages that portal authors face: TS/Python/YAML, CEL, the OpenFGA DSL, TSX and WIT/WASM
    ([principal-review.md](./principal-review.md) §4.3 flagged this and it was rejected).
- **People** [P]: 26 commits since 2026-11-18, 25 authored by Claude and one by a single human. The
  principal review assumed "3–5 people"; the repository shows one.
- **Base rates** [S]: Tidelift 2024: ~60% of maintainers are unpaid and 61% of unpaid maintainers work
  alone; ~60% have quit or considered quitting (attributed to the 2023 or 2024 report; sources
  conflict).
- **Precedents:**
  - Earthly shut its cloud service in July 2025 and moved the open-source tool to critical fixes only,
    citing "open-source cannibalisation" [S].
  - Lyft Clutch is archived (pushed 2026-02-13, `archived: true`) [P].
  - Kuzu archived in Oct 2025 ([research.md](./research.md)).
  - Torii stale since 2024 ([research.md](./research.md)).
- **Monetisation tension** [P/I]: Feldera keeps the hard production features (checkpoints, state-preserving
  changes) in its Enterprise edition. An open-source Seikika has to rebuild them for free, with no
  revenue model ([red-team.md](./red-team.md) A6 is still open).

**Verdict: refuted as currently staffed.** It could survive with a funded team or a foundation, but
none is in sight.

### H9. No existing open-source project fits, so the gap is worth filling

**Evidence** [P]:
- [research.md](./research.md) §5 says the Rust agent-first niche "is beginning to fill", which
  "validates both the language choice and the timing".
- Current star counts: Frontkeep **3**, OpsCat **2**, Decathlon idp-core **10**, Cortex engrams 75.
- Meanwhile the projects with traction are Backstage (34.9k), OpenChoreo (1.9k, built on Backstage)
  and the GitHub MCP server (34.6k).

**Inference.** A gap that only near-zero-traction projects occupy is weak evidence of demand. It may
be a gap because the combination isn't wanted.

**Verdict: survives literally, but doesn't imply value.**

### H10. Self-hosted open source avoids vendor risk

**Evidence** [S]:
- Compass end of sale, with scorecards not migrating to DX.
- Port and Cortex price per seat with enterprise tiers by quote.
- Feldera's open-core gating shows how vendors move features behind licences.

**Verdict: survives.** This is the strongest point *for* Seikika. Backstage (Apache-2.0, CNCF) already
satisfies it, though, so it argues for open source, not specifically for Seikika.

---

## 4. Alternatives compared

Figures are list prices or third-party estimates [S] unless marked. "1,000 eng" assumes 1,000
contributing developers; enterprise discounts are typical and unknown [I].

| Option | Cost signal | Time to value | Ongoing maintenance | Agent readiness (MCP/API) | Fit for 500–5,000 eng |
|---|---|---|---|---|---|
| **Backstage, self-hosted** | Free licence. "3+ FTE", "$200k–$1M+/yr" (vendor-sourced, [research.md](./research.md) §2) | 6–12 months to production (vendor-sourced) | High: upgrades are the #1 pain (56%, secondhand) | **Good and improving:** MCP + OAuth (CIMD), audit, skills, `AiResource` [P] | Strong if you already run it; weak at < 500 eng |
| **Roadie** (managed Backstage) | Teams ~$24/dev/month (50–150 devs); Growth by quote → ~$288k/yr list at 1,000 eng [I] | Days to weeks | Low (vendor runs it) | MCP Context Store (four servers) | Good |
| **Spotify Portal / RHDH / Harness IDP** | By quote (AWS Marketplace listings) | Weeks | Low to medium | MCP across all three | Good; RHDH fits OpenShift shops |
| **Port** | Free ≤ 15 seats; ~$30–40/seat/month; enterprise by quote → ~$360k–480k/yr list at 1,000 eng [I] | "An afternoon" for a GitHub catalog ([red-team.md](./red-team.md) A1) | Low; config drift between Terraform and the DB is possible | Remote MCP, AI Builder, agents | Strong; vendor-dependency risk |
| **Cortex** | Vendr median **$52.8k/yr**; estimates ~$25–65/user/month | Weeks | Low | MCP GA, Context Graph | Strong for scorecards |
| **OpsLevel** | Vendr median ~$24k/yr (range $12k–65k, n=18) | Weeks | Low | MCP since 2025; agent entities | Good, mid-market |
| **DX (Atlassian)** | By quote | Weeks | Low | (not verified) | Atlassian shops; Compass users are being forced to migrate |
| **Datadog IDP** | Bundled with Datadog | Days if already on APM | Low | No IDP MCP found | Good for Datadog-centric orgs |
| **Vibe-coded portal** (Next.js/Django/Retool + an agent) | Agent + engineer time; no licence | Fast for CRUD screens [I] | **Unknown and likely high:** duplication and churn (GitClear), ~45% insecure output (Veracode); 93% of tech leaders worry about vibe-coded tools in production (Retool, n=307) [S] | Whatever you build | OK for a narrow catalog; poor for permissions and audit |
| **No portal / minimal** (catalog-info + CODEOWNERS + custom properties + existing MCP servers + Steampipe/CloudQuery MCP) | Near zero | GitHub did ownership in < 45 days [S] | Low | GitHub MCP (34.6k stars), Steampipe MCP, CloudQuery MCP; **no unified permission model** | Good for ownership; weak for cross-tool scorecards and actions |
| **Other OSS** (OpenChoreo, ForgePortal, OpsCat, Frontkeep, idp-core) | Free | Varies | Your own risk | Some MCP | Only OpenChoreo has traction, and it is Kubernetes- and Backstage-based |
| **Seikika** | Free licence; self-hosted Postgres + object storage + connector hosting + on-call | Unmeasured ([red-team.md](./red-team.md) A1; starter portals deferred) | Framework maintenance on an unproven project with one maintainer | Designed agent-first (unbuilt) | Unproven; no reference deployment |

---

## 5. Market facts (with source quality)

| Fact | Source quality |
|---|---|
| 90% of organisations have adopted platform engineering; 76% have dedicated platform teams (DORA 2025, n≈5,000) | [S], survey; definitions loose |
| Gartner: portals at 60% of platform-team organisations in 2025 → 85% by 2028 | [S], vendor summary of a paywalled report |
| 45.3% struggle with adoption; 29.6% don't measure; organisational barriers dominate (platformengineering.org Vol. 4, n=518) | [S], community survey |
| Backstage ~3,400 known adopters; ~270 *public* adopters (Oct 2024) | [S], CNCF/Roadie |
| "~10% internal adoption" outside Spotify | [S], attributed quote via secondary blogs; low quality |
| IDP market size: $1B–$10B (2025), depending on definition; portal-only ~9% CAGR | [S], market-research firms; low quality |
| Port $800M valuation; DX ~$1B exit; Cortex ~$470M | [S], press |
| Compass end of sale May 2026; end of support 2027 | [S], Atlassian partner posts |
| Retool 2026 (n=817 of its own users): 35% replaced at least one SaaS tool with a custom build; internal admin tools 33% at risk | [S], vendor survey with a biased sample |

---

## 6. Build-vs-maintain economics

| Cost driver | Seikika | Backstage | Buy (Port/Cortex) |
|---|---|---|---|
| Who maintains the framework | The Seikika project: currently one person and agents | Spotify + CNCF community (36 contributors in one release) [P] | The vendor |
| Languages in the core | Rust, Go (cgo), TS, Python, React/TSX, CEL, OpenFGA DSL, WIT | TypeScript | n/a |
| Pre-1.0 dependencies on the critical path | `dbsp` 0.385, DBOS Rust 0.6, cel-rust 0.16 (+ a new type checker) [P] | Mostly stable; frontend system 1.0 in v1.57 [P] | n/a |
| Production features that are paid elsewhere | DBSP checkpoints and state-preserving reconfiguration (Feldera Enterprise) [P] | None | Included |
| Adopter-side staffing | Unknown. Self-host + connector hosting + on-call ([red-team.md](./red-team.md) A1) | "3+ FTE" (secondary) | Low |
| Exit cost if the project stalls | High: a bespoke runtime, protocol and SDKs | Medium: large ecosystem, many vendors | Medium: data export, Compass-style migrations |

**Inference.** The adopter pays twice: once to run Seikika, and again as risk on a framework whose
maintenance is unfunded. For a 500–5,000-engineer company, that is strictly worse than Backstage on
risk, and worse than buying on time to value. Seikika wins only on licence cost and on a set of
design properties no survey says buyers rank.

---

## 7. Experiment: naive full recomputation

**Question.** Is incremental view maintenance needed to meet [plan.md](./plan.md) §6.6?

**Setup** [P]:
- **Machine and toolchain:** 4 vCPU Intel Xeon @ 2.1 GHz; rustc/cargo 1.97; release build.
- **CEL:** cel-rust `cel` 0.16.0 at commit `004a4b8`, used as an unmodified interpreter.
- **Data:** synthetic entities with 15 fields (strings, ints, bools, lists, a map).
- **Rules:** 100 rules from 10 templates covering `has()`, `in`, `exists`, `all`, ternaries,
  `startsWith` and `size`.
- **Per-entity work:** a fresh CEL context per entity (deliberately wasteful); rayon for parallelism.
- **Graph:** closure over 1,000 teams in a five-way tree and 10,000 users with three memberships each,
  plus a services-per-team roll-up.
- **Code:** [`experiments/full-recompute`](../experiments/full-recompute) (cel-rust pinned to `004a4b8`).
  Re-run on the same machine type: 7.15 s single-threaded, 1.75 s on 4 threads at 100k × 100.

**Results:**

| Entities | Rules | Evaluations | 1 thread | 4 threads |
|---|---|---|---|---|
| 100,000 | 100 | 10M | 6.32 s (632 ns/eval) | 2.00 s |
| 250,000 | 100 | 25M | 17.85 s | 4.40 s |
| Closure + roll-up | — | — | 5–7 ms | — |

**Caveats.**
- No cross-entity aggregates, 2M-relation joins, authz tuple derivation or search indexing.
- Synthetic rules; no I/O. Building the entity values took 0.4 s and was excluded.

**Reading.**
- Even with these omissions, a full recompute sits about **50× (1 thread) to 150× (4 threads) below** the < 5 min full-rescore SLO
  and well inside the < 60 s freshness SLO.
- Re-scoring only the entities touched by a webhook costs microseconds.
- This doesn't prove DBSP is useless. It shows DBSP isn't *required* for v1, so its maturity risks
  (H5) are optional.

---

## 8. The strongest single argument against building Seikika

> **Seikika spends its novelty where buyers can't see it, and competes where the market has already
> converged.**
>
> What a platform lead buys is permission-aware, agent-accessible answers about ownership and standards,
> with an audit trail. By late 2026, Backstage (free, CNCF), Port, Cortex, OpsLevel, Roadie, Harness and
> RHDH all ship this over MCP. The minimal version (CODEOWNERS + custom properties + the GitHub MCP
> server) costs almost nothing, and GitHub did its own ownership cleanup in under 45 days.
>
> Seikika's distinctive investments are:
> - an incremental circuit, unnecessary at the stated scale (§7);
> - embedded Go-in-Rust authorization with no precedent;
> - polyglot SDKs that Pulumi needed a nine-figure company to sustain;
> - a new connector protocol.
>
> None of these appears in any survey of why platforms fail. The surveys say platforms fail on adoption,
> measurement and culture. One maintainer cannot carry this scope, and the one experiment that would
> justify the thesis (agents do better with Seikika's context) is scheduled last.

---

## 9. The cheapest alternative that gets ~80% of the value

For an organisation **without** a portal today:

1. **Ownership record:**
   - `catalog-info.yaml` in each repo, so it stays Backstage-compatible and portable.
   - CODEOWNERS as the authority, as Seikika itself chose in D17.
   - GitHub custom properties (`ownership-type`, `ownership-name`) synced from them, the pattern
     GitHub used.
2. **Standards:** a scheduled GitHub Action (or rulesets plus a small script) that evaluates checks
   per repo and writes results to custom properties and a dashboard issue.
3. **Agent access:**
   - The GitHub MCP server, plus PagerDuty, Datadog and IdP MCP servers where available.
   - One **thin read-only "ownership MCP" server** (Postgres or SQLite, full recompute on webhook)
     with three or four tools: `owner_of`, `dependents_of`, `standards_for`, `explain`.
   - Authorization mirrors GitHub team membership.
4. **Cloud inventory, if needed:** Steampipe or CloudQuery MCP.

For an organisation **with** Backstage: enable `@backstage/plugin-mcp-actions-backend` with OAuth
(CIMD) and add Tech Insights or Soundcheck scorecards. If the budget allows, buy Roadie or Spotify
Portal to offload upgrades.

**What this loses vs. Seikika [I]:**
- field-level source of truth;
- `plan` previews of permission and score diffs;
- facet-level information-flow control;
- durable actions with approvals;
- one authorization model across all interfaces.

These are real, but they are the remaining ~20%, and each could ship as a Backstage module or a
standalone tool.

---

## 10. Kill criteria and cheap experiments

Run these **before** further investment in the circuit, the FFI, the SDKs or TCP. Each has a
pre-registered threshold. No dates are given (D25); order them by cost.

| # | Experiment (cheap) | Falsifies | Kill or pivot if… |
|---|---|---|---|
| E1 | **Demand interviews:** platform leads at ten 500–5,000-engineer organisations; ask them to rank Seikika's differentiators against their top three portal pains, and ask for a design-partner commitment | H7 | Fewer than 3 commit to a pilot with a named problem that Backstage MCP or Port doesn't solve → **kill** |
| E2 | **Agent-value A/B, moved from Phase 4 to first:** 30 scripted "who owns / what depends / is X compliant and why" tasks on a real org. Arms: (a) GitHub MCP + CODEOWNERS only; (b) Backstage MCP actions; (c) the thin ownership MCP from §9; (d) a Seikika prototype. Measure success, tokens and wrong answers | H1, H2 | (d) does not beat the best of (a)–(c) by ≥ 15 percentage points at equal tokens → **kill the framework**; ship (c) |
| E3 | **Thin-server ceiling:** an agent builds §9's ownership MCP on Postgres with full recompute. Score it against the Phase 1 exit criteria ([plan.md](./plan.md) §8) | H1, H4 | It meets ≥ 80% of the Phase 1 exit criteria → **pivot** to that architecture |
| E4 | **Extend §7:** add aggregate rules, 2M relations and authz tuple derivation at 100k entities | H4 | A full recompute stays < 30 s → **drop DBSP from v1** (D21 reversed) |
| E5 | **Novel-stack agent productivity:** have agents implement the Phase 0 "manifest → DBSP circuit with recursive closure" spike, and an equivalent feature on Next.js + Postgres. Log human interventions and defects found by tests | H6 | The novel stack needs ≥ 3× the human interventions → **drop** the "built with agents, so scope is fine" premise and cut scope |
| E6 | **Backstage parity audit:** for every Phase 1 feature, check Backstage 1.57 + existing plugins | H2, H9 | ≥ 80% present → **pivot** to Backstage modules (CODEOWNERS-authority processor, OpenFGA permission policy, `plan`-style catalog diff in CI) |
| E7 | **Sustainability gate** | H8 | No second committed human maintainer and no funding or foundation path before Phase 1 → **do not release** a framework that others would depend on |
| E8 | **Dependency gate:** DBOS Rust reaches a production-supported release; cel-rust or Seikika ships a type checker passing the cel-spec checker tests; an OpenFGA c-archive passes the conformance corpus under signal and shutdown stress | H5 | Any of these fails in Phase 0 → use the boring fallback (Temporal or Restate, or a Postgres job table; cel-go via a sidecar or a CEL subset; OpenFGA as a sidecar) |

---

## 11. What would have to be true for Seikika to be worth it

1. **E2 passes.** Agents measurably do better with a curated, permission-shaped ownership and standards
   graph than with direct MCP access to sources and incumbents. Today there is no evidence for this,
   and the adjacent evidence (ETH 2026) points the other way.
2. **There is a segment incumbents won't serve:** organisations that cannot buy SaaS (sovereign,
   regulated or air-gapped), refuse to run a TypeScript monolith, and want Git-reviewed config with
   `plan` diffs. This segment exists, but its size is unknown. E1 should size it.
3. **The scope shrinks to a wedge:** the "ownership and standards for agents" release on boring
   technology:
   - Postgres with full recompute;
   - an OpenFGA sidecar;
   - a YAML manifest plus one TypeScript SDK;
   - no TCP spec yet.
   This is what [principal-review.md](./principal-review.md) §5 proposed. Seikika's real
   differentiators (`plan` on the PR, CODEOWNERS authority, field-level sources, facets) survive this
   cut.
4. **People and money:** at least two committed human maintainers and a funding or foundation path
   ([red-team.md](./red-team.md) A6).
5. **Backstage compatibility as a bridge:** read `catalog-info.yaml` and be usable *from* Backstage, so
   adopters don't have to bet the organisation on a new framework.

If 1 and 2 hold, a narrowed Seikika is defensible. If 1 fails, nothing else in the design rescues it.

---

## 12. Corrections to existing documents (not edited here)

| Document | Claim | Correction |
|---|---|---|
| [research.md](./research.md) §2, §4 | Backstage MCP "external auth relies on static tokens … a temporary workaround" | Out of date. Backstage MCP actions support **OAuth via CIMD**; static tokens are a fallback only for apps on the old frontend system (`docs/ai/mcp-actions.md`, v1.50–1.54) [P] |
| [research.md](./research.md) §5 | The Rust agent-first niche "is beginning to fill … validates both the language choice and the timing" | Frontkeep has 3 stars and OpsCat 2 [P]. This is not validation |
| [research.md](./research.md) §3 | Compass "offers `compass.yml` config as code" | Compass reached **end of sale on 13 May 2026** and is being folded into DX [S] |
| [design/incremental-computation.md](./design/incremental-computation.md) §2 | DBSP is "embeddable" with checkpoints and spill as mitigations | True of the MIT crate, but Feldera ships checkpoints, fault tolerance and state-preserving pipeline changes **only in Enterprise**. Seikika must build that orchestration itself [P] |
| [decisions.md](./decisions/README.md) Q10 | "cel-rust conformance level: measure in Phase 0" | Measured: 688 of ~2,771 cel-spec conformance tests skipped; no type checker (commit `004a4b8`) [P] |

---

## 13. Claims I could not verify

- **Backstage "10% internal adoption":** I could not reach the original New Stack article. The figure
  circulates via secondary blogs and forums.
- **Pricing for Port, Roadie, Cortex, OpsLevel and Spotify Portal:** all third-party listings or Vendr
  medians; no vendor pricing page was reachable.
- **METR figures:** both the 2025 RCT and the Feb 2026 update come from search excerpts and
  secondary write-ups. metr.org and arxiv.org were unreachable.
- **ETH AGENTS.md study magnitudes:** I saw the abstract-level claims only. Per-condition numbers come
  from secondary blogs.
- **DORA 2025, Gartner 2025, Perforce 2026, platformengineering.org Vol. 4:** all from summaries.
- **GitHub's "durable owner" post:** read via mirrors; repository counts conflict (11k vs 14k).
- **Compass end-of-support dates:** sources conflict (June–December 2027).
- **Pulumi funding and headcount:** sources conflict widely.

**Primary sources read locally (commits):**
- `backstage/backstage` `184acad`
- `feldera/feldera` `8ff1805`
- `dbos-inc/dbos-transact-rust` `bad825f`
- `cel-rust/cel-rust` `004a4b8`
- Repository statistics via `gh api search/repositories`, 2026-11-19.

---

## Sources

Primary (read from GitHub or run locally):
- Backstage: [docs/ai/mcp-actions.md](https://github.com/backstage/backstage/blob/master/docs/ai/mcp-actions.md) ·
  [docs/ai/skills.md](https://github.com/backstage/backstage/blob/master/docs/ai/skills.md) ·
  [docs/ai/index.md](https://github.com/backstage/backstage/blob/master/docs/ai/index.md) ·
  [v1.57.0 release notes](https://github.com/backstage/backstage/blob/master/docs/releases/v1.57.0.md) ·
  [v1.54.0](https://github.com/backstage/backstage/blob/master/docs/releases/v1.54.0.md) ·
  [v1.55.0](https://github.com/backstage/backstage/blob/master/docs/releases/v1.55.0.md) ·
  [v1.51.0](https://github.com/backstage/backstage/blob/master/docs/releases/v1.51.0.md) ·
  [v1.50.0](https://github.com/backstage/backstage/blob/master/docs/releases/v1.50.0.md)
- Feldera: [LICENSE (MIT + Enterprise)](https://github.com/feldera/feldera/blob/main/LICENSE) ·
  [fault-tolerance-overview.md](https://github.com/feldera/feldera/blob/main/docs.feldera.com/docs/pipelines/fault-tolerance-overview.md) ·
  [modifying.md (bootstrapping, Enterprise-only)](https://github.com/feldera/feldera/blob/main/docs.feldera.com/docs/pipelines/modifying.md) ·
  [changelog.md](https://github.com/feldera/feldera/blob/main/docs.feldera.com/docs/changelog.md) ·
  [crates/connectors/src/controller.rs](https://github.com/feldera/feldera/blob/main/crates/connectors/src/controller.rs) ·
  [crates/dbsp](https://github.com/feldera/feldera/tree/main/crates/dbsp)
- [dbos-inc/dbos-transact-rust README](https://github.com/dbos-inc/dbos-transact-rust)
- cel-rust: [README](https://github.com/cel-rust/cel-rust) ·
  [conformance/src/bin/ignored.txt](https://github.com/cel-rust/cel-rust/blob/master/conformance/src/bin/ignored.txt)
- OpenFGA issues: [#3346](https://github.com/openfga/openfga/issues/3346) · [#3393](https://github.com/openfga/openfga/issues/3393)
- Repositories (stars/archived via `gh api search/repositories`): [lyft/clutch](https://github.com/lyft/clutch) ·
  [github/github-mcp-server](https://github.com/github/github-mcp-server) · [glemmestad/frontkeep](https://github.com/glemmestad/frontkeep) ·
  [sumant1122/opscat](https://github.com/sumant1122/opscat) · [Decathlon/internal-developer-platform](https://github.com/Decathlon/internal-developer-platform) ·
  [cortexapps/engrams](https://github.com/cortexapps/engrams) · [openchoreo/openchoreo](https://github.com/openchoreo/openchoreo) ·
  [turbot/steampipe](https://github.com/turbot/steampipe) · [cloudquery/cloudquery](https://github.com/cloudquery/cloudquery)

AI productivity and agent context (secondary):
- [METR RCT, arXiv 2507.09089](https://arxiv.org/abs/2507.09089) · [METR uplift update, Feb 2026](https://metr.org/blog/2026-02-24-uplift-update/) ·
  [Rob Bowley on the 2026 update](https://blog.robbowley.net/2026/04/04/metrs-developer-productivity-research-2026-update/)
- [Evaluating AGENTS.md, arXiv 2602.11988](https://arxiv.org/abs/2602.11988) · [ICLR 2026 listing](https://iclr.cc/virtual/2026/10021142)
- [DORA 2025 report (PDF)](https://services.google.com/fh/files/misc/2025_state_of_ai_assisted_software_development.pdf) ·
  [InfoQ on DORA 2025](https://www.infoq.com/news/2025/09/dora-state-of-ai-in-dev-2025)
- [GitClear coverage (LeadDev)](https://leaddev.com/software-quality/how-ai-generated-code-accelerates-technical-debt) ·
  [Veracode 2025 GenAI code security](https://www.veracode.com/press-release/ai-generated-code-poses-major-security-risks-in-nearly-half-of-all-development-tasks-veracode-research-reveals/)
- [Anthropic: code execution with MCP](https://www.anthropic.com/engineering/code-execution-with-mcp) ·
  [Anthropic: advanced tool use](https://www.anthropic.com/engineering/advanced-tool-use)
- [CodeScaleBench (Sourcegraph)](https://sourcegraph.com/resources/ebooks/code-scale-bench-report)
- [Retool 2026 Build vs. Buy](https://retool.com/blog/ai-build-vs-buy-report-2026) · [Retool AI governance report 2026](https://retool.com/blog/ai-governance-report-2026)

Market, surveys and funding (secondary):
- [Gartner Market Guide for IDPs (abstract)](https://www.gartner.com/en/documents/6306515) ·
  [Cortex summary of Gartner 2025](https://www.cortex.io/post/cortex-recognized-again-as-a-representative-vendor-in-the-2025-gartner-market-guide-for-internal-developer-portals)
- [platformengineering.org Vol. 4](https://platformengineering.org/blog/announcing-the-state-of-platform-engineering-vol-4) ·
  [Biggest challenges 2026](https://platformengineering.org/blog/the-biggest-challenges-platform-engineering-teams-are-facing-in-2026)
- [Perforce/Puppet 2026 State of Platform Engineering](https://www.puppet.com/resources/2026-state-of-platform-engineering) ·
  [InfoQ analysis](https://infoq.com/news/2026/08/perforce-maturity-ai-success)
- [Port 2025 State of IDPs](https://www.port.io/state-of-internal-developer-portals) ·
  [TechCrunch: Port $100M at $800M](https://techcrunch.com/2025/12/11/port-raises-100m-at-800m-valuation-to-take-on-spotifys-backstage) ·
  [Port: Backstage is dead](https://newsletter.port.io/p/backstage-is-dead)
- [Cortex Series C](https://www.cortex.io/posts/announcing-series-c) · [Bloomberg/BNN: Cortex $470M](https://bnnbloomberg.ca/business/company-news/2024/09/04/sequoia-backs-developer-startup-cortex-at-470-million-valuation)
- [Atlassian: The next chapter for Compass](https://www.atlassian.com/blog/announcements/the-next-chapter-for-compass) ·
  [Adaptavist: Compass end of sale](https://www.adaptavist.com/blog/what-you-need-to-know-about-atlassian-compass-end-of-sale) ·
  [Cortex: Compass end of life](https://www.cortex.io/post/atlassian-compass-end-of-life-what-to-do-next)
- [Roadie: Backstage Weekly 117 (3,400 adopters)](https://roadie.io/backstage-weekly/117-spotify-plugins-v-1-43-tech-insights-legacy-backend-eol/) ·
  [CNCF: Backstage certification (3,000+ adopters)](https://www.cncf.io/blog/2024/11/15/internal-developer-platforms-at-scale-with-the-certified-backstage-associate-cba-certification/) ·
  [tianpan.co forum (10% / 89% claims, low quality)](https://tianpan.co/forum/t/backstage-holds-89-market-share-but-only-10-internal-adoption-why/417)
- [Feldera $21.5M](https://www.thesaasnews.com/news/feldera-raises-21-5m-series-a/) · [DBOS $8.5M seed](https://blocksandfiles.com/2024/03/13/dbos-raises-8-5m/) ·
  [Pulumi funding (Salestools)](https://salestools.io/report/pulumi-145m-series-d-2024) · [GeekWire: Pulumi $41M](https://www.geekwire.com/2023/cloud-startup-pulumi-raises-41m-from-madrona-nea-to-grow-infrastructure-as-code-platform/)
- IDP market size (low quality): [SNS Insider](https://www.snsinsider.com/reports/internal-developer-platform-market-10643) ·
  [Mordor Intelligence](https://www.mordorintelligence.com/industry-reports/platform-engineering-and-internal-developer-platform-idp-market)

Vendors' agent features and pricing (secondary):
- [Toolradar: Port pricing](https://toolradar.com/tools/port/pricing) · [Port pricing page](https://www.port.io/pricing-new) ·
  [Toolradar: Roadie pricing](https://toolradar.com/tools/roadie/pricing) · [Vendr: Cortex](https://www.vendr.com/marketplace/cortex) ·
  [Vendr: OpsLevel](https://vendr.com/marketplace/opslevel)
- [Roadie MCP servers](https://roadie.io/docs/api/roadie-mcp/) · [Roadie Context Store MCP](https://roadie.io/docs/context/mcp-servers/overview/) ·
  [OpsLevel MCP docs](https://docs.opslevel.com/docs/mcp) · [OpsLevel: richer context for agents](https://www.opslevel.com/resources/richer-context-for-humans-and-ai-agents) ·
  [Cortex MCP GA](https://www.cortex.io/post/a-new-era-for-your-developer-portal-cortex-mcp) ·
  [Harness MCP server redesign](https://www.harness.io/blog/harness-mcp-server-redesign) ·
  [Red Hat: MCP in Developer Hub](https://developers.redhat.com/articles/2025/11/10/mcp-red-hat-developer-hub-chat-your-catalog) ·
  [SD Times: RHDH agent-ready](https://sdtimes.com/open-source/red-hat-advances-red-hat-developer-hub-as-an-agent-ready-enterprise-developer-portal/) ·
  [SD Times: Spotify Portal GA](https://sdtimes.com/softwaredev/spotify-portal-now-generally-available-and-packed-with-features-for-improving-dev-experience/) ·
  [InfoQ: Spotify Portal Studio](https://www.infoq.com/news/2026/03/spotify-portal-studio) ·
  [Datadog IDP](https://docs.datadoghq.com/internal_developer_portal/)
- [Spotify Soundcheck performance (in-memory catalog)](https://backstage.spotify.com/docs/plugins/soundcheck/performance) ·
  [Roadie: Scaling Backstage](https://roadie.io/blog/scaling-backstage-intro)

Minimal and "no portal" options:
- [GitHub: How GitHub gave every repository a durable owner](https://github.blog/security/application-security/how-github-gave-every-repository-a-durable-owner/) ·
  [GitHub custom properties best practices](https://wellarchitected.github.com/library/governance/recommendations/managing-repositories-at-scale/custom-properties-best-practices/)
- [Steampipe MCP](https://steampipe.io/docs/query/mcp) · [CloudQuery: AI for cloud asset inventory](https://www.cloudquery.io/blog/ai-for-analyzing-your-cloud-asset-inventory)
- [Ben Edmunds: AI skills are the new platform interface](https://benedmunds.com/management/2026/04/02/ai-skills-are-the-new-platform-interface)

Open-source sustainability:
- [Tidelift 2024 maintainer report (Cyentia library)](https://library.cyentia.com/report/report_023871.html) ·
  [Socket: the unpaid backbone of open source](https://socket.dev/blog/the-unpaid-backbone-of-open-source) ·
  [Sonar summary](https://www.sonarsource.com/resources/library/open-source-maintainers/)
- [Earthly: a message about Earthly (2025)](https://earthly.dev/blog/shutting-down-earthfiles-cloud/)

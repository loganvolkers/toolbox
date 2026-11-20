# Federation: Standards That Start in One Repo and Graduate to the Company

*Status: design only, 2026-11-20 (D68). **Nothing here is implemented, and no implementation is
planned until repo portals have been used by a few teams.** Builds on
[repo-portals.md](./repo-portals.md) (what exists today), [ratchets.md](./ratchets.md),
[meta-catalog.md](./meta-catalog.md), [source-of-truth.md](./source-of-truth.md) and
[authorization.md](../authorization.md).*

---

## 1. Conclusion

**Federate configuration down and results up, with no runtime coupling between portals.**

A Toolbox deployment becomes a tree (or DAG) of portals:
- **company → division → org → team → repo**, where every level is optional;
- each portal is an ordinary Toolbox portal: a manifest, its connectors, its data.

| Flow | What moves | How (proposed) |
|---|---|---|
| **Standards down** | Kinds, fields, scorecards, policies and checks published by a parent | Versioned **standard packages** a child `include`s by name and digest. The parent's rules arrive read-only, so a child can add and tighten but never weaken them |
| **Results up** | Each child's scorecard results, evidence counts and its own entities | The parent **reads children's committed report files** (or their ingest pushes, later) as ordinary connector records. No portal calls another at runtime |
| **Graduation** | A standard moves from one team to the company | Each move is the same rule changing home: team package → org package → company package. A stage tag (`experimental → advisory → mandatory`) controls enforcement, and `toolbox plan` shows the fleet impact first |

Why this shape:
1. It matches how standards actually spread. A team invents a CODEOWNERS check. Neighbours copy it.
   The platform org measures it everywhere. Then the company mandates it with a ratchet, so nobody
   gets worse and a burn-down lets everyone get better. Betterer stopped at the copying step, because
   it has no fleet view and no promotion path ([ratchets.md](./ratchets.md) §2).
2. It answers the go-to-market critique ([adversarial-critique.md](../adversarial-critique.md)).
   Toolbox can start in a handful of repos with zero infrastructure (`toolbox report` in CI) and grow
   into central governance without a migration. Every repo portal is already a future child.
3. It keeps charter goal G1. A child portal is a file in a repo plus CI, and only the root needs a
   server. G3 holds too: packages and report files are plain, documented formats.

## 2. Vocabulary

| Term | Meaning |
|---|---|
| **Portal node** | One portal in the tree: root (company), intermediate (division, org, team) or leaf (repo). Leaves usually run only `toolbox report`; inner nodes may run `serve` |
| **Standard package** | A versioned bundle of manifest fragments (kinds, fields, scorecards, policies, connector configs) published by a node, e.g. `@acme/standards-company@3.2.0`. The unit of graduation |
| **Subgraph** | The entities and fields a node owns. Borrowed from Apollo Federation (secondary source): subgraphs own types and fields, and the gateway composes them. Here a child may *extend* a parent's kind with its own fields but not redefine the parent's |
| **Stage** | A scorecard's or rule's enforcement level: `experimental` (visible only in its own node), `advisory` (reported up, never fails CI), `mandatory` (ratcheted in CI where it applies) |
| **Waiver** | An owned, expiring exception to a mandatory rule for one entity, recorded as data (a native value with author, reason and expiry), never as a manifest edit in the child |

## 3. Standards down: packages and composition

```yaml
# repo portal: payments-api/.toolbox/toolbox.yaml
include:
  - package: "@acme/standards-company@^3"   # mandatory + advisory company standards
    digest: sha256:…                        # pinned; `toolbox plan` shows the impact of upgrades
  - package: "@acme/standards-payments@^1"  # the payments org's own (some still experimental)
scorecards:
  - name: payments-idempotency               # this team's experiment
    stage: experimental
    …
```

**Composition rules**

1. **Namespaces.** Each package owns its scorecard and kind names, e.g. `company/ownership`. A child
   refers to a parent's names but can't redefine them.
2. **Monotonic tightening.** A child may:
   - add rules to its own scorecards;
   - add stricter `appliesTo` exclusions only through waivers (§6);
   - raise its own target level.

   It may not remove, weaken or re-level a parent's rule. `toolbox validate` enforces this. Parent
   rules are read-only, like ownership in [source-of-truth.md](./source-of-truth.md).
3. **Diamonds by list, not tree.** Matrix organisations and platform teams that span orgs simply
   include several packages. Conflicts between packages (the same kind with different field types)
   are validation errors that name both owners.
4. **Pinned and planned.** A package upgrade is a manifest change: `toolbox plan` shows the
   entities whose level would drop, before the upgrade lands.

This reuses the existing `include` (D54) and the canonical manifest (D53). What's new is fetching
and pinning packages from a registry: an OCI artifact, a git ref, or npm, alongside the
`@toolbox/manifest` builder.

## 4. Results up: how a parent sees its children

| Option | Mechanism | Pros | Cons |
|---|---|---|---|
| **A. Pull committed reports** (recommended first) | Children commit `toolbox report` baselines (`toolbox-report/v1`, already built). The parent's `git-files` connector reads them across the org's repos | No new protocol; works with leaf CI only; history is git history; the parent can't be "down" for children | Coarse: levels and evidence counts, not entity details; as fresh as the default branch |
| **B. Push to the parent's ingest API** | Child CI runs `toolbox report -o json` and posts records to the parent as connector `child:<repo>` | Fresher; richer (child entities and fields) | Needs credentials per child; the parent must be up; the trust boundary is subtle (a child asserting facts about itself) |
| **C. Parent runs the child's checks** | The parent clones repos and runs their `toolbox report` itself (Scorecard-style fleet scan) | Uniform; doesn't trust child CI | Compute cost; needs build environments for every ecosystem |

**Recommendation.** Start with A. Use C for checks that don't need a build: the static
`workspace`-type facts that OpenSSF Scorecard proves can run fleet-wide. Use B only for children
that run their own `serve`.

**Trust.**
- Results a child reports about itself are claims. A parent scorecard can require them to be
  *verified*: computed by the parent (C), or signed by CI (OIDC-attested report artifacts).
- This mirrors CODEOWNERS-verified ownership (D17).

**Identity.**
- Entity ids are global (`repository:acme/payments-api`), so a child's report merges onto the
  parent's entity.
- Precedence follows the source-of-truth rules: the parent's own connector facts outrank
  child-reported claims for parent-owned fields.

## 5. Multi-layer hierarchies

```
company (root; serve; owns company/*)
├── division: consumer          (serve or report; owns consumer/*)
│   ├── org: payments           (report in its standards repo; owns payments/*)
│   │   ├── team: checkout      (package only, no portal)
│   │   └── repo: payments-api  (report in CI: includes company/*, consumer/*, payments/*)
│   └── repo: web-shell
└── org: platform               (serve; owns platform/*)
```

- **Any node may be skipped.** A repo can include the company package directly.
- **Where the hierarchy comes from.** Group membership comes from the IdP (SCIM, as today). The node
  tree is configuration: each package declares its owning group. The root's meta catalog
  ([meta-catalog.md](./meta-catalog.md)) shows the tree, which package versions each child runs, and
  adoption per stage.
- **Who sees what.** OpenFGA relations follow the tree:
  - a parent's operators can read their descendants' results;
  - a team sees its own results and the standards it inherits;
  - siblings see each other only if the parent grants it.
- **Rolling up scores.** A parent scorecard can aggregate children ("org level = the lowest level any
  repo reaches"). These are computed fields over inbound relations, but they need multi-hop
  derivation ([toolbox-for-toolbox.md](./toolbox-for-toolbox.md) F3) once trees are deeper than one
  level.

## 6. Graduation: a worked example (CODEOWNERS quality)

| Stage | Where the rule lives | Enforcement | What `toolbox` shows |
|---|---|---|---|
| 1. **Experiment** | `checkout` team's repo portal: `codeowners-quality` with rules `has-codeowners`, `owners-are-teams`, `covers-catalog-info`; `stage: experimental` | Only in that repo's CI (ratchet) | Its own report |
| 2. **Adopted** | Moved into `@acme/standards-payments`; 6 repos include it | Each including repo's ratchet | The payments org's view: 6 repos, levels, burn-down |
| 3. **Proposed** | Copied into `@acme/standards-company` as `stage: advisory` | None; results roll up | `toolbox plan` on the company package: "1,240 repos in scope; 410 would fail bronze". The owner, now platform, reviews fixes and false positives |
| 4. **Mandated** | `stage: mandatory` in the company package; old copies removed from the payments package | Every repo's ratchet: **no regressions from today**, so existing failures are baselined, not blocking. Waivers for justified exceptions, with expiry | Burn-down per org; waivers expiring; repos still below bronze |
| 5. **Target** | A level date: "bronze everywhere by Q3" is the goal, the ratchet is the path | Optional hard gate once burn-down is done | — |
| 6. **Retired** | Removed or replaced; `plan` shows nobody depends on it | — | — |

The ratchet is what makes mandating safe. Mandating a standard never breaks a build on day one: every
repo is baselined at its current state (as ESLint's bulk suppressions do for one repo, [ratchets.md](./ratchets.md) §6),
and only regressions fail.

## 7. What federation must not become

- **Runtime coupling.** No portal calls another to answer a query. Parents read artifacts and records
  on their own schedule.
- **Configuration drift by copy.** Standards travel as versioned packages, never copy-pasted YAML. A
  copied rule is a fork with a new owner.
- **Silent weakening.** A child that excludes itself from a mandatory rule must do it with a visible,
  owned, expiring waiver.
- **A second source of truth.** A child's report is an input (a claim) to the parent. The parent's
  own facts win for fields it owns.

## 8. Phasing (no implementation now)

| Phase | Scope | Exit |
|---|---|---|
| F0 (done) | Repo portals: `toolbox report`, ratchet baselines, static HTML ([repo-portals.md](./repo-portals.md)) | This repo's CI runs its own ratchet |
| F1 | Standard packages: remote `include` by name and digest; namespaced names; monotonic-tightening validation; `plan` over package upgrades | One package used by two repo portals; a weakening attempt fails validation |
| F2 | Results up (option A): a root portal reads children's committed reports; adoption and burn-down views | A root shows three repos' ratchets and their package versions |
| F3 | Stages and waivers: `stage:` on scorecards and rules; waivers as owned, expiring native values; enforcement by stage | The CODEOWNERS example (§6) runs end to end on three repos |
| F4 | Hierarchies: node tree in the meta catalog; OpenFGA relations by subtree; roll-up scorecards (needs multi-hop derivation, F3 of toolbox-for-toolbox) | A two-level org rolls up correctly with per-subtree visibility |
| F5 | Push (option B) and verified results (signed CI artifacts or parent-run checks) | A parent rejects an unsigned child claim for a verified rule |

## 9. Open questions

| # | Question |
|---|---|
| FQ1 | 🚩 **Researched; recommendation awaiting decision ([packages.md](./packages.md) §4: git-first with an OCI mirror, pinned by digest).** The package registry: OCI artifacts (fits "one image"), git refs (fits portal repos), or npm (fits the TS builder)? Possibly all three behind one resolver. Phase F1 can't start until this is researched |
| FQ2 | Should a child be able to *extend* a parent kind with fields, Apollo-style, or only add its own kinds? Extension is powerful but makes the parent's schema depend on children |
| FQ3 | How are waivers approved? This is the approvals design from M5, which is paused (D64), so revisit after the self-service review |
| FQ4 | Do repo portals need entity identity beyond the repo (e.g. services inside a monorepo) to merge cleanly at the parent? This is [identity-resolution.md](./identity-resolution.md) at the federation boundary |
| FQ5 | What is the minimal parent: does a root need `serve`, or can a static site aggregate children's reports ([repo-portals.md](./repo-portals.md) §5)? |

## Sources

- Prior art (secondary unless noted):
  - Apollo Federation (subgraphs composed into a supergraph, entity ownership);
  - Renovate shared presets and ESLint shareable configs (configuration inherited from org-level
    packages);
  - Kustomize overlays (layered configuration without editing the base);
  - OpenSSF Scorecard's per-repo CLI and fleet-wide scan (primary: `ossf/scorecard` README, read
    2026-11-20);
  - ESLint bulk suppressions as a one-repo baseline (primary: `eslint/eslint`
    `docs/src/use/suppressions.md`).
- Toolbox: [ratchets.md](./ratchets.md), [repo-portals.md](./repo-portals.md),
  [toolbox-for-toolbox.md](./toolbox-for-toolbox.md), [meta-catalog.md](./meta-catalog.md),
  [source-of-truth.md](./source-of-truth.md), [adversarial-critique.md](../adversarial-critique.md),
  [decisions.md](../decisions.md) D17, D53, D54.

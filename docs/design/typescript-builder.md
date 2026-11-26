# TypeScript Builder (`@seikika/manifest`)

*Status: design, 2026-11-20 (decisions D53–D55). Replaces the Pulumi-style SDK sketches in
[plan.md](../plan.md) §2 and [extensibility.md](./extensibility.md) for v1. Prototype and measurements:
[experiments/typescript-builder](../../experiments/typescript-builder/README.md). **Implementation status:** prototype only; `include`-split YAML is implemented in `seikika validate`, and the `@seikika/manifest` package and `seikika build` are not yet built.*

---

## 1. Conclusion

The builder is a **typed library that only produces data**: it prints the same manifest JSON a
person could write in YAML, and nothing at runtime depends on it. The prototype rebuilds the reference
portal's `seikika.yaml` exactly, catches eight classes of mistakes at compile time, and type-checks
in 0.38 s. It needed **no change to the manifest design**, but it settles three questions:

| # | Consequence | Decision |
|---|---|---|
| 1 | The canonical artefact is **one sorted manifest JSON** produced by `seikika build`, whatever the source (one YAML file, several YAML files, or a TS program). The server, `plan` and the manifest hash only see that JSON | D53 |
| 2 | **YAML sources may be split** with a top-level `include` list (globs), so security can own `policies/approved-apps.yaml` through CODEOWNERS, as TS modules allow. `include` exists only in sources; the canonical manifest never contains it | D54 |
| 3 | **The builder never resolves environment variables or secrets.** It emits `${env:…}` and `${secret:…}` placeholders, which the server resolves at start-up. This keeps the manifest identical across environments, so a plan in CI is valid for production | D55 |

## 2. What the builder is and isn't

| It is | It isn't |
|---|---|
| A thin typed layer over the manifest schema | A runtime: the image never contains Node for configuration |
| Ordinary TS: modules, functions, loops, packages on npm | A Pulumi-style engine with language hosts, gRPC or resource graphs (deferred, D41) |
| Optional: YAML remains first-class | A second source of truth: the JSON Schema is |
| Deterministic: same source, same JSON | Allowed to read the clock, randomness, the network or the environment |

`seikika build` (and `seikika dev`) run `seikika.config.ts` with Node when it exists, take its
printed JSON, merge nothing else, validate it, and embed it in the image. Node is needed on developer
machines and in CI only.

## 3. API shape (from the prototype)

```ts
import { action, cel, connector, field, kind, scorecard, toManifest } from "@seikika/manifest";

const github = connector("github", {
  package: "builtin:github", runtime: "in-image",
  records: ["repository", "codeowners", "app", "installation"],
  freshness: { repository: "15m" },                 // keys checked against `records`
});

const Repository = kind("Repository", {
  identityFrom: [github.source],                      // "connector:github"
  fields: {
    tier: field.enum(["0", "1", "2", "3"]),
    archived: field.boolean(),
    ownership: field.codeowners({ claim: "ownerClaim", rules: "codeowners",
                                  path: "catalog-info.yaml", teamField: "Group.githubTeam" }),
    // …
  },
  sources: { default: [github.source], fields: { tier: ["git:catalog-info", "native"] } },  // keys checked
  map: [{ from: github.record("repository"),          // only declared record kinds
          id: cel`'repository:' + record.name`,
          set: cel`{'archived': record.archived}` }],
});

const R = Repository.f;                               // typed field paths: R.archived === "entity.archived"
scorecard("ownership", {
  kind: Repository, owner: "group:platform", levels: ["bronze", "silver"],
  rules: [{ name: "branch-protected", level: "silver", check: cel`${R.defaultBranchProtected}` }],
});
```

Rules the prototype established:

- **Objects carry their names.** `kind`, `connector` and `uiBundle` return values that other
  definitions reference directly (`kind: Repository`, `refresh: [{ connector: github }]`), so renames
  are refactors and dangling names are compile errors.
- **CEL stays a string**, written with the `cel` tag. Interpolated `FieldRef`s (`${R.tier}`) are
  checked by `tsc`; the rest of the expression is checked by `seikika validate` (CEL compile with the
  declared fields). Typing CEL fully in TypeScript isn't worth it: it would duplicate the CEL checker.
- **Literal unions need `NoInfer`.** Without it, `levels: ["bronze"]` plus a rule at `"gold"` widens
  the union instead of failing. The builder requires TypeScript ≥ 5.4.
- **Actions must choose** `expect` or `unverified: true` (a discriminated union), matching the
  schema's `anyOf`.

## 4. What tsc catches vs what `seikika validate` catches

| Mistake | tsc | `seikika validate` |
|---|---|---|
| Undeclared record kind in `map.from`, `freshness`, `refresh` | ✓ | ✓ |
| `sources.fields` key that isn't a field | ✓ | ✓ |
| Malformed source id, duration | ✓ | ✓ |
| Rule level not in `levels` | ✓ | ✓ |
| Typed field path that doesn't exist | ✓ | ✓ (CEL compile) |
| Action without `expect` or `unverified` | ✓ | ✓ |
| A source in a precedence list that no map rule provides | — | ✓ |
| CEL syntax and types, variables per site | — | ✓ |
| UI bundle, connector or kind referenced but not included in the portal | — (unless passed as objects) | ✓ |
| OpenFGA relation named by `permission` exists in the model | — | ✓ |

`seikika validate` is the authority; the builder only moves common errors earlier.

## 5. Types and the schema

The prototype hand-wrote its types. For the real package:
- **Generate the data types from `manifest.schema.json`** (e.g. json-schema-to-typescript) so the schema
  stays the single source of truth.
- Keep a small hand-written generic layer on top (record-kind keys, field keys, levels, discriminated
  actions).
- CI runs the round-trip (TS reference portal ≡ YAML reference portal), schema validation of the
  output, and the negative type tests.
- The package version tracks `apiVersion` (`seikika.com/v1alpha1` → `@seikika/manifest@0.1.x`).

## 6. Effect on other designs

- **Manifest schema:** add `include` (sources only, D54). No other change.
- **Operator SDLC:** `seikika build` produces `manifest.json` (canonical) next to the image; `plan` hashes it;
  CI caches on it.
- **Dev loop:** `seikika dev` re-runs the config program on change (≈0.3 s) before recomputing.
- **Agent plugin:** the `seikika-authoring` skill should prefer the builder when `seikika.config.ts`
  exists and YAML otherwise.

## Sources

- Prototype, results and pinned versions: [experiments/typescript-builder](../../experiments/typescript-builder/README.md).
- TypeScript `NoInfer` (TypeScript 5.4 release notes, secondary; not re-read in this session).

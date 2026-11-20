# typescript-builder

**Question** ([v1-spec.md](../../docs/v1-spec.md) §5, D41): can a small typed TypeScript library produce
the v1 manifest exactly, catch real mistakes at compile time, and stay fast enough for the dev loop,
without changing the manifest design?

**Answer (2026-11-20): yes, with three design consequences** (recorded as D53–D55 and explained in
[typescript-builder.md](../../docs/design/typescript-builder.md)).

| Check | Result |
|---|---|
| [`portal/acme.ts`](./portal/acme.ts) rebuilds [`examples/reference-portal/toolbox.yaml`](../../examples/reference-portal/toolbox.yaml) | **Identical** (598 canonical JSON lines; CEL and prose compared with whitespace collapsed). Changing one value is detected |
| Builder output against `manifest.schema.json` | 0 errors |
| [`tests/type-errors.ts`](./tests/type-errors.ts): 8 deliberate mistakes | All 8 are compile errors (undeclared record kind, freshness key, unknown field in `sources`, malformed source id, rule at an undeclared level, unknown typed field path, action without `expect`/`unverified`, bad duration) |
| Finding | The level check first passed silently: TypeScript widened the level union from the rules. Fixed with `NoInfer` (TypeScript ≥ 5.4) |
| Speed (4 vCPU) | `tsc --noEmit` 0.38 s (TypeScript 7.0.2); printing the manifest 0.34 s (tsx) |

Run:

```bash
npm install
npm run check      # tsc (including the negative type tests) + round-trip against the YAML
npm run print      # canonical manifest JSON
```

Pinned: typescript 7.0.2, tsx 4.23.15, yaml 2.9.1, @types/node 24.10.1.

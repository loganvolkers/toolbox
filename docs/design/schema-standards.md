# Schema Standards: Cataloguing the JSON Schemas We Compile

*Status: deterministic catalog and scorecard built, 2026-11-20 (D77). Seikika for Seikika catalogues
each document in `schema/` and scores it with `maintainer-schema`. The schema compiler is
unchanged. Related: [package-standards.md](./package-standards.md),
[seikika-for-seikika.md](./seikika-for-seikika.md).*

---

## 1. Conclusion

**A schema in this repository is a contract only if the binary compiles it as Draft 2020-12.**

`crates/seikika-model` embeds the three documents and builds them with Draft 2020-12. It does not
fetch a `$ref`. A file that names another dialect, or a schema that lives on the network, is not
the contract the server runs. The checks that do not need a second validator:

| Question | What answers it | In this repo |
|---|---|---|
| Is it Draft 2020-12, does it have a canonical id, and does the root say what it is? | `$schema`, `$id`, `description` | bronze |
| Does it have a title, and does the compiler or the reference validator load it? | `title`, a mention in `schema.rs` or `validate.py` | silver |
| Do its `$ref`s stay inside the file, and is there no secret in it? | `$ref` values, secret shapes | gold |

Property descriptions and `additionalProperties` on payload objects (`config`, `inputs`, `data`)
are not gated. Those objects are open because the caller defines them. Requiring a description on
every property would fail all three documents for a standard the dialect does not require.

## 2. What the field says

| Source | What it decides | Adopted |
|---|---|---|
| **JSON Schema 2020-12 core** | `$schema` identifies the dialect. `$id` is the base URI of the resource and should be absolute for a published schema. `$ref` is a URI reference; a fragment (`#/...`) stays in the document | The dialect, an `https` `$id`, and no remote `$ref` |
| **This repository's compiler** | `schema.rs` hardcodes `Draft::Draft202012` and `include_str!` of the three files. `examples/reference-portal/tools/validate.py` applies the same files | `draft-2020-12` and `wired-in`. A schema nothing loads is not a contract |
| **The three documents** | Each already sets `$schema`, an `https` `$id`, a title and a description, and every `$ref` starts with `#` | All three reach gold. The scorecard is the ratchet for the next document |

## 3. The catalog

`schemas` glob: `schema/*.json`. Id `json-schema:<path>`.

| Fact | What it is |
|---|---|
| `dialect`, `draft202012` | The `$schema` URI, and whether it is Draft 2020-12 |
| `id` | `$id` |
| `titleChars`, `descriptionChars` | Length of the root `title` and `description` |
| `externalRefs` | `$ref` values that do not start with `#` |
| `wired` | `schema.rs` or the reference validator names the filename |
| `secretHits` | High-precision secret shapes |

## 4. The scorecard

`maintainer-schema`, kind `JsonSchema`. Maintainer SDLC: these documents are the configuration
and protocol contracts of this repository.

| Level | Rule | Passes when |
|---|---|---|
| bronze | `draft-2020-12` | `$schema` is `https://json-schema.org/draft/2020-12/schema` |
| bronze | `has-id` | `$id` is an absolute `https` URI |
| bronze | `described` | The root `description` is non-empty |
| silver | `titled` | The root `title` is non-empty |
| silver | `wired-in` | The compiler or the reference validator names the file |
| gold | `local-refs` | Every `$ref` is a fragment in this file |
| gold | `no-hardcoded-secrets` | No secret shape |

## 5. This repository

| Document | Level | Notes |
|---|---|---|
| `schema/manifest.schema.json` | gold | Draft 2020-12, `$id` under `seikika.com`, compiled in, local `$ref`s |
| `schema/connector-messages.schema.json` | gold | The same |
| `schema/executor-messages.schema.json` | gold | The same |

Fixtures: `json-schema:schema/demo/focused.schema.json` at gold,
`json-schema:schema/demo/partial.schema.json` at bronze (no title, nothing compiles it),
`json-schema:schema/demo/unsafe.schema.json` at none (Draft 07, no `$id`, a remote `$ref`, a
secret). Every rule passes on one fixture and fails on another (D71).

## Sources

- JSON Schema 2020-12 core, `$schema`, `$id` and `$ref`:
  https://json-schema.org/draft/2020-12/json-schema-core
- `crates/seikika-model/src/schema.rs`, which embeds the documents and compiles them as Draft 2020-12.
- `examples/reference-portal/tools/validate.py`, which applies the same three files.

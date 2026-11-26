# SDK documentation audit

The connector SDK and both public npm packages now document the entry points readers use. Rustdoc reported 55 missing Rust comments at the starting revision. The npm audit found 40 missing comments or typed declarations and 62 gaps in the package references. The [SDK documentation comments guide](../../site/src/content/docs/maintainers/sdk-docs.md) sets the writing and publication conventions. Seikika measures the reference coverage in `maintainer-sdk-docs`.

## Scope and method

This is maintainer SDLC work on the box's published programming interfaces for connector and portal authors. The baseline was commit `25a208a`. The Rust scope is the full public library of `seikika-connect`, including the hub, conformance kit, and helper modules. The release guide identifies it as the connector SDK. The npm scope is every `exports` entry in the two public pnpm packages, including the expanded Vue wildcard. It includes public types and options, Vue props and slots, Astro props and slots, and the theme's documented customization surface. The other Rust workspace crates ship the CLI or its dependencies. The [manifest builder](../design/typescript-builder.md) remains a private prototype, and [typed portal clients](../design/user-sdlc.md) are planned. [Rust publishing decision](../decisions/D01M8BEC7ZX7TFBTSC47W1XQVV4.md), [npm package catalog](../../site/src/data/npm-packages.json)

The Rust inventory comes from default-visibility rustdoc and its `missing_docs` diagnostics on the library target with all features. For npm, the checker reads the package exports, uses the TypeScript compiler to inspect declarations, uses Vue's single-file component parser, and checks README sections against the exported paths, slots, and theme properties. The audit also traced the public signatures to implementation, connector protocol schema, and smoke-test fixtures to identify misleading or incomplete comments. [SDK checker](../../scripts/sdk-docs.mjs)

## Research findings

The comparison covers common SDK documentation problems: parsing and error conditions, async runtime requirements, HTTP configuration defaults, generated declaration comments, and editor-centered discovery. The projects represent several widely used approaches. On 2026-11-26 the GitHub API showed 5,697 stars for serde_json, 33,809 for Tokio, and 11,963 for reqwest. The npm downloads API reported about 146.3 million downloads for Axios, 3.8 million for Octokit, and 77.1 million for `@aws-sdk/client-s3` for November 19–25. Stars and downloads are separate selection signals. [serde_json repository](https://github.com/serde-rs/json), [Tokio repository](https://github.com/tokio-rs/tokio), [reqwest repository](https://github.com/seanmonstar/reqwest), [Axios downloads](https://api.npmjs.org/downloads/point/2026-11-19:2026-11-25/axios), [Octokit downloads](https://api.npmjs.org/downloads/point/2026-11-19:2026-11-25/octokit), [AWS S3 client downloads](https://api.npmjs.org/downloads/point/2026-11-19:2026-11-25/@aws-sdk/client-s3)

| Primary reference | Observed practice | Adopted in Seikika |
|---|---|---|
| [serde_json `from_str`](https://docs.rs/serde_json/1.0.154/serde_json/fn.from_str.html) | Starts with the operation, shows a typed parsing example, and explains ways valid JSON can still fail to deserialize. | Explain failure conditions a signature cannot convey. Show the relevant input and how to handle failure. |
| [Tokio `spawn`](https://docs.rs/tokio/1.54.1/tokio/task/fn.spawn.html) | Shows the feature requirement and explains scheduling, runtime context, shutdown, and panic conditions. | Describe runtime and lifecycle constraints beside the affected method. Link to the type or transport that owns the behavior. |
| [reqwest `ClientBuilder::timeout`](https://docs.rs/reqwest/0.13.5/reqwest/struct.ClientBuilder.html#method.timeout), [reqwest `Client`](https://docs.rs/reqwest/0.13.5/reqwest/struct.Client.html) | Explains what a timeout covers, its default, and related settings. Explains when `Client::new` can panic and points to the fallible builder. | Record units, scope, defaults, and caller-relevant panic conditions. Link alternatives where they solve a real caller problem. |
| [Axios declarations](https://github.com/axios/axios/blob/v1.x/index.d.ts), [Axios error guide](https://axios-http.com/docs/handling_errors) | Ships typed request and response contracts. Its guide explains how to distinguish server responses, missing responses, and other failures. | Put core contracts in the declarations. Use package guidance for failure diagnosis and configuration involving multiple steps. |
| [Octokit README](https://github.com/octokit/octokit.js), [GitHub scripting guide](https://docs.github.com/en/rest/guides/scripting-with-the-rest-api-and-javascript) | Shows the client import and authentication flow. Documents which invocation style gives consumers IDE completion. | Show the exact installed import and required setup. Keep editor advice tied to the entry point used in the example. |
| [S3 `GetObjectCommand` source](https://github.com/aws/aws-sdk-js-v3/blob/main/clients/client-s3/src/commands/GetObjectCommand.ts), [AWS S3 package manifest](https://github.com/aws/aws-sdk-js-v3/blob/main/clients/client-s3/package.json) | Generated commands document input and output, runnable usage shape, and named exceptions. The package builds and ships declarations and extracts API documentation. | Keep the semantic descriptions on the consumer-facing declarations. Put descriptions in generator inputs if an SDK is generated later. |

The rustdoc book confirms the use of a short summary, crate overviews, runnable examples, and conditional failure sections. TypeScript publishing guidance ties declarations to package entry points, and TSDoc specifies the tags that carry parameter descriptions, exceptions, and examples. For the current two npm packages, shipped comments plus the package READMEs cover the reference, with Storybook for rendered examples. [Rustdoc writing guide](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html), [Rustdoc doctests](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html), [Docs.rs metadata](https://docs.rs/about/metadata), [TypeScript publishing](https://www.typescriptlang.org/docs/handbook/declaration-files/publishing.html), [TSDoc tag kinds](https://tsdoc.org/pages/spec/tag_kinds/)

## Gap analysis and repairs

| Public surface | Missing comments or declarations before | Reference gaps before | Repairs |
|---|---:|---:|---|
| Rust crate root and `sdk` | 6 | 1 | Added docs.rs metadata and a crate entry route. Documented `Source::name` and all five `Out` methods. Explained the scopes, when to mark a snapshot complete, and the transports' failure behavior. Added a complete connector doctest. |
| Rust `hub` and its root re-exports | 45 | 0 | Covered every missing status variant, command and request field, registration field, and hub method. Explained deadlines, coalescing, input versions, runtime requirements, and timestamp limitations. Added a timestamp round trip doctest. |
| Rust `conformance` | 4 | 0 | Documented `CheckResult` and its fields. Clarified how to interpret partial output and how to check success after a driven resync. |
| Rust `assessment` and `files` | 0 | 0 | Corrected and expanded existing summaries about citation resolution, review freshness, file fallback behavior, and glob limits. Added a glob doctest. |
| `@seikika/ui` | 36 | 58 | Documented the entity functions and types plus all seven Vue components and their props. The README covers nine import paths, 20 slots, and 29 theme properties. |
| `@seikika/astro-report` | 4 | 4 | Documented the default export, option defaults, and the layout's typed title. The README covers three import paths and the layout slot. The guide explains file resolution, render failures, and snapshot exposure. |

The two npm packages' declaration findings and README findings count different checks. For example, a component can have fully documented props and still need a README entry for its slots. The final scan reports zero findings in both categories for all three SDKs.

## Seikika score and regression check

The `surfaces` connector runs the standalone checker through the existing NDJSON runner. Seikika catalogs one `SdkDocs` entity per SDK. The scorecard gives bronze for a public entry point, silver when the declarations have comments, and gold when the reference checks pass. The last two rules report the count and evidence for each gap. The maintained baseline and CI gate require gold. The baseline stores this zero-gap result as an empty scorecard because Seikika compacts scorecards that require their top level.

| SDK | Starting score under this standard | Final score |
|---|---|---|
| `seikika-connect` | Bronze | Gold |
| `@seikika/ui` | Bronze | Gold |
| `@seikika/astro-report` | Bronze | Gold |

The new scorecard is about documentation coverage. Behavioral review and example execution use the approach in the style guide. [Scorecard rules](../../portal/standards/sdk-docs.yaml), [Test cases](../../portal/tests/standards.test.yaml), [Seikika baseline](../../portal/results.json)

The CI ratchet returned exit 1 when the comment for `Hub::new`, the `PageFrame` `label` prop, and the Astro layout's default-slot reference were temporarily removed. Seikika named all three gaps. After restoring the documentation, the clean run matched the baseline. The new scorecard tests cover every level and both outcomes of each rule.

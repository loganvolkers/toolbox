# Package Standards: Cataloguing npm Manifests

*Status: deterministic catalog and scorecard built, 2026-11-20 (D76); pnpm workspace lockfiles and
frozen installs recognised 2026-11-21 (D01M7Y57FVX0XYNGMX0A0XJ4GK9). Seikika for Seikika catalogues
each `package.json` and scores it with `maintainer-package`. A licence field is not required by the scorecard (D76); manifests use MIT (D88).
Advisory databases are not a gate. Related: [dependency-standards.md](./dependency-standards.md),
[seikika-for-seikika.md](./seikika-for-seikika.md).*

---

## 1. Conclusion

**A private package is an application: name it, pin it with a lockfile, and install that lockfile.**

npm requires `name` and `version` only when a package is published. `private: true` makes `npm
publish` refuse the package, which is the right intent for an application that lives in this
repository. The checks that do not need a registry or a vulnerability database:

| Question | What answers it | In this repo |
|---|---|---|
| Is the package named, and is a publish either refused or versioned? Are its ranges bounded registry versions? | `name`, `private` or `version`, the dependency ranges | bronze |
| Is there a lockfile beside it or at the workspace root, and a script CI can run? | `package-lock.json` (or shrinkwrap, pnpm, yarn) and `test` / `build` / `check` / `typecheck` | silver |
| Does it say what it is, which Node it needs, and does CI install from the lockfile? | `description`, `engines.node`, `npm ci` or `pnpm install --frozen-lockfile` rather than an unlocked install | gold |

Caret and tilde ranges stay bounded. A `*`, `x`, `latest` or `>=` range does not. `workspace:`,
`file:` and `link:` are local and are not treated as wildcards. A git URL or an `https` tarball is
not a registry version. `npm audit` and a licence allow-list need a database and a licence policy,
so they do not gate.

## 2. What the field says

| Source | What it decides | Adopted |
|---|---|---|
| **npm `package.json`** | `name` and `version` are required to publish and optional otherwise. "Put a description in it." `private: true` makes npm refuse to publish. `engines` records the Node version and is advisory unless `engine-strict` is set | The name, the private-or-version rule, the description, `engines.node` |
| **npm developers** | Have a smoke test as `scripts.test`. Use `engines` to say which Node the program needs | A `test`, `build`, `check` or `typecheck` script counts. This repository's CI runs `build` and `check` |
| **OpenSSF npm best practices** | A lockfile hash-pins every installed package via `integrity`. For an application, commit the lockfile and in CI run only commands that treat it as read-only. `npm ci` and `pnpm install --frozen-lockfile` do. `npm install` / unlocked `pnpm install` may update it | A sibling or workspace-root lockfile at silver. Lockfile-faithful CI (`npm ci` or frozen pnpm) in that package's job at gold |
| **This repository** | `ui/` and `experiments/typescript-builder/` are private, exact-version, and each has a `package-lock.json`. CI runs `npm install` in both jobs | Both are silver. The gold gaps are a description, `engines.node`, and `npm ci`. The workflows are not changed here |

## 3. The catalog

`packages` glob: `**/package.json`. The walk already skips `node_modules`. Id
`package-json:<path>`. Direct npm dependencies are not separate entities. The Cargo catalog (D74)
covers the libraries the Rust build links. This catalog is the manifest.

| Fact | What it is |
|---|---|
| `name`, `version`, `private` | The identity and whether npm will publish it |
| `descriptionChars` | Length of `description` |
| `engines` | `engines.node` is a non-empty string |
| `requirements`, `wildcardRanges`, `gitRanges` | Direct ranges from dependencies, devDependencies, optionalDependencies and peerDependencies |
| `lockfile` | `package-lock.json`, `npm-shrinkwrap.json`, `pnpm-lock.yaml` or `yarn.lock` beside the manifest, or the same names walked up to the checkout root (pnpm/yarn workspaces) |
| `checkScripts` | How many of `test`, `build`, `check`, `typecheck` exist |
| `ciUsesInstall` | A workflow job whose `working-directory` is this package runs `npm install`, or `pnpm install` / `pnpm i` without `--frozen-lockfile` |
| `secretHits` | High-precision secret shapes in the manifest |

## 4. The scorecard

`maintainer-package`, kind `PackageJson`. Maintainer SDLC: these manifests are part of building
this repository.

| Level | Rule | Passes when |
|---|---|---|
| bronze | `named` | `name` is set |
| bronze | `publish-intent` | `private` is true, or `version` is set |
| bronze | `version-bounded` | No `*`, `x`, `latest` or `>=` range |
| bronze | `from-a-registry` | No git URL and no tarball URL |
| silver | `locked` | A lockfile sits beside the manifest or at a workspace root above it |
| silver | `has-a-check` | At least one of `test`, `build`, `check`, `typecheck` |
| gold | `described` | `description` is non-empty |
| gold | `engines-declared` | `engines.node` is set |
| gold | `ci-installs-from-the-lockfile` | CI does not run an unlocked install (`npm install`, or pnpm without `--frozen-lockfile`) for this package |
| gold | `no-hardcoded-secrets` | No secret shape |

## 5. This repository

| Package | Level | Notes |
|---|---|---|
| `ui/package.json` | silver | Private, `package-lock.json`, `build`. No description, no `engines.node`. The `ui` job runs `npm install` |
| `experiments/typescript-builder/package.json` | silver | Private, `package-lock.json`, `check` and `typecheck`. The same gold gaps. The `typescript-builder` job runs `npm install` |

Fixtures: `package-json:packages/demo/focused/package.json` at gold,
`package-json:packages/demo/partial/package.json` at bronze (no lockfile, no check script),
`package-json:packages/demo/unsafe/package.json` at none (no name, public with no version, a star
range, a git URL, `npm install`, a secret). Every rule passes on one fixture and fails on another
(D71).

## 6. Monorepo semantics — do not reuse this scorecard blindly

`maintainer-package` scores **each** `package.json` as an application package. Authors building
pnpm/Expo **workspace** standards must not copy it as the whole story
([D01M7Y6X38AERPBT0YH1AG5TBRD](../decisions/D01M7Y6X38AERPBT0YH1AG5TBRD.md)): use
[`packages/pnpm-expo`](../../packages/pnpm-expo/) (`JsWorkspace` / `ExpoApp`) instead.

| Fact | Semantics today (`npm.rs`) | Author pitfall |
|---|---|---|
| `lockfile` | Lockfile **beside** the manifest **or** walked up to the checkout root (`package-lock.json`, shrinkwrap, `pnpm-lock.yaml`, `yarn.lock`) | Older mental model was “sibling-only”. Nested app manifests without a local lockfile can still be `locked` via a root `pnpm-lock.yaml` — that does **not** mean workspace hygiene (packageManager pin, linker, frozen CI) is covered |
| `ciUsesInstall` | True when a workflow job with `working-directory` for **this** package path runs unlocked install: `npm install`, or `pnpm install` / `pnpm i` **without** `--frozen-lockfile` | Root-only CI (`working-directory` omitted or `.`) is not attributed to nested packages. Faithful installs: `npm ci`, `pnpm install --frozen-lockfile`, `pnpm i --frozen-lockfile`, and (for workspace scorecards) `pnpm ci` / plain `pnpm install` in CI as documented by `JsWorkspace` facts — not the same field |
| Scorecard shape | Per-manifest bronze→gold for name, ranges, sibling/root lockfile, check scripts, description, engines | Does not score `pnpm-workspace.yaml`, linker policy, mise pins, Expo/EAS, or toolchain alignment across the monorepo |

**Rule of thumb:** enable `packages: ["**/package.json"]` on a consumer portal only when you mean
`maintainer-package`-style per-manifest gates (or after you include pnpm-aware rules such as
`pnpm-expo`). The starter README says the same
([repo-portal-starter](../../examples/repo-portal-starter/README.md)).

## Sources

- npm documentation, `package.json` (name, version, description, private, engines), read 2026-11-20:
  https://docs.npmjs.com/cli/v12/configuring-npm/package-json/
- npm documentation, developers (a smoke test in `scripts.test`, be explicit with `engines`):
  https://docs.npmjs.com/cli/v12/using-npm/developers/
- OpenSSF, package-manager best practices for npm (lockfile integrity hashes; application projects
  run `npm ci` in CI): https://github.com/ossf/package-manager-best-practices/blob/main/published/npm.md
- This repository's `ui/package.json`, `experiments/typescript-builder/package.json`, and the `ui`
  and `typescript-builder` jobs in `.github/workflows/ci.yml`.

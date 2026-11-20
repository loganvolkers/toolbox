# Package Standards: Cataloguing npm Manifests

*Status: deterministic catalog and scorecard built, 2026-11-20 (D76). Toolbox for Toolbox catalogues
each `package.json` and scores it with `maintainer-package`. A licence field is not required by the scorecard (D76); manifests use MIT (D88).
Advisory databases are not a gate. Related: [dependency-standards.md](./dependency-standards.md),
[toolbox-for-toolbox.md](./toolbox-for-toolbox.md).*

---

## 1. Conclusion

**A private package is an application: name it, pin it with a lockfile, and install that lockfile.**

npm requires `name` and `version` only when a package is published. `private: true` makes `npm
publish` refuse the package, which is the right intent for an application that lives in this
repository. The checks that do not need a registry or a vulnerability database:

| Question | What answers it | In this repo |
|---|---|---|
| Is the package named, and is a publish either refused or versioned? Are its ranges bounded registry versions? | `name`, `private` or `version`, the dependency ranges | bronze |
| Is there a lockfile beside it, and a script CI can run? | `package-lock.json` (or shrinkwrap, pnpm, yarn) and `test` / `build` / `check` / `typecheck` | silver |
| Does it say what it is, which Node it needs, and does CI install from the lockfile? | `description`, `engines.node`, `npm ci` rather than `npm install` | gold |

Caret and tilde ranges stay bounded. A `*`, `x`, `latest` or `>=` range does not. `workspace:`,
`file:` and `link:` are local and are not treated as wildcards. A git URL or an `https` tarball is
not a registry version. `npm audit` and a licence allow-list need a database and a licence policy,
so they do not gate.

## 2. What the field says

| Source | What it decides | Adopted |
|---|---|---|
| **npm `package.json`** | `name` and `version` are required to publish and optional otherwise. "Put a description in it." `private: true` makes npm refuse to publish. `engines` records the Node version and is advisory unless `engine-strict` is set | The name, the private-or-version rule, the description, `engines.node` |
| **npm developers** | Have a smoke test as `scripts.test`. Use `engines` to say which Node the program needs | A `test`, `build`, `check` or `typecheck` script counts. This repository's CI runs `build` and `check` |
| **OpenSSF npm best practices** | A lockfile hash-pins every installed package via `integrity`. For an application, commit the lockfile and in CI run only commands that treat it as read-only. `npm ci` does. `npm install` may update it | A sibling lockfile at silver. `npm ci` in that package's CI job at gold |
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
| `lockfile` | `package-lock.json`, `npm-shrinkwrap.json`, `pnpm-lock.yaml` or `yarn.lock` beside the manifest |
| `checkScripts` | How many of `test`, `build`, `check`, `typecheck` exist |
| `ciUsesInstall` | A workflow job whose `working-directory` is this package runs `npm install` |
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
| silver | `locked` | A lockfile sits beside the manifest |
| silver | `has-a-check` | At least one of `test`, `build`, `check`, `typecheck` |
| gold | `described` | `description` is non-empty |
| gold | `engines-declared` | `engines.node` is set |
| gold | `ci-installs-from-the-lockfile` | CI does not run `npm install` for this package |
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

## Sources

- npm documentation, `package.json` (name, version, description, private, engines), read 2026-11-20:
  https://docs.npmjs.com/cli/v12/configuring-npm/package-json/
- npm documentation, developers (a smoke test in `scripts.test`, be explicit with `engines`):
  https://docs.npmjs.com/cli/v12/using-npm/developers/
- OpenSSF, package-manager best practices for npm (lockfile integrity hashes; application projects
  run `npm ci` in CI): https://github.com/ossf/package-manager-best-practices/blob/main/published/npm.md
- This repository's `ui/package.json`, `experiments/typescript-builder/package.json`, and the `ui`
  and `typescript-builder` jobs in `.github/workflows/ci.yml`.

# OSS monorepo federation testbed

**Question:** Can Toolbox push shared standards into real OSS monorepo conventions
(Babel, Jest, pnpm), let each leaf extend with *only* its existing tooling, reject
weakenings, and roll results into a readable aggregated portal?

**Answer (this experiment):** yes, as a docs-as-code fleet under `experiments/`
(not discoverable `packages/`). Selection write-up (project store):
`docs/oss-monorepo-federation-testbed.md`. Design:
[docs/design/federation.md](../../docs/design/federation.md) + federation
prototype pattern.

## Fleet

| Leaf | Upstream | Package manager | Leaf-only standards (mined, not invented) |
|---|---|---|---|
| `leaves/babel` | [babel/babel](https://github.com/babel/babel) (MIT) | Yarn Berry 4.17 | Husky + lint-staged, Jest, Prettier `printWidth: 80`, Renovate, `AGENTS.md` |
| `leaves/jest` | [jestjs/jest](https://github.com/jestjs/jest) (MIT) | Yarn Berry 4.18 | Lerna-lite, Prettier `singleQuote` / no `bracketSpacing`, Jest CI configs, `CLAUDE.md` |
| `leaves/pnpm` | [pnpm/pnpm](https://github.com/pnpm/pnpm) (MIT) | pnpm 12.22 | Changesets, custom githooks (not Husky), rustfmt / Clippy `-D warnings` / cargo-deny, Renovate, `AGENTS.md` |

Shared push (`oss-fleet`): `packageManager` pin, workspace layout, ESLint flat,
TypeScript presence, agent instruction file, Prettier (when used), advisory
`deps-automation` campaign.

Sparse fixtures live under `leaves/*/fixtures/` — see [ATTRIBUTION.md](ATTRIBUTION.md).

## Run

```bash
python3 experiments/oss-monorepo-federation/compose.py check
python3 experiments/oss-monorepo-federation/compose.py weaken
python3 experiments/oss-monorepo-federation/compose.py report -o experiments/oss-monorepo-federation/out/federation-report.json
python3 experiments/oss-monorepo-federation/render_portal.py
python3 experiments/oss-monorepo-federation/tests/test_compose.py
python3 experiments/oss-monorepo-federation/compose.py bench --rounds 30
```

Open the aggregated portal:

```bash
# file:// or any static server
python3 -m http.server 8765 --bind 127.0.0.1 -d experiments/oss-monorepo-federation/out
# then http://127.0.0.1:8765/portal.html
```

Committed artifact: [out/portal.html](out/portal.html).

## What the tests prove

| Flow | Proof |
|---|---|
| **Push** | Each leaf compose includes shared scorecards (`package-manager-pin`, …) |
| **Extend** | Babel → husky/jest/yarn; Jest → lerna/prettier-style; pnpm → changesets/rust |
| **Weaken rejected** | `prettierPrintWidth: 120` against default 100 fails with `weakens` |
| **Results up** | Parent report rolls adoption + `deps-automation` (babel+pnpm meet; jest does not — no Renovate observed) |
| **Catalog** | Merged Repository / Library / Component across three repos |
| **Evidence** | Fixture `packageManager` / Prettier width match upstream files |

## Performance (measured)

Host: cloud agent VM, Python 3, PyYAML, **30 rounds**, fixture fleet only (no
full clones, no `toolbox` binary required).

| Command | mean | min | max |
|---|---|---|---|
| `compose.py check` | **25.4 ms** | 24.3 ms | 30.5 ms |
| `compose.py report` | **33.1 ms** | 32.0 ms | 42.0 ms |
| `compose.py weaken` | **7.2 ms** | 6.8 ms | 8.7 ms |

Re-measure: `python3 experiments/oss-monorepo-federation/compose.py bench --rounds 30`
(redirect stdout if you only want the JSON summary from a quieter wrapper).

## Explicit non-goals

- No invented Lefthook mandate for Babel (they use Husky).
- No CodeQL / Ubuntu / JUnit campaigns unless upstream evidence exists.
- No full monorepo clones in-tree; large configs are `*.evidence.json` stubs.
- Does not weaken T4T / yamark / D79; fixtures stay out of `packages/`.

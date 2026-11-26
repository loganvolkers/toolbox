# build-cost

**Question:** what do the heavy Rust dependencies proposed in [plan.md](../../docs/plan.md) §3
(dbsp, wasmtime, tantivy, async-graphql) cost in clean and incremental builds, compared with a
"boring" set (axum, sqlx, rmcp, cel-rust)? And what does the proposed OpenFGA c-archive cost to build?

**Answer (2026-11-19, 4 vCPU Xeon @ 2.1 GHz):** the heavy set makes clean builds **5×** slower and
binaries **5×** larger, but the wide-and-shallow layout keeps edit-loop rebuilds near one second. The
first attempt failed outright: **`dbsp` 0.384.0 crashed rustc 1.97.0** (internal compiler error) under
the dev profile from [developer-velocity.md](../../docs/design/developer-velocity.md) §3.2. It builds
with rustc 1.99.0, the version Feldera pins.

| Measure (rustc 1.99.0) | Boring (axum, sqlx, rmcp, cel) | Full (+ dbsp, wasmtime, tantivy, async-graphql) | Ratio |
|---|---|---|---|
| Crates compiled | 167 | 494 | 3.0× |
| Clean dev build | 96 s | 496 s | 5.1× |
| Clean release build (thin LTO) | 64 s | 363 s | 5.7× |
| Dev / release binary | 18.0 / 10.1 MB | 101.4 / 52.9 MB | 5.6× / 5.2× |
| Rebuild after editing the binary crate | 0.45–0.48 s | 1.27–1.29 s | 2.7× |
| Rebuild after editing a leaf crate (`leaf-cel`) | 0.50 s | 1.21 s | 2.4× |
| First `cargo check` after an edit (no check artifacts yet) | 16.5 s | 67.4 s | 4.1× |
| Warm `cargo check` after editing a leaf | 0.30 s | 0.50 s | 1.7× |

OpenFGA c-archive ([fga-carchive](./fga-carchive), OpenFGA v1.22.1, Go 1.26.11,
`-buildmode=c-archive`): **45 s** cold build cache (modules already downloaded), **2.0 s** warm,
**78 MB** archive.

## Layout

- `crates/model`: a tiny shared crate.
- One leaf crate per heavy dependency, each *using* it: `leaf-ivm` (a DBSP recursive transitive
  closure), `leaf-wasm` (wasmtime instantiates and calls a WAT module), `leaf-search` (tantivy in-RAM
  index and query), `leaf-graphql` (async-graphql dynamic schema built at runtime), `leaf-mcp` (rmcp
  server types), `leaf-store` (sqlx Postgres pool), `leaf-web` (axum router), `leaf-cel` (cel-rust).
- `crates/app`: a binary with features `boring` and `full`; it runs each leaf once.
- The workspace uses the dev profile proposed in developer-velocity.md §3.2
  (`debug = "line-tables-only"`, dependencies at `opt-level = 1`, `dbsp` at `opt-level = 2`).
- The heavy dependencies are pinned with `=` in `Cargo.toml`; everything is locked in `Cargo.lock` (683 packages).

## Run

```bash
cargo fetch
RUSTUP_TOOLCHAIN=1.99.0 TARGET_ROOT=/scratch/build-cost ./run.sh > results.txt
# OpenFGA c-archive
cd fga-carchive && GOTOOLCHAIN=go1.26.11 go build -buildmode=c-archive -o /scratch/libseikikafga.a .
```

`run.sh` builds each set from an empty target directory, then edits `app` and `leaf-cel` (comment
changes) and times the rebuilds. Raw output: [results-rustc-1.99.0.txt](./results-rustc-1.99.0.txt) and,
for the failed attempt, [results-rustc-1.97.0.txt](./results-rustc-1.97.0.txt).

## The rustc 1.97.0 internal compiler error

Compiling `dbsp` 0.384.0 with `-C opt-level=2 -C debug-assertions=on` panicked in
`instantiate_and_check_impossible_predicates` for `operator::dynamic::multijoin::star_join::StarJoinFuncTrait`
(reproduced on every retry). `dbsp` declares `rust-version = "1.96.1"`, but Feldera's own
`rust-toolchain.toml` pins `1.99.0`. Other profiles were not tried on 1.97.

## Caveats

- Minimal usage of each dependency; a real crate using more generic instantiations of `dbsp` would
  compile longer (Feldera's own experience, cited in developer-velocity.md §1).
- One 4-vCPU machine; no sccache; Cargo's default parallelism.
- The release build of the full set is faster than its dev build because the dev profile compiles
  `dbsp` at `opt-level = 2` with debug assertions.

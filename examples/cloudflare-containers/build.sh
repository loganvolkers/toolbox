#!/usr/bin/env bash
# Build and stage the Worker, Container, and portal for Workers Builds.
set -euo pipefail

example_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_dir=$(cd -- "$example_dir/../.." && pwd)
portal_dir=$(cd -- "${1:-"$example_dir/seikika-forks"}" && pwd)
cd -- "$repo_dir"

staging=$(mktemp -d "$example_dir/.build.XXXXXX")
trap 'rm -rf -- "$staging"' EXIT

export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
if ! command -v cargo >/dev/null 2>&1; then
  installer="$staging/rustup-init"
  curl --fail --silent --show-error --location --retry 3 \
    https://static.rust-lang.org/rustup/archive/1.29.3/x86_64-unknown-linux-gnu/rustup-init \
    --output "$installer"
  printf '%s  %s\n' b1c8cb14e30889e7e2999e6b69e62d9e9b9ffc0f4df8180564d306d9680e6c55 "$installer" \
    | sha256sum --check --status
  chmod 0755 "$installer"
  "$installer" -y --no-modify-path --profile minimal --default-toolchain none
  rm -- "$installer"
fi

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-"$repo_dir/target"}"
cargo build --locked --release --workspace
bash "$example_dir/stage.sh" "$portal_dir" "$staging"

worker_name=${WRANGLER_CI_OVERRIDE_NAME:-}
if [[ -n "$worker_name" ]]; then
  if [[ ! "$worker_name" =~ ^[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?$ ]]; then
    printf 'Invalid Worker name from Workers Builds: %s\n' "$worker_name" >&2
    exit 1
  fi
  sed -i "0,/^name = .*/s//name = \"$worker_name\"/" "$staging/wrangler.toml"
fi

"$staging/bin/seikika" validate "$staging/portal"
npm ci --ignore-scripts --no-audit --no-fund --prefix "$staging"
npm --prefix "$staging" run check

rm -rf -- "$example_dir/.build"
mv -- "$staging" "$example_dir/.build"

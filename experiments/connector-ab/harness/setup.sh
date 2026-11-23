#!/usr/bin/env bash
# Prepare /tmp/connector-ab: Forge fixtures, framework pins, per-arm working copies.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
AB_HOME="${AB_HOME:-/tmp/connector-ab}"
OCEAN_PIN=f9f3948e83a20769f32fd3ca9fe0581cc8755926
BACKSTAGE_PIN=feeadd4d4efcbda9f1573b95e356e0f3fc0b5d17

mkdir -p "$AB_HOME"/{arms,pins,bin,logs}
cp -a "$ROOT/experiments/connector-ab/fixtures" "$AB_HOME/fixtures"

# --- pins (open-source frameworks agents may read) ---------------------------------
if [[ ! -d "$AB_HOME/pins/ocean/.git" ]]; then
  git clone --depth 1 https://github.com/port-labs/ocean.git "$AB_HOME/pins/ocean"
  git -C "$AB_HOME/pins/ocean" fetch --depth 1 origin "$OCEAN_PIN"
  git -C "$AB_HOME/pins/ocean" checkout "$OCEAN_PIN"
fi
if [[ ! -d "$AB_HOME/pins/backstage/.git" ]]; then
  git clone --depth 1 --filter=blob:none --sparse https://github.com/backstage/backstage.git "$AB_HOME/pins/backstage"
  git -C "$AB_HOME/pins/backstage" sparse-checkout set \
    plugins/catalog-backend \
    plugins/catalog-node \
    plugins/catalog-backend-module-github \
    packages/catalog-model \
    docs
  git -C "$AB_HOME/pins/backstage" fetch --depth 1 origin "$BACKSTAGE_PIN" || true
  git -C "$AB_HOME/pins/backstage" checkout "$BACKSTAGE_PIN" || true
fi

# --- toolbox binaries on PATH for arm A scoring ------------------------------------
if [[ -x "$ROOT/target/debug/toolbox" ]]; then
  ln -sfn "$ROOT/target/debug/toolbox" "$AB_HOME/bin/toolbox"
fi

# --- arm working copies from scaffolds ---------------------------------------------
for arm in A B C; do
  dest="$AB_HOME/arms/$arm"
  rm -rf "$dest"
  mkdir -p "$dest/OUT"
  case "$arm" in
    A)
      cp -a "$ROOT/experiments/connector-ab/scaffolds/toolbox/." "$dest/"
      # Point the path dependency at this checkout
      sed -i "s|__TOOLBOX_ROOT__|$ROOT|g" "$dest/Cargo.toml"
      # Arm A must use the repo toolchain (edition2024); without this, cargo 1.83 fails.
      cp "$ROOT/rust-toolchain.toml" "$dest/rust-toolchain.toml"
      echo "export PATH=/usr/local/cargo/bin:\$PATH" > "$dest/env.sh"
      ;;
    B)
      cp -a "$ROOT/experiments/connector-ab/scaffolds/ocean/." "$dest/"
      echo "$AB_HOME/pins/ocean" > "$dest/OCEAN_ROOT.txt"
      ;;
    C)
      cp -a "$ROOT/experiments/connector-ab/scaffolds/backstage/." "$dest/"
      echo "$AB_HOME/pins/backstage" > "$dest/BACKSTAGE_ROOT.txt"
      ;;
  esac
  cat > "$dest/FORGE.md" <<EOF
# Forge (mock Jira Cloud)

Tenant A: http://127.0.0.1:9743  token tenant-a-token
Tenant B: http://127.0.0.1:9744  token tenant-b-token

Endpoints:
- GET /rest/api/3/project?startAt=&maxResults=
- GET /rest/api/3/search?startAt=&maxResults=
- GET /rest/api/3/project/{key}/components   (required from phase 3)

Header: Authorization: Bearer <token>
EOF
done

# Write pin record for the catalog / results
cat > "$AB_HOME/pins/PINS.txt" <<EOF
port-labs/ocean@$OCEAN_PIN
backstage/backstage@$BACKSTAGE_PIN
toolbox=$(git -C "$ROOT" rev-parse HEAD)
EOF

echo "AB_HOME=$AB_HOME ready"
cat "$AB_HOME/pins/PINS.txt"

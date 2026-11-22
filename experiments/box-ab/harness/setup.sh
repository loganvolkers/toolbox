#!/usr/bin/env bash
# Box A/B setup: the pinned tools and clean copies of the PostHog fork, rebuilt from nothing, so a
# reclaimed container costs one rerun of this script. Idempotent.
#
#   experiments/box-ab/harness/setup.sh            # tools, toolbox, the reference checkout
#   AB_HOME=/somewhere experiments/box-ab/harness/setup.sh
#
# Layout under $AB_HOME (default /tmp/box-ab):
#   bin/          what both arms get on PATH: semgrep, ruff (PostHog's pin), zizmor, actionlint
#   box-bin/      arm B only: toolbox and its connectors
#   upgrade/      ruff 0.15.0, for phase 4
#   posthog/      the fork at $COMMIT, never edited (truth runs here)
set -euo pipefail

AB_HOME=${AB_HOME:-/tmp/box-ab}
COMMIT=153f1b8d3a427aea26d8df84e8d9c57cead0a1e0 # loganvolkers/posthog, 2026-06-01
REPO=$(cd "$(dirname "$0")/../../.." && pwd)
SEMGREP=1.163.0 # PostHog's CI image (ci-security.yaml)
RUFF=0.14.11    # PostHog's pyproject pin (ruff~=0.14.11)
RUFF_UP=0.15.0  # phase 4: reports 28 more (UP042) on PostHog at $COMMIT
ZIZMOR=1.32.0   # the github-actions kit's pin
mkdir -p "$AB_HOME"/{bin,box-bin,upgrade,venv}

# semgrep in its own venv. Its startup version check stalls ~190 s when the network is closed
# (docs/research-log.md); the wrapper turns it off for both arms alike.
if [ ! -x "$AB_HOME/venv/semgrep/bin/semgrep" ]; then
  python3 -m venv "$AB_HOME/venv/semgrep"
  "$AB_HOME/venv/semgrep/bin/pip" install -q "semgrep==$SEMGREP" pyyaml click
fi
cat > "$AB_HOME/bin/semgrep" <<WRAP
#!/bin/sh
# Add --disable-version-check unless it's there (semgrep refuses it twice) or this is --version.
case " \$* " in *" --version "*|*" --disable-version-check "*) ;; *) set -- "\$@" --disable-version-check ;; esac
SEMGREP_ENABLE_VERSION_CHECK=0 exec "$AB_HOME/venv/semgrep/bin/semgrep" "\$@"
WRAP
chmod +x "$AB_HOME/bin/semgrep"

pin() { # pin <name> <version> <dir>: pip --target with --require-hashes (OpenSSF)
  [ -x "$3/bin/$1" ] && return 0
  req=$(mktemp)
  case "$1==$2" in
    ruff==0.14.11) echo 'ruff==0.14.11 --hash=sha256:a35c9da08562f1598ded8470fcfef2afb5cf881996e6c0a502ceb61f4bc9c8a3' >"$req" ;;
    ruff==0.15.0) echo 'ruff==0.15.0 --hash=sha256:bcbca3d40558789126da91d7ef9a7c87772ee107033db7191edefa34e2c7f1b4' >"$req" ;;
    zizmor==1.32.0) echo 'zizmor==1.32.0 --hash=sha256:84a6acb975accc13408786a2bd274b572537ca19c933ebbc13e78f3988562071' >"$req" ;;
    *) echo "no hash pin for $1==$2" >&2; rm -f "$req"; return 1 ;;
  esac
  python3 -m pip install -q --target "$3" --require-hashes -r "$req"
  rm -f "$req"
}
pin ruff "$RUFF" "$AB_HOME/venv/ruff-$RUFF" && ln -sf "$AB_HOME/venv/ruff-$RUFF/bin/ruff" "$AB_HOME/bin/ruff"
pin ruff "$RUFF_UP" "$AB_HOME/upgrade/ruff-$RUFF_UP"
pin zizmor "$ZIZMOR" "$AB_HOME/venv/zizmor-$ZIZMOR" && ln -sf "$AB_HOME/venv/zizmor-$ZIZMOR/bin/zizmor" "$AB_HOME/bin/zizmor"
[ -x "$AB_HOME/bin/actionlint" ] || "$REPO/scripts/ci-tools.sh" "$AB_HOME/bin" actionlint

# Toolbox for arm B, from this checkout (release build: --against and report rerun connectors).
(cd "$REPO" && cargo build -q --release --locked)
for b in toolbox toolbox-connector-github toolbox-connector-git-files; do
  [ -x "$REPO/target/release/$b" ] && ln -sf "$REPO/target/release/$b" "$AB_HOME/box-bin/$b"
done

# The reference checkout: exactly $COMMIT, shallow.
if [ ! -d "$AB_HOME/posthog/.git" ]; then
  git init -q "$AB_HOME/posthog"
  # The fork attached to the session, if any (add_repo loganvolkers/posthog), else GitHub.
  SRC=${POSTHOG_SRC:-/home/user/loganvolkers/posthog}
  [ -d "$SRC/.git" ] || SRC=https://github.com/loganvolkers/posthog
  git -C "$AB_HOME/posthog" fetch -q --depth 1 "$SRC" "$COMMIT"
  git -C "$AB_HOME/posthog" checkout -q FETCH_HEAD
fi
[ "$(git -C "$AB_HOME/posthog" rev-parse HEAD)" = "$COMMIT" ] || { echo "posthog is not at $COMMIT" >&2; exit 1; }

PATH="$AB_HOME/bin:$PATH"
semgrep --version | sed 's/^/semgrep /'; ruff --version; zizmor --version; actionlint -version | head -1
"$AB_HOME/upgrade/ruff-$RUFF_UP/bin/ruff" --version | sed 's/^/upgrade: /'
"$AB_HOME/box-bin/toolbox" --version 2>/dev/null || echo "toolbox built"
echo "ready: $AB_HOME"

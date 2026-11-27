#!/usr/bin/env bash
# Verify that setup refuses a valid archive with the wrong published checksum.
set -euo pipefail

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/bin" "$work/seikika-cli-x86_64-unknown-linux-gnu"
candidate="$work/seikika-cli-x86_64-unknown-linux-gnu/seikika"
printf '#!/usr/bin/env bash\necho "seikika should not run"\n' >"$candidate"
chmod +x "$candidate"
archive=seikika-cli-x86_64-unknown-linux-gnu.tar.xz
tar -cJf "$work/$archive" -C "$work" seikika-cli-x86_64-unknown-linux-gnu

cat >"$work/bin/gh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
[[ $1 == release && $2 == download ]]
while [[ $1 != --dir ]]; do shift; done
dest=$2
cp "$FIXTURE_ARCHIVE" "$dest/"
printf '%064d\n' 0 > "$dest/${FIXTURE_ARCHIVE##*/}.sha256"
EOF
chmod +x "$work/bin/gh"
touch "$work/path" "$work/output"

status=0
PATH="$work/bin:$PATH" FIXTURE_ARCHIVE="$work/$archive" \
  RUNNER_TEMP="$work" RUNNER_OS=Linux RUNNER_ARCH=X64 \
  GITHUB_PATH="$work/path" GITHUB_OUTPUT="$work/output" \
  SEIKIKA_REPO=seikika/seikika SEIKIKA_VERSION=0.2.1 SEIKIKA_BIN='' \
  bash "$root/actions/setup/run.sh" >"$work/log" 2>&1 || status=$?
cat "$work/log"
test "$status" = 2
grep -qF "checksum mismatch for $archive" "$work/log"
test ! -s "$work/path"
test ! -s "$work/output"
shopt -s nullglob
installed=("$work"/seikika-bin.*/seikika)
test "${#installed[@]}" = 0
echo "Setup rejected the archive before installing or registering it."

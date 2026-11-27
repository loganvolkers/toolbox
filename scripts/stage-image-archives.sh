#!/usr/bin/env bash
# Stage the checksummed cargo-dist archives for the Seikika image.
set -euo pipefail

archives=${1:?usage: stage-image-archives.sh ARCHIVE_DIR IMAGE_DIR [RELEASE_TAG]}
image=${2:?specify the image context}
tag=${3-}

mkdir -p "$archives"
for pair in amd64:x86_64 arm64:aarch64; do
  arch=${pair%%:*}
  target=${pair##*:}-unknown-linux-gnu
  mkdir -p "$image/bin/$arch"
  for binary in seikika seikika-connector-github seikika-connector-git-files seikika-connector-runner seikika-connector-workspace seikika-connector-scim; do
    app=$binary
    if [[ $binary == seikika ]]; then
      app=seikika-cli
    fi
    stem=$app-$target
    archive=$stem.tar.xz
    if [[ -n $tag ]]; then
      gh release download --repo "${GITHUB_REPOSITORY:?set GITHUB_REPOSITORY}" \
        --dir "$archives" --pattern "$archive" --pattern "$archive.sha256" --clobber -- "$tag"
    fi
    (
      cd "$archives"
      digest=$(awk 'NR == 1 {print $1}' "$archive.sha256")
      [[ $digest =~ ^[[:xdigit:]]{64}$ ]] || {
        echo "Invalid checksum in $archive.sha256" >&2
        exit 1
      }
      printf '%s  %s\n' "$digest" "$archive" | sha256sum -c -
      tar -xJf "$archive"
    )
    install -m 0755 "$archives/$stem/$binary" "$image/bin/$arch/$binary"
  done
done

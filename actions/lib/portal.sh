# shellcheck shell=bash
# Shared checkout preparation for repo-portal actions. Call from the consumer checkout.
prepare_portal() {
  local title=$1 executable=$2 directory=$3 portal=$4
  if ! cd -- "$directory"; then
    printf '::error title=%s::Cannot enter the working directory. Check the checkout and working-directory input.\n' "$title"
    return 2
  fi
  if [[ ! -f "$portal/seikika.yaml" ]]; then
    printf '::error title=%s::No seikika.yaml in the portal directory. Set the portal input or run seikika init.\n' "$title"
    return 2
  fi
  if [[ -f "$portal/seikika.lock" ]] && ! "$executable" install "$portal" --frozen; then
    printf '::error title=%s::seikika install --frozen failed. Check the lockfile and package refs.\n' "$title"
    return 2
  fi
}

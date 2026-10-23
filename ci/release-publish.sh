#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PYTHONPATH="$root" python3 -c 'import sys; from ci.release import version_from_tag; version_from_tag(sys.argv[1])' "$RELEASE_TAG"
(cd release-assets && sha256sum --check --strict SHA256SUMS)

check_tag() {
  PYTHONPATH="$root" python3 -m ci.release_remote
}

ensure_draft() {
  local draft
  if draft="$(gh release view "$RELEASE_TAG" --json isDraft --jq .isDraft)"; then
    [[ "$draft" == true ]] && return 0
    echo "A release já está publicada; seus artefatos não serão substituídos." >&2
    return 1
  fi
  check_tag
  gh release create "$RELEASE_TAG" --verify-tag --draft \
    --title "Sparky $RELEASE_TAG" --generate-notes
}

check_tag
ensure_draft
PYTHONPATH="$root" python3 -m ci.release_draft prune
gh release upload "$RELEASE_TAG" release-assets/* --clobber
PYTHONPATH="$root" python3 -m ci.release_draft verify
check_tag
gh release edit "$RELEASE_TAG" --draft=false

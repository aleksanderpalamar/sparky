#!/usr/bin/env bash
set -euo pipefail

: "${RUNNER_TEMP:?Defina RUNNER_TEMP para uma pasta temporária}"
mkdir -p "$RUNNER_TEMP"
archive="$RUNNER_TEMP/actionlint-1.7.12.tar.gz"
curl --fail --location --retry 3 \
  --output "$archive" \
  https://github.com/rhysd/actionlint/releases/download/v1.7.12/actionlint_1.7.12_linux_amd64.tar.gz
printf '%s  %s\n' \
  8aca8db96f1b94770f1b0d72b6dddcb1ebb8123cb3712530b08cc387b349a3d8 \
  "$archive" | sha256sum --check --strict
tar -xzf "$archive" -C "$RUNNER_TEMP" actionlint
"$RUNNER_TEMP/actionlint" -color

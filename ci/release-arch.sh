#!/usr/bin/env bash
set -euo pipefail

[[ "${GITHUB_ACTIONS:-}" == true ]]
pacman -Syu --noconfirm --needed base-devel git rustup dbus python
git config --global --add safe.directory "$PWD"
startdir="$PWD/packaging/arch"
source "$startdir/PKGBUILD"
pacman -S --noconfirm --needed "${depends[@]}"
useradd --create-home --uid "$SPARKY_BUILDER_UID" builder
runuser -u builder -- bash -euo pipefail -c '
  rustup toolchain install stable --profile minimal
  env -u DISPLAY -u WAYLAND_DISPLAY -u XDG_RUNTIME_DIR -u SPARKY_SOCKET \
    DBUS_SESSION_BUS_ADDRESS=unix:path=/nonexistent \
    DBUS_SYSTEM_BUS_ADDRESS=unix:path=/nonexistent \
    bash packaging/build.sh pacman
'

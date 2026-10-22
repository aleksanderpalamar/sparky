#!/usr/bin/env bash
set -euo pipefail

exec env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS \
  -u DBUS_SYSTEM_BUS_ADDRESS -u XDG_RUNTIME_DIR -u SPARKY_SOCKET \
  DBUS_SESSION_BUS_ADDRESS=unix:path=/nonexistent \
  DBUS_SYSTEM_BUS_ADDRESS=unix:path=/nonexistent \
  cargo "$@"

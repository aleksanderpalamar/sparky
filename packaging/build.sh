#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="$root/packaging/out"
target="x86_64-unknown-linux-gnu"
glibc="2.36"
package_release="$(sed -n 's/^pkgrel=//p' "$root/packaging/arch/PKGBUILD")"
binaries=(sparky sparky-bar sparky-msg sparky-shot sparky-files sparky-image sparky-monitor sparky-settings sparky-launcher sparky-keyring sparky-keyring-prompt sparky-terminal sparky-notifications polkit-sparky)

usage() {
  echo "uso: packaging/build.sh [deb|rpm|pacman|todos]" >&2
  exit 2
}

compile() {
  RUSTFLAGS="-C link-arg=-Wl,--allow-shlib-undefined" \
    cargo zigbuild --release --locked --workspace --target "$target.$glibc"
}

newest_glibc() {
  objdump -T "$1" | grep -o 'GLIBC_[0-9.]*' | sed 's/GLIBC_//' | sort -V | tail -1
}

check_glibc() {
  local binary newest
  for binary in "${binaries[@]}"; do
    newest="$(newest_glibc "$root/target/$target/release/$binary")"
    if [[ -z "$newest" ]]; then
      echo "$binary não foi compilado" >&2
      return 1
    fi
    if [[ "$(printf '%s\n%s\n' "$newest" "$glibc" | sort -V | tail -1)" != "$glibc" ]]; then
      echo "$binary pede glibc $newest, acima de $glibc" >&2
      return 1
    fi
  done
}

portable() {
  compile
  check_glibc
}

deb() {
  local arguments=(--package sparky-compositor --no-build --target "$target" --output "$out" --deb-revision "$package_release")
  cargo deb "${arguments[@]}"
  cargo deb "${arguments[@]}" --variant wallpapers
}

rpm() {
  local arguments=(--package crates/sparky-compositor --target "$target" --output "$out" --set-metadata "release=\"$package_release\"")
  cargo generate-rpm "${arguments[@]}"
  cargo generate-rpm "${arguments[@]}" --variant wallpapers
}

pacman() {
  (cd "$root/packaging/arch" && PKGDEST="$out" makepkg --force --cleanbuild)
}

main() {
  [[ "$package_release" =~ ^[1-9][0-9]*$ ]] || { echo "pkgrel inválido no PKGBUILD" >&2; return 1; }
  mkdir -p "$out"
  cd "$root"
  case "${1:-todos}" in
    deb)
      portable
      deb
      ;;
    rpm)
      portable
      rpm
      ;;
    pacman) pacman ;;
    todos)
      portable
      deb
      rpm
      pacman
      ;;
    *) usage ;;
  esac
}

main "$@"

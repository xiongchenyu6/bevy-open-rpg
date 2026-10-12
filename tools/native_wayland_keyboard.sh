#!/usr/bin/env bash
# Persistent standard-US keyboard for an explicitly owned test compositor.
# nix develop --command bash -c 'WAYLAND_DISPLAY=wayland-2 tools/native_wayland_keyboard.sh'
# Send tap KEY / hold KEY MS / press KEY / release KEY / quit on stdin.
# No per-command keymap replacement, game hooks, or application-state access.
set -euo pipefail
: "${WAYLAND_DISPLAY:?Set WAYLAND_DISPLAY to the owned test compositor}"
tool_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
build_dir=$(mktemp -d "${TMPDIR:-/tmp}/love-rpg-native-keyboard.XXXXXX")
trap 'rm -rf "$build_dir"' EXIT
if command -v wayland-scanner >/dev/null; then
    scanner=$(command -v wayland-scanner)
else
    scanner_root=$(nix build --no-link --print-out-paths nixpkgs#wayland-scanner)
    scanner="$scanner_root/bin/wayland-scanner"
fi
"$scanner" client-header "$tool_dir/virtual-keyboard-unstable-v1.xml" "$build_dir/virtual-keyboard.h"
"$scanner" private-code "$tool_dir/virtual-keyboard-unstable-v1.xml" "$build_dir/virtual-keyboard.c"
read -r -a flags <<< "$(pkg-config --cflags --libs xkbcommon wayland-client)"
cc -Wall -I"$build_dir" "$tool_dir/native_wayland_keyboard.c" "$build_dir/virtual-keyboard.c" "${flags[@]}" -o "$build_dir/controller"
"$build_dir/controller"

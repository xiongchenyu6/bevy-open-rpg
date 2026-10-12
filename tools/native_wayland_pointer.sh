#!/usr/bin/env bash
# Run inside nix develop with an explicit owned compositor:
# WAYLAND_DISPLAY=wayland-2 tools/native_wayland_pointer.sh
# Keep the process alive; send move X Y / press / release / click / quit on stdin.
# Absolute coordinates use the project's 1280 x 720 native capture viewport.
set -euo pipefail
: "${WAYLAND_DISPLAY:?Set WAYLAND_DISPLAY to the owned test compositor}"
tool_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
build_dir=$(mktemp -d "${TMPDIR:-/tmp}/love-rpg-native-pointer.XXXXXX")
trap 'rm -rf "$build_dir"' EXIT
if command -v wayland-scanner >/dev/null; then
    scanner=$(command -v wayland-scanner)
else
    scanner_root=$(nix build --no-link --print-out-paths nixpkgs#wayland-scanner)
    scanner="$scanner_root/bin/wayland-scanner"
fi
"$scanner" client-header "$tool_dir/wlr-virtual-pointer-unstable-v1.xml" "$build_dir/virtual-pointer.h"
"$scanner" private-code "$tool_dir/wlr-virtual-pointer-unstable-v1.xml" "$build_dir/virtual-pointer.c"
read -r -a flags <<< "$(pkg-config --cflags --libs wayland-client)"
cc -I"$build_dir" "$tool_dir/native_wayland_pointer.c" "$build_dir/virtual-pointer.c" "${flags[@]}" -o "$build_dir/controller"
"$build_dir/controller"

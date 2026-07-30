#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${1:-$ROOT/screenshots/map-harness}"

MAP_NAMES=(
  village bamboo cave moon-corridor river-town river-reed
  plague-village plague-shrine capital capital-mansion mirror-gallery
  southern-road thunder-drum final-sanctum dream-waterway
)

mkdir -p "$OUT"
cd "$ROOT"

echo "[map-harness] checking topology, connectivity, space, and route variants"
cargo test --lib map_topology_

echo "[map-harness] building deterministic offscreen capture"
cargo build --bin capture

frames=()
for index in "${!MAP_NAMES[@]}"; do
  number="$(printf '%02d' "$((index + 1))")"
  name="${MAP_NAMES[$index]}"
  destination="$OUT/$number-$name"
  log="$destination/capture.log"
  mkdir -p "$destination"
  echo "[map-harness] $number/15 $name"
  if ! RUST_LOG=warn cargo run --bin capture -- \
    "$destination" 1 "route-map-$number" >"$log" 2>&1; then
    tail -n 80 "$log"
    exit 1
  fi
  frames+=("$destination/frame00000.png")
done

if command -v magick >/dev/null 2>&1; then
  magick montage "${frames[@]}" \
    -set label '%d' -tile 5x3 -geometry 320x180+6+20 \
    "$OUT/map-contact-sheet.png"
  echo "[map-harness] contact sheet: $OUT/map-contact-sheet.png"
else
  echo "[map-harness] ImageMagick missing; inspect each frame00000.png"
fi

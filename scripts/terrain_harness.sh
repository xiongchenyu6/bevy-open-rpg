#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${1:-$ROOT/screenshots/terrain-harness}"

mkdir -p "$OUT/run-scene" "$OUT/legacy-explore"
cd "$ROOT"

echo "[terrain-harness] checking world-space UV continuity"
cargo test --lib terrain_

echo "[terrain-harness] building deterministic offscreen capture"
cargo build --bin capture

capture_preset() {
  local destination="$1"
  local preset="$2"
  local log="$destination/capture.log"
  if ! RUST_LOG=warn cargo run --bin capture -- "$destination" 1 "$preset" >"$log" 2>&1; then
    tail -n 80 "$log"
    return 1
  fi
}

echo "[terrain-harness] capturing centered RunScene terrain blend"
capture_preset "$OUT/run-scene" terrain-harness

echo "[terrain-harness] capturing legacy Explore river town"
capture_preset "$OUT/legacy-explore" river-town

if command -v magick >/dev/null 2>&1; then
  magick montage \
    "$OUT/run-scene/frame00000.png" \
    "$OUT/legacy-explore/frame00000.png" \
    -tile 2x1 -geometry 640x360+8+8 \
    "$OUT/terrain-contact-sheet.png"
  echo "[terrain-harness] contact sheet: $OUT/terrain-contact-sheet.png"
else
  echo "[terrain-harness] ImageMagick missing; inspect the two frame00000.png files"
fi

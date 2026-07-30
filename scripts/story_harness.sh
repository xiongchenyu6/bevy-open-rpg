#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${1:-$ROOT/screenshots/story-harness}"

mkdir -p "$OUT/southern-drum" "$OUT/finale-bell"
cd "$ROOT"

echo "[story-harness] checking capacity, authored choices, and no-repeat draws"
cargo test --lib story_content_

echo "[story-harness] building deterministic dialogue captures"
cargo build --bin capture

capture_story() {
  local destination="$1"
  local preset="$2"
  local log="$destination/capture.log"
  if ! RUST_LOG=warn cargo run --bin capture -- \
    "$destination" 1 "$preset" >"$log" 2>&1; then
    tail -n 80 "$log"
    return 1
  fi
}

capture_story "$OUT/southern-drum" rogue-story-south
capture_story "$OUT/finale-bell" rogue-story-final

if command -v magick >/dev/null 2>&1; then
  magick montage \
    "$OUT/southern-drum/frame00000.png" \
    "$OUT/finale-bell/frame00000.png" \
    -tile 2x1 -geometry 640x360+8+8 \
    "$OUT/story-contact-sheet.png"
  echo "[story-harness] contact sheet: $OUT/story-contact-sheet.png"
else
  echo "[story-harness] ImageMagick missing; inspect the two frame00000.png files"
fi

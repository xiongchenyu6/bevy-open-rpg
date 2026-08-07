#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT="${1:-$ROOT/docs/showcase.gif}"
FPS="${README_GIF_FPS:-8}"
WIDTH="${README_GIF_WIDTH:-960}"
MAX_BYTES="${README_GIF_MAX_BYTES:-12582912}"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/love-rpg-readme-gif.XXXXXX")"

cleanup() {
  if [[ "${KEEP_README_GIF_FRAMES:-0}" == "1" ]]; then
    printf '[readme-gif] kept capture frames: %s\n' "$WORK"
  else
    rm -rf "$WORK"
  fi
}
trap cleanup EXIT

cd "$ROOT"
for command in cargo ffmpeg magick rg; do
  if ! command -v "$command" >/dev/null 2>&1; then
    printf '[readme-gif] missing command: %s\n' "$command" >&2
    exit 69
  fi
done

capture() {
  local preset="$1"
  local frames="$2"
  local destination="$WORK/$preset"
  local log="$destination/capture.log"

  mkdir -p "$destination"
  printf '[readme-gif] capture %-20s %s frames\n' "$preset" "$frames"
  if ! RUST_LOG=error cargo run --quiet --bin capture -- \
    "$destination" "$frames" "$preset" >"$log" 2>&1; then
    tail -n 100 "$log" >&2
    exit 1
  fi
  if rg -q 'Path not found|\bERROR\b|panicked at' "$log"; then
    rg -n 'Path not found|\bERROR\b|panicked at' "$log" >&2
    exit 1
  fi
}

cargo build --quiet --bin capture
capture route-map-12 1
capture chapter-card 380
capture rogue-story-south 1
capture battle-final 120

sequence="$WORK/sequence"
mkdir -p "$sequence"
sequence_index=0

add_frame() {
  local source="$1"
  local destination
  destination="$(printf '%s/frame%05d.png' "$sequence" "$sequence_index")"
  ln -s "$source" "$destination"
  sequence_index=$((sequence_index + 1))
}

add_hold() {
  local source="$1"
  local count="$2"
  for _ in $(seq 1 "$count"); do
    add_frame "$source"
  done
}

add_range() {
  local directory="$1"
  local first="$2"
  local last="$3"
  local step="$4"
  local frame
  for frame in $(seq "$first" "$step" "$last"); do
    add_frame "$(printf '%s/frame%05d.png' "$directory" "$frame")"
  done
}

# Establish the world, prove the tracked main task, then move through story and combat.
add_hold "$WORK/route-map-12/frame00000.png" 12
add_hold "$WORK/chapter-card/frame00208.png" 10
add_hold "$WORK/chapter-card/frame00260.png" 12
add_hold "$WORK/rogue-story-south/frame00000.png" 12
add_range "$WORK/battle-final" 0 116 4
add_hold "$WORK/route-map-12/frame00000.png" 4

mkdir -p "$(dirname "$OUTPUT")"
printf '[readme-gif] encode %s frames at %sfps, width=%s\n' "$sequence_index" "$FPS" "$WIDTH"
ffmpeg -hide_banner -loglevel error -y \
  -framerate "$FPS" -i "$sequence/frame%05d.png" \
  -filter_complex \
  "[0:v]scale=${WIDTH}:-2:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=128:stats_mode=diff[p];[s1][p]paletteuse=dither=bayer:bayer_scale=4:diff_mode=rectangle" \
  -loop 0 "$OUTPUT"

width="$(magick identify -format '%w' "${OUTPUT}[0]")"
height="$(magick identify -format '%h' "${OUTPUT}[0]")"
frame_count="$(magick identify "$OUTPUT" | wc -l)"
size_bytes="$(stat -c '%s' "$OUTPUT")"

if [[ "$width" != "$WIDTH" || "$height" -le 0 || "$frame_count" -lt 60 ]]; then
  printf '[readme-gif] invalid output: %sx%s, frames=%s\n' \
    "$width" "$height" "$frame_count" >&2
  exit 65
fi
if ((size_bytes > MAX_BYTES)); then
  printf '[readme-gif] output too large: %s bytes (limit %s)\n' \
    "$size_bytes" "$MAX_BYTES" >&2
  exit 65
fi

printf '[readme-gif] PASS: %s (%sx%s, %s frames, %s bytes)\n' \
  "$OUTPUT" "$width" "$height" "$frame_count" "$size_bytes"

#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if [[ $# -lt 2 || $# -gt 3 ]]; then
  printf 'usage: %s INPUT OUTPUT [SIZE]\n' "$0" >&2
  exit 64
fi

input="$1"
output="$2"
size="${3:-512}"
model="${CUTOUT_MODEL:-u2net}"

if [[ ! -f "$input" ]]; then
  printf 'cutout input not found: %s\n' "$input" >&2
  exit 66
fi
if [[ ! "$size" =~ ^[1-9][0-9]*$ ]]; then
  printf 'cutout size must be a positive integer: %s\n' "$size" >&2
  exit 64
fi
if ! command -v magick >/dev/null 2>&1; then
  printf 'ImageMagick 7 is required (missing magick)\n' >&2
  exit 69
fi

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/bevy-cutout.XXXXXX")"
trap 'rm -rf "$tmp_dir"' EXIT
segmented="$tmp_dir/segmented.png"

if command -v rembg >/dev/null 2>&1; then
  rembg i -m "$model" "$input" "$segmented"
elif command -v uvx >/dev/null 2>&1; then
  uvx --python 3.11 --from 'rembg[cpu,cli]' rembg i -m "$model" "$input" "$segmented"
else
  printf 'semantic cutout requires rembg or uvx\n' >&2
  exit 69
fi

mkdir -p "$(dirname "$output")"
magick "$segmented" \
  -trim +repage \
  -resize "${size}x${size}" \
  -background none -gravity center -extent "${size}x${size}" \
  "$output"

IFS='|' read -r width height channels opaque alpha_mean <<<"$(
  magick identify -format '%w|%h|%[channels]|%[opaque]|%[fx:mean.a]' "$output"
)"

if [[ "$width" != "$size" || "$height" != "$size" ]]; then
  printf 'cutout size check failed: %sx%s (expected %sx%s)\n' "$width" "$height" "$size" "$size" >&2
  exit 65
fi
if [[ "$channels" != *a* || "$opaque" != "False" ]]; then
  printf 'cutout alpha check failed: channels=%s opaque=%s\n' "$channels" "$opaque" >&2
  exit 65
fi
if ! awk -v alpha="$alpha_mean" 'BEGIN { exit !(alpha >= 0.02 && alpha <= 0.90) }'; then
  printf 'cutout coverage check failed: alpha_mean=%s (expected 0.02..0.90)\n' "$alpha_mean" >&2
  exit 65
fi

printf 'cutout PASS: %s -> %s (%sx%s, alpha_mean=%s, model=%s)\n' \
  "$input" "$output" "$width" "$height" "$alpha_mean" "$model"

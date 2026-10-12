#!/usr/bin/env bash
# Run inside nix develop. Captures the actual GamePlugin, not an atlas preview.
set -euo pipefail
cd "$(dirname "$0")/.."
out="${1:-screenshots/enemy-motion}"
mkdir -p "$out"
cargo fmt --check > "$out/fmt.log" 2>&1
cargo check --all-targets > "$out/check.log" 2>&1
cargo test --lib > "$out/tests.log" 2>&1
cargo build --bin capture --bin love-rpg > "$out/build.log" 2>&1
export BEVY_ASSET_ROOT="$PWD" RUST_LOG=warn CAPTURE_UI_AUDIT=1
target/debug/capture "$out" 450 rogue-enemy-motion > "$out/capture.log" 2>&1
ffmpeg -y -loglevel error -framerate 30 -i "$out/frame%05d.png" \
  -c:v libx264 -pix_fmt yuv420p -preset medium -crf 21 -movflags +faststart "$out/video.mp4"

#!/usr/bin/env bash
# Run inside nix develop. Normal native mouse/keyboard review is separate.
set -euo pipefail
repo_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
out_dir=${1:-"$repo_dir/screenshots/location-harness"}
mkdir -p "$out_dir"
cd "$repo_dir"
cargo fmt --check
cargo check --all-targets
cargo test --lib
cargo build --bin capture --bin playtest
export BEVY_ASSET_ROOT="$repo_dir" RUST_LOG=warn
for location in $(seq 1 15); do
  mkdir -p "$out_dir/map-$location"
  ./target/debug/capture "$out_dir/map-$location" 1 "route-map-$location" >"$out_dir/map-$location/capture.log" 2>&1
done
mkdir -p "$out_dir/tour" "$out_dir/full-route"
./target/debug/capture "$out_dir/tour" 450 rogue-place-tour >"$out_dir/tour/capture.log" 2>&1
ffmpeg -hide_banner -loglevel error -y -framerate 30 -i "$out_dir/tour/frame%05d.png" -c:v libx264 -pix_fmt yuv420p "$out_dir/video.mp4"
./target/debug/capture "$out_dir/full-route" 60000 rogue-audit >"$out_dir/full-route/capture.log" 2>&1

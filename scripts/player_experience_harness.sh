#!/usr/bin/env bash
# Run inside nix develop. Native input evidence is recorded separately.
set -euo pipefail
repo_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
out_dir=${1:-"$repo_dir/screenshots/player-experience"}
mkdir -p "$out_dir"
cd "$repo_dir"
cargo fmt --check
cargo check
cargo test --lib
cargo build --bin capture
export BEVY_ASSET_ROOT="$repo_dir"
export RUST_LOG=warn
capture_case() {
    local name=$1 preset=$2 size=$3 frames=$4
    mkdir -p "$out_dir/$name"
    CAPTURE_SIZE="$size" ./target/debug/capture "$out_dir/$name" "$frames" "$preset" >"$out_dir/$name/capture.log" 2>&1
}
capture_case puzzle-choice rogue-puzzle-choice 1280x720 1
capture_case puzzle-wrong rogue-puzzle-wrong 1280x720 1
capture_case dialogue-portrait rogue-story-opening 720x1280 1
capture_case dialogue-small rogue-story-opening 960x540 1
capture_case full-route rogue-audit 1280x720 60000

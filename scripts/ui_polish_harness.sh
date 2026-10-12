#!/usr/bin/env bash
# Run inside nix develop: real gameplay screenshots, UI bounds and dense scroll proofs.
set -euo pipefail
cd "$(dirname "$0")/.."
out="${1:-screenshots/ui-polish}"
mkdir -p "$out"
export RUST_LOG=warn
export WGPU_BACKEND=vulkan
export CAPTURE_UI_AUDIT=1
cargo fmt --check > "$out/fmt.log" 2>&1
cargo check > "$out/check.log" 2>&1
cargo test --lib > "$out/tests.log" 2>&1
cargo build > "$out/build.log" 2>&1
capture() {
    local preset="$1" frames="$2" destination="$3"
    cargo run --bin capture -- "$destination" "$frames" "$preset" > "$destination.log" 2>&1
    printf '[ui-polish] %s: %s frames\n' "$preset" "$frames"
}
capture rogue-party-battle 450 "$out"
capture route-map-02 24 "$out/map"
capture rogue-story-final 80 "$out/story"
capture rogue-polish-inventory 105 "$out/inventory"
capture rogue-polish-ending 90 "$out/ending"
capture rogue-status 450 "$out/status"
capture rogue-polish-skills 95 "$out/menu"
capture rogue-skill 900 "$out/skills"
CAPTURE_SIZE=960x540 capture rogue-party-battle 90 "$out/small"
ffmpeg -y -loglevel error -framerate 30 -i "$out/frame%05d.png" \
    -c:v libx264 -pix_fmt yuv420p -preset medium -crf 21 -movflags +faststart "$out/video.mp4"
printf '[ui-polish] proof bundle: %s\n' "$out"

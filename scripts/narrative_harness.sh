#!/usr/bin/env bash
# Run inside nix develop. All rewards and simulations use production systems.
set -euo pipefail
cd "$(dirname "$0")/.."
out="${1:-screenshots/narrative-harness}"
mkdir -p "$out"
cargo fmt --check > "$out/fmt.log" 2>&1
cargo check > "$out/check.log" 2>&1
cargo test --lib > "$out/tests.log" 2>&1
NARRATIVE_REWARD_OUT="$out/reward-contracts.csv" cargo test --lib narrative_reward_audit -- --ignored --nocapture > "$out/reward-audit.log" 2>&1
NARRATIVE_COMBAT_OUT="$out/reward-combat.csv" cargo test --lib narrative_reward_combat_sweep -- --ignored --nocapture > "$out/reward-combat.log" 2>&1
NARRATIVE_SIDE_COMBAT_OUT="$out/side-combat.csv" cargo test --lib side_story_reward_combat_sweep -- --ignored --nocapture > "$out/side-combat.log" 2>&1
python3 tools/summarize_narrative_rewards.py "$out"
cargo build > "$out/build.log" 2>&1
export RUST_LOG=warn
export BEVY_ASSET_ROOT="$PWD"
export CAPTURE_UI_AUDIT=1
cargo run --bin capture -- "$out/fox" 450 rogue-side-fox > "$out/fox.log" 2>&1
cargo run --bin capture -- "$out/funds" 240 rogue-side-funds > "$out/funds.log" 2>&1
cargo run --bin capture -- "$out/qin" 240 rogue-side-qin > "$out/qin.log" 2>&1
cargo run --bin capture -- "$out/skill" 240 rogue-story-reward > "$out/skill.log" 2>&1
CAPTURE_SIZE=960x540 cargo run --bin capture -- "$out/small" 240 rogue-story-reward > "$out/small.log" 2>&1
cargo run --bin capture -- "$out/cost" 120 rogue-cost-proof > "$out/cost.log" 2>&1
ffmpeg -y -loglevel error -framerate 30 -i "$out/fox/frame%05d.png" -c:v libx264 -pix_fmt yuv420p -preset medium -crf 21 -movflags +faststart "$out/video.mp4"
ffmpeg -y -loglevel error -framerate 30 -i "$out/skill/frame%05d.png" -c:v libx264 -pix_fmt yuv420p -preset medium -crf 21 -movflags +faststart "$out/story-reward.mp4"
if command -v magick >/dev/null 2>&1; then
  magick montage "$out/fox/frame00030.png" "$out/fox/frame00185.png" "$out/fox/frame00320.png" "$out/funds/frame00065.png" "$out/qin/frame00185.png" "$out/skill/frame00185.png" -tile 3x2 -geometry 640x360+8+8 "$out/contact-sheet.png"
fi
cargo run --bin capture -- "$out/run-audit" 45000 rogue-audit > "$out/run-audit.log" 2>&1
rg '^run_audit_status=PASS$' "$out/run-audit/run-audit.txt"

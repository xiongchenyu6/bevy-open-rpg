#!/usr/bin/env bash
# Run inside nix develop; reuses production combat systems for every verification.
set -euo pipefail
cd "$(dirname "$0")/.."
out="${1:-screenshots/combat-harness}"
mkdir -p "$out"
cargo fmt --check > "$out/fmt.log" 2>&1
cargo check > "$out/check.log" 2>&1
cargo test --lib > "$out/tests.log" 2>&1
COMBAT_SWEEP_OUT="$out/combat-sweep.csv" cargo test --lib combat_policy_sweep -- --ignored --nocapture > "$out/sweep.log" 2>&1
python tools/summarize_combat_sweep.py "$out/combat-sweep.csv" "$out/combat-sweep.md"
cargo build > "$out/build.log" 2>&1
export RUST_LOG=warn
export CAPTURE_UI_AUDIT=1
cargo run --bin capture -- "$out" 450 rogue-tactics > "$out/capture.log" 2>&1
CAPTURE_SIZE=960x540 cargo run --bin capture -- "$out/small" 240 rogue-tactics > "$out/small.log" 2>&1
ffmpeg -y -loglevel error -framerate 30 -i "$out/frame%05d.png" -c:v libx264 -pix_fmt yuv420p -preset medium -crf 21 -movflags +faststart "$out/video.mp4"
cargo run --bin capture -- "$out/run-audit" 45000 rogue-audit > "$out/run-audit.log" 2>&1
rg '^run_audit_status=PASS$' "$out/run-audit/run-audit.txt"

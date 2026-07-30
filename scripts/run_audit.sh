#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${1:-$ROOT/screenshots/run-audit}"
MAX_FRAMES="${MAX_FRAMES:-45000}"
VERIFY_DETERMINISM="${VERIFY_DETERMINISM:-0}"
LOG="$OUT/capture.log"
REPORT="$OUT/run-audit.txt"

mkdir -p "$OUT"
cd "$ROOT"

echo "[run-audit] checking runtime ledger and boss-task settlement"
cargo test --lib run_play_time_tracks_total_and_chapter_ledgers
cargo test --lib chapter_clear_archives_unclaimed_boss_task_once

echo "[run-audit] building capture before the deterministic run"
cargo build --bin capture
rm -f "$REPORT"

echo "[run-audit] traversing all 41 stages (frame budget: $MAX_FRAMES)"
if ! RUST_LOG=error cargo run --quiet --bin capture -- \
  "$OUT" "$MAX_FRAMES" rogue-audit >"$LOG" 2>&1; then
  test -f "$REPORT" && sed -n '1,28p' "$REPORT"
  tail -n 80 "$LOG"
  exit 1
fi

if ! rg -q '^run_audit_status=PASS$' "$REPORT"; then
  echo "[run-audit] capture exited successfully without a PASS report"
  sed -n '1,28p' "$REPORT"
  exit 1
fi

if [[ "$VERIFY_DETERMINISM" == "1" ]]; then
  REPEAT_OUT="$OUT/repeat"
  REPEAT_LOG="$REPEAT_OUT/capture.log"
  REPEAT_REPORT="$REPEAT_OUT/run-audit.txt"
  DIFF="$OUT/determinism.diff"
  mkdir -p "$REPEAT_OUT"
  rm -f "$REPEAT_REPORT" "$DIFF"
  echo "[run-audit] repeating the fixed-seed run for byte-level comparison"
  if ! RUST_LOG=error cargo run --quiet --bin capture -- \
    "$REPEAT_OUT" "$MAX_FRAMES" rogue-audit >"$REPEAT_LOG" 2>&1; then
    test -f "$REPEAT_REPORT" && sed -n '1,28p' "$REPEAT_REPORT"
    tail -n 80 "$REPEAT_LOG"
    exit 1
  fi
  if ! diff -u \
    <(rg -v '^wall_seconds=' "$REPORT") \
    <(rg -v '^wall_seconds=' "$REPEAT_REPORT") >"$DIFF"; then
    echo "[run-audit] fixed-seed reports diverged"
    cat "$DIFF"
    exit 1
  fi
  rm -f "$DIFF"
  echo "[run-audit] deterministic repeat matched"
fi

sed -n '1,25p' "$REPORT"
echo "[run-audit] report: $REPORT"

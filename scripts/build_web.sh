#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# Guard: WebGPU has no rgba16unorm. bevy loads 16-bit PNGs as TextureFormat::Rgba16Unorm
# and the web build then panics at boot. Fail early if any 16-bit PNG snuck in.
sixteen_bit_pngs="$(find assets -name '*.png' -type f -print0 2>/dev/null |
  xargs -0 -r file | grep -i '16-bit' | cut -d: -f1 || true)"
if [ -n "$sixteen_bit_pngs" ]; then
  echo "ERROR: 16-bit PNG(s) found — WebGPU has no Rgba16Unorm. Downconvert to 8-bit:" >&2
  echo "$sixteen_bit_pngs" | sed 's/^/  /' >&2
  echo "  fix: magick FILE -depth 8 PNG32:FILE" >&2
  exit 1
fi

engine_inputs_sha256="$(python3 tools/web_engine_inputs.py)"
if [ -n "${WASM_REUSE_BASE_URL:-}" ] && python3 tools/reuse_web_engine.py --base-url "$WASM_REUSE_BASE_URL" --output web --fingerprint "$engine_inputs_sha256"; then
  echo "Reusing verified published engine bytes."
else
# The wasm-bindgen CLI MUST match the wasm-bindgen crate version from Cargo.lock.
WASM_BINDGEN_VERSION="${WASM_BINDGEN_VERSION:-$(
  awk '/^name = "wasm-bindgen"$/ {getline; print; exit}' Cargo.lock |
    sed -E 's/.*"([0-9.]+)".*/\1/'
)}"
echo ">> wasm-bindgen crate version: ${WASM_BINDGEN_VERSION}"

wasm_bindgen_version_ok() {
  "$1" --version 2>/dev/null | grep -q "wasm-bindgen ${WASM_BINDGEN_VERSION}\$"
}

resolve_wasm_bindgen() {
  local tool_root="$ROOT/target/web-tools/wasm-bindgen-${WASM_BINDGEN_VERSION}"
  local isolated="$tool_root/bin/wasm-bindgen"
  if [ -x "$isolated" ] && wasm_bindgen_version_ok "$isolated"; then
    printf '%s\n' "$isolated"
    return
  fi
  local candidate
  for candidate in "${CARGO_HOME:-$HOME/.cargo}/bin/wasm-bindgen" "$(command -v wasm-bindgen 2>/dev/null || true)"; do
    if [ -x "$candidate" ] && wasm_bindgen_version_ok "$candidate"; then
      mkdir -p "$tool_root/bin"
      cp "$candidate" "$isolated"
      if wasm_bindgen_version_ok "$isolated"; then
        printf '%s\n' "$isolated"
        return
      fi
    fi
  done
  echo ">> installing isolated wasm-bindgen-cli ${WASM_BINDGEN_VERSION}" >&2
  cargo install -q --locked wasm-bindgen-cli --version "$WASM_BINDGEN_VERSION" --root "$tool_root"
  wasm_bindgen_version_ok "$isolated" || return 1
  printf '%s\n' "$isolated"
}

WASM_BINDGEN_BIN="$(resolve_wasm_bindgen)"

# Pin binaryen (wasm-opt): apt's build misassigns the externref table and the
# web build boot-fails. Use a pinned recent release; on failure ship
# un-optimized (but valid) wasm.
BINARYEN_VERSION="${BINARYEN_VERSION:-version_130}"
BINARYEN_MIN_NUM="${BINARYEN_VERSION#version_}"
wasm_opt_num() { "$1" --version 2>/dev/null | sed -nE 's/.*wasm-opt version ([0-9]+).*/\1/p'; }
resolve_wasm_opt() {
  local cargo_home="${CARGO_HOME:-$HOME/.cargo}"
  local pin_bin="$cargo_home/bin/wasm-opt"
  local cand n
  for cand in "$pin_bin" "$(command -v wasm-opt 2>/dev/null || true)"; do
    [ -n "$cand" ] && [ -x "$cand" ] || continue
    n="$(wasm_opt_num "$cand")"
    if [ -n "$n" ] && [ "$n" -ge "$BINARYEN_MIN_NUM" ]; then
      printf '%s\n' "$cand"
      return 0
    fi
  done
  echo ">> downloading binaryen ${BINARYEN_VERSION} (system wasm-opt missing/too old)" >&2
  local tmp
  tmp="$(mktemp -d)"
  if curl -fsSL "https://github.com/WebAssembly/binaryen/releases/download/${BINARYEN_VERSION}/binaryen-${BINARYEN_VERSION}-x86_64-linux.tar.gz" -o "$tmp/b.tgz" &&
    tar xzf "$tmp/b.tgz" -C "$tmp"; then
    mkdir -p "$cargo_home/bin"
    cp "$tmp/binaryen-${BINARYEN_VERSION}/bin/wasm-opt" "$pin_bin"
    rm -rf "$tmp"
    printf '%s\n' "$pin_bin"
    return 0
  fi
  rm -rf "$tmp"
  return 1
}
WASM_OPT_BIN="$(resolve_wasm_opt || true)"
[ -n "$WASM_OPT_BIN" ] && echo ">> wasm-opt: $("$WASM_OPT_BIN" --version)"

# Only the game bin — the capture bin is a native-only offscreen recorder.
cargo build --release --target wasm32-unknown-unknown --features webgpu --bin love-rpg

rm -rf web/pkg
mkdir -p web/pkg
"$WASM_BINDGEN_BIN" \
  --target web \
  --out-dir web/pkg \
  --out-name love_rpg \
  target/wasm32-unknown-unknown/release/love-rpg.wasm

if [ -n "$WASM_OPT_BIN" ]; then
  BG="web/pkg/love_rpg_bg.wasm"
  # Enable exactly the stable, browser-shipped post-MVP features rustc emits.
  # Do NOT use --all-features (turns on experimental proposals the browser
  # can't parse). Retry a few times, ship un-optimized on repeated failure.
  opt_ok=0
  for attempt in 1 2 3; do
    rm -f "$BG.opt"
    if "$WASM_OPT_BIN" -Oz \
      --enable-reference-types \
      --enable-bulk-memory \
      --enable-nontrapping-float-to-int \
      --enable-sign-ext \
      --enable-mutable-globals \
      --enable-multivalue \
      -o "$BG.opt" "$BG" && [ -f "$BG.opt" ]; then
      mv "$BG.opt" "$BG"
      opt_ok=1
      break
    fi
    echo ">> wasm-opt attempt $attempt failed, retrying..." >&2
    sleep 2
  done
  if [ "$opt_ok" != 1 ]; then
    echo ">> wasm-opt failed 3×; shipping the un-optimized wasm (functional, larger)" >&2
  fi
fi

# Precompressed copy the loader streams with a real progress bar.
gzip -n -9 -kf web/pkg/love_rpg_bg.wasm

fi

python3 - "$engine_inputs_sha256" <<'PYBUILD'
import hashlib, json, pathlib, shutil, subprocess, sys
sys.path.insert(0, 'tools')
from web_engine_inputs import fingerprint
if fingerprint(pathlib.Path.cwd())[0] != sys.argv[1]:
    raise SystemExit('Compiled inputs changed during build; rebuild before publishing.')
root=pathlib.Path('web')
shutil.rmtree(root/'assets', ignore_errors=True)
shutil.copytree('assets',root/'assets',ignore=lambda directory,names: ['generated'] if pathlib.Path(directory)==pathlib.Path('assets') else [])
wasm=root/'pkg/love_rpg_bg.wasm'
(root/'build.json').write_text(json.dumps({'revision':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'engine_inputs_sha256':sys.argv[1],'wasm_sha256':hashlib.sha256(wasm.read_bytes()).hexdigest(),'wasm_bytes':wasm.stat().st_size},indent=2)+'\n')
PYBUILD
python3 tools/generate_web_manifest.py web --revision "$(git rev-parse HEAD)" --template tools/sw-template.js

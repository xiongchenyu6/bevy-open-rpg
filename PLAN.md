# PLAN — love-rpg (仙剑-style demo)

## Verification criteria (what "done" means)
- App launches on desktop with no errors/`B0004`/missing-asset warnings.
- Overworld: player walks 4 directions, blocked by trees/water, Chinese HUD renders.
- NPC dialogue box opens on Space and advances through lines.
- Stepping into grass can trigger a battle (state switch).
- Battle: command menu (攻击/仙术/物品/逃跑) navigable; damage, MP cost, healing,
  enemy AI turn, HP bars, win/lose/flee all function and return to the map.
- Chinese text renders correctly (bundled unifont).
- A `screenshots/result/N/` bundle (video.mp4 + raw frames) proves the loop.

## Risk
- **R1 Bevy 0.19 API correctness** — RESOLVED via bevy-help; compiles clean.
- **R2 Native build deps (alsa/udev/wayland pkg-config)** — RESOLVED by trimming
  audio/gamepad deps and providing Wayland/pkg-config deps in `flake.nix`.
- **R3 CJK text rendering** — bundled unifont.otf; verify on screenshot.
- **R4 Offscreen capture path** — to build for the proof bundle.

## Stages
1. Scaffold (Cargo.toml, lib, state, core) — **Complete**
2. Overworld explore (map/movement/NPC/dialogue/encounter) — **Complete**
3. Turn-based battle (menu/HP-MP/AI/exp) — **Complete**
4. Build + runtime verification — **Complete** (headless lavapipe render)
5. Offscreen capture binary + proof bundle — **Complete**

## Status — DONE
- `cargo fmt`/`check`/`build` clean.
- Visually verified via offscreen capture (lavapipe): overworld map + HUD,
  NPC dialogue box, and turn-based battle (menu/HP bars/log) all render with
  correct Chinese text (bundled unifont).
- Proof bundle: `screenshots/result/1/` (450 frames + `video.mp4`, 30 fps / 15s).
- See MEMORY.md for the exact env needed to run the capture binary.

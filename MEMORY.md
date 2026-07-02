# Memory — love-rpg

Key discoveries and decisions for resuming work.

## Engine version
- Targeting **Bevy 0.19.0**, NOT the scaffold's default 0.18.1. Reason: the local
  `bevy-help` docs cache is 0.19.0, so matching code to available docs avoids API churn.

## Build environment (important)
- Host is NixOS-like; native runtime libraries exist in `/nix/store` but are
  **not on the dynamic loader path** unless the dev shell exposes them.
- Therefore Bevy `default-features = false` with a trimmed 2D feature set. Dropped:
  - `bevy_audio` (→ alsa-sys / pkg-config) and `bevy_gilrs` (→ libudev-sys / pkg-config)
- Kept `wayland` and dropped `x11`, matching `/home/freeman.xiong/Desktop/protect-carrot`.
  `flake.nix` provides `wayland`, `libxkbcommon`, `pkg-config`, and `vulkan-loader`
  so the Wayland backend can build and run.
- `WAYLAND_DISPLAY=wayland-1` is available → real desktop `cargo run` works inside
  `nix develop` / `direnv`. `xvfb-run` is NOT installed.
- `ffmpeg` IS available for encoding capture video.

## Bevy 0.19 API notes (verified via bevy-help)
- 0.19 split sprite/UI render: need BOTH `bevy_sprite`+`bevy_sprite_render` and
  `bevy_ui_widgets`(includes bevy_ui)+`bevy_ui_render`, else components compile but never draw.
- `WindowResolution: From<(u32,u32)>` — pass integers, not floats.
- States: derive `States` + `init_state` + `DespawnOnExit(state)` (StateScoped is gone);
  `NextState::set`.
- Text: `Text`/`Text2d` are newtypes (`.0` string), `TextFont{font, font_size}` (f32 → FontSize
  via .into()), `TextColor`, `BackgroundColor`; layout via `Node`.
- `Sprite::from_color(color, size)` for solid sprites (no image asset needed).
- Default embedded font is Latin-only → bundled `assets/fonts/unifont.otf` for Chinese.

## Font
- `unifont.otf` copied from nix store (`unifont-17.0.04`). Single-face OTF covering CJK.
  Noto/Sarasa were `.ttc`/variable collections → skipped to avoid extraction complexity.

## Headless capture environment (hard-won)
- Offscreen capture binary: `src/bin/capture.rs`. Manually pumps `app.update()`
  (no winit, no ScheduleRunnerPlugin) per Bevy's `externally_driven_headless_renderer`.
- GPU: surfaceless wgpu finds NO adapter by default here. Fix = software Vulkan
  (lavapipe). Required env when running the capture binary:
  - `VK_ICD_FILENAMES=/run/opengl-driver/share/vulkan/icd.d/lvp_icd.x86_64.json`
  - `LD_LIBRARY_PATH=/nix/store/d6vnmfmcz5b299180issiaad4m96wh8k-vulkan-loader-1.4.341.0/lib:/run/opengl-driver/lib`
    (the `d6vnmfmc...` loader is the **64-bit** one; `1ajccbq...` is 32-bit — wrong ELF class, silently ignored).
  - `BEVY_ASSET_ROOT="$(pwd)"` (bare `./target/debug/capture` otherwise looks in `target/debug/assets`).
- Windowed app previously used X11 and failed because winit couldn't dlopen
  `libX11.so.6`; fixed by switching Bevy to the Wayland backend and adding the
  Wayland runtime/build libraries to `flake.nix`.
- ICU4X "No segmentation model for ja" lines during CJK text shaping are warnings, not errors.
- RESOLVED message-init panic: `bevy_ui_widgets::text_input::apply_queued_select_all`
  needs `MessageReader<Pointer<Release>>` (picking), which isn't initialized without
  the picking feature. Fix: use the `bevy_ui` Cargo feature instead of `bevy_ui_widgets`
  (we only use plain `Node`/`Text`, no interactive widgets). To name a panicking system,
  temporarily add the `debug` bevy feature (reveals ECS system/param names).

## Design choices
- No `rand` crate; tiny xorshift `Rng` resource instead.
- Grid-step movement with a 0.14s cooldown. Encounters only on grass tiles (16%).
- Player position persists across battles via `PlayerPos` resource; scenes fully
  despawn/respawn on state change.

## 肉鸽改造 (2026-07)
- 主循环换成 roguelike run:`AppState` 默认 `Title`;旧 capture preset 依赖
  Explore 起始,capture.rs 里对非 rogue preset 显式 `NextState::set(Explore)`。
- battle.rs 以 `Option<Res<RunState>>` 门控双流:有 RunState=肉鸽(无经验、
  法宝钩子、Reward/Ending 路由),无=旧线性战役,老逻辑零改动。
- 平衡:boss 定义按练级战役调的,run 模式统一打 0.72/0.68 折;第一章普通敌
  0.75/0.70 倍,否则开局竹林敌(21 atk)会灭掉 80 血新角色(实测 2/80 险胜)。
- `DespawnOnExit<AppState>` 不是 Copy——多实体共用作用域时写
  `let scope = || DespawnOnExit(...)` 闭包。
- rogue capture preset 用节奏脚本(非魔法帧号):非战斗每 18 帧 confirm、
  每 90 帧 down;战斗内每 14 帧 confirm(1.6x tempo 下 timer≈15 帧)。
  title_input 会用墙钟异或 Rng,故 rogue 捕获非严格确定,但足够出证明视频。

# AGENTS.md

This file provides guidance to coding agents when working with code in this repository.

## What this repo is

**love-rpg** (仙剑 · 御剑情缘) — a 仙剑奇侠传-style 2D turn-based RPG built
code-first in **Bevy 0.19.1**, generated and maintained by the `godogen` skill.
It is a single Cargo package (edition 2024), not a workspace.

- **`STRUCTURE.md` is the architecture source of truth.** Read it before
  changing modules; update it whenever module ownership, state, asset
  contracts, or verification commands change.
- `GAME_SCOPE.md` defines the 7-act story scope and what is implemented so far;
  expand content in chapters, not isolated scenes.
- `ASSETS.md` documents the generated-asset pipeline (remote ComfyUI via
  `tools/comfy_batch_assets.mjs`).
- Use `/godogen` to generate, rebuild, or substantially extend the game.
- Use the `bevy-help` skill for any Bevy question — API lookup, ECS ownership,
  rendering/UI, scheduling. Prefer it over guessing an API from memory.
- The `.Codex/skills/godogen/` stage files (`task-execution.md`,
  `scene-generation.md`, `quirks.md`, `capture.md`, asset-gen docs) are the
  detailed playbooks for each stage of work.

**love-rpg** (仙剑 · 御剑情缘) — a 仙剑奇侠传-style 2D turn-based RPG built
code-first in **Bevy 0.19.0**, generated and maintained by the `godogen` skill.
It is a single Cargo package (edition 2024), not a workspace.

- **`STRUCTURE.md` is the architecture source of truth.** Read it before
  changing modules; update it whenever module ownership, state, asset
  contracts, or verification commands change.
- `GAME_SCOPE.md` defines the 7-act story scope and what is implemented so far;
  expand content in chapters, not isolated scenes.
- `ASSETS.md` documents the generated-asset pipeline (remote ComfyUI via
  `tools/comfy_batch_assets.mjs`).
- Use `/godogen` to generate, rebuild, or substantially extend the game.
- Use the `bevy-help` skill for any Bevy question — API lookup, ECS ownership,
  rendering/UI, scheduling. Prefer it over guessing an API from memory.
- The `.claude/skills/godogen/` stage files (`task-execution.md`,
  `scene-generation.md`, `quirks.md`, `capture.md`, asset-gen docs) are the
  detailed playbooks for each stage of work.

## Environment & commands

Build/run must happen inside `nix develop` / direnv (NixOS host: Wayland,
libxkbcommon, Vulkan runtime libs come from `flake.nix` via `LD_LIBRARY_PATH`).

The standard loop (do not skip `cargo check`):

```bash
cargo fmt           # format before checking
cargo check         # fix compiler/Bevy API errors here first — cheaper than build
cargo test --lib    # verify gameplay and content invariants
cargo build         # only after check is clean
cargo run           # desktop smoke test (needs WAYLAND_DISPLAY)
```

Verification includes library unit tests, build, runtime, and visual inspection
of captured frames. When code and captured media disagree, trust the media.

### Offscreen capture (proof/screenshots)

Screenshots and video come from the dedicated headless capture binary
(`src/bin/capture.rs`), which renders the real `GamePlugin` offscreen with a
deterministic scripted `Intent` — not from `cargo run`:

```bash
cargo run --bin capture -- <out_dir> <frame_count> [preset]
```

Presets start the script at specific maps/scenes (`river-town`, `moon-cave`,
`capital`, `battle-river`, …) — see the doc comment at the top of
`src/bin/capture.rs` for the full list. Works headless via lavapipe. Details in
`.Codex/skills/godogen/capture.md`.

```bash
cargo fmt           # format before checking
cargo check         # fix compiler/Bevy API errors here first — cheaper than build
cargo build         # only after check is clean
cargo run           # desktop smoke test (needs WAYLAND_DISPLAY)
```

There is **no test suite**; verification is build + runtime + visual inspection
of captured frames. When code and captured media disagree, trust the media.

Presets start the script at specific maps/scenes (`river-town`, `moon-cave`,
`capital`, `battle-river`, …) — see the doc comment at the top of
`src/bin/capture.rs` for the full list. Works headless via lavapipe. Details in
`.claude/skills/godogen/capture.md`.

## Architecture

- `src/main.rs` — thin entrypoint calling `love_rpg::run()`.
- `src/lib.rs` — `App` construction: window config, plugin registration.
- `src/game/mod.rs` — `GamePlugin`, the integration point wiring feature
  modules: `state` (AppState: Explore | Battle), `core` (shared
  constants/PlayerStats/Rng), `explore`, `battle`, `quest`, `paperdoll`,
  `animation`, `lighting`, `fog`.
- Dialogue is an overlay inside Explore (a resource flag), not a state.
- Scene entities use `DespawnOnExit(<state>)`; persistent resources
  (`PlayerStats`, `PlayerPos`, `Rng`, `GameFont`) survive state switches.
- `assets/` — runtime-loaded files only; load paths strip the `assets/` prefix.
  All Chinese text must use `GameFont` (`assets/fonts/unifont.otf`) — the
  embedded default font is Latin-only.
- `vendor/bevy_firefly` — local `[patch.crates-io]` of `bevy_firefly`; keep the
  patch when touching `Cargo.toml`.

## Key constraints

- **Preserve Bevy 0.19.1 and the trimmed feature set** in `Cargo.toml`:
  default features are off; audio (`bevy_audio`/alsa) and gamepad
  (`bevy_gilrs`/udev) are deliberately disabled to avoid native system-lib
  deps; windowing is **Wayland only**. Only migrate versions or re-enable
  features when the user explicitly asks.
- Keep app wiring (`main.rs`/`lib.rs`) stable; adjust state, plugin, and module
  boundaries before filling in gameplay details.
- Imported/generated assets are a second pass: keep the code-owned wrapper
  entity and placement logic even when the visible asset is a sprite/scene.
  White or missing textures are usually a load path or missing image-format
  feature (e.g. `png`), not a spawn-code bug — check runtime logs first.

- **Preserve Bevy 0.19.0 and the trimmed feature set** in `Cargo.toml`:
  default features are off; audio (`bevy_audio`/alsa) and gamepad
  (`bevy_gilrs`/udev) are deliberately disabled to avoid native system-lib
  deps; windowing is **Wayland only**. Only migrate versions or re-enable
  features when the user explicitly asks.
- Keep app wiring (`main.rs`/`lib.rs`) stable; adjust state, plugin, and module
  boundaries before filling in gameplay details.
- Imported/generated assets are a second pass: keep the code-owned wrapper
  entity and placement logic even when the visible asset is a sprite/scene.
  White or missing textures are usually a load path or missing image-format
  feature (e.g. `png`), not a spawn-code bug — check runtime logs first.

## Git

`.gitignore` excludes `/target`, `/screenshots`, `.bevy-help.log`, and the
`.Codex`/`AGENTS.md` working files. Keep `Cargo.lock` tracked (this is an
application, not a library).

`.gitignore` excludes `/target`, `/screenshots`, `.bevy-help.log`, and the
`.claude`/`AGENTS.md` working files. Keep `Cargo.lock` tracked (this is an
application, not a library).

# 仙剑 · 御剑情缘 (love-rpg)

A 仙剑奇侠传-style 2D turn-based RPG demo built code-first in Bevy.

## Runtime

- Bevy **0.19.0** (default features off; trimmed 2D feature set — see `Cargo.toml`).
  Audio (`bevy_audio`/alsa) and gamepad (`bevy_gilrs`/udev) are disabled to avoid
  native system-lib build deps. Windowing is **Wayland only**, following
  `/home/freeman.xiong/Desktop/protect-carrot`'s native setup. On NixOS-like
  hosts, run inside `nix develop` / `direnv` so Wayland, libxkbcommon, and Vulkan
  runtime libraries are on `LD_LIBRARY_PATH`.
- Package: `love-rpg`, edition 2024.
- Dimension: 2D (`Camera2d`, sprites, `Text2d`, Bevy UI).

## App Entry

- `src/main.rs` → `love_rpg::run()`
- `src/lib.rs` → builds `App`, configures the 1280×720 window, registers `GamePlugin`.

## Plugins / Modules (`src/game/`)

### `mod.rs` — `GamePlugin`
- `init_state::<AppState>()`, inserts `PlayerStats` + `Rng`.
- `Startup` `boot`: loads the CJK font (`fonts/unifont.otf`) into `GameFont`, spawns
  the single persistent `Camera2d` (shared by both states + used as the UI camera).
- Registers `ExplorePlugin` and `BattlePlugin`.

### `state.rs` — `AppState`
- `Explore` (overworld) and `Battle` (turn-based combat). Dialogue is an overlay
  inside Explore (a `Dialogue` resource flag), **not** a state, so the map stays
  visible behind the text box.

### `core.rs` — shared types
- Constants `TILE`, `MAP_W`, `MAP_H`; `tile_to_world(col,row)` (row 0 = top, centered).
- `GameFont(Handle<Font>)` with `.text_font(size)` helper.
- `PlayerStats` (persistent resource): name/level/hp/mp/atk/def/potions/exp,
  `gain_exp` (auto level-up), `full_restore`. Constants `SPELL_COST`, `POTION_HEAL`.
- `Rng` (xorshift, no `rand` dependency): `unit`, `chance`, `range`.

### `explore.rs` — `ExplorePlugin`
- Static `MAP` (`[&str; MAP_H]`) parsed into `MapData` (Wall/Water/Path/Grass/Npc).
- `PlayerPos` (persistent: col/row/facing) — default scans `MAP` for `P`.
- `OnEnter(Explore)` `spawn_explore`: tiles, NPC sprites + name tags, player sprite,
  HUD (top-left stats), hint line, hidden dialogue box. All `DespawnOnExit(Explore)`.
- `Update` chain (run_if in Explore): `explore_input` (grid-step movement w/ cooldown,
  collision, facing, NPC talk on Space, 16% grass encounter → `Battle`),
  `sync_player_transform`, `update_hud`, `update_dialogue_ui`.

### `battle.rs` — `BattlePlugin`
- `ENEMIES` table (妖蛇/山魈/黑山鬼). `BattleState` resource holds the rolled
  `EnemyInstance`, a `Phase` state machine, timer, menu index, and message.
- `OnEnter(Battle)` `spawn_battle`: picks an enemy via `Rng`, builds the battle scene
  (background, enemy + HP bar, player sprite, enemy info, bottom command panel with
  message/player-info/menu). All `DespawnOnExit(Battle)`.
- `Update` chain (run_if in Battle):
  - `battle_input` (Menu phase only): Up/Down cursor; Enter/Space confirms
    攻击 / 仙术(御剑术, costs MP) / 物品(药水) / 逃跑(50%).
  - `battle_tick`: timed `Phase` machine — PlayerActing → enemy death check / enemy
    turn; EnemyActing → player death check / back to Menu; Won/Lost/Fled → back to
    Explore (Lost auto-restores HP/MP for the demo).
  - `update_battle_ui`: refreshes texts, menu cursor/highlight, enemy HP bar (shrinks
    from the left via custom_size + x offset).

## State / Resources summary

- `AppState` (Explore | Battle)
- Persistent: `PlayerStats`, `Rng`, `PlayerPos`, `GameFont`
- Explore-scoped: `MapData` (re-inserted each enter), `MoveCooldown`, `Dialogue`
- Battle-scoped: `BattleState`

## Assets

- Runtime root: `assets/`
- `assets/fonts/unifont.otf` — CJK-capable font (the embedded default font is
  Latin-only). Pixel look suits the theme. All Chinese text uses `GameFont`.

## Controls

- 方向键 / WASD: move · 空格(Space)/Enter: talk to NPC, confirm, advance dialogue
- In battle: Up/Down select command, Space/Enter confirm.

## Verification

- `cargo fmt` · `cargo check` · `cargo build`
- Desktop smoke test: `cargo run` inside `nix develop` / `direnv` (Wayland / `WAYLAND_DISPLAY` required)
- Capture: dedicated offscreen path (see `.claude/skills/godogen/capture.md`)

# 御剑行 · 轮回 (love-rpg)

A 仙侠-flavoured **roguelike turn-based RPG** built code-first in Bevy. One run
= one life: Title → random chapter node map → battles / events / story / rests
→ chapter boss → next chapter → final boss → one of two endings. No levels or
grinding — growth comes from 法宝 (relics), rewards, and per-chapter
breakthroughs. The pre-roguelike free-roam overworld (`Explore`) is retained
for legacy capture presets but is no longer the main loop.

## Runtime

- Bevy **0.19.0** (default features off; trimmed 2D feature set — see `Cargo.toml`).
  Audio (`bevy_audio`/alsa) and gamepad (`bevy_gilrs`/udev) are disabled to avoid
  native system-lib build deps. Windowing is **Wayland only**. On NixOS-like
  hosts, run inside `nix develop` / `direnv` so Wayland, libxkbcommon, and Vulkan
  runtime libraries are on `LD_LIBRARY_PATH`.
- Package: `love-rpg`, edition 2024.
- Dimension: 2D (`Camera2d`, sprites, `Text2d`, Bevy UI).

## App Entry

- `src/main.rs` → `love_rpg::run()`
- `src/lib.rs` → builds `App`, configures the 1280×720 window, registers `GamePlugin`.

## States (`src/game/state.rs`)

`AppState`: `Title` (default) | `NodeMap` | `Battle` | `Reward` | `Ending` |
`Explore` (legacy). Run-mode dialogue/events are an overlay resource inside
`NodeMap`, not a state. `Battle` is shared by both flows — the presence of the
`RunState` resource marks a roguelike battle.

## Roguelike core (`src/game/roguelike/`)

- **`mod.rs`** — `RoguelikePlugin` (system registration for all run screens);
  `RunState` (chapter, node graph, position, relics, 道心/情缘 counters,
  outcome, revive flag); `Relic` enum (22 法宝,加成按持有求和叠加) with battle-modifier query
  methods; `CHAPTERS: [ChapterDef; 4]` (title, encounter zones, boss pool,
  enemy hp/atk multipliers); `RunBattleMods` + `battle_mods_for` (normal /
  elite ×1.4/×1.15 / boss flat ×0.72/×0.68 discounts); breakthrough constants
  (per-chapter stat gains replacing levels).
- **`graph.rs`** — `NodeGraph::generate`: layered DAG per chapter (entry 2–3
  wide → depth random layers with one single-node Story bottleneck → rest →
  boss), non-crossing links, full reachability. `NodeKind`:
  Fight战/Elite袭/Event遇/Story缘/Rest歇/Market市/Boss魔.
- **`content.rs`** — all narrative data: 4 chapter cards, 11 story scenes(每章一个池,「缘」节点随机抽取), 22 random events (options
  with success chance + outcome effects; run 内不重复抽取), market prices,
  rest options, 4 endings (情缘 / 道心 / 双全隐藏 / defeat). `Effect` enum applied by `event::apply_effect`.
- **`event.rs`** — `RunDialogue` overlay resource (chapter cards, story,
  events, rests, markets all render through it; market purchases keep the
  stall open until离开); `run_dialogue_input` (line advance +
  option pick + probabilistic resolution); relic granting; overlay UI.
- **`map_ui.rs`** — node-map scene (edge sprites, node sprites + `Text2d`
  glyphs, pulsing cursor ring, run HUD with relics/道心/情缘), `node_map_input`
  (up/down picks a reachable node, confirm travels & dispatches: battles set
  `PendingEncounter`+`RunBattleMods`+`AppState::Battle`; other kinds open the
  overlay). Chapter card opens on first map entry per chapter.
- **`screens.rs`** — Title (starts a run: resets `PlayerStats` [+2 atk/+1 def/
  +1 potion baseline], reseeds `Rng` with wall clock, inserts `RunState`);
  Reward (three-choice loot post-battle, relics guaranteed on elite/boss;
  boss reward also applies breakthrough + `next_chapter`, or rolls the ending
  after the finale); Ending (text + run stats, confirm → remove `RunState` →
  Title).

## Shared plugins (`src/game/`)

### `mod.rs` — `GamePlugin`
Registers fonts/paperdoll/lighting/animation assets, `AppState`, persistent
resources (`PlayerStats`, `Rng`, `Intent`, `EncounterRate`, `QuestLog`), and
plugins: animation, paperdoll runtime, explore, battle, fog, roguelike.
Deliberately no camera/input — `DesktopInputPlugin` (keyboard → `Intent`,
spawns `Camera2d`) and the capture binary wire those separately.

### `battle.rs` — `BattlePlugin` (shared by run + legacy)
- `ENEMIES[9]` zone pools + 6 boss defs; `PendingEncounter{zone, kind}` is the
  entry API from both flows.
- Run-mode hooks (all gated on `Option<Res<RunState>>` /
  `Option<Res<RunBattleMods>>`): enemy stat scaling & elite naming at spawn;
  relic modifiers in `battle_input` (attack/spell bonuses, spell-cost delta,
  potion bonus, guaranteed flee, elite/boss ×1.25 hunter multiplier) and
  `begin_enemy_turn` (flat damage reduction); 雷泽鼓 opening strike; 1.6×
  message-timer tempo; victory grants gold only (no exp) + 嗜血珠 on-kill
  heal → `Reward`; defeat tries 檀木符 revive else → `Ending`; flee →
  `NodeMap` (blocked in boss fights). Legacy (no `RunState`) paths unchanged
  (exp/levels, back to `Explore`).
- `PlayerStats.gain_exp` (`core.rs`) is now used **only** by the legacy flow.

### `explore.rs` / `quest.rs` (legacy campaign, intact)
Free-roam grid maps, NPC dialogue, 41-stage linear quest chain. Reachable only
via legacy capture presets (which set `AppState::Explore` explicitly).

### `core.rs` — shared types
`TILE/MAP_W/MAP_H`, `tile_to_world`, `GameFont`, `Intent` (move/confirm/up/
down abstraction driven by keyboard or capture scripts), `PlayerStats`,
xorshift `Rng` (drives all run randomness).

### `animation.rs` / `paperdoll.rs` / `lighting.rs` / `fog.rs`
Atlas animation, paperdoll portraits, 2D lights, fog-of-war (fog is
Explore-only).

## State / Resources summary

- `AppState` (Title | NodeMap | Battle | Reward | Ending | Explore)
- Persistent: `PlayerStats`, `Rng`, `GameFont`, `Intent`, `QuestLog`
- Run-scoped: `RunState` (inserted at title confirm, removed at ending),
  `RunBattleMods` (per battle), `RunDialogue`, `MapCursor`, `RewardChoices`
- Battle-scoped: `BattleState`; Explore-scoped: `MapData`, `Dialogue`, …

## Controls

- 节点图: 上/下 选路 · 空格 前进/继续对话/确认选项
- 战斗: 上/下 选指令 · 空格/Enter 确认(攻击/仙术/合击/物品/逃跑)
- 标题/奖励/结局: 上/下 + 空格

## Verification

- `cargo fmt` · `cargo check` · `cargo build`
- Desktop: `cargo run --bin love-rpg` inside `nix develop` (needs
  `WAYLAND_DISPLAY`)
- Headless proof: `cargo run --bin capture -- <out> <frames> rogue` with
  lavapipe (`VK_ICD_FILENAMES=/run/opengl-driver/share/vulkan/icd.d/lvp_icd.x86_64.json`);
  the `rogue` preset walks title → card → nodes → battles → rewards on a
  cadence script. Legacy presets still work (capture sets `Explore` for them).
- Latest proof bundle: `screenshots/result/3/` (900 frames + video.mp4, 30s).

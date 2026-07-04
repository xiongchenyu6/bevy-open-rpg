# 御剑行 · 轮回 (love-rpg)

A 仙侠-flavoured **roguelike turn-based RPG** built code-first in Bevy. One run
= one life, played entirely on real walkable tile maps: Title → chapter map
chain (each stage scatters unknown「?」mist markers that reveal battles /
events / story / rest / market on contact; grass tiles roll random
encounters) → portal to the next map → chapter boss gate → next chapter →
final boss → one of four endings. No levels or
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

`AppState`: `Title` (default) | `NodeMap` (zero-frame hop that rolls the next
stage) | `RunScene` (the walkable map — the run's main state) | `Battle` |
`Reward` | `Ending` | `Explore` (legacy). Run-mode dialogue/events are an
overlay resource inside `RunScene`, not a state. `Battle` is shared by both
flows — the presence of the `RunState` resource marks a roguelike battle.

## Roguelike core (`src/game/roguelike/`)

- **`mod.rs`** — `RoguelikePlugin` (system registration for all run screens);
  `RunState` (chapter, node graph, position, relics, 道心/情缘 counters,
  outcome, revive flag); `Relic` enum (22 法宝,加成按持有求和叠加) with battle-modifier query
  methods; `CHAPTERS: [ChapterDef; 4]` (title, encounter zones, boss pool,
  enemy hp/atk multipliers); `RunBattleMods` + `battle_mods_for` (normal /
  elite ×1.4/×1.15 / boss flat ×0.72/×0.68 discounts); breakthrough constants
  (per-chapter stat gains replacing levels).
- **`graph.rs`** — `NodeKind` marker kinds:
  Fight战/Elite袭/Event遇/Story缘/Rest歇/Market市/Boss魔 (the abstract node
  graph was removed in favour of playing directly on maps).
- **`content.rs`** — all narrative data: 4 chapter cards, 11 story scenes(每章一个池,「缘」节点随机抽取), 22 random events (options
  with success chance + outcome effects; run 内不重复抽取), market prices,
  rest options, 4 endings (情缘 / 道心 / 双全隐藏 / defeat). `Effect` enum applied by `event::apply_effect`.
- **`event.rs`** — `RunDialogue` overlay resource (chapter cards, story,
  events, rests, markets all render through it; market purchases keep the
  stall open until离开); `run_dialogue_input` (line advance +
  option pick + probabilistic resolution); relic granting; overlay UI.
- **`scene.rs`** — node-map scene (edge sprites, node sprites + `Text2d`
  glyphs, pulsing cursor ring, run HUD with relics/道心/情缘), `node_map_input`
  (up/down picks a reachable node, confirm travels & dispatches: battles set
  `PendingEncounter`+`RunBattleMods`+`AppState::Battle`; other kinds open the
  overlay). Chapter card opens on first map entry per chapter.
- **`scene.rs`** — the run's core: `advance_stage` (on the `NodeMap` hop)
  rolls a chapter tileset (no immediate repeats) and **generates organic
  terrain procedurally** (cellular-automata tree walls smoothed into blobs,
  largest-region connectivity pass, blob lakes, scattered grass patches —
  no hand-authored rectangles; layout persists in `RunSceneState.tiles`),
  then scatters 2–3 markers
  (pairwise-spread walkable tiles, story beat guaranteed before the boss
  map; boss stage = single demon gate); `spawn_run_scene` renders tiles via
  explore's `tile_sprite`, hero, mist markers (unknown「?」until touched),
  界门 spirit-gate sprite over the portal (boss gates reuse it violet-tinted), fill lights, HUD, and rebuilds losslessly after battles
  (`RunSceneState` persists). `run_scene_movement`: grid movement, marker
  contact fires the payload (battle → `AppState::Battle`; others open the
  overlay in place), grass tiles roll 8% random encounters, portal advances
  the stage once all mandatory markers are cleared (`run.stage += 1` → hop).
  Optional visible loot never blocks the gate: chests (`NodeKind::Chest`,
  ai_chest prop; 20% mimic elite fight / relic / potion / gold) and springs
  (`NodeKind::Spring`, ai_spring prop; one-shot 35% heal). 3–5 瘴气 hazard
  tiles (`RunSceneState.hazards`, violet mist) deal 8% max-hp poison on step
  (never lethal) with rising float text (`SceneFloatText`). BFS
  multi-source `flow` steers the capture driver.
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
Deliberately no camera/input — `DesktopInputPlugin` (keyboard → `Intent`
incl. Esc→cancel; spawns `Camera2d` with `ScalingMode::Fixed` 1280×720 so the
game fills any window) and the capture binary wire those separately. While in
`RunScene` the camera switches to a zoomed `FixedVertical(560)` follow-view
clamped to the bordered map (`scene::zoom_camera_in/out`, `camera_follow`).

### `battle.rs` — `BattlePlugin` (shared by run + legacy)
- `ENEMIES[9]` zone pools + 6 boss defs; `PendingEncounter{zone, kind}` is the
  entry API from both flows.
- **Decision layer**: every enemy telegraphs its next move (`EnemyIntent`:
  Strike / Heavy 1.8× / Gather heal+def / Drain mp-steal, shown in the enemy
  info line; boss special turns telegraph as Heavy). The 6-item menu adds
  御守 (guard: −65% damage this turn, +4 mp). Attack/spell build 气势
  momentum (max 3, shown as ●●○); at full stacks the run-mode menu offers
  绝技·剑气爆发 (~2× atk, resets momentum) in the combo slot.
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

- 地图: 方向键/WASD 移动 · 走到「?」迷雾揭晓内容 · 清完标记走「门」过图
- ESC: 打开/关闭行囊(纸娃娃、属性、法宝、道心/情缘)
- 对话/选项: 上/下 选择 · 空格 确认
- 战斗: 上/下 选指令 · 空格/Enter 确认(攻击/仙术/合击/物品/逃跑)
- 标题/奖励/结局: 上/下 + 空格

## Verification

- `cargo fmt` · `cargo check` · `cargo build`
- Desktop: `cargo run --bin love-rpg` inside `nix develop` (needs
  `WAYLAND_DISPLAY`)
- Headless proof: `cargo run --bin capture -- <out> <frames> rogue` with
  lavapipe (`VK_ICD_FILENAMES=/run/opengl-driver/share/vulkan/icd.d/lvp_icd.x86_64.json`);
  the `rogue` preset walks title → card → maps on a cadence script and
  auto-paths through walkable scenes via `RunSceneState.flow`;
  `rogue-inventory` opens the Esc inventory for a still. Legacy presets still work (capture sets `Explore` for them).
- Chapter-start presets `rogue-ch2|rogue-ch3|rogue-ch4` begin a run directly
  at chapter 2/3/finale (used to proof per-chapter tilesets).
- Latest proof bundle: `screenshots/result/10/` (per-chapter map stills; full
  run video in `result/9/`).

## UI art (`assets/ui/`, generated via remote ComfyUI)

- `title_bg.png` — 标题主视觉(月下剑侣崖景); `map_bg.png` — 节点图水墨群山底;
  `reward_bg.png` — 战利祭坛; `ending_bg.png` — 月夜渡口(结局)。
- `assets/npcs/ai_linger.png` — 灵儿月轮立绘卡(剧情缘节点、歇脚谈心)。
- `assets/tiles/ai_*_wall|floor.png` — 每张章节地图的专属无缝地砖
  (18 张,见 `ASSETS.md` 对照表);`explore::tile_sprite` 按 `MapKind`
  选贴图,tint 近白。
- Run 对话框为「立绘卡 + 文本」双栏;奇遇立绘复用 NPC/怪物/道具切图,
  映射在 `content::event_portrait`。全屏界面文字均垫深色半透明底板。

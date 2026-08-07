# 御剑行 · 轮回 (love-rpg)

A 仙侠-flavoured **roguelike turn-based RPG** built code-first in Bevy. One run
= one life, played entirely on real walkable tile maps: Title → chapter map
chain (each stage scatters unknown「?」mist markers that reveal battles /
events / story / rest / market on contact; grass tiles roll random
encounters) → portal to the next map → chapter boss gate → next chapter →
final boss → one of four endings. No levels or
grinding — growth comes from 法宝 (relics), rewards, and boss-earned chapter
seals / breakthroughs. The pre-roguelike free-roam overworld (`Explore`) is retained
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
  `RunState` (seven-chapter, 41-step journey route, position, relics, 道心/情缘 counters,
  chapter-local `缘忆` no-repeat history, outcome, revive flag); `Relic` enum
  (22 法宝,加成按持有求和叠加) with battle-modifier query
  methods; `CHAPTERS: [ChapterDef; 7]` + `JOURNEY_ROUTE_STAGES = 41`
  (title, encounter zones, boss pool,
  enemy hp/atk multipliers) plus `JOURNEY_BEATS` (named route beat, place,
  objective, gate prompt, generated `主线签` receipt, and explicit
  `待签收 -> 已追踪 -> 已归档` task state for every map stage)
  plus `CHAPTER_PACING` (7 chapter minute budgets totaling 600 minutes, with
  stage estimates derived from `JOURNEY_MARKER_PLANS`)
  and `JOURNEY_MARKER_PLANS`
  (the required Fight/Elite/Event/Story/Rest/Market/Boss pacing for each
  route beat), plus `JOURNEY_INTROS` (one-shot authored scene and party lines
  shown when each generated stage opens); `RunCampTactic` + `RunState`
  camp-scene tracking (`营火照应` plus one next-battle tactic: 调息/剑守/灵护/凝灵);
  `RunBattleMods` + `battle_mods_for` (normal / elite ×1.4/×1.15 / boss flat
  ×0.72/×0.68 discounts); `CHAPTER_CLEAR_REWARDS` (Boss-earned 章印, chapter
  epilogue line, and per-chapter stat gains replacing levels).
- **`graph.rs`** — `NodeKind` marker kinds:
  Fight战/Elite袭/Event遇/Story缘/Rest歇/Market市/Puzzle机/Boss魔 plus
  optional Chest宝/Spring泉/Guide人 (the abstract node graph was removed in
  favour of playing directly on maps).
- **`content.rs`** — all narrative data: 7 chapter cards and 22 selectable
  story scenes (7 primary plus 15 chapter-pool extras; required「缘」nodes draw
  unseen scenes first, including dedicated southern old-drum and six-scene
  finale pools), 22 random events (options
  with success chance + outcome effects; run 内不重复抽取) plus 7 **chain
  events** past `CHAIN_START` (白狐三遇 branching on the first-meeting
  choice, 盲女琴师二遇 — driven by `RunState::draw_chain_or_event`, never in
  the random pool), seven `boss_taunt` confrontation scripts (played before
  the boss gate battle via `RunDialogue.boss_battle_after`), market prices,
  rest options, 4 endings (情缘 / 道心 / 双全隐藏 / defeat) with
  chain-echo recap lines and run `本卷誓记` summary on the ending screen.
  `Effect` enum applied by `event::apply_effect`.
- **`event.rs`** — `RunDialogue` overlay resource (chapter cards, story,
  events, rests, markets all render through it; rest nodes now build
  chapter/party-specific camp dialogue and prepare a run `营策`; market purchases
  keep the stall open until离开); `run_dialogue_input` (line advance +
  option pick + probabilistic resolution); run 「缘」 choices record the
  chapter's `本卷誓记`; story portraits follow the scene speaker (灵儿/林月衡/
  南瑶); relic granting; overlay UI.
- **`scene.rs`** — node-map scene (edge sprites, node sprites + `Text2d`
  glyphs, pulsing cursor ring, run HUD with relics/道心/情缘), `node_map_input`
  (up/down picks a reachable node, confirm travels & dispatches: battles set
  `PendingEncounter`+`RunBattleMods`+`AppState::Battle`; other kinds open the
  overlay). Chapter card opens on first map entry per chapter.
- **`scene.rs`** — the run's core: `advance_stage` (on the `NodeMap` hop)
  reads the current route beat's `JOURNEY_MAPS` map kind and uses that place's
  authored 30×16 map as a topology blueprint. Per-stage transforms, wall-edge
  softening, and biome-specific water/grass walks vary repeat visits; an exact
  largest-component pass then requires at least 150 connected walkable cells
  and a 24-step exploration span (layout persists in `RunSceneState.tiles`),
  then scatters the beat's required markers
  (pairwise-spread walkable tiles, plus a small chance of one extra surprise
  marker; boss stage = single demon gate); `spawn_run_scene` renders the whole
  logical map through explore's `TerrainMaterial` — a 30×16 terrain-id texture
  drives world-space sampling of floor/grass/wall/water sources, while the WGSL
  shader feathers and noise-warps neighboring terrain boundaries. Gameplay
  remains grid-based without rendering one opaque sprite per cell —
  hero, mist markers (unknown「?」until touched), a top journey banner with
  the current authored route beat/place/objective,
  one-shot `open_journey_intro` route dialogue after chapter cards close,
  run-stage `主线签` pickup/progress text in both HUD and inventory, including
  the required marker mix, approximate stage minutes, and chapter runtime focus,
  `sync_run_party_followers` dedicated generated companion cutouts that appear by route progress,
  界门 spirit-gate sprite over the portal (boss gates reuse it violet-tinted), fill lights, HUD, and rebuilds losslessly after battles
  (`RunSceneState` persists). `run_scene_movement`: grid movement, marker
  contact fires the payload (battle → `AppState::Battle`; others open the
  overlay in place), grass tiles roll 8% random encounters, portal advances
  the stage once all mandatory markers are cleared (`run.stage += 1` → hop).
  Route puzzle maps add required visible mechanisms (`NodeKind::Puzzle`) for
  moon crystals, river lanterns, plague bells, mansion mirrors, thunder drums,
  and dream lamps; solving them increments `机关破除` and gives a small route
  reward.
  Optional visible map life never blocks the gate: route contacts
  (`NodeKind::Guide`, local AI NPC sprite, portrait dialogue, `路人签`
  pickup/tracking plus `路人回声` receipt reward and chapter-local `路况`
  encounter pressure reduction plus `首领照应` boss-opening preparation), chests
  (`NodeKind::Chest`, ai_chest prop; 20%
  mimic elite fight / relic / potion / gold), and springs
  (`NodeKind::Spring`, ai_spring prop; one-shot 35% heal). 3–5 瘴气 hazard
  tiles (`RunSceneState.hazards`, violet mist) deal 8% max-hp poison on step
  (never lethal) with rising float text (`SceneFloatText`). BFS
  multi-source `flow` steers the capture driver.
- **`screens.rs`** — Title (starts a run: resets `PlayerStats` [+2 atk/+1 def/
  +1 potion baseline], reseeds `Rng` with wall clock, inserts `RunState`);
  Reward (three-choice loot post-battle, relics guaranteed on elite/boss;
  boss reward atomically archives the boss-stage main-task receipt and records
  a chapter seal, applies that chapter's breakthrough + `next_chapter`, or rolls
  the ending after the finale); Ending (text + chapter-seal summary, run stats,
  and total/per-chapter play time, confirm → remove `RunState` → Title).

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
- `ENEMIES[9]` zone pools + 7 boss defs, each with a distinct generated
  `creatures/boss_*.png` cutout; `PendingEncounter{zone, kind}` is the
  entry API from both flows.
- **Decision layer**: every enemy telegraphs its next move (`EnemyIntent`:
  Strike / Heavy 1.8× / Gather heal+def / Drain mp-steal, shown in the enemy
  info line; boss special turns telegraph as Heavy). The 6-item menu adds
  御守 (guard: −65% damage this turn, +4 mp). Attack/spell build 气势
  momentum (max 3, shown as ●●○); at full stacks the run-mode menu offers
  绝技·剑气爆发 (~2× atk, resets momentum) in the combo slot.
- **Boss phase 2**: at half HP every boss takes a transform turn
  (`boss_phase2_transform`, enemy line tagged ·真身) and gains a signature
  mechanic — MoonWraith drains mp per hit, RiverDemon rages (+atk/−def),
  MiasmaRoot regenerates each turn, MirrorMinister reflects 20% of player
  strikes (`mirror_backlash`), ThunderQilin chains Heavy intents,
  DreamEclipse casts specials every 2 turns (`boss_special_cadence`).
- **Spell impact**: `spawn_spell_impact` keeps player spells as enemy-side hit
  effects; `spell_damage_texts` splits 御剑术 into three damage floaters and
  万剑诀 into seven chapter-tinted hit numbers across the enemy body.
- Legacy boss openings (when there is no `RunState`) call
  `apply_legacy_boss_preparation_to_boss`: chapter bond+camp照应, both local
  commissions, and chapter companion scenes are scored into
  `首领照应·周全/半备/欠备`, adjusting initial boss HP/ATK and player HP/MP with
  a battle log line.
- Run-mode hooks (all gated on `Option<Res<RunState>>` /
  `Option<Res<RunBattleMods>>`): enemy stat scaling & elite naming at spawn;
  relic modifiers in `battle_input` (attack/spell bonuses, spell-cost delta,
  potion bonus, guaranteed flee, elite/boss ×1.25 hunter multiplier), consumed
  run camp tactics (opening HP/MP recovery, damage bonuses, mitigation, and HUD
  summary), run-party companions for battle UI/support/追击/回灵 and late spell naming/VFX, and
  `begin_enemy_turn` (flat damage reduction); run-mode boss spawns read the
  current chapter's `本卷誓记` (`护心誓` lowers attack, `破势誓` lowers max HP,
  `同心誓` lightly lowers both); 雷泽鼓 opening strike; 1.6×
  message-timer tempo; victory grants gold only (no exp) + 嗜血珠 on-kill
  heal → `Reward`; defeat tries 檀木符 revive else → `Ending`; flee →
  `NodeMap` (blocked in boss fights). Legacy (no `RunState`) paths unchanged
  (exp/levels, chapter boss breakthrough growth, back to `Explore`).
- `PlayerStats.gain_exp` (`core.rs`) is now used **only** by the legacy flow.

### `explore.rs` / `quest.rs` (legacy campaign, intact)
Free-roam grid maps, NPC dialogue, and a 41-stage linear quest chain. `QuestLog`
owns main-task state plus `MainTaskLedger` route-book metadata, so every
authored main handoff can show chapter step, receipt id, location, contact, and
current action in the NPC confirmation card, HUD task book, and persistent
tracker. The Explore task tracker appends `任务引路`, using the current `MapKind`
to mark main tasks, local commissions, and active NPC errands as current-map
targets, delivery points, or named next destinations. Successful portal moves
now open a short `界门` transition dialogue with origin/destination, route-book
receipt, landing-target status, and objective before movement resumes. It also archives exploration chapter seals (`章印 x/7`) from village
handoff and boss clears, feeds the seal summary into HUD/tracker, and reflects
it in the finale epilogue. Local commissions derive authored step plans from
`SideQuest::field_steps()` without adding save fields: task-board detail,
`委托契约`, HUD tracking, and `委托推进` messages all show the full signed flow
and current next step while still using `side_progress` as the source of truth.
Pickup also emits a shared `签收回执` block so board previews, confirmed
acceptance, task notices, and HUD intake text all point at the same first step,
field choice, route hint, and turn-in target. Field traces store their one-time
handling in `commission_traces`, plus a `side_field_confront` bit for the
`细查现场` vs `快断余妖` branch; the branch is reflected in contracts, turn-in
receipts, route-pressure tails, and local NPC `现场回声`.
Local commissions also store a delivery resolution bitset: default `稳妥封存`
preserves the existing completion flow, while player-selected `追查余波` is
shown in task/NPC text and gives the matching route a stronger `委托清障`
pressure reduction. Route detours also track one-time local reports
(`route_detour_reports_claimed`); after a detour is resolved, non-critical NPC
dialogue can file the report, pay a small `报路回礼`, and update HUD `路报 x/6`.
Companion personal scenes keep a separate `companion_aftermath_claimed` bitset;
non-critical NPCs on the matching route can play an authored `小传回访`, pay
each completed vignette's one-time `小传回礼`, and settle both finale scenes in
one conversation while missed scenes remain unrewarded route consequences.
The first three missable scenes also use `companion_revisits_active` to drive
cross-chapter `同伴补访` contracts. `explore.rs` maps each contract to a later NPC
and route mark without role conflicts, exposes pickup/field/turn-in prompts and markers,
restores the original `CompanionScene` at the field beat, blocks generic local
aftermath claims until formal turn-in, and adds recovered-route relief plus
`补访回声` reactions.
`NpcErrand` adds eight cross-map non-board NPC tasks with separate active and
completed bitsets, confirmation choices, HUD/tracker summaries, delivery
markers, one-time `托付回礼` rewards, and completed-errand `托付回声` route
relief for matching hub/route maps.
Reachable only via legacy capture presets (which set `AppState::Explore`
explicitly).

### `core.rs` — shared types
`TILE/MAP_W/MAP_H`, `tile_to_world`, `GameFont`, `Intent` (move/confirm/up/
down abstraction driven by keyboard or capture scripts), `PlayerStats`,
xorshift `Rng` (drives all run randomness).

### `animation.rs` / `paperdoll.rs` / `lighting.rs` / `fog.rs`
Atlas animation, paperdoll portraits, 2D lights, fog-of-war. `fog.rs` owns the
legacy Explore overlay; roguelike run maps keep their own per-stage revealed
grid in `RunSceneState` and draw `RunFogTile` overlays from `scene.rs`.

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
- Terrain regression: `scripts/terrain_harness.sh <out>` runs UV/category unit
  tests, captures a centered mixed-terrain RunScene plus legacy Explore, and
  writes `terrain-contact-sheet.png` when ImageMagick is available.
- Map regression: `scripts/map_harness.sh <out>` checks all 15 biome topologies,
  all 41 stages' required-marker/mechanism/contact placement capacity, and
  repeated finale variants, then captures `route-map-01..15` from the real
  RunScene renderer into `map-contact-sheet.png`.
- Story regression: `scripts/story_harness.sh <out>` verifies authored scene
  capacity against every required「缘」marker, complete two-choice outcomes,
  speaker portraits, and chapter-local no-repeat draws, then captures the
  southern old-drum and finale bell-oath choices into `story-contact-sheet.png`.
- Full-run regression: `scripts/run_audit.sh <out>` drives normal movement,
  dialogue, battle-menu, item/guard, and reward inputs at a fixed time step. It
  must reach a victory after visiting all 41 stages, archiving all 41 main-task
  receipts, and earning all seven chapter seals. The report includes real
  total/per-chapter runtime ledgers but explicitly does not count as a human
  ten-hour playtest. `VERIFY_DETERMINISM=1 scripts/run_audit.sh <out>` repeats
  the fixed-seed run and requires byte-identical reports after excluding host
  wall time.
- Chapter-start presets `rogue-ch2|rogue-ch3|rogue-ch4` begin a run directly
  at chapter 2/3/finale (used to proof per-chapter tilesets).
- Latest proof bundle: `screenshots/result/10/` (per-chapter map stills; full
  run video in `result/9/`).

## UI art (`assets/ui/`, generated via remote ComfyUI + imagegen)

- `title_bg.png` — 标题主视觉(月下剑侣崖景); `map_bg.png` — 节点图水墨群山底;
  `reward_bg.png` — 战利祭坛; `ending_bg.png` — 月夜渡口(结局)。
- `assets/npcs/ai_linger.png` — 灵儿月轮立绘卡(剧情缘节点、歇脚谈心)。
- `assets/tiles/ai_*_wall|floor.png` — 每张章节地图的专属地表源图
  (18 张,见 `ASSETS.md` 对照表);`explore::TerrainMaterial` 按 `MapKind`
  选贴图并在地图级 shader 中连续采样、混合边界。
- `assets/ui/chapter1..7_art.png` / `assets/ui/anim/chapter1..7_sheet.png` —
  七章卷章静帧与 32 帧 atlas;开放 RPG 主线卷章卡播放对应全屏动画,后 5-7
  章由 imagegen 静帧再生成 atlas 补齐。
- Run 对话框为「立绘卡 + 文本」双栏;奇遇立绘复用 NPC/怪物/道具切图,
  映射在 `content::event_portrait`。全屏界面与对话/菜单面板统一使用
  `ui/panel_frame.png` 金纹九宫格框(`event::panel_slicer()`)。

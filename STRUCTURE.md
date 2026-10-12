# 御剑行 · 轮回 (love-rpg)

A 仙侠-flavoured **roguelike turn-based RPG** built code-first in Bevy. One run
= one life, played entirely on real walkable tile maps: Title → chapter map
chain (each stage shows category icons for required objectives and optional
contacts; Space/Enter deliberately interacts nearby; grass tiles roll random
encounters after the opening map) → portal to the next map → chapter boss gate → next chapter →
final boss → one of four endings. No levels or
grinding — growth comes from 法宝 (relics), rewards, and boss-earned chapter
seals / breakthroughs. The pre-roguelike free-roam overworld (`Explore`) is retained
for legacy capture presets but is no longer the main loop.

## Runtime

- Bevy **0.19.1** (default features off; trimmed 2D feature set — see `Cargo.toml`).
  Audio (`bevy_audio`/alsa) and gamepad (`bevy_gilrs`/udev) are disabled to avoid
  native system-lib build deps. Windowing is **Wayland only**. On NixOS-like
  hosts, run inside `nix develop` / `direnv` so Wayland, libxkbcommon, and Vulkan
  runtime libraries are on `LD_LIBRARY_PATH`.
- Package: `love-rpg`, edition 2024.
- `vendor/paperdoll-tar` patches version0.1.1 to decode `.ppd` manifests and
  images directly from the archive in memory. The upstream reader uses a
  temporary directory, which panics in browsers. Keep this patch alongside
  `vendor/bevy_firefly`; it preserves the existing paperdoll asset format.
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
  chapter-local `缘忆` chronological history, outcome, revive flag); `Relic` enum
  (22 法宝,加成按持有求和叠加) with battle-modifier query
  methods; `CHAPTERS: [ChapterDef; 7]` + `JOURNEY_ROUTE_STAGES = 41`
  (title, encounter zones, boss pool,
  enemy hp/atk multipliers) plus `JOURNEY_BEATS` (named route beat, place,
  objective, gate prompt, internal journey ledger automatically tracked on entry and archived on exit)
  plus `CHAPTER_PACING` (7 chapter minute budgets totaling 600 minutes, with
  stage estimates derived from `JOURNEY_MARKER_PLANS`)
  and `JOURNEY_MARKER_PLANS`
  (the required Fight/Elite/Event/Story/Rest/Market/Boss pacing for each
  route beat), plus `JOURNEY_INTROS` (one-shot authored scene and party lines
  shown when each generated stage opens); `RunCampTactic` + `RunState`
  camp-scene tracking (`营火照应` plus one next-battle tactic: 调息/剑守/灵护/凝灵);
  `RunBattleMods` + `battle_mods_for` (normal / elite ×1.4/×1.15 / boss chapter-growing
  HP/attack multipliers (0.72/0.68 plus 0.13 per cleared chapter)); `CHAPTER_CLEAR_REWARDS` (Boss-earned 章印, chapter
  epilogue line, and per-chapter stat gains replacing levels).
- **`graph.rs`** — `NodeKind` marker kinds:
  Fight战/Elite袭/Event遇/Story缘/Rest歇/Market市/Puzzle机/Boss魔 plus
  optional Chest宝/Spring泉/Guide人 (the abstract node graph was removed in
  favour of playing directly on maps).
- **`content.rs`** — 7 chapter cards, 22 named story scenes (7 primary plus
  15 chapter extras in authored order), 22 map-eligible ambient events and
  7 chain-event variants (阿狸's three encounters and 苏弦's two encounters),
  option costs/risks/rewards, seven boss confrontations, market prices,
  camp setup prose and four endings. The fox relationship can be repaired;
  the musician's first choice changes her reunion gift.
- **`narrative.rs`** — atomic `Cost`, reward previews and actual application,
  named skill/relic duplicate compensation, resolved choice journal,
  geography eligibility, chain completion and honest ending echoes.
- **`dialogue_ui.rs`** — shared real choice/continue buttons, selected-option
  previews and keyboard controls for every run dialogue.
- **`event.rs`** — `RunDialogue` overlay for chapter cards, stories, events,
  rests and markets. It checks costs before RNG, records resolved choices
  once, settles pending local commissions, and displays selected-choice
  resources/costs/rewards/risks. Camps prepare a next-battle tactic; market
  purchases keep the stall open until goodbye. Story portraits follow
  the speaker and a chapter's first story choice records its vow.
- **`scene.rs`** — the run's core: `advance_stage` (on the `NodeMap` hop)
  reads the current route beat's `JOURNEY_MAPS` map kind and uses that place's
  authored 30×16 map as a topology blueprint for natural areas. Urban/ritual
  places use courtyards, canal crossings and partitioned approaches owned by
  `scene_layout.rs`; per-stage doors/bridges vary return visits. `scene_places.rs`
  owns matching generated architecture and reserved collision footprints. An exact
  largest-component pass then requires at least 150 connected walkable cells
  and a 24-step exploration span (layout persists in `RunSceneState.tiles`),
  then scatters the beat's required markers
  (pairwise-spread walkable tiles, plus a small chance of one extra surprise
  marker; boss stage = single demon gate); `spawn_run_scene` renders the whole
  logical map through explore's `TerrainMaterial` — a 30×16 terrain-id texture
  drives world-space sampling of floor/grass/wall/water sources, while the WGSL
  shader selects one actual terrain surface per pixel and draws narrow contact
  shadows/shore rims, rather than mixing several painted scenes. Water uses a
  dedicated ripple texture without islands or decorative land. Gameplay
  remains grid-based without rendering one opaque sprite per cell —
  hero, gold category icons for required objectives, a top journey banner with
  the current authored route beat/place/objective,
  one-shot `open_journey_intro` route dialogue after chapter cards close,
  automatic route progress and nearby interaction hints in the HUD,
  `sync_run_party_followers` dedicated generated companion cutouts that appear by route progress,
  界门 spirit-gate sprite over the portal (boss gates reuse it violet-tinted), fill lights, HUD, and rebuilds losslessly after battles
  (`RunSceneState` persists). `run_scene_movement`: grid movement, marker
  Space/Enter beside a marker fires its payload (battle → `AppState::Battle`;
  others open the overlay in place); walking past is harmless. Grass tiles roll
  8% random encounters outside the opening map. Portal advances
  the stage once all mandatory markers are cleared (`run.stage += 1` → hop).
  Route puzzle maps add required visible mechanisms (`NodeKind::Puzzle`) for
  moon crystals, river lanterns, plague bells, mansion mirrors, thunder drums,
  and dream lamps. `puzzle.rs` owns six authored clue-based choices, rotated
  answers, harmless retry and retreat; successful resolution settles once in
  scene movement, increments `机关破除`, and pays the actual route reward.
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
- **`skill.rs`** — the complete run-mode spell catalog: exactly 20 curated
  `SkillId` entries with hand-authored names, grades, costs, power, mechanics,
  optional poison/burn/freeze/stun application, and one unique `VfxStyle` each.
  `ALL_SKILLS` is the sole reward/menu/inventory source; the retired 10x10
  generated catalog has no compatibility path.
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
plugins: animation, paperdoll runtime, explore, skill VFX, battle, fog,
roguelike.
Deliberately no camera/input — `DesktopInputPlugin` (keyboard → `Intent`
incl. Esc→cancel; spawns `Camera2d` with `ScalingMode::Fixed` 1280×720 so the
game fills any window) and the capture binary wire those separately. While in
`RunScene` the camera switches to a zoomed `FixedVertical(560)` follow-view
clamped to the bordered map (`scene::zoom_camera_in/out`, `camera_follow`).

### `battle.rs` — `BattlePlugin` (shared by run + legacy)
- `battle/enemy_animation.rs` owns identity-specific enemy atlas construction
  and phase-clock frame selection. Water-serpent motion has three 8-frame rows
  (idle, bite, hit), 384px cells. Bite contact follows the existing lunge peak;
  charge/gather/brace and non-damaging support actions remain idle. Numerical
  damage resolution stays in `battle.rs`.
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
- **Curated skill resolution**: the run-mode `仙术·秘卷` lists up to five
  learned skills per page from the 20-entry catalog. Poison ticks before the
  enemy action and can prevent it; freeze/stun then skip the action; burn ticks
  after it. Poison stacks to three, burn grows/extends to five turns, and a
  boss gains one turn of freeze immunity after breaking free.
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

### `skill_vfx.rs` — `SkillVfxPlugin`

Owns all run-mode skill and status rendering. Each `VfxStyle` dispatches to a
dedicated three-or-more-beat spawn script built from procedural 2D meshes,
soft sprites, layered line cores/halos, trails, overshoot, pulsing, flicker and
colored Firefly lights. `StatusVfxState` mirrors battle status into persistent
enemy auras; poison/burn/freeze/stun/resist also have separate trigger bursts.

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
- Battle-scoped: `BattleState`, `StatusVfxState`, `BattleAutomationView`;
  Explore-scoped: `MapData`, `Dialogue`, …

## Controls

- 地图: 方向键/WASD 移动 · 走到「?」迷雾揭晓内容 · 清完标记走「门」过图
- ESC: 打开/关闭行囊(主角、属性、法宝、道心/情缘)
- 对话/选项: 上/下 选择 · 空格 确认
- 战斗: 上/下 选指令 · 空格/Enter 确认(攻击/仙术/合击/物品/逃跑)
- 标题/奖励/结局: 上/下 + 空格

## Verification

- `cargo fmt` · `cargo check` · `cargo test --lib` · `cargo build`
- Desktop: `cargo run --bin love-rpg` inside `nix develop` (needs
  `WAYLAND_DISPLAY`)
- Headless proof: `cargo run --bin capture -- <out> <frames> rogue` with
  lavapipe (`VK_ICD_FILENAMES=/run/opengl-driver/share/vulkan/icd.d/lvp_icd.x86_64.json`);
  the `rogue` preset walks title → card → maps on a cadence script and
  auto-paths through walkable scenes via `RunSceneState.flow`;
  `rogue-inventory` opens the Esc inventory for a still. `rogue-skill` plays all
  20 curated effects in catalog order; `rogue-status` uses the real battle menu
  and turn loop to prove poison, burn, freeze and stun. Legacy presets still
  work (capture sets `Explore` for them).
- Terrain regression: `scripts/terrain_harness.sh <out>` runs UV/category unit
  tests, captures a centered mixed-terrain RunScene plus legacy Explore, and
  writes `terrain-contact-sheet.png` when ImageMagick is available.
- Map regression: `scripts/map_harness.sh <out>` checks all 15 biome topologies,
  all 41 stages' required-marker/mechanism/contact placement capacity, and
  repeated finale variants, then captures `route-map-01..15` from the real
  RunScene renderer into `map-contact-sheet.png`.
- Story regression: `scripts/story_harness.sh <out>` verifies authored scene
  capacity against every required「缘」marker, complete two-choice outcomes,
  speaker portraits, and chapter-local chronological draws, then captures the
  opening, southern old-drum and finale bell-oath choices into `story-contact-sheet.png`.
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
- Latest UI proof bundle: `screenshots/result/22/` (15-second/450-frame video,
  battle/map/dialogue/reward/inventory/ending captures, 20-skill and status
  stills, spell-page/Return proof, 960×540 UI bounds audit, and check/test/build logs).

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
  映射在 `content::event_portrait`。全屏界面与对话/菜单面板统一使用 `game::ui::panel` 的深墨底、细金边与柔阴影。


## Presentation polish (2026-10-06)

- `game/ui.rs` owns the ink/gold palette, panel chrome, clipped vital bars,
  soft contact shadows and automatic `UiScale` for the camera viewport.
- `game/battle/ui.rs` owns enemy health/intent, compact party vitals, contextual
  action descriptions and six keyboard/mouse command cards. `SkillPage` retains
  the current five-spell page when Return is selected; pointer page controls
  and normal keyboard navigation use the same combat rules. Combat log display
  keeps the latest nonempty consequence; the model retains the full message.
- `game/ui_scroll.rs` scrolls the active inventory/ending with wheel, PageUp,
  PageDown, Home and End. Every item/epilogue stays accessible inside a bounded
  viewport; content is not removed to make the panel fit.
- RunScene HUD has one objective tracker, separate location card and compact
  hero vitals; complete route receipts and build details stay in the inventory.
- Battle scenery is eight coherent painted perspective environments under
  `assets/backgrounds/battle-*.png`, selected by `EncounterZone`. A single
  generated enemy body avoids overlapping animated body slices. Hero/party
  feet have soft shadows; player attacks have anticipation/recovery and recoil.
- `animation.rs` uses `actors/hero-motion.png`: 4×6 cells, 256×256 each,
  eight idle frames (0–7), eight walk frames (8–15), eight sword attack frames
  (16–23). Attacks play once. Noto Sans CJK SC Regular is shared by all text;
  its SIL OFL is shipped at `assets/fonts/OFL.txt`.
- `skill_vfx.rs` keeps the 20 authored timings and replaces hard annulus/triangle
  fills with softly shaded engraved-ring/faceted-shard textures.
- Proof options: `CAPTURE_SIZE=960x540` exercises viewport scaling;
  `CAPTURE_UI_AUDIT=1` asserts visible command/reward buttons stay on screen
  and writes `ui-audit.txt`. `rogue-polish-inventory` supplies all 22 relics,
  14 hexes and 20 spells; `rogue-polish-skills` browses all four five-spell pages and Return through normal inputs. `rogue-polish-ending` supplies all seals, vows and
  chain echoes. Both presets feed real wheel messages to demonstrate scrolling.

Full UI regression: `nix develop --command bash scripts/ui_polish_harness.sh <out>` runs formatting, compile/test/build, real party combat, map, dialogue, full inventory/ending scrolls, status/20-skill VFX and a 960×540 capture. It creates a 15-second 30-fps clip plus raw frames and UI bounds reports.

UI regression result (2026-10-06): 193 library tests passed. Native Wayland launch ran for 12 seconds without a crash; capture reports contain no asset-load errors or panics. Upstream Parley/ICU emits a missing CJK word-segmentation diagnostic; Chinese line breaking is separate and was checked visually. The dependency/engine feature set remains unchanged.


## Tactical combat (2026-10-07)

- `battle/tactics.rs` owns four enemy rhythms, pressure limits (6 normally,
  7 armored, 8 boss), one-action breaks, exposed followups and guard ripostes.
  Damage modifiers are applied before HP and float text; support never pressures.
- `battle.rs` owns honest Charge→Heavy/Special intent execution, actual enemy
  attack flags, per-action duration, contact-distance sword movement and shared
  `scaled_enemy` construction. Heavy/signature guards grant one counter and
  momentum. Break/controls cancel armed charges; no fake flinch on no-attack beats.
- `battle/ui.rs` presents pressure, opportunities and actionable descriptions.
- `battle/sim_tests.rs` runs production `battle_input`/`battle_tick` in a CPU-only
  ECS app with manual time, actual skill/menu navigation and only visual assets
  substituted. It preserves internal ECS entities when collecting visual spawns.
- `battle_mods_for` grows boss HP/ATK with chapters; first-chapter multipliers
  remain 0.72/0.68, increasing by 0.13 each subsequent chapter.
- `rogue-tactics` capture logs real outcomes to `tactics-trace.csv` and requires
  at least one break and one guarded counter. Full-run audit navigates affordable
  learned skills rather than blindly confirming slot zero.
- `scripts/combat_harness.sh <out>` runs standard checks, 1,536-fight policy
  sweep, 15-second footage/scaled UI and all41-stage audit. Latest proof is
  `screenshots/result/24/`; 204 tests passed, zero sweep timeouts, full-run PASS.

### Narrative continuity

Main-run `缘` scenes are authored in chronological order: chapter primary scene,
then `STORY_EXTRA`. `draw_story` does not consume RNG and never resets history.
Chapter cards explain the clue leading from the previous boss to the next region;
companions join at the existing party boundaries (Linger after the first stage,
Lin Yueheng in chapter 4, Nanyao in chapter 6). Random encounters remain random.
`cargo test --lib story_content_` checks ordering across 64 seeds, retained
history, authored choices, and finale setup before the last bell.

## Narrative contracts and side stories (2026-10-07)

- `roguelike/narrative.rs` owns `Cost`, reward previews/application, named relic
  and skill duplicate compensation, `NarrativeRecord`, journal/ending echoes,
  and map eligibility for ambient events. `content.rs` authors option costs,
  rewards and named scenes. `event.rs` validates costs before RNG, resolves
  choices, records one-time receipts and only then advances chain progress.
- `RunState.narrative_log` is the resolved-choice/reward ledger; drawn-scene
  memory is not evidence that a choice has completed. `fox_kind == 2` denotes
  a repaired relationship; `qin_kind` remembers patron or quiet-listener choice.
- Guide markers remain available after asking directions. Their visited flag
  removes them from navigation targets while `is_visitable()` preserves
  interaction and visuals after battle returns. `RunDialogue` holds
  a pending local commission; Event/Story/Rest resolve it after a real choice,
  Market after goodbye, Puzzle after its completed mechanism.
- The selected choice displays its cost, reward and risk before confirmation.
  Actual receipts and the inventory’s 缘事簿 expose the same resolved history.
- `scripts/narrative_harness.sh` runs the choice audit, real battle sweeps,
  keyboard-driven captures (`rogue-side-fox`, `rogue-side-funds`,
  `rogue-side-qin`, `rogue-story-reward`), scaled capture and full journey audit.
  `narrative_tests.rs` exercises production resolution, insufficient resources,
  branch repair, chronology, duplicate rewards, commissions and geography.

Fallback spell input and UI/automation pricing share `battle::fallback_spell_cost`
(base cost + relic modifier + hex surcharge). `rogue-cost-proof` demonstrates
MoonBite pricing and a rejected low-mana input using the real battle screen.
Full-run audit reports terminal dialogue/menu/marker state and writes an actual
`final-frame.png`, so a stalled route has a reproducible visible diagnosis.

Learned-spell `battle::skill_cost` is shared by input, UI, inventory and capture.
`skill_hit_damage` applies relic/camp bonuses once to a completed offensive
cast (including multi-hit totals), then the existing hunter and tactical
modifiers. Wards/heals do not get damage-only relic/camp bonuses. Real-input
regressions cover spell relic +9, pendant mana charges, multi-hit camp totals,
and normal/elite/boss hunter behavior.

## Player experience verification

The opening story is within three walkable steps of spawn and visible through
fog. Main objectives track automatically; portals archive them without a hand-in
modal. The ordinary desktop session is driven through Wayland keyboard/pointer
input by `tools/native_play_input.py`; it neither reads nor changes game state.
See `docs/playtests/player-review-2026-10-07.md` for observations and limitations.

`battle/playtest.rs` exercises 96 continuous imperfect-strategy journeys with
persistent resources and real battle/reward input. It excludes overworld rewards,
and reports deaths/timeouts rather than calling its results human experience.
Learned skills gain +2 power per chapter from chapter four, capped at +12 (+2/+4/+8/+12 from chapters four through seven). Boss
healing consumes a shared budget of 25% maximum HP, including passive/special
healing, preventing the observed Miasma Root stalemate. `rogue-audit` measures
accelerated battle/dialogue/exploration frames separately; none is human playtime.


## Location and novice difficulty verification

`scene_places.rs` reserves the ground occupied by each landmark and renders the
same placement after every battle rebuild. Transparent sprites retain feet
anchors; entrances/bridges stay reachable. The complete41-route CPU contract
checks footprint collision, approach reachability and asset existence. Existing
map tests require fifteen distinct biomes and different repeated finale layouts.
Blocked movement displays a throttled obstacle/water hint without trapping input.
`rogue-place-tour` captures normal intent-driven movement/dialogue/combat on the
revealed first location; `route-map-1` through15 cover all biomes.

Late bosses cap attack growth at1.20 in chapters six/seven, while HP/earlier
chapters remain unchanged. Fixed-loot and actual-story-reward imperfect journeys
are separate measurements: the latter resolves genuine mixed story choices and
applies real relic/camp/companion effects. Neither measures human enjoyment.

`src/bin/playtest.rs` is a developer-only ordinary window fixture runner for
opening/capital/final-battle checkpoints. It reuses `GamePlugin` and normal
`DesktopInputPlugin`; checkpoint construction applies actual narrative outcomes
and chapter growth, without scripting live player inputs. It does not replace
the main binary or constitute an inherited full-run human save.

Surface rendering uses four pure ground textures plus the pure water texture.
`Tile::Structure` is blocked and encoded4, but samples the biome floor rather
than unrelated trees/cliffs. Shader boundary class treats it as a blocker.
Architecture depth switches when the hero walks behind its foot row; bridges
remain underneath actors. Map generation reserves structure footprints before
objective placement, including a walkable central celestial-arch opening.

Bridge cells encode5 and are walkable while rendering river water underneath
the bridge sprite. Reservation keeps water adjoining the span/pier instead of
clearing it into dry strips. Architecture and shadows use any known footprint
cell for fog visibility; centre-only reveal must not hide a nearby building.
The full audit drives every real dialogue through Intent immediately, while
keeping ordinary presentation captures at their original readable cadence.

Portrait map HUD locally compensates viewport/DPI scaling for readable text,
using46%/36% top panels and44%/44% bottom panels with physical padding. Landscape
retains its original layout. The camera computes clamp bounds from actual viewport
aspect. Combat keeps enemy results when companion support resolves; guard/camp
messages quote actual capped mana restoration. Visible首领 tags remain Chinese.
Contacts/objectives avoid the projected overhang of architecture, rather than
appearing to stand on roofs.

Occluding architecture temporarily fades while the hero is behind its overhang,
keeping the player visible without changing collision/depth. Persistent native
input uses `tools/native_wayland_keyboard.sh` and `native_wayland_pointer.sh`;
`native_play_input.py` wraps the fixed-US keyboard, replacing the unreliable
one-key wtype mapping strategy. These tools never inspect/mutate game state.

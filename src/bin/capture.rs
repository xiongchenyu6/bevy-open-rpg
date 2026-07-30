//! Headless offscreen capture binary.
//!
//! Renders the real game (via `GamePlugin`) into an offscreen image target and
//! writes one PNG per frame, driven by a deterministic scripted `Intent` so the
//! proof video is reproducible. The update loop is pumped manually (no winit, no
//! ScheduleRunnerPlugin) following Bevy's `externally_driven_headless_renderer`
//! example. Usage:
//!   capture <out_dir> <frame_count> [preset]
//! Presets:
//!   scripted      default quest-pickup route
//!   chapter-card  start by the first main NPC and show the first chapter card
//!   chapter-card-final start by the final oracle and show the finale chapter card
//!   river-town    start directly on the Act 3 river-town map
//!   moon-cave     start directly in the Act 2 moon cave trial
//!   moon-corridor start directly on the Act 2 moon-echo corridor route map
//!   moon-crystal  start at the Act 2 moon-crystal puzzle
//!   route-mark    start at an optional route-mark checkpoint
//!   route-detour  start at an optional route branch choice
//!   task-board-next start by a board after its first local commission is done
//!   task-board-turn-in start by a completed task-board errand ready to claim
//!   task-contact start by a local NPC who can hand off a commission
//!   task-tracker  start with an accepted commission in the quest tracker
//!   commission-trace start by an accepted commission's on-map field trace
//!   service-favor start by a discounted service after local commissions
//!   npc-route-branch start by a local NPC after resolving a route branch
//!   camp-rest     start by the village rest NPC and show a camp scene
//!   battle-river  start directly in a companion battle against the river boss
//!   battle-bond   start directly in a companion battle after the first bond scene
//!   battle-camp   start directly in a battle with camp preparation active
//!   plague-village start directly on the Act 4 plague-village map
//!   plague-shrine-path start directly on the Act 4 shrine route map
//!   plague-ward  start at the Act 4 plague-ward bell puzzle
//!   battle-plague start directly in the Act 4 boss battle
//!   capital       start directly on the Act 5 capital map
//!   capital-mansion start directly on the Act 5 mansion infiltration map
//!   mansion-gallery start directly on the Act 5 mirror-gallery route map
//!   mansion-mirror start at the Act 5 mansion mirror-array puzzle
//!   battle-capital start directly in the Act 5 boss battle
//!   thunder-drum-path start directly on the Act 6 thunder-drum route map
//!   thunder-drum start at the Act 6 thunder-drum totem puzzle
//!   river-lantern start at the Act 3 river-lantern order puzzle
//!   treasure-cache start by a one-time exploration reward cache
//!   shrine-offering start by an explored shrine ready for battle blessing
//!   shop-gear    start by the village merchant's one-time gear purchase
//!   battle-blessing start directly in a battle with shrine blessing active
//!   bond-choice start at a companion lantern response choice
//!   companion-scene start at a companion personal scene after the first bond
//!   npc-reaction start by a villager reacting to an accepted task-board errand
//!   npc-reaction-complete start by a villager after one board errand is completed
//!   final-task-board start by the Finale task board
//!   final-waterway start directly on the Finale old-dream waterway route map
//!   final-lamp start at the Finale memory-lamp puzzle
//!   final-epilogue start after the final boss by the gatekeeper ending scene
//!   terrain-harness fixed mixed-terrain scene for seam/blending regression
//!   route-map-01..15 fixed full-map proofs for every run-mode biome topology
//!   rogue         roguelike run mode: title → node map → battles/events/rewards
//!   rogue-audit   accelerated full-run traversal without writing frame PNGs
//!   rogue-rest    start directly by a run-mode rest marker with the late party
//!   rogue-guide   start by a run-mode route contact offering a local errand
//!   rogue-puzzle  start directly by a required run-mode route mechanism
//!   rogue-story-south show the new southern old-drum story choice
//!   rogue-story-final show the new finale bell-oath story choice
//!   rogue-party-battle start directly in a late run-mode battle with full party

use bevy::{
    camera::RenderTarget,
    prelude::*,
    render::{
        RenderPlugin,
        render_resource::{TextureFormat, TextureUsages},
        view::screenshot::{Screenshot, save_to_disk},
    },
    time::TimeUpdateStrategy,
    window::ExitCondition,
    winit::WinitPlugin,
};
use std::{fmt::Write as _, time::Duration};

use love_rpg::{
    AppState, EncounterRate, GamePlugin, Intent,
    game::{
        battle::{EncounterKind, EncounterZone, PendingEncounter},
        explore::{CurrentMap, MapKind, PlayerPos},
        lighting,
        quest::{
            BondResponse, BossKind, FinalLamp, MansionMirrorNode, MoonCrystal, PlagueWard,
            QuestLog, QuestRole, RiverLantern, RouteDetour, RouteDetourApproach, ShopGear,
            ShrineBlessing, SideQuest, ThunderDrum, TreasureCache,
        },
    },
};

const R: IVec2 = IVec2::new(1, 0);
const L: IVec2 = IVec2::new(-1, 0);
const U: IVec2 = IVec2::new(0, -1);
const D: IVec2 = IVec2::new(0, 1);

const SETTLE: u32 = 24;
const RUN_AUDIT_SEED: u64 = 0xC0FFEE_5EED;

#[derive(Resource, Default)]
struct CaptureBattleDriver {
    in_battle: bool,
    frame: u32,
}

#[derive(Resource, Default)]
struct CaptureUseCombo(bool);

#[derive(Resource, Default)]
struct CaptureTaskBoard(bool);

#[derive(Resource, Default)]
struct CaptureTaskContact(bool);

#[derive(Resource, Default)]
struct CaptureCommissionTrace(bool);

#[derive(Resource, Default)]
struct CaptureChapterCard(bool);

#[derive(Resource, Default)]
struct CaptureTreasureCache(bool);

#[derive(Resource, Default)]
struct CaptureShrineOffering(bool);

#[derive(Resource, Default)]
struct CaptureNpcReaction(bool);

#[derive(Resource, Default)]
struct CaptureBondChoice(bool);

#[derive(Resource, Default)]
struct CaptureCampRest(bool);

#[derive(Resource, Default)]
struct CaptureRiverLantern(bool);

#[derive(Resource, Default)]
struct CaptureMoonCrystal(bool);

#[derive(Resource, Default)]
struct CaptureRouteMark(bool);

#[derive(Resource, Default)]
struct CaptureRouteDetour(bool);

#[derive(Resource, Default)]
struct CapturePlagueWard(bool);

#[derive(Resource, Default)]
struct CaptureMansionMirror(bool);

#[derive(Resource, Default)]
struct CaptureThunderDrum(bool);

#[derive(Resource, Default)]
struct CaptureFinalEpilogue(bool);

#[derive(Resource, Default)]
struct CaptureFinalLamp(bool);

#[derive(Resource, Default)]
struct CaptureRogue(bool);

#[derive(Resource, Default)]
struct CaptureInventory(bool);

#[derive(Resource, Default)]
struct CaptureRouteMap(bool);

#[derive(Resource, Default)]
struct CaptureStoryProof(bool);

#[derive(Resource, Default)]
struct CaptureRunAudit(bool);

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| "screenshots/cap".to_string());
    let frames: u32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(450);
    let preset = args.get(3).map(String::as_str).unwrap_or("scripted");
    std::fs::create_dir_all(&out).ok();

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: ExitCondition::DontExit,
                ..default()
            })
            .set(RenderPlugin {
                // Ensure shaders are ready for the first rendered frame.
                synchronous_pipeline_compilation: true,
                ..default()
            })
            // We own the update loop, so winit is not needed.
            .disable::<WinitPlugin>(),
    )
    .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
        1.0 / 30.0,
    )));

    // Allocate the offscreen render target before any system reads it.
    let mut image = Image::new_target_texture(1280, 720, TextureFormat::Rgba8UnormSrgb, None);
    image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
    let handle = app.world_mut().resource_mut::<Assets<Image>>().add(image);

    app.add_plugins(GamePlugin);
    app.init_resource::<CaptureBattleDriver>();
    app.init_resource::<CaptureUseCombo>();
    app.init_resource::<CaptureTaskBoard>();
    app.init_resource::<CaptureTaskContact>();
    app.init_resource::<CaptureCommissionTrace>();
    app.init_resource::<CaptureChapterCard>();
    app.init_resource::<CaptureTreasureCache>();
    app.init_resource::<CaptureShrineOffering>();
    app.init_resource::<CaptureNpcReaction>();
    app.init_resource::<CaptureBondChoice>();
    app.init_resource::<CaptureCampRest>();
    app.init_resource::<CaptureRiverLantern>();
    app.init_resource::<CaptureMoonCrystal>();
    app.init_resource::<CaptureRouteMark>();
    app.init_resource::<CaptureRouteDetour>();
    app.init_resource::<CapturePlagueWard>();
    app.init_resource::<CaptureMansionMirror>();
    app.init_resource::<CaptureThunderDrum>();
    app.init_resource::<CaptureFinalEpilogue>();
    app.init_resource::<CaptureFinalLamp>();
    app.init_resource::<CaptureRogue>();
    app.init_resource::<CaptureInventory>();
    app.init_resource::<CaptureRouteMap>();
    app.init_resource::<CaptureStoryProof>();
    app.init_resource::<CaptureRunAudit>();
    app.add_systems(PreUpdate, reveal_route_map_capture);
    match preset {
        "terrain-harness" => {
            use love_rpg::game::{
                core::{MAP_H, MAP_W},
                explore::{MapKind, Tile},
                roguelike::{RunState, scene::RunSceneState},
            };
            app.world_mut().resource_mut::<CaptureRogue>().0 = true;
            let mut run = {
                let mut rng = app.world_mut().resource_mut::<love_rpg::game::Rng>();
                rng.0 = 0x7E22_A11;
                RunState::new(&mut rng)
            };
            run.chapter = 5;
            run.card_shown = true;
            run.accept_journey_task();
            app.world_mut().insert_resource(run);

            let mut tiles = vec![vec![Tile::Path; MAP_W as usize]; MAP_H as usize];
            for row in 0..MAP_H as usize {
                tiles[row][0] = Tile::Wall;
                tiles[row][MAP_W as usize - 1] = Tile::Wall;
            }
            for col in 0..MAP_W as usize {
                tiles[0][col] = Tile::Wall;
                tiles[MAP_H as usize - 1][col] = Tile::Wall;
            }
            for row in 4..8 {
                for col in 9..14 {
                    tiles[row][col] = Tile::Grass;
                }
                for col in 17..22 {
                    tiles[row][col] = Tile::Water;
                }
            }
            for row in 8..12 {
                for col in 9..14 {
                    tiles[row][col] = Tile::Wall;
                }
                for col in 17..22 {
                    tiles[row][col] = Tile::Grass;
                }
            }
            app.world_mut().insert_resource(RunSceneState {
                map: MapKind::SouthernRoad,
                tiles,
                revealed: vec![vec![true; MAP_W as usize]; MAP_H as usize],
                col: 15,
                row: 8,
                facing_left: false,
                markers: Vec::new(),
                portals: Vec::new(),
                flow: Vec::new(),
                hazards: Vec::new(),
                intro_shown: true,
                cooldown: 0.0,
            });
            app.world_mut()
                .resource_mut::<NextState<AppState>>()
                .set(AppState::RunScene);
        }
        preset if preset.starts_with("route-map-") => {
            prepare_route_map_preset(&mut app, preset);
        }
        "rogue" => {
            app.world_mut().resource_mut::<CaptureRogue>().0 = true;
            // Deterministic-ish start (the title screen still mixes in wall
            // clock; the script is cadence-based, not frame-exact).
            app.world_mut().resource_mut::<love_rpg::game::Rng>().0 = 0xC0FFEE_5EED;
        }
        "rogue-audit" => {
            app.world_mut().resource_mut::<CaptureRogue>().0 = true;
            app.world_mut().resource_mut::<CaptureRunAudit>().0 = true;
            app.world_mut().resource_mut::<love_rpg::game::Rng>().0 = RUN_AUDIT_SEED;
            *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
                TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(0.16));
        }
        "rogue-ch2" | "rogue-ch3" | "rogue-ch4" | "rogue-ch5" | "rogue-ch6" | "rogue-ch7" => {
            use love_rpg::game::roguelike::RunState;
            app.world_mut().resource_mut::<CaptureRogue>().0 = true;
            let chapter = match preset {
                "rogue-ch2" => 1,
                "rogue-ch3" => 2,
                "rogue-ch4" => 3,
                "rogue-ch5" => 4,
                "rogue-ch6" => 5,
                _ => 6,
            };
            let mut run = {
                let mut rng = app.world_mut().resource_mut::<love_rpg::game::Rng>();
                rng.0 = 0xC0FFEE_5EED ^ (chapter as u64) << 8;
                RunState::new(&mut rng)
            };
            run.chapter = chapter;
            run.card_shown = true;
            app.world_mut().insert_resource(run);
            app.world_mut()
                .resource_mut::<NextState<AppState>>()
                .set(AppState::NodeMap);
        }
        "rogue-inventory" => {
            use love_rpg::game::explore::MapKind;
            use love_rpg::game::roguelike::{
                Relic, RunState,
                graph::NodeKind,
                scene::{RunSceneState, SceneMarker},
            };
            app.world_mut().resource_mut::<CaptureRogue>().0 = true;
            app.world_mut().resource_mut::<CaptureInventory>().0 = true;
            let mut run = {
                let mut rng = app.world_mut().resource_mut::<love_rpg::game::Rng>();
                rng.0 = 0xC0FFEE_5EED;
                RunState::new(&mut rng)
            };
            run.card_shown = true;
            run.relics = vec![Relic::SwordTassel, Relic::SandalCharm, Relic::PixiuPouch];
            run.daoxin = 2;
            run.qingyuan = 3;
            run.accept_journey_task();
            app.world_mut().insert_resource(run);
            let mut scene = RunSceneState {
                map: MapKind::Village,
                tiles: Vec::new(),
                revealed: Vec::new(),
                col: -1,
                row: -1,
                facing_left: false,
                markers: vec![SceneMarker {
                    kind: NodeKind::Fight,
                    col: 24,
                    row: 12,
                    cleared: false,
                }],
                portals: Vec::new(),
                flow: Vec::new(),
                hazards: Vec::new(),
                intro_shown: true,
                cooldown: 0.0,
            };
            love_rpg::game::roguelike::scene::seed_flow(&mut scene);
            app.world_mut().insert_resource(scene);
            app.world_mut()
                .resource_mut::<NextState<AppState>>()
                .set(AppState::RunScene);
        }
        "rogue-boss" => {
            use love_rpg::game::explore::MapKind;
            use love_rpg::game::roguelike::{
                RunChapterVow, RunState,
                graph::NodeKind,
                scene::{RunSceneState, SceneMarker},
            };
            app.world_mut().resource_mut::<CaptureRogue>().0 = true;
            let mut run = {
                let mut rng = app.world_mut().resource_mut::<love_rpg::game::Rng>();
                rng.0 = 0xB055_F16D;
                RunState::new(&mut rng)
            };
            run.card_shown = true;
            run.accept_journey_task();
            run.record_chapter_vow(run.chapter, RunChapterVow::Heart);
            app.world_mut().insert_resource(run);
            {
                // 相当于打满一章的积累,让取证局能撑到 boss 二阶段。
                let mut stats = app
                    .world_mut()
                    .resource_mut::<love_rpg::game::PlayerStats>();
                stats.atk += 8;
                stats.def += 4;
                stats.max_hp += 80;
                stats.hp = stats.max_hp;
                stats.potions = 6;
            }
            // 单魔门标记:直达对峙台词 → 章 boss 战(取证二阶段变身)。
            let mut scene = RunSceneState {
                map: MapKind::Bamboo,
                tiles: Vec::new(),
                revealed: Vec::new(),
                col: -1,
                row: -1,
                facing_left: false,
                markers: vec![SceneMarker {
                    kind: NodeKind::Boss,
                    col: 15,
                    row: 8,
                    cleared: false,
                }],
                portals: Vec::new(),
                flow: Vec::new(),
                hazards: Vec::new(),
                intro_shown: true,
                cooldown: 0.0,
            };
            love_rpg::game::roguelike::scene::seed_flow(&mut scene);
            app.world_mut().insert_resource(scene);
            app.world_mut()
                .resource_mut::<NextState<AppState>>()
                .set(AppState::RunScene);
        }
        "rogue-market" => {
            use love_rpg::game::explore::MapKind;
            use love_rpg::game::roguelike::{
                RunState,
                graph::NodeKind,
                scene::{RunSceneState, SceneMarker},
            };
            app.world_mut().resource_mut::<CaptureRogue>().0 = true;
            let mut run = {
                let mut rng = app.world_mut().resource_mut::<love_rpg::game::Rng>();
                rng.0 = 0xC0FFEE_5EED;
                RunState::new(&mut rng)
            };
            run.card_shown = true; // skip the chapter card, go straight to the stalls
            run.accept_journey_task();
            app.world_mut().insert_resource(run);
            // A hand-built Village stage with a single market marker; the
            // scene spawner fills in hero position and the flow field is
            // recomputed by the driver as it steers.
            let mut scene = RunSceneState {
                map: MapKind::Village,
                tiles: Vec::new(),
                revealed: Vec::new(),
                col: -1,
                row: -1,
                facing_left: false,
                markers: vec![SceneMarker {
                    kind: NodeKind::Market,
                    col: 15,
                    row: 8,
                    cleared: false,
                }],
                portals: Vec::new(),
                flow: Vec::new(),
                hazards: Vec::new(),
                intro_shown: true,
                cooldown: 0.0,
            };
            love_rpg::game::roguelike::scene::seed_flow(&mut scene);
            app.world_mut().insert_resource(scene);
            app.world_mut()
                .resource_mut::<NextState<AppState>>()
                .set(AppState::RunScene);
        }
        "rogue-rest" => {
            use love_rpg::game::explore::MapKind;
            use love_rpg::game::roguelike::{
                RunState,
                graph::NodeKind,
                scene::{RunSceneState, SceneMarker},
            };
            app.world_mut().resource_mut::<CaptureRogue>().0 = true;
            let mut run = {
                let mut rng = app.world_mut().resource_mut::<love_rpg::game::Rng>();
                rng.0 = 0xCAFE_F1E0;
                RunState::new(&mut rng)
            };
            run.chapter = 5;
            run.stage = 0;
            run.card_shown = true;
            run.accept_journey_task();
            app.world_mut().insert_resource(run);
            {
                let mut stats = app
                    .world_mut()
                    .resource_mut::<love_rpg::game::PlayerStats>();
                stats.hp = (stats.max_hp - 30).max(1);
                stats.mp = (stats.max_mp - 12).max(0);
            }
            let mut scene = RunSceneState {
                map: MapKind::SouthernRoad,
                tiles: Vec::new(),
                revealed: Vec::new(),
                col: -1,
                row: -1,
                facing_left: false,
                markers: vec![SceneMarker {
                    kind: NodeKind::Rest,
                    col: 15,
                    row: 8,
                    cleared: false,
                }],
                portals: Vec::new(),
                flow: Vec::new(),
                hazards: Vec::new(),
                intro_shown: true,
                cooldown: 0.0,
            };
            love_rpg::game::roguelike::scene::seed_flow(&mut scene);
            app.world_mut().insert_resource(scene);
            app.world_mut()
                .resource_mut::<NextState<AppState>>()
                .set(AppState::RunScene);
        }
        "rogue-guide" => {
            use love_rpg::game::{
                core::{MAP_H, MAP_W},
                explore::{MapKind, Tile},
                roguelike::{
                    RunState,
                    graph::NodeKind,
                    scene::{RunSceneState, SceneMarker},
                },
            };
            app.world_mut().resource_mut::<CaptureRogue>().0 = true;
            let mut run = {
                let mut rng = app.world_mut().resource_mut::<love_rpg::game::Rng>();
                rng.0 = 0x51_6EED;
                RunState::new(&mut rng)
            };
            run.chapter = 1;
            run.stage = 1;
            run.card_shown = true;
            run.accept_journey_task();
            app.world_mut().insert_resource(run);
            let mut tiles = vec![vec![Tile::Path; MAP_W as usize]; MAP_H as usize];
            for row in 0..MAP_H as usize {
                tiles[row][0] = Tile::Wall;
                tiles[row][MAP_W as usize - 1] = Tile::Wall;
            }
            for col in 0..MAP_W as usize {
                tiles[0][col] = Tile::Wall;
                tiles[MAP_H as usize - 1][col] = Tile::Wall;
            }
            let mut scene = RunSceneState {
                map: MapKind::MoonEchoCorridor,
                tiles,
                revealed: Vec::new(),
                col: -1,
                row: -1,
                facing_left: false,
                markers: vec![
                    SceneMarker {
                        kind: NodeKind::Guide,
                        col: 3,
                        row: 1,
                        cleared: false,
                    },
                    SceneMarker {
                        kind: NodeKind::Event,
                        col: 8,
                        row: 1,
                        cleared: false,
                    },
                ],
                portals: Vec::new(),
                flow: Vec::new(),
                hazards: Vec::new(),
                intro_shown: true,
                cooldown: 0.0,
            };
            love_rpg::game::roguelike::scene::seed_flow(&mut scene);
            app.world_mut().insert_resource(scene);
            app.world_mut()
                .resource_mut::<NextState<AppState>>()
                .set(AppState::RunScene);
        }
        "rogue-puzzle" => {
            use love_rpg::game::explore::MapKind;
            use love_rpg::game::roguelike::{
                RunState,
                graph::NodeKind,
                scene::{RunSceneState, SceneMarker},
            };
            app.world_mut().resource_mut::<CaptureRogue>().0 = true;
            let mut run = {
                let mut rng = app.world_mut().resource_mut::<love_rpg::game::Rng>();
                rng.0 = 0xC0FFEE_5EED;
                RunState::new(&mut rng)
            };
            run.chapter = 1;
            run.stage = 2;
            run.card_shown = true;
            run.accept_journey_task();
            app.world_mut().insert_resource(run);
            let mut scene = RunSceneState {
                map: MapKind::MoonEchoCorridor,
                tiles: Vec::new(),
                revealed: Vec::new(),
                col: -1,
                row: -1,
                facing_left: false,
                markers: vec![SceneMarker {
                    kind: NodeKind::Puzzle,
                    col: 15,
                    row: 8,
                    cleared: false,
                }],
                portals: Vec::new(),
                flow: Vec::new(),
                hazards: Vec::new(),
                intro_shown: true,
                cooldown: 0.0,
            };
            love_rpg::game::roguelike::scene::seed_flow(&mut scene);
            app.world_mut().insert_resource(scene);
            app.world_mut()
                .resource_mut::<NextState<AppState>>()
                .set(AppState::RunScene);
        }
        "rogue-story-south" => {
            prepare_story_proof_preset(
                &mut app,
                5,
                3,
                MapKind::ThunderDrumPath,
                2,
                "剧情 · 南疆旧鼓",
            );
        }
        "rogue-story-final" => {
            prepare_story_proof_preset(
                &mut app,
                6,
                9,
                MapKind::DreamWaterway,
                6,
                "剧情 · 终门铃誓",
            );
        }
        "rogue-ending" | "rogue-ending-defeat" => {
            use love_rpg::game::roguelike::{RunOutcome, RunState};
            app.world_mut().resource_mut::<CaptureRogue>().0 = true;
            let mut run = {
                let mut rng = app.world_mut().resource_mut::<love_rpg::game::Rng>();
                rng.0 = 0xC0FFEE_5EED;
                RunState::new(&mut rng)
            };
            run.outcome = Some(if preset == "rogue-ending-defeat" {
                RunOutcome::Defeat
            } else {
                RunOutcome::VictoryLove
            });
            run.qingyuan = 7;
            run.daoxin = 3;
            run.fights_won = 12;
            app.world_mut().insert_resource(run);
            app.world_mut()
                .resource_mut::<NextState<AppState>>()
                .set(AppState::Ending);
        }
        "rogue-party-battle" => {
            use love_rpg::game::roguelike::{FightRank, RunState, battle_mods_for};
            app.world_mut().resource_mut::<CaptureRogue>().0 = true;
            let mut run = {
                let mut rng = app.world_mut().resource_mut::<love_rpg::game::Rng>();
                rng.0 = 0x51_5A11;
                RunState::new(&mut rng)
            };
            run.chapter = 3;
            run.stage = 0;
            run.card_shown = true;
            run.current_fight = Some(FightRank::Elite);
            run.qingyuan = 6;
            run.daoxin = 5;
            let mods = battle_mods_for(&run, FightRank::Elite);
            app.world_mut().insert_resource(PendingEncounter {
                zone: EncounterZone::FinalSanctum,
                kind: EncounterKind::Random,
            });
            app.world_mut().insert_resource(mods);
            app.world_mut().insert_resource(run);
            {
                let mut stats = app
                    .world_mut()
                    .resource_mut::<love_rpg::game::PlayerStats>();
                stats.atk += 10;
                stats.def += 5;
                stats.max_hp += 60;
                stats.hp = stats.max_hp - 24;
                stats.max_mp += 20;
                stats.mp = stats.max_mp;
                stats.potions = 5;
            }
            app.world_mut()
                .resource_mut::<NextState<AppState>>()
                .set(AppState::Battle);
        }
        "chapter-card" => prepare_chapter_card_preset(&mut app),
        "chapter-card-final" => prepare_final_chapter_card_preset(&mut app),
        "task-board" => prepare_task_board_preset(&mut app),
        "task-contact" => prepare_task_contact_preset(&mut app),
        "task-board-next" => prepare_task_board_next_preset(&mut app),
        "task-board-turn-in" => prepare_task_board_turn_in_preset(&mut app),
        "task-tracker" => prepare_task_tracker_preset(&mut app),
        "commission-trace" => prepare_commission_trace_preset(&mut app),
        "camp-rest" => prepare_camp_rest_preset(&mut app),
        "treasure-cache" => prepare_treasure_cache_preset(&mut app),
        "shrine-offering" => prepare_shrine_offering_preset(&mut app),
        "shop-gear" => prepare_shop_gear_preset(&mut app),
        "npc-reaction" => prepare_npc_reaction_preset(&mut app),
        "npc-reaction-complete" => prepare_npc_reaction_complete_preset(&mut app),
        "npc-care" => prepare_npc_care_preset(&mut app),
        "npc-route-branch" => prepare_npc_route_branch_preset(&mut app),
        "service-favor" => prepare_service_favor_preset(&mut app),
        "river-town" => prepare_river_town_preset(&mut app),
        "river-lantern" => prepare_river_lantern_preset(&mut app),
        "moon-cave" => prepare_moon_cave_preset(&mut app),
        "moon-corridor" => prepare_moon_corridor_preset(&mut app),
        "moon-crystal" => prepare_moon_crystal_preset(&mut app),
        "route-mark" => prepare_route_mark_preset(&mut app),
        "route-detour" => prepare_route_detour_preset(&mut app),
        "battle-river" => prepare_battle_river_preset(&mut app),
        "battle-bond" => prepare_battle_bond_preset(&mut app),
        "battle-camp" => prepare_battle_camp_preset(&mut app),
        "battle-blessing" => prepare_battle_blessing_preset(&mut app),
        "bond-choice" => prepare_bond_choice_preset(&mut app),
        "companion-scene" => prepare_companion_scene_preset(&mut app),
        "river-reed" => prepare_river_reed_preset(&mut app),
        "plague-village" => prepare_plague_village_preset(&mut app),
        "plague-shrine-path" => prepare_plague_shrine_path_preset(&mut app),
        "plague-ward" => prepare_plague_ward_preset(&mut app),
        "battle-plague" => prepare_battle_plague_preset(&mut app),
        "capital" => prepare_capital_preset(&mut app),
        "capital-mansion" => prepare_capital_mansion_preset(&mut app),
        "mansion-gallery" => prepare_mansion_gallery_preset(&mut app),
        "mansion-mirror" => prepare_mansion_mirror_preset(&mut app),
        "battle-capital" => prepare_battle_capital_preset(&mut app),
        "southern-road" => prepare_southern_road_preset(&mut app),
        "thunder-drum-path" => prepare_thunder_drum_path_preset(&mut app),
        "thunder-drum" => prepare_thunder_drum_preset(&mut app),
        "battle-southern" => prepare_battle_southern_preset(&mut app),
        "final-sanctum" => prepare_final_sanctum_preset(&mut app),
        "final-task-board" => prepare_final_task_board_preset(&mut app),
        "final-waterway" => prepare_final_waterway_preset(&mut app),
        "final-lamp" => prepare_final_lamp_preset(&mut app),
        "final-epilogue" => prepare_final_epilogue_preset(&mut app),
        "battle-final" => prepare_battle_final_preset(&mut app),
        _ => {}
    }
    // Legacy explore presets predate the roguelike Title state and expect to
    // start on the free-roam overworld. Battle presets set their own state.
    if preset_starts_in_explore(preset) {
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Explore);
    }
    app.finish();
    app.cleanup();

    // Camera renders into the offscreen image. `IsDefaultUiCamera` is required so
    // Bevy UI (HUD, dialogue, battle menu) renders to this image-target camera
    // rather than looking for a windowed default UI camera.
    let render_target: RenderTarget = handle.clone().into();
    app.world_mut().spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::Fixed {
                width: 1280.0,
                height: 720.0,
            },
            ..OrthographicProjection::default_2d()
        }),
        render_target,
        IsDefaultUiCamera,
        lighting::camera_config(),
    ));

    // The scripted route first demonstrates quest pickup, then enables
    // guaranteed grass encounters after the dialogue proof.
    app.world_mut().resource_mut::<EncounterRate>().0 = 0.0;

    // Warm-up: let the scene spawn and the font load before capturing.
    for _ in 0..SETTLE {
        clear_intent(&mut app);
        app.update();
    }

    if app.world().resource::<CaptureRunAudit>().0 {
        run_full_audit(&mut app, frames, &out);
        return;
    }

    for f in 0..frames {
        set_intent(&mut app, f);
        let path = format!("{}/frame{:05}.png", out, f);
        app.world_mut()
            .spawn(Screenshot::image(handle.clone()))
            .observe(save_to_disk(path));
        app.update();
    }

    // Flush pending async screenshot read-backs / disk writes.
    for _ in 0..12 {
        clear_intent(&mut app);
        app.update();
    }

    println!("[capture] wrote {frames} frames to {out}");
}

fn run_full_audit(app: &mut App, max_frames: u32, out: &str) {
    use love_rpg::game::{
        Rng,
        roguelike::{CHAPTER_COUNT, JOURNEY_ROUTE_STAGES, RunOutcome, RunState},
    };

    let started = std::time::Instant::now();
    let mut visited_stages = Vec::new();
    let mut state_trace = Vec::new();
    let mut previous_state = None;
    let mut movement_inputs = 0u32;
    let mut confirm_inputs = 0u32;
    let mut menu_inputs = 0u32;
    let mut frames_run = 0u32;
    let mut outcome = None;
    let mut play_time = "00:00:00".to_string();
    let mut chapter_times = String::new();
    let mut accepted_tasks = 0usize;
    let mut completed_tasks = 0usize;
    let mut chapter_seals = 0usize;
    let mut fights_won = 0u32;
    let mut story_memories = 0usize;
    let mut route_puzzles = 0u32;
    let mut route_contacts = 0u32;
    let mut camp_scenes = 0usize;
    let mut chapter_vows = 0usize;
    let mut final_chapter = 0usize;
    let mut final_stage = 0usize;
    let mut audit_seed_applied = false;

    for frame in 0..max_frames {
        set_intent(app, frame);
        {
            let intent = app.world().resource::<Intent>();
            movement_inputs += u32::from(intent.move_dir.is_some());
            confirm_inputs += u32::from(intent.confirm);
            menu_inputs += u32::from(intent.up || intent.down || intent.cancel);
        }
        app.update();
        frames_run = frame + 1;

        // Title startup intentionally mixes wall-clock entropy into normal
        // runs. Replace that first RunState before NodeMap spawns so this audit
        // owns both the generated route and every later random draw.
        if !audit_seed_applied && app.world().contains_resource::<RunState>() {
            let run = {
                let mut rng = app.world_mut().resource_mut::<Rng>();
                rng.0 = RUN_AUDIT_SEED;
                RunState::new(&mut rng)
            };
            app.world_mut().insert_resource(run);
            audit_seed_applied = true;
        }

        let state = *app.world().resource::<State<AppState>>().get();
        if previous_state != Some(state) {
            state_trace.push(format!("{state:?}@{frame}"));
            previous_state = Some(state);
        }

        if let Some(run) = app.world().get_resource::<RunState>() {
            let stage = (run.chapter, run.stage);
            if !visited_stages.contains(&stage) {
                visited_stages.push(stage);
                println!(
                    "[run-audit] entered chapter {} stage {} ({}/{})",
                    run.chapter + 1,
                    run.stage + 1,
                    visited_stages.len(),
                    JOURNEY_ROUTE_STAGES
                );
            }
            outcome = run.outcome;
            play_time = run.play_time_summary();
            chapter_times = run.chapter_play_time_archive_summary();
            accepted_tasks = run.accepted_journey_tasks.len();
            completed_tasks = run.completed_journey_tasks.len();
            chapter_seals = run.chapter_seals.len();
            fights_won = run.fights_won;
            story_memories = run.seen_story_scenes.len();
            route_puzzles = run.route_puzzles_solved;
            route_contacts = run.route_contacts_helped;
            camp_scenes = run.camp_scenes_seen.len();
            chapter_vows = run.chapter_vows.len();
            final_chapter = run.chapter;
            final_stage = run.stage;
        }

        if state == AppState::Ending && outcome.is_some() {
            break;
        }
    }

    let (hp, max_hp, mp, max_mp, atk, def, potions, gold) = {
        let stats = app.world().resource::<love_rpg::game::PlayerStats>();
        (
            stats.hp,
            stats.max_hp,
            stats.mp,
            stats.max_mp,
            stats.atk,
            stats.def,
            stats.potions,
            stats.gold,
        )
    };
    let victory = matches!(
        outcome,
        Some(RunOutcome::VictoryLove | RunOutcome::VictoryResolve | RunOutcome::VictoryBoth)
    );
    let passed = audit_seed_applied
        && victory
        && visited_stages.len() == JOURNEY_ROUTE_STAGES
        && accepted_tasks == JOURNEY_ROUTE_STAGES
        && completed_tasks == JOURNEY_ROUTE_STAGES
        && chapter_seals == CHAPTER_COUNT;
    let status = if passed { "PASS" } else { "FAIL" };
    let outcome_label = outcome
        .map(|value| format!("{value:?}"))
        .unwrap_or_else(|| "None".to_string());
    let mut report = String::new();
    writeln!(report, "run_audit_status={status}").unwrap();
    writeln!(report, "human_runtime_proof=false").unwrap();
    writeln!(report, "seed=0x{RUN_AUDIT_SEED:X}").unwrap();
    writeln!(report, "seed_applied={audit_seed_applied}").unwrap();
    writeln!(report, "max_frames={max_frames}").unwrap();
    writeln!(report, "frames_run={frames_run}").unwrap();
    writeln!(
        report,
        "wall_seconds={:.3}",
        started.elapsed().as_secs_f64()
    )
    .unwrap();
    writeln!(report, "accelerated_play_time={play_time}").unwrap();
    writeln!(report, "outcome={outcome_label}").unwrap();
    writeln!(
        report,
        "final_route={}-{}",
        final_chapter + 1,
        final_stage + 1
    )
    .unwrap();
    writeln!(
        report,
        "visited_stages={}/{}",
        visited_stages.len(),
        JOURNEY_ROUTE_STAGES
    )
    .unwrap();
    writeln!(
        report,
        "accepted_main_tasks={accepted_tasks}/{}",
        JOURNEY_ROUTE_STAGES
    )
    .unwrap();
    writeln!(
        report,
        "completed_main_tasks={completed_tasks}/{}",
        JOURNEY_ROUTE_STAGES
    )
    .unwrap();
    writeln!(report, "chapter_seals={chapter_seals}/{CHAPTER_COUNT}").unwrap();
    writeln!(report, "fights_won={fights_won}").unwrap();
    writeln!(report, "story_memories={story_memories}").unwrap();
    writeln!(report, "route_puzzles={route_puzzles}").unwrap();
    writeln!(report, "route_contacts={route_contacts}").unwrap();
    writeln!(report, "camp_scenes={camp_scenes}").unwrap();
    writeln!(report, "chapter_vows={chapter_vows}").unwrap();
    writeln!(report, "movement_inputs={movement_inputs}").unwrap();
    writeln!(report, "confirm_inputs={confirm_inputs}").unwrap();
    writeln!(report, "menu_inputs={menu_inputs}").unwrap();
    writeln!(
        report,
        "final_stats=hp:{hp}/{max_hp},mp:{mp}/{max_mp},atk:{atk},def:{def},potions:{potions},gold:{gold}"
    )
    .unwrap();
    writeln!(report, "chapter_times=").unwrap();
    writeln!(report, "{chapter_times}").unwrap();
    writeln!(report, "state_trace={}", state_trace.join(" -> ")).unwrap();
    writeln!(
        report,
        "note=accelerated automation validates reachability and bookkeeping; it is not a human ten-hour playtest"
    )
    .unwrap();

    let path = format!("{out}/run-audit.txt");
    std::fs::write(&path, &report).expect("write run audit report");
    print!("{report}");
    println!("[run-audit] report: {path}");
    if !passed {
        eprintln!("[run-audit] full-run invariants failed");
        std::process::exit(2);
    }
}

fn prepare_route_map_preset(app: &mut App, preset: &str) {
    const ROUTE_MAP_STAGES: [(usize, usize); 15] = [
        (0, 0), // Village
        (0, 1), // Bamboo
        (0, 3), // Cave
        (0, 2), // MoonEchoCorridor
        (2, 0), // RiverTown
        (2, 1), // RiverReedBed
        (3, 1), // PlagueVillage
        (3, 0), // PlagueShrinePath
        (4, 0), // Capital
        (4, 1), // CapitalMansion
        (4, 2), // MansionMirrorGallery
        (5, 0), // SouthernRoad
        (5, 1), // ThunderDrumPath
        (6, 0), // FinalSanctum
        (6, 1), // DreamWaterway
    ];
    let index = preset
        .strip_prefix("route-map-")
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|index| (1..=ROUTE_MAP_STAGES.len()).contains(index))
        .unwrap_or_else(|| panic!("invalid route-map preset: {preset}"));
    let (chapter, stage) = ROUTE_MAP_STAGES[index - 1];
    let mut run = {
        let mut rng = app.world_mut().resource_mut::<love_rpg::game::Rng>();
        rng.0 = 0xA71A_5000_0000_0000 ^ ((chapter as u64) << 12) ^ stage as u64;
        love_rpg::game::roguelike::RunState::new(&mut rng)
    };
    run.chapter = chapter;
    run.stage = stage;
    run.card_shown = true;
    run.accept_journey_task();
    app.world_mut().insert_resource(run);
    app.world_mut().resource_mut::<CaptureRouteMap>().0 = true;
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::NodeMap);
}

fn prepare_story_proof_preset(
    app: &mut App,
    chapter: usize,
    stage: usize,
    map: MapKind,
    roll: usize,
    title: &str,
) {
    use love_rpg::game::{
        core::{MAP_H, MAP_W},
        roguelike::{RunState, event::RunDialogue, scene::RunSceneState},
    };

    let mut run = {
        let mut rng = app.world_mut().resource_mut::<love_rpg::game::Rng>();
        rng.0 = 0x5707_1000_0000_0000 ^ ((chapter as u64) << 12) ^ stage as u64;
        RunState::new(&mut rng)
    };
    run.chapter = chapter;
    run.stage = stage;
    run.card_shown = true;
    run.accept_journey_task();
    run.seen_story_scenes.push((chapter, roll));
    app.world_mut().insert_resource(run);
    app.world_mut().resource_mut::<CaptureStoryProof>().0 = true;

    let mut scene = RunSceneState {
        map,
        tiles: Vec::new(),
        revealed: vec![vec![true; MAP_W as usize]; MAP_H as usize],
        col: -1,
        row: -1,
        facing_left: false,
        markers: Vec::new(),
        portals: Vec::new(),
        flow: Vec::new(),
        hazards: Vec::new(),
        intro_shown: true,
        cooldown: 0.0,
    };
    love_rpg::game::roguelike::scene::seed_flow(&mut scene);
    app.world_mut().insert_resource(scene);
    {
        let mut dialogue = app.world_mut().resource_mut::<RunDialogue>();
        dialogue.open_story(chapter, roll);
        dialogue.title = title.to_string();
        dialogue.idx = dialogue.lines.len().saturating_sub(1);
    }
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::RunScene);
}

fn reveal_route_map_capture(
    capture: Res<CaptureRouteMap>,
    scene: Option<ResMut<love_rpg::game::roguelike::scene::RunSceneState>>,
) {
    if !capture.0 {
        return;
    }
    let Some(mut scene) = scene else { return };
    scene.intro_shown = true;
    scene.revealed = vec![
        vec![true; love_rpg::game::core::MAP_W as usize];
        love_rpg::game::core::MAP_H as usize
    ];
}

fn preset_starts_in_explore(preset: &str) -> bool {
    !preset.starts_with("rogue")
        && !preset.starts_with("terrain-")
        && !preset.starts_with("route-map-")
        && !matches!(
            preset,
            "battle-river"
                | "battle-bond"
                | "battle-camp"
                | "battle-blessing"
                | "battle-plague"
                | "battle-capital"
                | "battle-southern"
                | "battle-final"
        )
}

fn prepare_task_board_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::Village;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 21;
        pos.row = 2;
        pos.facing = U;
    }
    app.world_mut().resource_mut::<CaptureTaskBoard>().0 = true;
}

fn prepare_task_contact_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::Village;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 6;
        pos.row = 8;
        pos.facing = U;
    }
    app.world_mut().resource_mut::<CaptureTaskContact>().0 = true;
}

fn prepare_chapter_card_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::Village;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 22;
        pos.row = 2;
        pos.facing = U;
    }
    app.world_mut().resource_mut::<CaptureChapterCard>().0 = true;
}

fn prepare_final_chapter_card_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::FinalSanctum;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 13;
        pos.row = 7;
        pos.facing = U;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        advance_to_southern_road_complete(&mut quest);
    }
    app.world_mut().resource_mut::<CaptureChapterCard>().0 = true;
}

fn prepare_task_board_turn_in_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::Village;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 21;
        pos.row = 2;
        pos.facing = U;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        quest.interact_side_quest(SideQuest::VillageTrail);
        quest.record_side_victory();
        quest.record_side_victory();
    }
    app.world_mut().resource_mut::<CaptureTaskBoard>().0 = true;
}

fn prepare_task_board_next_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::Village;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 21;
        pos.row = 2;
        pos.facing = U;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        quest.interact_side_quest(SideQuest::VillageTrail);
        quest.record_side_victory();
        quest.record_side_victory();
        quest.interact_side_quest(SideQuest::VillageTrail);
    }
    app.world_mut().resource_mut::<CaptureTaskBoard>().0 = true;
}

fn prepare_task_tracker_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::Village;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 18;
        pos.row = 7;
        pos.facing = D;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        quest.interact_side_quest(SideQuest::VillageTrail);
    }
}

fn prepare_commission_trace_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::Village;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 15;
        pos.row = 12;
        pos.facing = D;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        quest.interact_side_quest(SideQuest::VillageTrail);
    }
    app.world_mut().resource_mut::<CaptureCommissionTrace>().0 = true;
}

fn prepare_camp_rest_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::Village;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 6;
        pos.row = 8;
        pos.facing = U;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
    }
    app.world_mut().resource_mut::<CaptureCampRest>().0 = true;
}

fn prepare_treasure_cache_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::Village;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 20;
        pos.row = 5;
        pos.facing = U;
    }
    app.world_mut().resource_mut::<CaptureTreasureCache>().0 = true;
}

fn prepare_shrine_offering_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::Village;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 20;
        pos.row = 5;
        pos.facing = U;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        quest.interact_treasure(TreasureCache::VillageShrine);
    }
    app.world_mut().resource_mut::<CaptureShrineOffering>().0 = true;
}

fn prepare_shop_gear_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::Village;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 16;
        pos.row = 9;
        pos.facing = U;
    }
    app.world_mut().resource_mut::<CaptureNpcReaction>().0 = true;
}

fn prepare_npc_reaction_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::Village;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 6;
        pos.row = 8;
        pos.facing = U;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        quest.interact_side_quest(SideQuest::VillageTrail);
    }
    app.world_mut().resource_mut::<CaptureNpcReaction>().0 = true;
}

fn prepare_npc_reaction_complete_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::Village;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 6;
        pos.row = 8;
        pos.facing = U;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        quest.interact_side_quest(SideQuest::VillageTrail);
        quest.record_side_victory();
        quest.record_side_victory();
        quest.interact_side_quest(SideQuest::VillageTrail);
    }
    app.world_mut().resource_mut::<CaptureNpcReaction>().0 = true;
}

fn prepare_npc_care_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::Bamboo;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 5;
        pos.row = 7;
        pos.facing = U;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
        quest.interact_bond_scene();
        quest.interact_camp_scene();
    }
    app.world_mut().resource_mut::<CaptureNpcReaction>().0 = true;
}

fn prepare_npc_route_branch_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::RiverTown;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 24;
        pos.row = 1;
        pos.facing = R;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        advance_to_opening_complete(&mut quest);
        quest.complete_route_detour(RouteDetour::ReedHiddenFord, RouteDetourApproach::Scout);
    }
    app.world_mut().resource_mut::<CaptureNpcReaction>().0 = true;
}

fn prepare_service_favor_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::Village;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 16;
        pos.row = 9;
        pos.facing = U;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        complete_side_quest(&mut quest, SideQuest::VillageTrail);
        complete_side_quest(&mut quest, SideQuest::VillageHerbs);
        quest.record_shop_gear(ShopGear::VillageSwordTassel);
    }
    app.world_mut().resource_mut::<CaptureNpcReaction>().0 = true;
}

fn prepare_river_town_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::RiverTown;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 1;
        pos.row = 1;
        pos.facing = R;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        advance_to_opening_complete(&mut quest);
    }
}

fn prepare_river_lantern_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::RiverReedBed;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 9;
        pos.row = 1;
        pos.facing = D;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        advance_to_river_lantern_puzzle(&mut quest);
    }
    app.world_mut().resource_mut::<CaptureRiverLantern>().0 = true;
}

fn prepare_moon_cave_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::Cave;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 12;
        pos.row = 4;
        pos.facing = U;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
        quest.talk(QuestRole::StarMage);
        quest.record_victory();
        quest.record_victory();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Merchant);
        quest.talk(QuestRole::BambooScout);
        quest.talk(QuestRole::CavePriestess);
    }
}

fn prepare_moon_corridor_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::MoonEchoCorridor;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 1;
        pos.row = 1;
        pos.facing = R;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
        quest.talk(QuestRole::StarMage);
        quest.record_victory();
        quest.record_victory();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Merchant);
        quest.talk(QuestRole::BambooScout);
        quest.talk(QuestRole::CavePriestess);
    }
}

fn prepare_moon_crystal_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::MoonEchoCorridor;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 12;
        pos.row = 4;
        pos.facing = U;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
        quest.talk(QuestRole::StarMage);
        quest.record_victory();
        quest.record_victory();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Merchant);
        quest.talk(QuestRole::BambooScout);
        quest.talk(QuestRole::CavePriestess);
        quest.record_victory();
        quest.record_victory();
        quest.record_victory();
    }
    app.world_mut().resource_mut::<CaptureMoonCrystal>().0 = true;
}

fn prepare_route_mark_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::MoonEchoCorridor;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 26;
        pos.row = 1;
        pos.facing = R;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
        quest.talk(QuestRole::StarMage);
        quest.record_victory();
        quest.record_victory();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Merchant);
        quest.talk(QuestRole::BambooScout);
        quest.talk(QuestRole::CavePriestess);
    }
    app.world_mut().resource_mut::<CaptureRouteMark>().0 = true;
}

fn prepare_route_detour_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::MoonEchoCorridor;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 19;
        pos.row = 6;
        pos.facing = L;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
        quest.talk(QuestRole::StarMage);
        quest.record_victory();
        quest.record_victory();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Merchant);
        quest.talk(QuestRole::BambooScout);
        quest.talk(QuestRole::CavePriestess);
    }
    app.world_mut().resource_mut::<CaptureRouteDetour>().0 = true;
}

fn prepare_battle_river_preset(app: &mut App) {
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
    }
    app.world_mut().insert_resource(PendingEncounter {
        zone: EncounterZone::RiverTown,
        kind: EncounterKind::Boss(BossKind::RiverDemon),
    });
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Battle);
}

fn prepare_battle_bond_preset(app: &mut App) {
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
        quest.interact_bond_scene();
        quest.record_bond_response(BondResponse::Courage);
        quest.talk(QuestRole::StarMage);
        quest.record_victory();
        quest.record_victory();
        quest.talk(QuestRole::SwordSister);
    }
    app.world_mut().insert_resource(PendingEncounter {
        zone: EncounterZone::RiverTown,
        kind: EncounterKind::Boss(BossKind::RiverDemon),
    });
    app.world_mut().resource_mut::<CaptureUseCombo>().0 = true;
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Battle);
}

fn prepare_battle_camp_preset(app: &mut App) {
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
        quest.interact_camp_scene();
    }
    app.world_mut().insert_resource(PendingEncounter {
        zone: EncounterZone::Village,
        kind: EncounterKind::Random,
    });
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Battle);
}

fn prepare_battle_blessing_preset(app: &mut App) {
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        quest.set_shrine_blessing(ShrineBlessing::Guard);
    }
    app.world_mut().insert_resource(PendingEncounter {
        zone: EncounterZone::Village,
        kind: EncounterKind::Random,
    });
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Battle);
}

fn prepare_bond_choice_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::Village;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 6;
        pos.row = 9;
        pos.facing = U;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
    }
    app.world_mut().resource_mut::<CaptureBondChoice>().0 = true;
}

fn prepare_companion_scene_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::Village;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 6;
        pos.row = 9;
        pos.facing = U;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
        quest.interact_bond_scene();
        quest.talk(QuestRole::StarMage);
        quest.record_victory();
        quest.record_victory();
        quest.talk(QuestRole::SwordSister);
    }
    app.world_mut().resource_mut::<CaptureBondChoice>().0 = true;
}

fn prepare_river_reed_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::RiverReedBed;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 1;
        pos.row = 1;
        pos.facing = R;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        advance_to_river_town_complete(&mut quest);
    }
}

fn prepare_plague_village_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::PlagueVillage;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 1;
        pos.row = 1;
        pos.facing = R;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        advance_to_river_town_complete(&mut quest);
    }
}

fn prepare_plague_shrine_path_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::PlagueShrinePath;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 1;
        pos.row = 1;
        pos.facing = R;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        advance_to_plague_ward_puzzle(&mut quest);
    }
}

fn prepare_plague_ward_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::PlagueShrinePath;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 12;
        pos.row = 6;
        pos.facing = U;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        advance_to_plague_ward_puzzle(&mut quest);
    }
    app.world_mut().resource_mut::<CapturePlagueWard>().0 = true;
}

fn prepare_battle_plague_preset(app: &mut App) {
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        advance_to_miasma_boss(&mut quest);
    }
    app.world_mut().insert_resource(PendingEncounter {
        zone: EncounterZone::PlagueVillage,
        kind: EncounterKind::Boss(BossKind::MiasmaRoot),
    });
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Battle);
}

fn prepare_capital_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::Capital;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 1;
        pos.row = 1;
        pos.facing = R;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        advance_to_plague_village_complete(&mut quest);
    }
}

fn prepare_capital_mansion_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::CapitalMansion;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 1;
        pos.row = 1;
        pos.facing = R;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        advance_to_plague_village_complete(&mut quest);
        quest.talk(QuestRole::CapitalEnvoy);
    }
}

fn prepare_mansion_gallery_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::MansionMirrorGallery;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 1;
        pos.row = 1;
        pos.facing = R;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        advance_to_mansion_mirror_puzzle(&mut quest);
    }
}

fn prepare_mansion_mirror_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::MansionMirrorGallery;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 12;
        pos.row = 7;
        pos.facing = U;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        advance_to_mansion_mirror_puzzle(&mut quest);
    }
    app.world_mut().resource_mut::<CaptureMansionMirror>().0 = true;
}

fn prepare_battle_capital_preset(app: &mut App) {
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        advance_to_mirror_boss(&mut quest);
    }
    app.world_mut().insert_resource(PendingEncounter {
        zone: EncounterZone::Capital,
        kind: EncounterKind::Boss(BossKind::MirrorMinister),
    });
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Battle);
}

fn prepare_southern_road_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::SouthernRoad;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 1;
        pos.row = 1;
        pos.facing = R;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        advance_to_capital_intrigue_complete(&mut quest);
    }
}

fn prepare_thunder_drum_path_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::ThunderDrumPath;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 1;
        pos.row = 1;
        pos.facing = R;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        advance_to_thunder_drum_puzzle(&mut quest);
    }
}

fn prepare_thunder_drum_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::ThunderDrumPath;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 12;
        pos.row = 6;
        pos.facing = U;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        advance_to_thunder_drum_puzzle(&mut quest);
    }
    app.world_mut().resource_mut::<CaptureThunderDrum>().0 = true;
}

fn prepare_battle_southern_preset(app: &mut App) {
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        advance_to_thunder_qilin_boss(&mut quest);
    }
    app.world_mut().insert_resource(PendingEncounter {
        zone: EncounterZone::SouthernRoad,
        kind: EncounterKind::Boss(BossKind::ThunderQilin),
    });
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Battle);
}

fn prepare_final_sanctum_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::FinalSanctum;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 1;
        pos.row = 1;
        pos.facing = R;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        advance_to_southern_road_complete(&mut quest);
        quest.talk(QuestRole::FinalOracle);
    }
}

fn prepare_final_task_board_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::FinalSanctum;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 2;
        pos.row = 4;
        pos.facing = R;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        advance_to_southern_road_complete(&mut quest);
        quest.talk(QuestRole::FinalOracle);
    }
    app.world_mut().resource_mut::<CaptureTaskBoard>().0 = true;
}

fn prepare_final_waterway_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::DreamWaterway;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 1;
        pos.row = 1;
        pos.facing = R;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        advance_to_southern_road_complete(&mut quest);
        quest.talk(QuestRole::FinalOracle);
    }
}

fn prepare_final_lamp_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::DreamWaterway;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 8;
        pos.row = 13;
        pos.facing = U;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        advance_to_southern_road_complete(&mut quest);
        quest.talk(QuestRole::FinalOracle);
    }
    app.world_mut().resource_mut::<CaptureFinalLamp>().0 = true;
}

fn prepare_final_epilogue_preset(app: &mut App) {
    app.world_mut().resource_mut::<CurrentMap>().0 = MapKind::FinalSanctum;
    {
        let mut pos = app.world_mut().resource_mut::<PlayerPos>();
        pos.col = 13;
        pos.row = 7;
        pos.facing = U;
    }
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        advance_to_final_boss(&mut quest);
        quest.record_boss_victory(BossKind::DreamEclipse);
    }
    app.world_mut().resource_mut::<CaptureFinalEpilogue>().0 = true;
}

fn prepare_battle_final_preset(app: &mut App) {
    {
        let mut quest = app.world_mut().resource_mut::<QuestLog>();
        advance_to_final_boss(&mut quest);
    }
    app.world_mut().insert_resource(PendingEncounter {
        zone: EncounterZone::FinalSanctum,
        kind: EncounterKind::Boss(BossKind::DreamEclipse),
    });
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Battle);
}

fn advance_to_opening_complete(quest: &mut QuestLog) {
    quest.talk(QuestRole::SwordSister);
    quest.talk(QuestRole::Linger);
    quest.talk(QuestRole::StarMage);
    quest.record_victory();
    quest.record_victory();
    quest.talk(QuestRole::SwordSister);
    quest.talk(QuestRole::Merchant);
    quest.talk(QuestRole::BambooScout);
    quest.talk(QuestRole::CavePriestess);
    quest.record_victory();
    quest.record_victory();
    quest.record_victory();
    quest.activate_moon_crystal(MoonCrystal::North);
    quest.activate_moon_crystal(MoonCrystal::South);
    quest.record_boss_victory(BossKind::MoonWraith);
    quest.talk(QuestRole::Linger);
}

fn complete_side_quest(quest: &mut QuestLog, side: SideQuest) {
    quest.interact_side_quest(side);
    for _ in 0..quest.side_quest_goal(side) {
        quest.record_side_victory();
    }
    quest.interact_side_quest(side);
}

fn advance_to_river_town_complete(quest: &mut QuestLog) {
    advance_to_river_lantern_puzzle(quest);
    quest.activate_river_lantern(RiverLantern::Upstream);
    quest.activate_river_lantern(RiverLantern::Midstream);
    quest.activate_river_lantern(RiverLantern::Dock);
    quest.record_boss_victory(BossKind::RiverDemon);
}

fn advance_to_river_lantern_puzzle(quest: &mut QuestLog) {
    advance_to_opening_complete(quest);
    quest.talk(QuestRole::HerbHealer);
    quest.record_victory();
    quest.record_victory();
    quest.talk(QuestRole::HerbHealer);
    quest.talk(QuestRole::RiverBoatman);
}

fn advance_to_miasma_boss(quest: &mut QuestLog) {
    advance_to_plague_ward_puzzle(quest);
    quest.seal_plague_ward(PlagueWard::OldShrine);
    quest.seal_plague_ward(PlagueWard::BitterWell);
    quest.seal_plague_ward(PlagueWard::Sickroom);
    quest.talk(QuestRole::ShrineKeeper);
}

fn advance_to_plague_ward_puzzle(quest: &mut QuestLog) {
    advance_to_river_town_complete(quest);
    quest.talk(QuestRole::PlagueElder);
    quest.talk(QuestRole::ShrineKeeper);
    quest.record_victory();
    quest.record_victory();
    quest.record_victory();
}

fn advance_to_plague_village_complete(quest: &mut QuestLog) {
    advance_to_miasma_boss(quest);
    quest.record_boss_victory(BossKind::MiasmaRoot);
}

fn advance_to_mirror_boss(quest: &mut QuestLog) {
    advance_to_mansion_mirror_puzzle(quest);
    quest.align_mansion_mirror(MansionMirrorNode::Ledger);
    quest.align_mansion_mirror(MansionMirrorNode::Witness);
    quest.talk(QuestRole::MansionSpy);
}

fn advance_to_mansion_mirror_puzzle(quest: &mut QuestLog) {
    advance_to_plague_village_complete(quest);
    quest.talk(QuestRole::CapitalEnvoy);
    quest.talk(QuestRole::MansionSpy);
    quest.record_victory();
    quest.record_victory();
}

fn advance_to_capital_intrigue_complete(quest: &mut QuestLog) {
    advance_to_mirror_boss(quest);
    quest.record_boss_victory(BossKind::MirrorMinister);
}

fn advance_to_thunder_qilin_boss(quest: &mut QuestLog) {
    advance_to_thunder_drum_puzzle(quest);
    quest.align_thunder_drum(ThunderDrum::Wind);
    quest.align_thunder_drum(ThunderDrum::Cloud);
    quest.align_thunder_drum(ThunderDrum::Oath);
    quest.talk(QuestRole::TribalChief);
}

fn advance_to_thunder_drum_puzzle(quest: &mut QuestLog) {
    advance_to_capital_intrigue_complete(quest);
    quest.talk(QuestRole::SpiritGuide);
    quest.talk(QuestRole::TribalChief);
    quest.record_victory();
    quest.record_victory();
    quest.record_victory();
}

fn advance_to_southern_road_complete(quest: &mut QuestLog) {
    advance_to_thunder_qilin_boss(quest);
    quest.record_boss_victory(BossKind::ThunderQilin);
}

fn advance_to_final_boss(quest: &mut QuestLog) {
    advance_to_southern_road_complete(quest);
    quest.talk(QuestRole::FinalOracle);
    quest.light_final_lamp(FinalLamp::Memory);
    quest.light_final_lamp(FinalLamp::Vow);
    quest.light_final_lamp(FinalLamp::Fate);
    quest.talk(QuestRole::FinalOracle);
}

fn clear_intent(app: &mut App) {
    app.world_mut().resource_mut::<Intent>().clear();
}

/// Write the scripted `Intent` for gameplay frame `f`.
fn set_intent(app: &mut App, f: u32) {
    app.world_mut().resource_mut::<EncounterRate>().0 = if f < 620 { 0.0 } else { 1.0 };

    if app.world().resource::<CaptureRunAudit>().0
        && app
            .world()
            .get_resource::<love_rpg::game::roguelike::RunState>()
            .is_some_and(|run| run.outcome.is_some())
    {
        clear_intent(app);
        return;
    }

    if app.world().resource::<CaptureRouteMap>().0 || app.world().resource::<CaptureStoryProof>().0
    {
        clear_intent(app);
        return;
    }

    let battle = *app.world().resource::<State<AppState>>().get() == AppState::Battle;
    let battle_frame = {
        let mut driver = app.world_mut().resource_mut::<CaptureBattleDriver>();
        if battle {
            if driver.in_battle {
                driver.frame += 1;
            } else {
                driver.in_battle = true;
                driver.frame = 0;
            }
            Some(driver.frame)
        } else {
            driver.in_battle = false;
            driver.frame = 0;
            None
        }
    };

    // Roguelike run: cadence-driven confirms walk the whole loop (title →
    // chapter card → node picks → battles → rewards → events), with periodic
    // cursor-down presses to vary node/option choices.
    if app.world().resource::<CaptureRogue>().0 {
        let ui_state = *app.world().resource::<State<AppState>>().get();

        if app.world().resource::<CaptureRunAudit>().0 && ui_state == AppState::Battle {
            use love_rpg::game::battle::BattleAutomationView;
            let (menu_open, current, heavy, spell_cost, momentum) = {
                let view = app.world().resource::<BattleAutomationView>();
                (
                    view.menu_open,
                    view.menu_index,
                    view.enemy_heavy_intent,
                    view.spell_cost,
                    view.momentum,
                )
            };
            let (hp, max_hp, mp, potions) = {
                let stats = app.world().resource::<love_rpg::game::PlayerStats>();
                (stats.hp, stats.max_hp, stats.mp, stats.potions)
            };
            let desired = if potions > 0 && hp * 100 <= max_hp * 55 {
                4
            } else if heavy && hp * 100 <= max_hp * 85 {
                1
            } else if momentum >= 3 {
                3
            } else if mp >= spell_cost {
                2
            } else {
                0
            };
            let mut intent = app.world_mut().resource_mut::<Intent>();
            intent.clear();
            if menu_open {
                if current == desired {
                    intent.confirm = true;
                } else {
                    let down = (desired + 6 - current) % 6;
                    let up = (current + 6 - desired) % 6;
                    if down <= up {
                        intent.down = true;
                    } else {
                        intent.up = true;
                    }
                }
            }
            return;
        }

        // Walkable node scene: steer the hero along the BFS flow field
        // toward the objective; confirms advance any overlay that opens.
        if ui_state == AppState::RunScene && battle_frame.is_none() {
            if app.world().resource::<CaptureInventory>().0 {
                // Walk a little, then open the Esc inventory and hold it.
                let mut intent = app.world_mut().resource_mut::<Intent>();
                intent.clear();
                if f < 40 {
                    intent.move_dir = Some(R);
                } else if f == 46 {
                    intent.cancel = true;
                }
                return;
            }
            {
                use love_rpg::game::roguelike::event::{DialogueSource, RunDialogue};
                let route_tracking_choice =
                    app.world()
                        .get_resource::<RunDialogue>()
                        .is_some_and(|dialogue| {
                            dialogue.active
                                && matches!(
                                    dialogue.source,
                                    DialogueSource::RouteTask
                                        | DialogueSource::RouteCommission { .. }
                                )
                                && !dialogue.resolved
                                && !dialogue.options.is_empty()
                                && dialogue.idx + 1 >= dialogue.lines.len()
                        });
                if route_tracking_choice {
                    let mut intent = app.world_mut().resource_mut::<Intent>();
                    intent.clear();
                    intent.confirm = true;
                    return;
                }
            }
            use love_rpg::game::core::{MAP_H, MAP_W};
            use love_rpg::game::roguelike::scene::RunSceneState;
            let step = app
                .world()
                .get_resource::<RunSceneState>()
                .and_then(|scene| {
                    if scene.flow.is_empty() {
                        return None;
                    }
                    let mut best: Option<(IVec2, u16)> = None;
                    for (dc, dr) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                        let (nc, nr) = (scene.col + dc, scene.row + dr);
                        if nc < 0 || nr < 0 || nc >= MAP_W || nr >= MAP_H {
                            continue;
                        }
                        let d = scene.flow[nr as usize][nc as usize];
                        if d == u16::MAX {
                            continue;
                        }
                        if best.is_none_or(|(_, bd)| d < bd) {
                            best = Some((IVec2::new(dc, dr), d));
                        }
                    }
                    best.map(|(dir, _)| dir)
                });
            let mut intent = app.world_mut().resource_mut::<Intent>();
            intent.clear();
            intent.move_dir = step;
            if f % 18 == 12 {
                intent.confirm = true;
            }
            // Walk option lists downward over time so multi-option overlays
            // (e.g. the market's "leave" entry) always terminate.
            if f % 36 == 0 {
                intent.down = true;
            }
            return;
        }

        let in_reward = ui_state == AppState::Reward;
        let mut intent = app.world_mut().resource_mut::<Intent>();
        intent.clear();
        if let Some(battle_frame) = battle_frame {
            if battle_frame % 14 == 4 {
                intent.confirm = true;
            }
        } else if in_reward {
            // Let the three-choice loot screen breathe: browse, then take.
            if f % 44 == 20 {
                intent.down = true;
            }
            if f % 44 == 42 {
                intent.confirm = true;
            }
        } else if f >= 30 {
            // Hold the title screen for a second, then walk the run.
            if f % 18 == 12 {
                intent.confirm = true;
            }
            if f % 90 == 48 {
                intent.down = true;
            }
        }
        return;
    }

    let use_combo = app.world().resource::<CaptureUseCombo>().0;
    let task_board = app.world().resource::<CaptureTaskBoard>().0;
    let task_contact = app.world().resource::<CaptureTaskContact>().0;
    let commission_trace = app.world().resource::<CaptureCommissionTrace>().0;
    let chapter_card = app.world().resource::<CaptureChapterCard>().0;
    let treasure_cache = app.world().resource::<CaptureTreasureCache>().0;
    let shrine_offering = app.world().resource::<CaptureShrineOffering>().0;
    let npc_reaction = app.world().resource::<CaptureNpcReaction>().0;
    let bond_choice = app.world().resource::<CaptureBondChoice>().0;
    let camp_rest = app.world().resource::<CaptureCampRest>().0;
    let river_lantern = app.world().resource::<CaptureRiverLantern>().0;
    let moon_crystal = app.world().resource::<CaptureMoonCrystal>().0;
    let route_mark = app.world().resource::<CaptureRouteMark>().0;
    let route_detour = app.world().resource::<CaptureRouteDetour>().0;
    let plague_ward = app.world().resource::<CapturePlagueWard>().0;
    let mansion_mirror = app.world().resource::<CaptureMansionMirror>().0;
    let thunder_drum = app.world().resource::<CaptureThunderDrum>().0;
    let final_epilogue = app.world().resource::<CaptureFinalEpilogue>().0;
    let final_lamp = app.world().resource::<CaptureFinalLamp>().0;

    let mut intent = app.world_mut().resource_mut::<Intent>();
    intent.clear();
    if let Some(battle_frame) = battle_frame {
        // Cover skill commands deterministically: most battle captures cast 仙术,
        // while the bond capture moves down to 合击 first.
        if (use_combo && matches!(battle_frame, 6 | 8 | 10))
            || (!use_combo && matches!(battle_frame, 6 | 8))
        {
            intent.down = true;
        } else if battle_frame == 12 || (battle_frame > 74 && battle_frame % 28 == 0) {
            intent.confirm = true;
        }
        return;
    }

    if task_board {
        if (2..=1000).contains(&f) && (f - 2) % 26 == 0 {
            intent.confirm = true;
        }
        return;
    }

    if task_contact {
        if (2..=366).contains(&f) && (f - 2) % 26 == 0 {
            intent.confirm = true;
        }
        return;
    }

    if commission_trace {
        if matches!(f, 18 | 44 | 70 | 96 | 122) {
            intent.confirm = true;
        }
        return;
    }

    if chapter_card {
        if (2..=470).contains(&f) && (f - 2) % 26 == 0 {
            intent.confirm = true;
        }
        return;
    }

    if treasure_cache {
        if matches!(f, 2 | 28 | 54 | 80) {
            intent.confirm = true;
        }
        return;
    }

    if shrine_offering {
        if matches!(f, 2 | 28 | 54 | 80) {
            intent.confirm = true;
        }
        return;
    }

    if npc_reaction {
        if (2..=184).contains(&f) && (f - 2) % 26 == 0 {
            intent.confirm = true;
        }
        return;
    }

    if bond_choice {
        if matches!(f, 2 | 28 | 54 | 80 | 106 | 132 | 158 | 184 | 210) {
            intent.confirm = true;
        }
        return;
    }

    if camp_rest {
        if matches!(
            f,
            2 | 28 | 54 | 80 | 106 | 132 | 158 | 184 | 210 | 236 | 262 | 288
        ) {
            intent.confirm = true;
        }
        return;
    }

    if river_lantern {
        if matches!(f, 2 | 28 | 54 | 80 | 106) {
            intent.confirm = true;
        }
        return;
    }

    if moon_crystal {
        if matches!(f, 2 | 28 | 54 | 80 | 106) {
            intent.confirm = true;
        }
        return;
    }

    if route_mark {
        if matches!(f, 2 | 28 | 54 | 80 | 106) {
            intent.confirm = true;
        }
        return;
    }

    if route_detour {
        if matches!(f, 2 | 28 | 54 | 80 | 106 | 132 | 158 | 184 | 210) {
            intent.confirm = true;
        }
        return;
    }

    if plague_ward {
        if matches!(f, 2 | 28 | 54 | 80 | 106) {
            intent.confirm = true;
        }
        return;
    }

    if mansion_mirror {
        if matches!(f, 2 | 28 | 54 | 80 | 106) {
            intent.confirm = true;
        }
        return;
    }

    if thunder_drum {
        if matches!(f, 2 | 28 | 54 | 80 | 106) {
            intent.confirm = true;
        }
        return;
    }

    if final_lamp {
        if matches!(f, 2 | 28 | 54 | 80 | 106) {
            intent.confirm = true;
        }
        return;
    }

    if final_epilogue {
        if matches!(f, 2 | 28 | 54 | 80 | 106 | 132 | 158 | 184) {
            intent.confirm = true;
        }
        return;
    }

    // Grid movement is ~4 frames per tile (0.14s cooldown @ 30 fps). The route:
    // start (3,7) -> face red sword sister at (22,1) -> Linger (20,3)
    // for quest progress -> grass encounter proof.
    if f <= 42 {
        intent.move_dir = Some(D);
    } else if f <= 155 {
        intent.move_dir = Some(R);
    } else if f <= 221 {
        intent.move_dir = Some(U);
    } else if f <= 238 {
        intent.move_dir = Some(L);
    } else if f <= 380 {
        if matches!(f, 246 | 270 | 294 | 318 | 342 | 366) {
            intent.confirm = true;
        }
    } else if f <= 395 {
        intent.move_dir = Some(D);
    } else if f <= 412 {
        intent.move_dir = Some(L);
    } else if f <= 430 {
        intent.move_dir = Some(D); // face Linger
    } else if f <= 620 {
        if matches!(f, 442 | 466 | 490 | 514 | 538 | 562 | 586 | 610) {
            intent.confirm = true;
        }
    } else if f <= 645 {
        intent.move_dir = Some(L);
    } else if f <= 720 {
        intent.move_dir = Some(D);
    } else {
        // Post-battle wander so the back of the clip keeps moving.
        match (f / 20) % 4 {
            0 => intent.move_dir = Some(U),
            1 => intent.move_dir = Some(R),
            2 => intent.move_dir = Some(D),
            _ => intent.move_dir = Some(L),
        }
    }
}

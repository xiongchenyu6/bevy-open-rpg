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
//!   river-town    start directly on the Act 3 river-town map
//!   moon-cave     start directly in the Act 2 moon cave trial
//!   moon-corridor start directly on the Act 2 moon-echo corridor route map
//!   moon-crystal  start at the Act 2 moon-crystal puzzle
//!   task-board-next start by a board after its first local commission is done
//!   task-board-turn-in start by a completed task-board errand ready to claim
//!   task-tracker  start with an accepted commission in the quest tracker
//!   service-favor start by a discounted service after local commissions
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
//!   battle-blessing start directly in a battle with shrine blessing active
//!   bond-choice start at a companion lantern response choice
//!   npc-reaction start by a villager reacting to an accepted task-board errand
//!   npc-reaction-complete start by a villager after one board errand is completed
//!   final-waterway start directly on the Finale old-dream waterway route map
//!   final-lamp start at the Finale memory-lamp puzzle
//!   final-epilogue start after the final boss by the gatekeeper ending scene
//!   rogue         roguelike run mode: title → node map → battles/events/rewards

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
use std::time::Duration;

use love_rpg::{
    AppState, EncounterRate, GamePlugin, Intent,
    game::{
        battle::{EncounterKind, EncounterZone, PendingEncounter},
        explore::{CurrentMap, MapKind, PlayerPos},
        lighting,
        quest::{
            BondResponse, BossKind, FinalLamp, MansionMirrorNode, MoonCrystal, PlagueWard,
            QuestLog, QuestRole, RiverLantern, ShrineBlessing, SideQuest, ThunderDrum,
            TreasureCache,
        },
    },
};

const R: IVec2 = IVec2::new(1, 0);
const L: IVec2 = IVec2::new(-1, 0);
const U: IVec2 = IVec2::new(0, -1);
const D: IVec2 = IVec2::new(0, 1);

const SETTLE: u32 = 24;

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
    app.init_resource::<CaptureChapterCard>();
    app.init_resource::<CaptureTreasureCache>();
    app.init_resource::<CaptureShrineOffering>();
    app.init_resource::<CaptureNpcReaction>();
    app.init_resource::<CaptureBondChoice>();
    app.init_resource::<CaptureCampRest>();
    app.init_resource::<CaptureRiverLantern>();
    app.init_resource::<CaptureMoonCrystal>();
    app.init_resource::<CapturePlagueWard>();
    app.init_resource::<CaptureMansionMirror>();
    app.init_resource::<CaptureThunderDrum>();
    app.init_resource::<CaptureFinalEpilogue>();
    app.init_resource::<CaptureFinalLamp>();
    app.init_resource::<CaptureRogue>();
    match preset {
        "rogue" => {
            app.world_mut().resource_mut::<CaptureRogue>().0 = true;
            // Deterministic-ish start (the title screen still mixes in wall
            // clock; the script is cadence-based, not frame-exact).
            app.world_mut().resource_mut::<love_rpg::game::Rng>().0 = 0xC0FFEE_5EED;
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
            app.world_mut().insert_resource(run);
            // A hand-built Village stage with a single market marker; the
            // scene spawner fills in hero position and the flow field is
            // recomputed by the driver as it steers.
            let mut scene = RunSceneState {
                map: MapKind::Village,
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
                cooldown: 0.0,
            };
            love_rpg::game::roguelike::scene::seed_flow(&mut scene);
            app.world_mut().insert_resource(scene);
            app.world_mut()
                .resource_mut::<NextState<AppState>>()
                .set(AppState::RunScene);
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
        "chapter-card" => prepare_chapter_card_preset(&mut app),
        "task-board" => prepare_task_board_preset(&mut app),
        "task-board-next" => prepare_task_board_next_preset(&mut app),
        "task-board-turn-in" => prepare_task_board_turn_in_preset(&mut app),
        "task-tracker" => prepare_task_tracker_preset(&mut app),
        "camp-rest" => prepare_camp_rest_preset(&mut app),
        "treasure-cache" => prepare_treasure_cache_preset(&mut app),
        "shrine-offering" => prepare_shrine_offering_preset(&mut app),
        "npc-reaction" => prepare_npc_reaction_preset(&mut app),
        "npc-reaction-complete" => prepare_npc_reaction_complete_preset(&mut app),
        "npc-care" => prepare_npc_care_preset(&mut app),
        "service-favor" => prepare_service_favor_preset(&mut app),
        "river-town" => prepare_river_town_preset(&mut app),
        "river-lantern" => prepare_river_lantern_preset(&mut app),
        "moon-cave" => prepare_moon_cave_preset(&mut app),
        "moon-corridor" => prepare_moon_corridor_preset(&mut app),
        "moon-crystal" => prepare_moon_crystal_preset(&mut app),
        "battle-river" => prepare_battle_river_preset(&mut app),
        "battle-bond" => prepare_battle_bond_preset(&mut app),
        "battle-camp" => prepare_battle_camp_preset(&mut app),
        "battle-blessing" => prepare_battle_blessing_preset(&mut app),
        "bond-choice" => prepare_bond_choice_preset(&mut app),
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
        "final-waterway" => prepare_final_waterway_preset(&mut app),
        "final-lamp" => prepare_final_lamp_preset(&mut app),
        "final-epilogue" => prepare_final_epilogue_preset(&mut app),
        "battle-final" => prepare_battle_final_preset(&mut app),
        _ => {}
    }
    // Legacy presets predate the roguelike Title state and expect to start on
    // the free-roam overworld.
    if !preset.starts_with("rogue") {
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

        // Walkable node scene: steer the hero along the BFS flow field
        // toward the objective; confirms advance any overlay that opens.
        if ui_state == AppState::RunScene && battle_frame.is_none() {
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
    let chapter_card = app.world().resource::<CaptureChapterCard>().0;
    let treasure_cache = app.world().resource::<CaptureTreasureCache>().0;
    let shrine_offering = app.world().resource::<CaptureShrineOffering>().0;
    let npc_reaction = app.world().resource::<CaptureNpcReaction>().0;
    let bond_choice = app.world().resource::<CaptureBondChoice>().0;
    let camp_rest = app.world().resource::<CaptureCampRest>().0;
    let river_lantern = app.world().resource::<CaptureRiverLantern>().0;
    let moon_crystal = app.world().resource::<CaptureMoonCrystal>().0;
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
        if (use_combo && matches!(battle_frame, 6 | 8)) || (!use_combo && battle_frame == 6) {
            intent.down = true;
        } else if battle_frame == 12 || (battle_frame > 74 && battle_frame % 28 == 0) {
            intent.confirm = true;
        }
        return;
    }

    if task_board {
        if (2..=366).contains(&f) && (f - 2) % 26 == 0 {
            intent.confirm = true;
        }
        return;
    }

    if chapter_card {
        if matches!(f, 2 | 28 | 54 | 80 | 106 | 132 | 158 | 184 | 210) {
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
        if matches!(f, 2 | 28 | 54 | 80 | 106 | 132 | 158) {
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

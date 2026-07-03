//! Walkable node scenes (实景节点).
//!
//! Picking a node on the chapter map no longer resolves it abstractly:
//! the player is dropped onto a real tile map (reusing the legacy Explore
//! chapter maps + art) and walks to a marked objective. Reaching it triggers
//! the node's payload — a battle, an event/story/rest/market overlay, or the
//! chapter boss — after which control returns to the node map.

use bevy::prelude::*;

use super::super::animation::{self, AnimationAssets, AnimationClip};
use super::super::battle::{EncounterZone, PendingEncounter};
use super::super::core::{GameFont, Intent, MAP_H, MAP_W, Rng, TILE, tile_to_world};
use super::super::explore::{ExploreAssets, MapData, MapKind, tile_sprite};
use super::super::lighting::{self, LightingAssets};
use super::super::paperdoll::{self, PaperdollAssets, PaperdollStyle};
use super::event::{self, RunDialogue};
use super::graph::NodeKind;
use super::{FightRank, RunState, battle_mods_for, encounter_kind_for};
use crate::game::state::AppState;

// ---------------------------------------------------------------------------
// Resources / components
// ---------------------------------------------------------------------------

/// Set up by the node map before entering `AppState::RunScene`, completed at
/// scene spawn. Public fields let the capture driver steer the hero.
#[derive(Resource)]
pub struct RunSceneState {
    pub kind: NodeKind,
    pub map: MapKind,
    pub col: i32,
    pub row: i32,
    pub facing_left: bool,
    pub objective: (i32, i32),
    /// BFS distance-to-objective per tile (u16::MAX = unreachable).
    pub flow: Vec<Vec<u16>>,
    /// The node payload has been triggered (battle entered / overlay opened).
    pub resolved: bool,
    pub cooldown: f32,
}

#[derive(Resource)]
pub struct RunSceneMap(pub MapData);

#[derive(Component)]
pub struct SceneHero;

#[derive(Component)]
pub struct SceneMarkerGlyph {
    base_y: f32,
}

fn map_zone(map: MapKind) -> EncounterZone {
    match map {
        MapKind::Village => EncounterZone::Village,
        MapKind::Bamboo => EncounterZone::Bamboo,
        MapKind::Cave | MapKind::MoonEchoCorridor => EncounterZone::Cave,
        MapKind::RiverTown | MapKind::RiverReedBed => EncounterZone::RiverTown,
        MapKind::PlagueVillage | MapKind::PlagueShrinePath => EncounterZone::PlagueVillage,
        MapKind::Capital | MapKind::CapitalMansion | MapKind::MansionMirrorGallery => {
            EncounterZone::Capital
        }
        MapKind::SouthernRoad | MapKind::ThunderDrumPath => EncounterZone::SouthernRoad,
        MapKind::FinalSanctum | MapKind::DreamWaterway => EncounterZone::FinalSanctum,
    }
}

/// BFS over walkable tiles from `from`; returns per-tile step distance.
fn distance_field(map: &MapData, from: (i32, i32)) -> Vec<Vec<u16>> {
    let mut dist = vec![vec![u16::MAX; MAP_W as usize]; MAP_H as usize];
    let mut queue = std::collections::VecDeque::new();
    dist[from.1 as usize][from.0 as usize] = 0;
    queue.push_back(from);
    while let Some((c, r)) = queue.pop_front() {
        let d = dist[r as usize][c as usize];
        for (dc, dr) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nc, nr) = (c + dc, r + dr);
            if nc < 0 || nr < 0 || nc >= MAP_W || nr >= MAP_H {
                continue;
            }
            if !map.at(nc, nr).walkable() {
                continue;
            }
            if dist[nr as usize][nc as usize] == u16::MAX {
                dist[nr as usize][nc as usize] = d + 1;
                queue.push_back((nc, nr));
            }
        }
    }
    dist
}

// ---------------------------------------------------------------------------
// Scene setup
// ---------------------------------------------------------------------------

pub(crate) fn spawn_run_scene(
    mut commands: Commands,
    font: Res<GameFont>,
    assets: Res<ExploreAssets>,
    anims: Res<AnimationAssets>,
    dolls: Res<PaperdollAssets>,
    lights: Res<LightingAssets>,
    asset_server: Res<AssetServer>,
    scene: Option<ResMut<RunSceneState>>,
    mut next: ResMut<NextState<AppState>>,
) {
    let Some(mut scene) = scene else {
        next.set(AppState::Title);
        return;
    };
    let scope = || DespawnOnExit(AppState::RunScene);
    let map = MapData::build(scene.map);

    // Tiles.
    for row in 0..MAP_H {
        for col in 0..MAP_W {
            let p = tile_to_world(col, row);
            commands.spawn((
                tile_sprite(map.at(col, row), scene.map, &assets),
                Transform::from_xyz(p.x, p.y, 0.0),
                scope(),
            ));
        }
    }

    // Soft fill lights so the whole map reads clearly (the ambient level is
    // tuned for the light-dense legacy explore scenes).
    for (fx, fy) in [(0.22, 0.28), (0.78, 0.28), (0.22, 0.74), (0.78, 0.74)] {
        let p = tile_to_world((MAP_W as f32 * fx) as i32, (MAP_H as f32 * fy) as i32);
        lighting::spawn_light(
            &mut commands,
            &lights,
            Vec3::new(p.x, p.y, 3.0),
            TILE * 11.0,
            Color::srgba(0.80, 0.86, 1.0, 0.10),
            AppState::RunScene,
        );
    }

    // Hero at the map's spawn point.
    let (col, row) = map.spawn();
    scene.col = col;
    scene.row = row;
    scene.cooldown = 0.0;
    scene.resolved = false;
    let p = tile_to_world(col, row);
    let hero = animation::spawn_animated_sprite(
        &mut commands,
        &anims,
        AnimationClip::HeroIdle,
        Vec3::new(p.x, p.y, 10.0),
        Vec2::splat(82.0),
        AppState::RunScene,
    );
    commands.entity(hero).insert(SceneHero);
    lighting::spawn_light(
        &mut commands,
        &lights,
        Vec3::new(p.x, p.y, 4.0),
        TILE * 4.1,
        Color::srgba(1.0, 0.86, 0.48, 0.36),
        AppState::RunScene,
    );

    // Objective = the reachable walkable tile farthest from the spawn.
    let from_spawn = distance_field(&map, (col, row));
    let mut best = (col, row, 0u16);
    for r in 0..MAP_H {
        for c in 0..MAP_W {
            let d = from_spawn[r as usize][c as usize];
            if d != u16::MAX && d > best.2 {
                best = (c, r, d);
            }
        }
    }
    scene.objective = (best.0, best.1);
    scene.flow = distance_field(&map, scene.objective);

    // Objective marker: art + light per node kind.
    let op = tile_to_world(best.0, best.1);
    let (glyph, light_color) = match scene.kind {
        NodeKind::Fight => ("战", Color::srgba(1.0, 0.42, 0.30, 0.42)),
        NodeKind::Elite => ("袭", Color::srgba(1.0, 0.25, 0.55, 0.46)),
        NodeKind::Event => ("遇", Color::srgba(0.40, 0.72, 1.0, 0.40)),
        NodeKind::Story => ("缘", Color::srgba(1.0, 0.78, 0.40, 0.42)),
        NodeKind::Rest => ("歇", Color::srgba(1.0, 0.72, 0.36, 0.40)),
        NodeKind::Market => ("市", Color::srgba(1.0, 0.86, 0.40, 0.40)),
        NodeKind::Boss => ("魔", Color::srgba(0.80, 0.40, 1.0, 0.52)),
    };
    match scene.kind {
        NodeKind::Story => {
            paperdoll::spawn_paperdoll(
                &mut commands,
                &dolls,
                PaperdollStyle::Linger,
                Vec3::new(op.x, op.y + 6.0, 9.0),
                58.0,
                AppState::RunScene,
            );
        }
        NodeKind::Boss => {
            commands.spawn((
                Sprite {
                    image: asset_server.load("props/ai_bamboo_gate.png"),
                    custom_size: Some(Vec2::splat(96.0)),
                    ..default()
                },
                Transform::from_xyz(op.x, op.y + 10.0, 9.0),
                scope(),
            ));
        }
        NodeKind::Market => {
            commands.spawn((
                Sprite {
                    image: asset_server.load("props/ai_quest_board.png"),
                    custom_size: Some(Vec2::splat(72.0)),
                    ..default()
                },
                Transform::from_xyz(op.x, op.y + 8.0, 9.0),
                scope(),
            ));
        }
        NodeKind::Rest | NodeKind::Event => {
            commands.spawn((
                Sprite {
                    image: asset_server.load("props/ai_spirit_lantern.png"),
                    custom_size: Some(Vec2::splat(60.0)),
                    ..default()
                },
                Transform::from_xyz(op.x, op.y + 6.0, 9.0),
                scope(),
            ));
        }
        NodeKind::Fight | NodeKind::Elite => {
            commands.spawn((
                Sprite {
                    image: lights.orb.clone(),
                    color: light_color.with_alpha(0.9),
                    custom_size: Some(Vec2::splat(64.0)),
                    ..default()
                },
                Transform::from_xyz(op.x, op.y + 4.0, 9.0),
                scope(),
            ));
        }
    }
    commands.spawn((
        SceneMarkerGlyph {
            base_y: op.y + 46.0,
        },
        Text2d::new(glyph),
        font.text_font(24.0),
        TextColor(Color::srgb(0.98, 0.92, 0.75)),
        Transform::from_xyz(op.x, op.y + 46.0, 11.0),
        scope(),
    ));
    lighting::spawn_light(
        &mut commands,
        &lights,
        Vec3::new(op.x, op.y, 4.0),
        TILE * 3.4,
        light_color,
        AppState::RunScene,
    );

    // HUD & hint (shared marker with the node-map HUD updater).
    commands.spawn((
        super::map_ui::RunHudText,
        Text::new(""),
        font.text_font(18.0),
        TextColor(Color::srgb(0.9, 0.92, 0.95)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(14.0),
            max_width: Val::Px(430.0),
            ..default()
        },
        scope(),
    ));
    commands.spawn((
        Text::new(format!(
            "{} · {} —— 方向键 移动,走到「{}」处",
            map.name(),
            scene.kind.label(),
            glyph
        )),
        font.text_font(17.0),
        TextColor(Color::srgba(0.9, 0.92, 0.95, 0.8)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            right: Val::Px(14.0),
            ..default()
        },
        scope(),
    ));

    // Dialogue overlay (event/story/rest/market resolve in place).
    event::spawn_run_dialogue_ui(&mut commands, &font, scope());

    commands.insert_resource(RunSceneMap(map));
}

// ---------------------------------------------------------------------------
// Movement + payload dispatch
// ---------------------------------------------------------------------------

pub fn run_scene_movement(
    mut commands: Commands,
    time: Res<Time>,
    mut intent: ResMut<Intent>,
    scene: Option<ResMut<RunSceneState>>,
    map: Option<Res<RunSceneMap>>,
    run: Option<ResMut<RunState>>,
    mut rng: ResMut<Rng>,
    mut dialogue: ResMut<RunDialogue>,
    mut next: ResMut<NextState<AppState>>,
    mut hero: Query<(&mut Transform, &mut Sprite), With<SceneHero>>,
) {
    let (Some(mut scene), Some(map), Some(mut run)) = (scene, map, run) else {
        return;
    };
    if dialogue.active || scene.resolved {
        return;
    }

    scene.cooldown -= time.delta_secs();
    if let Some(dir) = intent.move_dir {
        if scene.cooldown <= 0.0 {
            let (nc, nr) = (scene.col + dir.x, scene.row + dir.y);
            if map.0.at(nc, nr).walkable() {
                scene.col = nc;
                scene.row = nr;
                scene.cooldown = 0.14;
                if dir.x != 0 {
                    scene.facing_left = dir.x < 0;
                }
                if let Ok((mut transform, mut sprite)) = hero.single_mut() {
                    let p = tile_to_world(nc, nr);
                    transform.translation.x = p.x;
                    transform.translation.y = p.y;
                    sprite.flip_x = scene.facing_left;
                }
            } else {
                scene.cooldown = 0.05;
            }
        }
    }
    intent.clear();

    // Arrived at the objective: fire the node payload.
    if (scene.col, scene.row) != scene.objective {
        return;
    }
    scene.resolved = true;
    match scene.kind {
        NodeKind::Fight | NodeKind::Elite | NodeKind::Boss => {
            let rank = match scene.kind {
                NodeKind::Elite => FightRank::Elite,
                NodeKind::Boss => FightRank::Boss,
                _ => FightRank::Normal,
            };
            run.current_fight = Some(rank);
            commands.insert_resource(PendingEncounter {
                zone: map_zone(scene.map),
                kind: encounter_kind_for(&run, scene.kind),
            });
            commands.insert_resource(battle_mods_for(&run, rank));
            next.set(AppState::Battle);
        }
        NodeKind::Event => {
            let index = run.draw_event(&mut rng);
            dialogue.open_event(index);
        }
        NodeKind::Story => {
            let roll = rng.range(0, super::content::story_count(run.chapter) as i32 - 1) as usize;
            dialogue.open_story(run.chapter, roll);
        }
        NodeKind::Rest => dialogue.open_rest(),
        NodeKind::Market => dialogue.open_market(),
    }
}

/// Once an overlay payload has been resolved and closed, return to the map.
pub fn run_scene_finish(
    scene: Option<Res<RunSceneState>>,
    dialogue: Res<RunDialogue>,
    mut next: ResMut<NextState<AppState>>,
) {
    let Some(scene) = scene else { return };
    if !scene.resolved || dialogue.active {
        return;
    }
    if matches!(
        scene.kind,
        NodeKind::Event | NodeKind::Story | NodeKind::Rest | NodeKind::Market
    ) {
        next.set(AppState::NodeMap);
    }
}

/// Gentle bob on the objective glyph so it reads as interactive.
pub fn animate_marker_glyph(
    time: Res<Time>,
    mut glyphs: Query<(&SceneMarkerGlyph, &mut Transform)>,
) {
    for (glyph, mut transform) in &mut glyphs {
        transform.translation.y = glyph.base_y + (time.elapsed_secs() * 2.4).sin() * 4.0;
    }
}

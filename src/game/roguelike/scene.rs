//! Map stages (地图关卡) — the roguelike run IS the map.
//!
//! Each chapter is a chain of real walkable tile maps (reusing the legacy
//! Explore chapter maps + art). Every stage scatters 2–3 objective markers
//! (battles / events / story / rest / market); once all are cleared the
//! glowing portal tile leads to the next stage. The chapter's final stage is
//! the boss map: a single demon gate. Battles hop out to `AppState::Battle`
//! and return here through the Reward screen.

use bevy::prelude::*;

use super::super::animation::{self, AnimationAssets, AnimationClip};
use super::super::battle::{EncounterZone, PendingEncounter};
use super::super::core::{GameFont, Intent, MAP_H, MAP_W, PlayerStats, Rng, TILE, tile_to_world};
use super::super::explore::{ExploreAssets, MapData, MapKind, Tile, tile_sprite};
use super::super::lighting::{self, LightingAssets};
use super::super::paperdoll::PaperdollAssets;
use super::event::{self, RunDialogue};
use super::graph::NodeKind;
use super::{FightRank, RunState, battle_mods_for, encounter_kind_for};
use crate::game::state::AppState;

// ---------------------------------------------------------------------------
// Resources / components
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct SceneMarker {
    pub kind: NodeKind,
    pub col: i32,
    pub row: i32,
    pub cleared: bool,
}

/// The current map stage. Persists across battles (the scene rebuilds from it
/// on re-entry); replaced by `advance_stage` when moving to the next map.
/// Public fields let the capture driver steer the hero along `flow`.
#[derive(Resource)]
pub struct RunSceneState {
    pub map: MapKind,
    /// Hero grid position; col < 0 means "use the map's spawn point".
    pub col: i32,
    pub row: i32,
    pub facing_left: bool,
    pub markers: Vec<SceneMarker>,
    /// Portal tiles of this map (exit to the next stage).
    pub portals: Vec<(i32, i32)>,
    /// Multi-source BFS distance to the nearest active target (uncleared
    /// marker, or the portal once all are cleared). u16::MAX = unreachable.
    pub flow: Vec<Vec<u16>>,
    pub cooldown: f32,
}

#[derive(Resource)]
pub struct RunSceneMap(pub MapData);

#[derive(Component)]
pub struct SceneHero;

#[derive(Component)]
pub struct SceneMarkerVisual(pub usize);

#[derive(Component)]
pub struct SceneMarkerGlyph {
    base_y: f32,
}

#[derive(Component)]
pub struct RunHudText;

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

/// Multi-source BFS over walkable tiles; sources start at distance 0.
fn distance_field(map: &MapData, sources: &[(i32, i32)]) -> Vec<Vec<u16>> {
    let mut dist = vec![vec![u16::MAX; MAP_W as usize]; MAP_H as usize];
    let mut queue = std::collections::VecDeque::new();
    for &(c, r) in sources {
        if c >= 0 && r >= 0 && c < MAP_W && r < MAP_H {
            dist[r as usize][c as usize] = 0;
            queue.push_back((c, r));
        }
    }
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

/// Recompute the steering field: toward uncleared markers, else the portal.
fn recompute_flow(scene: &mut RunSceneState, map: &MapData) {
    let targets: Vec<(i32, i32)> = if scene.markers.iter().any(|m| !m.cleared) {
        scene
            .markers
            .iter()
            .filter(|m| !m.cleared)
            .map(|m| (m.col, m.row))
            .collect()
    } else {
        scene.portals.clone()
    };
    scene.flow = distance_field(map, &targets);
}

/// Build a stage's flow field from scratch (used by capture presets that
/// hand-craft a `RunSceneState`). Nudges markers off unwalkable tiles.
pub fn seed_flow(scene: &mut RunSceneState) {
    let map = MapData::build(scene.map);
    for marker in &mut scene.markers {
        if !map.at(marker.col, marker.row).walkable() {
            'search: for radius in 1..10 {
                for dr in -radius..=radius {
                    for dc in -radius..=radius {
                        if map.at(marker.col + dc, marker.row + dr).walkable() {
                            marker.col += dc;
                            marker.row += dr;
                            break 'search;
                        }
                    }
                }
            }
        }
    }
    recompute_flow(scene, &map);
}

// ---------------------------------------------------------------------------
// Stage generation (runs on the `NodeMap` hop state)
// ---------------------------------------------------------------------------

/// Roll a marker kind for a normal stage.
fn roll_marker_kind(rng: &mut Rng, stage: usize) -> NodeKind {
    let roll = rng.unit();
    if stage >= 1 && roll < 0.14 {
        NodeKind::Elite
    } else if roll < 0.52 {
        NodeKind::Fight
    } else if roll < 0.76 {
        NodeKind::Event
    } else if roll < 0.88 {
        NodeKind::Rest
    } else {
        NodeKind::Market
    }
}

/// Build the next stage into `RunSceneState` and enter the scene.
pub fn advance_stage(
    mut commands: Commands,
    run: Option<ResMut<RunState>>,
    mut rng: ResMut<Rng>,
    mut next: ResMut<NextState<AppState>>,
) {
    let Some(mut run) = run else {
        next.set(AppState::Title);
        return;
    };
    let map_kind = run.roll_map(&mut rng);
    let map = MapData::build(map_kind);
    let spawn = map.spawn();

    // Portals.
    let mut portals = Vec::new();
    for r in 0..MAP_H {
        for c in 0..MAP_W {
            if map.at(c, r) == Tile::Portal {
                portals.push((c, r));
            }
        }
    }

    // Candidate tiles: walkable, reasonably far from spawn and the portal.
    let from_spawn = distance_field(&map, &[spawn]);
    let mut candidates: Vec<(i32, i32)> = Vec::new();
    for r in 0..MAP_H {
        for c in 0..MAP_W {
            let d = from_spawn[r as usize][c as usize];
            if d != u16::MAX && d >= 6 && map.at(c, r) != Tile::Portal {
                candidates.push((c, r));
            }
        }
    }

    let mut markers: Vec<SceneMarker> = Vec::new();
    if run.is_boss_stage() {
        // Boss map: a single demon gate at the farthest reachable tile.
        let far = candidates
            .iter()
            .copied()
            .max_by_key(|&(c, r)| from_spawn[r as usize][c as usize])
            .unwrap_or(spawn);
        markers.push(SceneMarker {
            kind: NodeKind::Boss,
            col: far.0,
            row: far.1,
            cleared: false,
        });
    } else {
        // 2–3 spread-out objectives.
        let count = rng.range(2, 3) as usize;
        let mut guard = 0;
        while markers.len() < count && guard < 400 && !candidates.is_empty() {
            guard += 1;
            let pick = candidates[rng.range(0, candidates.len() as i32 - 1) as usize];
            let min_gap = if guard > 200 { 4 } else { 8 };
            let spread = markers
                .iter()
                .all(|m| (m.col - pick.0).abs() + (m.row - pick.1).abs() >= min_gap);
            if spread {
                markers.push(SceneMarker {
                    kind: roll_marker_kind(&mut rng, run.stage),
                    col: pick.0,
                    row: pick.1,
                    cleared: false,
                });
            }
        }
        // Guarantee the chapter's story beat before the boss map.
        let last_normal_stage = run.stage + 2 >= run.stage_count();
        if run.story_pending && !markers.is_empty() && (last_normal_stage || rng.chance(0.45)) {
            let index = rng.range(0, markers.len() as i32 - 1) as usize;
            markers[index].kind = NodeKind::Story;
            run.story_pending = false;
        }
    }

    let mut scene = RunSceneState {
        map: map_kind,
        col: -1,
        row: -1,
        facing_left: false,
        markers,
        portals,
        flow: Vec::new(),
        cooldown: 0.0,
    };
    recompute_flow(&mut scene, &map);
    commands.insert_resource(scene);
    next.set(AppState::RunScene);
}

// ---------------------------------------------------------------------------
// Scene setup (also re-runs after returning from a battle)
// ---------------------------------------------------------------------------

pub fn spawn_run_scene(
    mut commands: Commands,
    font: Res<GameFont>,
    assets: Res<ExploreAssets>,
    anims: Res<AnimationAssets>,
    dolls: Res<PaperdollAssets>,
    lights: Res<LightingAssets>,
    asset_server: Res<AssetServer>,
    run: Option<ResMut<RunState>>,
    scene: Option<ResMut<RunSceneState>>,
    mut dialogue: ResMut<RunDialogue>,
    mut next: ResMut<NextState<AppState>>,
) {
    let (Some(mut run), Some(mut scene)) = (run, scene) else {
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

    // Soft fill lights so the whole map reads clearly.
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

    // Hero (fresh stage: start at the map's spawn point).
    if scene.col < 0 {
        let (c, r) = map.spawn();
        scene.col = c;
        scene.row = r;
    }
    scene.cooldown = 0.0;
    let p = tile_to_world(scene.col, scene.row);
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

    // Markers.
    for (index, marker) in scene.markers.iter().enumerate() {
        if marker.cleared {
            continue;
        }
        spawn_marker_visual(
            &mut commands,
            &font,
            &dolls,
            &lights,
            &asset_server,
            index,
            marker,
        );
    }

    // Portal glow (exit to the next stage).
    for &(c, r) in &scene.portals {
        let p = tile_to_world(c, r);
        lighting::spawn_light(
            &mut commands,
            &lights,
            Vec3::new(p.x, p.y, 4.0),
            TILE * 2.6,
            Color::srgba(0.45, 0.95, 1.0, 0.4),
            AppState::RunScene,
        );
        commands.spawn((
            SceneMarkerGlyph { base_y: p.y + 34.0 },
            Text2d::new("门"),
            font.text_font(20.0),
            TextColor(Color::srgb(0.62, 0.95, 1.0)),
            Transform::from_xyz(p.x, p.y + 34.0, 11.0),
            scope(),
        ));
    }

    // HUD + stage banner.
    commands.spawn((
        RunHudText,
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
            "{} · {} (第 {}/{} 程)",
            run.chapter_def().title,
            map.name(),
            run.stage + 1,
            run.stage_count(),
        )),
        font.text_font(22.0),
        TextColor(Color::srgb(0.95, 0.85, 0.55)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            justify_self: JustifySelf::Center,
            ..default()
        },
        scope(),
    ));
    commands.spawn((
        Text::new("方向键 移动 · 探明所有「?」迷雾后从「门」离开 · 草丛有妖"),
        font.text_font(16.0),
        TextColor(Color::srgba(0.9, 0.92, 0.95, 0.75)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            right: Val::Px(14.0),
            ..default()
        },
        scope(),
    ));

    // Dialogue overlay (events/story/rest/market/chapter card resolve here).
    event::spawn_run_dialogue_ui(&mut commands, &font, scope());

    // First stage of a chapter: show the chapter card.
    if !run.card_shown && run.stage == 0 {
        run.card_shown = true;
        let chapter = run.chapter;
        dialogue.open_plain(
            run.chapter_def().title,
            super::content::CHAPTER_CARDS[chapter.min(3)],
        );
    }

    commands.insert_resource(RunSceneMap(map));
}

fn spawn_marker_visual(
    commands: &mut Commands,
    font: &GameFont,
    _dolls: &PaperdollAssets,
    lights: &LightingAssets,
    asset_server: &AssetServer,
    index: usize,
    marker: &SceneMarker,
) {
    let scope = || DespawnOnExit(AppState::RunScene);
    let op = tile_to_world(marker.col, marker.row);

    // Slay-the-Spire-style unknowns: every objective is a mysterious mist
    // light — what it holds (battle / event / story / rest / market) is only
    // revealed on contact. The chapter boss gate is the one visible landmark.
    if marker.kind == NodeKind::Boss {
        commands.spawn((
            SceneMarkerVisual(index),
            Sprite {
                image: asset_server.load("props/ai_bamboo_gate.png"),
                custom_size: Some(Vec2::splat(96.0)),
                ..default()
            },
            Transform::from_xyz(op.x, op.y + 10.0, 9.0),
            scope(),
        ));
        commands.spawn((
            SceneMarkerVisual(index),
            SceneMarkerGlyph {
                base_y: op.y + 58.0,
            },
            Text2d::new("魔"),
            font.text_font(24.0),
            TextColor(Color::srgb(0.92, 0.72, 1.0)),
            Transform::from_xyz(op.x, op.y + 58.0, 11.0),
            scope(),
        ));
        let light = lighting::spawn_light(
            commands,
            lights,
            Vec3::new(op.x, op.y, 4.0),
            TILE * 3.4,
            Color::srgba(0.80, 0.40, 1.0, 0.52),
            AppState::RunScene,
        );
        commands.entity(light).insert(SceneMarkerVisual(index));
        return;
    }

    let mist = Color::srgba(0.62, 0.58, 1.0, 0.42);
    commands.spawn((
        SceneMarkerVisual(index),
        Sprite {
            image: lights.orb.clone(),
            color: mist.with_alpha(0.85),
            custom_size: Some(Vec2::splat(58.0)),
            ..default()
        },
        Transform::from_xyz(op.x, op.y + 4.0, 9.0),
        scope(),
    ));
    commands.spawn((
        SceneMarkerVisual(index),
        SceneMarkerGlyph {
            base_y: op.y + 42.0,
        },
        Text2d::new("?"),
        font.text_font(26.0),
        TextColor(Color::srgb(0.90, 0.86, 1.0)),
        Transform::from_xyz(op.x, op.y + 42.0, 11.0),
        scope(),
    ));
    let light = lighting::spawn_light(
        commands,
        lights,
        Vec3::new(op.x, op.y, 4.0),
        TILE * 3.0,
        mist,
        AppState::RunScene,
    );
    commands.entity(light).insert(SceneMarkerVisual(index));
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
    visuals: Query<(Entity, &SceneMarkerVisual)>,
) {
    let (Some(mut scene), Some(map), Some(mut run)) = (scene, map, run) else {
        return;
    };
    if dialogue.active {
        return; // overlay swallows input in run_dialogue_input
    }

    scene.cooldown -= time.delta_secs();
    let mut moved = false;
    if let Some(dir) = intent.move_dir {
        if scene.cooldown <= 0.0 {
            let (nc, nr) = (scene.col + dir.x, scene.row + dir.y);
            if map.0.at(nc, nr).walkable() {
                scene.col = nc;
                scene.row = nr;
                scene.cooldown = 0.14;
                moved = true;
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
    if !moved {
        return;
    }

    // Stepped onto an uncleared marker: fire its payload.
    if let Some(index) = scene
        .markers
        .iter()
        .position(|m| !m.cleared && (m.col, m.row) == (scene.col, scene.row))
    {
        scene.markers[index].cleared = true;
        let kind = scene.markers[index].kind;
        for (entity, visual) in &visuals {
            if visual.0 == index {
                commands.entity(entity).despawn();
            }
        }
        recompute_flow(&mut scene, &map.0);

        match kind {
            NodeKind::Fight | NodeKind::Elite | NodeKind::Boss => {
                let rank = match kind {
                    NodeKind::Elite => FightRank::Elite,
                    NodeKind::Boss => FightRank::Boss,
                    _ => FightRank::Normal,
                };
                run.current_fight = Some(rank);
                commands.insert_resource(PendingEncounter {
                    zone: map_zone(scene.map),
                    kind: encounter_kind_for(&run, kind),
                });
                commands.insert_resource(battle_mods_for(&run, rank));
                next.set(AppState::Battle);
            }
            NodeKind::Event => {
                let index = run.draw_event(&mut rng);
                dialogue.open_event(index);
            }
            NodeKind::Story => {
                let roll =
                    rng.range(0, super::content::story_count(run.chapter) as i32 - 1) as usize;
                dialogue.open_story(run.chapter, roll);
            }
            NodeKind::Rest => dialogue.open_rest(),
            NodeKind::Market => dialogue.open_market(),
        }
        return;
    }

    // Grass rustle: classic random encounters keep every walk risky.
    if map.0.at(scene.col, scene.row) == Tile::Grass && rng.chance(0.08) {
        run.current_fight = Some(FightRank::Normal);
        commands.insert_resource(PendingEncounter {
            zone: map_zone(scene.map),
            kind: encounter_kind_for(&run, NodeKind::Fight),
        });
        commands.insert_resource(battle_mods_for(&run, FightRank::Normal));
        next.set(AppState::Battle);
        return;
    }

    // Stepped onto the portal: advance once the map is cleared.
    if map.0.at(scene.col, scene.row) == Tile::Portal {
        if scene.markers.iter().all(|m| m.cleared) {
            run.stage += 1;
            next.set(AppState::NodeMap); // hop → builds the next stage
        } else {
            let left = scene.markers.iter().filter(|m| !m.cleared).count();
            dialogue.open_plain(
                "路引",
                &[&format!("妖气未清,此门不开——图上还有 {left} 处发光标记。")],
            );
        }
    }
}

/// Gentle bob on marker glyphs so they read as interactive.
pub fn animate_marker_glyph(
    time: Res<Time>,
    mut glyphs: Query<(&SceneMarkerGlyph, &mut Transform)>,
) {
    for (glyph, mut transform) in &mut glyphs {
        transform.translation.y = glyph.base_y + (time.elapsed_secs() * 2.4).sin() * 4.0;
    }
}

pub fn update_run_hud(
    stats: Res<PlayerStats>,
    run: Option<Res<RunState>>,
    mut hud: Query<&mut Text, With<RunHudText>>,
) {
    let Some(run) = run else { return };
    let Ok(mut text) = hud.single_mut() else {
        return;
    };
    let relics = if run.relics.is_empty() {
        "无".to_string()
    } else {
        run.relics
            .iter()
            .map(|r| r.name())
            .collect::<Vec<_>>()
            .join("、")
    };
    text.0 = format!(
        "{}\n气血 {}/{} · 灵力 {}/{}\n药水 ×{} · 钱财 {} 文\n道心 {} · 情缘 {}\n法宝:{}",
        stats.name,
        stats.hp,
        stats.max_hp,
        stats.mp,
        stats.max_mp,
        stats.potions,
        stats.gold,
        run.daoxin,
        run.qingyuan,
        relics,
    );
}

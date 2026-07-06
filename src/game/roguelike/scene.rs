//! Map stages (地图关卡) — the roguelike run IS the map.
//!
//! Each chapter is a chain of real walkable tile maps (reusing the legacy
//! Explore chapter maps + art). Every stage scatters 2–3 objective markers
//! (battles / events / story / rest / market); once all are cleared the
//! glowing portal tile leads to the next stage. The chapter's final stage is
//! the boss map: a single demon gate. Battles hop out to `AppState::Battle`
//! and return here through the Reward screen.

use bevy::camera::ScalingMode;
use bevy::prelude::*;
use bevy::ui::widget::NodeImageMode;

use super::super::animation::{self, AnimationAssets, AnimationClip, SpriteAnimation};
use super::super::battle::{EncounterZone, PendingEncounter};
use super::super::core::{GameFont, Intent, MAP_H, MAP_W, PlayerStats, Rng, TILE, tile_to_world};
use super::super::explore::{ExploreAssets, MapData, MapKind, Tile, tile_sprite};
use super::super::lighting::{self, LightingAssets};
use super::super::paperdoll::{self, PaperdollAssets, PaperdollStyle};
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
    /// Procedurally generated terrain for this stage (persists across
    /// battles so the scene rebuilds identically).
    pub tiles: Vec<Vec<Tile>>,
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
    /// 瘴气毒格:踩上扣血(可反复),绕路与否是玩家的取舍。
    pub hazards: Vec<(i32, i32)>,
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

/// 章节开卷的全屏过场画,盖在地图之上、章节卡文字之下;
/// 章节卡对话一关就撤下。
#[derive(Component)]
pub struct ChapterArt;

/// 章节卡对话关闭后撤下全屏过场画。
pub fn clear_chapter_art(
    mut commands: Commands,
    dialogue: Res<RunDialogue>,
    art: Query<Entity, With<ChapterArt>>,
) {
    if dialogue.active {
        return;
    }
    for e in &art {
        commands.entity(e).despawn();
    }
}

/// Transient floating text on the map (hazard damage, pickups): rises and
/// fades, then despawns.
#[derive(Component)]
pub struct SceneFloatText {
    age: f32,
}

#[derive(Component)]
pub struct HudHpFill;

#[derive(Component)]
pub struct HudMpFill;

#[derive(Component)]
pub struct HudSubText;

/// The Esc inventory (行囊/纸娃娃) overlay is open; movement pauses.
#[derive(Resource, Default)]
pub struct InventoryOpen(pub bool);

#[derive(Component)]
pub struct InventoryUi;

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
    if scene.tiles.is_empty() {
        scene.tiles = MapData::build(scene.map).tiles;
    }
    let map = MapData::generated(scene.map, scene.tiles.clone());
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

/// Cellular-automata organic terrain: winding tree walls, blob lakes and
/// scattered grass — no hand-authored rectangles.
fn generate_stage_tiles(rng: &mut Rng) -> Vec<Vec<Tile>> {
    let (w, h) = (MAP_W as usize, MAP_H as usize);
    loop {
        // 1. Noise fill + guaranteed border.
        let mut walls = vec![vec![false; w]; h];
        for (r, row) in walls.iter_mut().enumerate() {
            for (c, cell) in row.iter_mut().enumerate() {
                let edge = r == 0 || c == 0 || r == h - 1 || c == w - 1;
                *cell = edge || rng.chance(0.40);
            }
        }
        // 2. Smooth into organic blobs.
        for _ in 0..4 {
            let snapshot = walls.clone();
            for r in 1..h - 1 {
                for c in 1..w - 1 {
                    let mut n = 0;
                    for dr in -1i32..=1 {
                        for dc in -1i32..=1 {
                            if snapshot[(r as i32 + dr) as usize][(c as i32 + dc) as usize] {
                                n += 1;
                            }
                        }
                    }
                    walls[r][c] = n >= 5;
                }
            }
        }
        // 3. Keep only the largest open region.
        let mut region = vec![vec![0u16; w]; h];
        let mut sizes = vec![0usize];
        for r in 0..h {
            for c in 0..w {
                if walls[r][c] || region[r][c] != 0 {
                    continue;
                }
                let id = sizes.len() as u16;
                let mut size = 0;
                let mut queue = std::collections::VecDeque::from([(c, r)]);
                region[r][c] = id;
                while let Some((qc, qr)) = queue.pop_front() {
                    size += 1;
                    for (dc, dr) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                        let (nc, nr) = (qc as i32 + dc, qr as i32 + dr);
                        if nc < 0 || nr < 0 || nc >= w as i32 || nr >= h as i32 {
                            continue;
                        }
                        let (nc, nr) = (nc as usize, nr as usize);
                        if !walls[nr][nc] && region[nr][nc] == 0 {
                            region[nr][nc] = id;
                            queue.push_back((nc, nr));
                        }
                    }
                }
                sizes.push(size);
            }
        }
        let Some((best_id, &best_size)) = sizes.iter().enumerate().skip(1).max_by_key(|(_, s)| **s)
        else {
            continue;
        };
        if best_size < 180 {
            continue; // too cramped — reroll
        }
        let mut tiles = vec![vec![Tile::Wall; w]; h];
        for r in 0..h {
            for c in 0..w {
                if region[r][c] == best_id as u16 {
                    tiles[r][c] = Tile::Path;
                }
            }
        }
        // 4. A blob lake or two (unwalkable, kept small to preserve routes).
        for _ in 0..rng.range(1, 2) {
            let mut placed = 0;
            let (mut c, mut r) = (rng.range(4, MAP_W - 5), rng.range(3, MAP_H - 4));
            for _ in 0..40 {
                if placed >= 10 {
                    break;
                }
                if tiles[r as usize][c as usize] == Tile::Path {
                    tiles[r as usize][c as usize] = Tile::Water;
                    placed += 1;
                }
                match rng.range(0, 3) {
                    0 => c = (c + 1).min(MAP_W - 2),
                    1 => c = (c - 1).max(1),
                    2 => r = (r + 1).min(MAP_H - 2),
                    _ => r = (r - 1).max(1),
                }
            }
        }
        // 5. Grass patches (walkable, roll encounters).
        for _ in 0..rng.range(5, 8) {
            let (mut c, mut r) = (rng.range(2, MAP_W - 3), rng.range(2, MAP_H - 3));
            for _ in 0..rng.range(8, 18) {
                if tiles[r as usize][c as usize] == Tile::Path {
                    tiles[r as usize][c as usize] = Tile::Grass;
                }
                match rng.range(0, 3) {
                    0 => c = (c + 1).min(MAP_W - 2),
                    1 => c = (c - 1).max(1),
                    2 => r = (r + 1).min(MAP_H - 2),
                    _ => r = (r - 1).max(1),
                }
            }
        }
        // 6. Water may have split the open area — keep the largest walkable
        // region only (stray pockets become walls).
        let map = MapData::generated(MapKind::Village, tiles.clone());
        let mut seed = None;
        'find: for r in 0..h {
            for c in 0..w {
                if tiles[r][c].walkable() {
                    seed = Some((c as i32, r as i32));
                    break 'find;
                }
            }
        }
        let Some(seed) = seed else { continue };
        let dist = distance_field(&map, &[seed]);
        // pick the true largest region: try a few seeds, keep best
        let mut best = (seed, dist);
        for _ in 0..4 {
            let (c, r) = (rng.range(1, MAP_W - 2), rng.range(1, MAP_H - 2));
            if tiles[r as usize][c as usize].walkable() {
                let d = distance_field(&map, &[(c, r)]);
                let count =
                    |f: &Vec<Vec<u16>>| f.iter().flatten().filter(|v| **v != u16::MAX).count();
                if count(&d) > count(&best.1) {
                    best = ((c, r), d);
                }
            }
        }
        let reach = best.1;
        let mut open = 0;
        for r in 0..h {
            for c in 0..w {
                if tiles[r][c].walkable() && reach[r][c] == u16::MAX {
                    tiles[r][c] = Tile::Wall;
                } else if tiles[r][c].walkable() {
                    open += 1;
                }
            }
        }
        if open < 150 {
            continue;
        }
        return tiles;
    }
}

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
    let mut tiles = generate_stage_tiles(&mut rng);

    // Spawn on a walkable tile near the left edge.
    let mut spawn = (1, MAP_H / 2);
    'spawn: for c in 1..MAP_W {
        let mut rows: Vec<i32> = (1..MAP_H - 1).collect();
        // shuffle-ish: random start offset
        let off = rng.range(0, rows.len() as i32 - 1) as usize;
        rows.rotate_left(off);
        for r in rows {
            if tiles[r as usize][c as usize].walkable() {
                spawn = (c, r);
                break 'spawn;
            }
        }
    }

    let map = MapData::generated(map_kind, tiles.clone());
    let from_spawn = distance_field(&map, &[spawn]);

    // Portal: the farthest reachable tile becomes the exit gate.
    let mut portal = spawn;
    let mut best_d = 0u16;
    for r in 0..MAP_H {
        for c in 0..MAP_W {
            let d = from_spawn[r as usize][c as usize];
            if d != u16::MAX && d > best_d {
                best_d = d;
                portal = (c, r);
            }
        }
    }
    tiles[portal.1 as usize][portal.0 as usize] = Tile::Portal;

    // Marker candidates: reachable, away from both spawn and portal.
    let mut candidates: Vec<(i32, i32)> = Vec::new();
    for r in 0..MAP_H {
        for c in 0..MAP_W {
            let d = from_spawn[r as usize][c as usize];
            let dp = (c - portal.0).abs() + (r - portal.1).abs();
            if d != u16::MAX && d >= 6 && dp >= 4 && tiles[r as usize][c as usize].walkable() {
                candidates.push((c, r));
            }
        }
    }

    let mut markers: Vec<SceneMarker> = Vec::new();
    if run.is_boss_stage() {
        // Boss map: the gate replaces the portal at the farthest tile.
        tiles[portal.1 as usize][portal.0 as usize] = Tile::Path;
        markers.push(SceneMarker {
            kind: NodeKind::Boss,
            col: portal.0,
            row: portal.1,
            cleared: false,
        });
    } else {
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

        // Optional visible loot: a chest (maybe a mimic) and/or a spirit
        // spring — worth a detour, never required to open the gate.
        for (kind, chance) in [(NodeKind::Chest, 0.55), (NodeKind::Spring, 0.40)] {
            if !rng.chance(chance) {
                continue;
            }
            let mut guard = 0;
            while guard < 200 && !candidates.is_empty() {
                guard += 1;
                let pick = candidates[rng.range(0, candidates.len() as i32 - 1) as usize];
                let spread = markers
                    .iter()
                    .all(|m| (m.col - pick.0).abs() + (m.row - pick.1).abs() >= 5);
                if spread {
                    markers.push(SceneMarker {
                        kind,
                        col: pick.0,
                        row: pick.1,
                        cleared: false,
                    });
                    break;
                }
            }
        }
    }

    // 瘴气毒格:少量散布在可达路面上,可见、可绕。
    let mut hazards: Vec<(i32, i32)> = Vec::new();
    if !run.is_boss_stage() {
        let want = rng.range(3, 5);
        let mut guard = 0;
        while (hazards.len() as i32) < want && guard < 300 {
            guard += 1;
            let c = rng.range(2, MAP_W - 3);
            let r = rng.range(2, MAP_H - 3);
            let d = from_spawn[r as usize][c as usize];
            if d == u16::MAX || d < 4 {
                continue;
            }
            if tiles[r as usize][c as usize] != Tile::Path {
                continue;
            }
            if (c, r) == portal
                || markers.iter().any(|m| (m.col, m.row) == (c, r))
                || hazards.iter().any(|&(hc, hr)| (hc, hr) == (c, r))
            {
                continue;
            }
            hazards.push((c, r));
        }
    }

    let portals = if run.is_boss_stage() {
        Vec::new()
    } else {
        vec![portal]
    };
    let mut scene = RunSceneState {
        map: map_kind,
        tiles,
        col: spawn.0,
        row: spawn.1,
        facing_left: false,
        markers,
        portals,
        flow: Vec::new(),
        hazards,
        cooldown: 0.0,
    };
    let map = MapData::generated(map_kind, scene.tiles.clone());
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
    if scene.tiles.is_empty() {
        scene.tiles = MapData::build(scene.map).tiles;
    }
    let map = MapData::generated(scene.map, scene.tiles.clone());

    // Tiles. All tile textures are 512×512 seamless: instead of squeezing the
    // whole sheet into one 48px cell (which blurs it into a kaleidoscope),
    // each cell samples its own 128×128 sub-rect by world position, so the
    // texture flows continuously across a 4×4-cell area with no mirror
    // symmetry. A light per-tile brightness jitter keeps large fields alive.
    let sub_rect = |col: i32, row: i32| {
        let x0 = col.rem_euclid(4) as f32 * 128.0;
        let y0 = row.rem_euclid(4) as f32 * 128.0;
        Rect::new(x0, y0, x0 + 128.0, y0 + 128.0)
    };
    for row in 0..MAP_H {
        for col in 0..MAP_W {
            let p = tile_to_world(col, row);
            let mut sprite = tile_sprite(map.at(col, row), scene.map, &assets);
            sprite.rect = Some(sub_rect(col, row));
            let hash =
                ((col as u32).wrapping_mul(73_856_093)) ^ ((row as u32).wrapping_mul(19_349_663));
            let tint = 0.92 + ((hash >> 3) % 8) as f32 * 0.015;
            let c = sprite.color.to_srgba();
            sprite.color = Color::srgb(c.red * tint, c.green * tint, c.blue * tint);
            commands.spawn((sprite, Transform::from_xyz(p.x, p.y, 0.0), scope()));
        }
    }

    // A thick ring of wall tiles beyond the map edge, so the camera never
    // shows a hard rectangular cutoff against black.
    for row in -7..(MAP_H + 7) {
        for col in -7..(MAP_W + 7) {
            if (0..MAP_W).contains(&col) && (0..MAP_H).contains(&row) {
                continue;
            }
            let p = tile_to_world(col, row);
            let mut sprite = tile_sprite(Tile::Wall, scene.map, &assets);
            sprite.rect = Some(sub_rect(col, row));
            let hash =
                ((col as u32).wrapping_mul(73_856_093)) ^ ((row as u32).wrapping_mul(19_349_663));
            let tint = 0.80 + ((hash >> 3) % 8) as f32 * 0.015;
            let c = sprite.color.to_srgba();
            sprite.color = Color::srgba(c.red * tint, c.green * tint, c.blue * tint, 0.9);
            commands.spawn((sprite, Transform::from_xyz(p.x, p.y, 0.0), scope()));
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

    // 瘴气毒格:紫雾明示危险,绕不绕路自己掂量。
    for &(c, r) in &scene.hazards {
        let p = tile_to_world(c, r);
        commands.spawn((
            Sprite {
                image: lights.orb.clone(),
                color: Color::srgba(0.62, 0.25, 0.85, 0.55),
                custom_size: Some(Vec2::splat(46.0)),
                ..default()
            },
            Transform::from_xyz(p.x, p.y, 8.0),
            scope(),
        ));
    }

    // The stage exit: a weathered spirit gate (界门) standing over the
    // portal tile, wrapped in teal light.
    for &(c, r) in &scene.portals {
        let p = tile_to_world(c, r);
        commands.spawn((
            Sprite {
                image: asset_server.load("props/ai_spirit_gate.png"),
                custom_size: Some(Vec2::new(86.0, 92.0)),
                ..default()
            },
            Transform::from_xyz(p.x, p.y + 18.0, 9.0),
            scope(),
        ));
        lighting::spawn_light(
            &mut commands,
            &lights,
            Vec3::new(p.x, p.y, 4.0),
            TILE * 3.2,
            Color::srgba(0.45, 0.95, 1.0, 0.42),
            AppState::RunScene,
        );
    }

    // HUD: hero avatar + HP/MP bars (details live in the Esc inventory).
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(12.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(10.0),
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.03, 0.07, 0.6)),
            scope(),
        ))
        .with_children(|hud| {
            hud.spawn((
                ImageNode::new(asset_server.load("npcs/ai_hero.png")),
                Node {
                    width: Val::Px(72.0),
                    height: Val::Px(72.0),
                    flex_shrink: 0.0,
                    ..default()
                },
            ));
            hud.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(5.0),
                ..default()
            })
            .with_children(|col| {
                for (marker, back, front) in [
                    (
                        true,
                        Color::srgba(0.25, 0.06, 0.06, 0.9),
                        Color::srgb(0.85, 0.25, 0.2),
                    ),
                    (
                        false,
                        Color::srgba(0.06, 0.10, 0.25, 0.9),
                        Color::srgb(0.3, 0.55, 0.95),
                    ),
                ] {
                    col.spawn((
                        Node {
                            width: Val::Px(180.0),
                            height: Val::Px(13.0),
                            ..default()
                        },
                        BackgroundColor(back),
                    ))
                    .with_children(|bar| {
                        let fill = (
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(front),
                        );
                        if marker {
                            bar.spawn((HudHpFill, fill));
                        } else {
                            bar.spawn((HudMpFill, fill));
                        }
                    });
                }
                col.spawn((
                    HudSubText,
                    Text::new(""),
                    font.text_font(15.0),
                    TextColor(Color::srgba(0.9, 0.92, 0.95, 0.9)),
                ));
            });
        });
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
        Text::new("方向键 移动 · 探明「?」后踏入界门 · 草丛有妖 · ESC 行囊"),
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
    event::spawn_run_dialogue_ui(
        &mut commands,
        &font,
        asset_server.load("ui/panel_frame.png"),
        scope(),
    );

    // First stage of a chapter: show the chapter card over a full-screen
    // chapter painting.
    if !run.card_shown && run.stage == 0 {
        run.card_shown = true;
        let chapter = run.chapter;
        dialogue.open_plain(
            run.chapter_def().title,
            super::content::CHAPTER_CARDS[chapter.min(3)],
        );
        commands.spawn((
            ChapterArt,
            scope(),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                ..default()
            },
            ImageNode::new(asset_server.load(format!("ui/chapter{}_art.png", chapter.min(3) + 1))),
            GlobalZIndex(40),
        ));
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
                image: asset_server.load("props/ai_spirit_gate.png"),
                color: Color::srgb(1.0, 0.62, 1.0),
                custom_size: Some(Vec2::new(104.0, 112.0)),
                ..default()
            },
            Transform::from_xyz(op.x, op.y + 22.0, 9.0),
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

    // Visible optional loot: chests and springs announce themselves with real
    // prop art — the detour (and the mimic risk) is the player's call.
    if marker.kind.optional() {
        let (image, size, glow) = if marker.kind == NodeKind::Spring {
            (
                "props/ai_spring.png",
                Vec2::new(62.0, 50.0),
                Color::srgba(0.30, 0.95, 0.85, 0.45),
            )
        } else {
            (
                "props/ai_chest.png",
                Vec2::new(52.0, 40.0),
                Color::srgba(1.0, 0.75, 0.30, 0.45),
            )
        };
        commands.spawn((
            SceneMarkerVisual(index),
            Sprite {
                image: asset_server.load(image),
                custom_size: Some(size),
                ..default()
            },
            Transform::from_xyz(op.x, op.y + 6.0, 9.0),
            scope(),
        ));
        let light = lighting::spawn_light(
            commands,
            lights,
            Vec3::new(op.x, op.y, 4.0),
            TILE * 2.6,
            glow,
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
    font: Res<GameFont>,
    mut intent: ResMut<Intent>,
    mut stats: ResMut<PlayerStats>,
    scene: Option<ResMut<RunSceneState>>,
    map: Option<Res<RunSceneMap>>,
    run: Option<ResMut<RunState>>,
    mut rng: ResMut<Rng>,
    mut dialogue: ResMut<RunDialogue>,
    inventory: Res<InventoryOpen>,
    mut next: ResMut<NextState<AppState>>,
    visuals: Query<(Entity, &SceneMarkerVisual)>,
) {
    let (Some(mut scene), Some(map), Some(mut run)) = (scene, map, run) else {
        return;
    };
    if dialogue.active || inventory.0 {
        return; // an overlay (dialogue / inventory) owns the input
    }
    // 对峙对话刚落幕:魔门之战开场。
    if dialogue.boss_battle_after {
        dialogue.boss_battle_after = false;
        run.current_fight = Some(FightRank::Boss);
        commands.insert_resource(PendingEncounter {
            zone: map_zone(scene.map),
            kind: encounter_kind_for(&run, NodeKind::Boss),
        });
        commands.insert_resource(battle_mods_for(&run, FightRank::Boss));
        next.set(AppState::Battle);
        return;
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
            NodeKind::Boss => {
                // 章末魔门:先礼后兵——对峙台词落幕才拔剑。
                let (title, lines) =
                    super::content::boss_taunt(run.boss, run.qingyuan > run.daoxin);
                dialogue.open_plain(title, lines);
                dialogue.boss_battle_after = true;
            }
            NodeKind::Fight | NodeKind::Elite => {
                let rank = match kind {
                    NodeKind::Elite => FightRank::Elite,
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
                let index = run.draw_chain_or_event(&mut rng);
                dialogue.open_event(index);
            }
            NodeKind::Story => {
                let roll =
                    rng.range(0, super::content::story_count(run.chapter) as i32 - 1) as usize;
                dialogue.open_story(run.chapter, roll);
            }
            NodeKind::Rest => dialogue.open_rest(),
            NodeKind::Market => dialogue.open_market(),
            NodeKind::Chest => {
                let roll = rng.unit();
                if roll < 0.20 {
                    // 宝箱妖!贪心有价。
                    run.current_fight = Some(FightRank::Elite);
                    commands.insert_resource(PendingEncounter {
                        zone: map_zone(scene.map),
                        kind: encounter_kind_for(&run, NodeKind::Elite),
                    });
                    commands.insert_resource(battle_mods_for(&run, FightRank::Elite));
                    next.set(AppState::Battle);
                } else if roll < 0.35 {
                    let line = super::event::grant_random_relic(&mut stats, &mut run, &mut rng);
                    dialogue.open_plain("宝箱", &["箱盖开处灵光扑面——", &line]);
                } else if roll < 0.60 {
                    stats.potions += 1;
                    dialogue.open_plain("宝箱", &["箱中静静躺着一瓶药水。药水 +1。"]);
                } else {
                    let gold = rng.range(30, 80) as u32;
                    stats.gold += gold;
                    dialogue.open_plain("宝箱", &[&format!("箱底散着碎银铜钱,共 {gold} 文。")]);
                }
            }
            NodeKind::Spring => {
                let heal = (stats.max_hp * 35 / 100)
                    .min(stats.max_hp - stats.hp)
                    .max(0);
                stats.hp += heal;
                dialogue.open_plain(
                    "灵泉",
                    &[&format!("掬一捧灵泉,暖流沿经脉散开——恢复 {heal} 点气血。")],
                );
            }
        }
        return;
    }

    // 瘴气毒格:踩上即中毒掉血(不致死),下次记得绕路。
    if scene
        .hazards
        .iter()
        .any(|&(c, r)| (c, r) == (scene.col, scene.row))
    {
        let hurt = (stats.max_hp * 8 / 100).max(3);
        stats.hp = (stats.hp - hurt).max(1);
        let p = tile_to_world(scene.col, scene.row);
        commands.spawn((
            SceneFloatText { age: 0.0 },
            Text2d::new(format!("瘴毒 -{hurt}")),
            font.text_font(20.0),
            TextColor(Color::srgb(0.85, 0.45, 1.0)),
            Transform::from_xyz(p.x, p.y + 30.0, 30.0),
            DespawnOnExit(AppState::RunScene),
        ));
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

    // Stepped onto the portal: advance once the map is cleared (visible
    // loot — chests and springs — never blocks the gate).
    if map.0.at(scene.col, scene.row) == Tile::Portal {
        if scene.markers.iter().all(|m| m.cleared || m.kind.optional()) {
            run.stage += 1;
            next.set(AppState::NodeMap); // hop → builds the next stage
        } else {
            let left = scene
                .markers
                .iter()
                .filter(|m| !m.cleared && !m.kind.optional())
                .count();
            dialogue.open_plain(
                "路引",
                &[&format!("妖气未清,此门不开——图上还有 {left} 处发光标记。")],
            );
        }
    }
}

/// Glide the hero sprite toward its logical tile and swap between the walk
/// and idle sheets, so movement reads as animation instead of a sliding
/// still image.
pub fn animate_hero(
    time: Res<Time>,
    anims: Res<AnimationAssets>,
    scene: Option<Res<RunSceneState>>,
    mut hero: Query<(&mut Transform, &mut Sprite, &mut SpriteAnimation), With<SceneHero>>,
) {
    let Some(scene) = scene else { return };
    if scene.col < 0 {
        return;
    }
    let Ok((mut transform, mut sprite, mut animation)) = hero.single_mut() else {
        return;
    };
    let target = tile_to_world(scene.col, scene.row);
    let pos = transform.translation.truncate();
    let delta = Vec2::new(target.x, target.y) - pos;
    let distance = delta.length();
    // Match the grid cadence: one tile (40u) per 0.14s cooldown.
    let step = (TILE / 0.14) * time.delta_secs();
    if distance > step {
        let next = pos + delta.normalize_or_zero() * step;
        transform.translation.x = next.x;
        transform.translation.y = next.y;
        animation::set_clip(&anims, &mut sprite, &mut animation, AnimationClip::HeroWalk);
    } else {
        transform.translation.x = target.x;
        transform.translation.y = target.y;
        if distance <= f32::EPSILON {
            animation::set_clip(&anims, &mut sprite, &mut animation, AnimationClip::HeroIdle);
        }
    }
    sprite.flip_x = scene.facing_left;
}

/// Rise-and-fade for transient map float text (hazard damage etc.).
pub fn animate_scene_float_text(
    mut commands: Commands,
    time: Res<Time>,
    mut texts: Query<(Entity, &mut SceneFloatText, &mut Transform, &mut TextColor)>,
) {
    for (entity, mut float, mut transform, mut color) in &mut texts {
        float.age += time.delta_secs();
        if float.age >= 1.3 {
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation.y += 26.0 * time.delta_secs();
        let alpha = (1.0 - float.age / 1.3).clamp(0.0, 1.0);
        color.0 = color.0.with_alpha(alpha);
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
    mut fills: ParamSet<(
        Query<&mut Node, With<HudHpFill>>,
        Query<&mut Node, With<HudMpFill>>,
    )>,
    mut sub: Query<&mut Text, With<HudSubText>>,
) {
    if let Ok(mut node) = fills.p0().single_mut() {
        node.width = Val::Percent((stats.hp.max(0) as f32 / stats.max_hp.max(1) as f32) * 100.0);
    }
    if let Ok(mut node) = fills.p1().single_mut() {
        node.width = Val::Percent((stats.mp.max(0) as f32 / stats.max_mp.max(1) as f32) * 100.0);
    }
    if let Ok(mut text) = sub.single_mut() {
        text.0 = format!("药水 ×{} · {} 文", stats.potions, stats.gold);
    }
}

// ---------------------------------------------------------------------------
// Camera: zoomed-in follow view while walking
// ---------------------------------------------------------------------------

/// View height in world units while exploring (14 tiles); the camera follows
/// the hero, clamped so the view stays on the bordered map.
const VIEW_HEIGHT: f32 = 560.0;

pub fn zoom_camera_in(
    mut cameras: Query<&mut Projection, With<Camera2d>>,
    mut inventory: ResMut<InventoryOpen>,
) {
    inventory.0 = false;
    for mut projection in &mut cameras {
        if let Projection::Orthographic(ortho) = projection.as_mut() {
            ortho.scaling_mode = ScalingMode::FixedVertical {
                viewport_height: VIEW_HEIGHT,
            };
        }
    }
}

pub fn zoom_camera_out(mut cameras: Query<(&mut Projection, &mut Transform), With<Camera2d>>) {
    for (mut projection, mut transform) in &mut cameras {
        if let Projection::Orthographic(ortho) = projection.as_mut() {
            ortho.scaling_mode = ScalingMode::Fixed {
                width: 1280.0,
                height: 720.0,
            };
        }
        transform.translation.x = 0.0;
        transform.translation.y = 0.0;
    }
}

pub fn camera_follow(
    scene: Option<Res<RunSceneState>>,
    mut cameras: Query<&mut Transform, With<Camera2d>>,
) {
    let Some(scene) = scene else { return };
    if scene.col < 0 {
        return;
    }
    let target = tile_to_world(scene.col, scene.row);
    // Clamp for a 16:9 view of VIEW_HEIGHT: half-extents 497x280 vs the
    // map's 600x320; the wall border covers wider aspect ratios.
    let cx = target.x.clamp(-103.0, 103.0);
    let cy = target.y.clamp(-40.0, 40.0);
    for mut transform in &mut cameras {
        transform.translation.x += (cx - transform.translation.x) * 0.12;
        transform.translation.y += (cy - transform.translation.y) * 0.12;
    }
}

// ---------------------------------------------------------------------------
// Esc inventory (行囊 · 纸娃娃)
// ---------------------------------------------------------------------------

pub fn inventory_toggle(
    mut commands: Commands,
    mut intent: ResMut<Intent>,
    mut inventory: ResMut<InventoryOpen>,
    dialogue: Res<RunDialogue>,
    font: Res<GameFont>,
    asset_server: Res<AssetServer>,
    dolls: Res<PaperdollAssets>,
    stats: Res<PlayerStats>,
    run: Option<Res<RunState>>,
    cameras: Query<&Transform, With<Camera2d>>,
    open_ui: Query<Entity, With<InventoryUi>>,
) {
    if !intent.cancel || dialogue.active {
        return;
    }
    intent.cancel = false;
    let Some(run) = run else { return };

    if inventory.0 {
        for entity in &open_ui {
            commands.entity(entity).despawn();
        }
        inventory.0 = false;
        return;
    }
    inventory.0 = true;

    // World-space dim + the hero paperdoll, anchored to the camera view.
    let cam = cameras
        .iter()
        .next()
        .map(|t| t.translation)
        .unwrap_or(Vec3::ZERO);
    commands.spawn((
        InventoryUi,
        Sprite::from_color(
            Color::srgba(0.01, 0.02, 0.05, 0.82),
            Vec2::new(2400.0, 1400.0),
        ),
        Transform::from_xyz(cam.x, cam.y, 40.0),
        DespawnOnExit(AppState::RunScene),
    ));
    let doll = paperdoll::spawn_paperdoll(
        &mut commands,
        &dolls,
        PaperdollStyle::Hero,
        Vec3::new(cam.x - 235.0, cam.y - 10.0, 50.0),
        250.0,
        AppState::RunScene,
    );
    commands.entity(doll).insert(InventoryUi);
    commands.spawn((
        InventoryUi,
        Text2d::new(stats.name.clone()),
        font.text_font(24.0),
        TextColor(Color::srgb(0.95, 0.88, 0.65)),
        Transform::from_xyz(cam.x - 235.0, cam.y - 160.0, 50.0),
        DespawnOnExit(AppState::RunScene),
    ));

    // Right panel: stats + relics.
    let relics = if run.relics.is_empty() {
        "(尚未寻得法宝)".to_string()
    } else {
        run.relics
            .iter()
            .map(|r| format!("【{}】{}", r.name(), r.desc()))
            .collect::<Vec<_>>()
            .join(
                "
",
            )
    };
    commands
        .spawn((
            InventoryUi,
            DespawnOnExit(AppState::RunScene),
            Node {
                position_type: PositionType::Absolute,
                right: Val::Percent(6.0),
                top: Val::Percent(8.0),
                bottom: Val::Percent(8.0),
                width: Val::Percent(46.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                padding: UiRect::new(Val::Px(30.0), Val::Px(30.0), Val::Px(56.0), Val::Px(24.0)),
                overflow: Overflow::clip_y(),
                ..default()
            },
            ImageNode {
                image: asset_server.load("ui/panel_frame.png"),
                image_mode: NodeImageMode::Sliced(event::panel_slicer()),
                ..default()
            },
            GlobalZIndex(80),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("行囊 · 纸娃娃"),
                font.text_font(28.0),
                TextColor(Color::srgb(0.95, 0.85, 0.55)),
            ));
            panel.spawn((
                Text::new(format!(
                    "气血 {}/{}    灵力 {}/{}
攻击 {}    防御 {}
药水 ×{}    钱财 {} 文
道心 {}    情缘 {}",
                    stats.hp,
                    stats.max_hp,
                    stats.mp,
                    stats.max_mp,
                    stats.atk,
                    stats.def,
                    stats.potions,
                    stats.gold,
                    run.daoxin,
                    run.qingyuan,
                )),
                font.text_font(20.0),
                TextColor(Color::srgb(0.9, 0.92, 0.95)),
            ));
            panel.spawn((
                Text::new(run.milestone_summary()),
                font.text_font(16.0),
                TextColor(Color::srgb(0.85, 0.78, 0.95)),
            ));
            panel.spawn((
                Text::new(format!("法宝({}):", run.relics.len())),
                font.text_font(20.0),
                TextColor(Color::srgb(0.95, 0.85, 0.55)),
            ));
            panel.spawn((
                Text::new(relics),
                font.text_font(17.0),
                TextColor(Color::srgb(0.82, 0.86, 0.94)),
            ));
            panel.spawn((
                Text::new(
                    "
ESC 关闭",
                ),
                font.text_font(15.0),
                TextColor(Color::srgba(0.8, 0.85, 0.9, 0.7)),
            ));
        });
}

//! Map stages (地图关卡) — the roguelike run IS the map.
//!
//! Each chapter is a chain of real walkable tile maps (reusing the authored
//! Explore map topology and art). Every normal stage scatters at least three
//! objective markers (battles / events / story / rest / market / mechanisms);
//! once all are cleared the
//! glowing portal tile leads to the next stage. The chapter's final stage is
//! the boss map: a single demon gate. Battles hop out to `AppState::Battle`
//! and return here through the Reward screen.

use bevy::camera::ScalingMode;
use bevy::prelude::*;

use super::super::animation::{self, AnimationAssets, AnimationClip, SpriteAnimation};
use super::super::battle::{EncounterZone, PendingEncounter};
use super::super::core::{GameFont, Intent, MAP_H, MAP_W, PlayerStats, Rng, TILE, tile_to_world};
use super::super::cutout::{COMPANION_OVERWORLD_SIZE, companion_cutout_path};
use super::super::explore::{
    ExploreAssets, MapData, MapKind, TerrainMaterial, Tile, spawn_terrain_map,
};
use super::super::lighting::{self, LightingAssets};
use super::super::paperdoll::PaperdollAssets;
use super::super::quest::Companion;
use super::event::RunDialogue;
use super::graph::NodeKind;
use super::{FightRank, RunState, battle_mods_for, encounter_kind_for};
use super::{dialogue_ui, puzzle};
use crate::game::state::AppState;
use crate::game::ui as theme;

#[derive(Component, Clone, Copy)]
pub struct SceneArchitecture {
    foot_row: i32,
    top_row: i32,
    left_col: i32,
    right_col: i32,
    walkover: bool,
    shadow: bool,
}

pub fn update_architecture_depth(
    scene: Option<Res<RunSceneState>>,
    mut buildings: Query<(
        &SceneArchitecture,
        &mut Transform,
        &mut Visibility,
        &mut Sprite,
    )>,
) {
    let Some(scene) = scene else { return };
    for (building, mut transform, mut visibility, mut sprite) in &mut buildings {
        let known = (building.top_row..=building.foot_row).any(|row| {
            (building.left_col..=building.right_col).any(|col| run_fog_known(&scene, col, row))
        });
        *visibility = if known {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        transform.translation.z = if building.shadow {
            1.0
        } else if !building.walkover && scene.row < building.foot_row {
            12.0
        } else {
            7.0
        };
        if !building.shadow {
            let covers_hero = !building.walkover
                && scene.row < building.foot_row
                && scene.row >= building.top_row - 3
                && (building.left_col..=building.right_col).contains(&scene.col);
            let alpha = if covers_hero { 0.55 } else { 1.0 };
            if sprite.color.alpha() != alpha {
                sprite.color = sprite.color.with_alpha(alpha);
            }
        }
    }
}

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

impl SceneMarker {
    /// Visited guides stop attracting navigation but remain available to talk.
    pub fn is_visitable(&self) -> bool {
        self.kind == NodeKind::Guide || !self.cleared
    }
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
    /// Exploration memory for this generated stage. Current vision clears the
    /// fog; previously visited cells stay dim instead of becoming hidden again.
    pub revealed: Vec<Vec<bool>>,
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
    /// One-shot authored route intro for this generated stage.
    pub intro_shown: bool,
    pub cooldown: f32,
}

#[derive(Resource)]
pub struct RunSceneMap(pub MapData);

#[derive(Component)]
pub struct SceneHero;

#[derive(Component)]
pub struct RunPartyFollower {
    companion: Companion,
}

#[derive(Component)]
pub struct RunFogTile {
    col: i32,
    row: i32,
}

#[derive(Component)]
pub struct RunFoggedEntity {
    col: i32,
    row: i32,
}

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

/// 章节过场动画播放状态(16fps 顺播,末帧回卷做呼吸循环)。
#[derive(Component, Default)]
pub struct ChapterArtAnim {
    timer: f32,
    frame: usize,
}

/// 逐帧推进章节过场动画图集。
pub fn animate_chapter_art(
    time: Res<Time>,
    mut anims: Query<(&mut ChapterArtAnim, &mut ImageNode)>,
) {
    for (mut anim, mut node) in &mut anims {
        anim.timer += time.delta_secs();
        if anim.timer < 1.0 / 16.0 {
            continue;
        }
        anim.timer = 0.0;
        anim.frame = (anim.frame + 1) % 32;
        if let Some(atlas) = node.texture_atlas.as_mut() {
            atlas.index = anim.frame;
        }
    }
}

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

#[derive(Component)]
pub struct HudVitalsText;

#[derive(Component)]
pub struct MapHudRoot;

#[derive(Component, Clone, Copy)]
pub(super) enum HudPanel {
    Vitals,
    Chapter,
    Objective,
    Controls,
}
#[derive(Component)]
pub(super) struct HudFont {
    base: f32,
    hint: bool,
}
#[derive(Component)]
pub(super) struct HudPortrait;

#[derive(Component)]
pub struct HudTaskText;

/// The Esc inventory (行囊/修行录) overlay is open; movement pauses.
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

fn route_contact_asset(map: MapKind) -> &'static str {
    match map {
        MapKind::Village => "npcs/ai_herb_healer.png",
        MapKind::Bamboo => "npcs/ai_bamboo_scout.png",
        MapKind::Cave | MapKind::MoonEchoCorridor => "npcs/ai_cave_priestess.png",
        MapKind::RiverTown | MapKind::RiverReedBed => "npcs/ai_wandering_merchant.png",
        MapKind::PlagueVillage => "npcs/ai_plague_elder.png",
        MapKind::PlagueShrinePath => "npcs/ai_shrine_keeper.png",
        MapKind::Capital | MapKind::CapitalMansion => "npcs/ai_capital_envoy.png",
        MapKind::MansionMirrorGallery => "npcs/ai_mansion_spy.png",
        MapKind::SouthernRoad | MapKind::ThunderDrumPath => "npcs/ai_tribal_chief.png",
        MapKind::FinalSanctum | MapKind::DreamWaterway => "npcs/ai_final_oracle.png",
    }
}

fn route_contact_name(map: MapKind) -> &'static str {
    match map {
        MapKind::Village => "采药婶",
        MapKind::Bamboo => "竹林斥候",
        MapKind::Cave | MapKind::MoonEchoCorridor => "洞天守阵人",
        MapKind::RiverTown | MapKind::RiverReedBed => "夜渡货郎",
        MapKind::PlagueVillage => "疫村长者",
        MapKind::PlagueShrinePath => "荒寺守铃人",
        MapKind::Capital | MapKind::CapitalMansion => "京城暗使",
        MapKind::MansionMirrorGallery => "府中线人",
        MapKind::SouthernRoad | MapKind::ThunderDrumPath => "南疆鼓使",
        MapKind::FinalSanctum | MapKind::DreamWaterway => "旧梦引路人",
    }
}

fn route_contact_clue(map: MapKind) -> &'static str {
    match map {
        MapKind::Village => "村里夜路不是没人走,只是人人都绕着湿脚印走。",
        MapKind::Bamboo => "十里坡风向变了,竹叶倒伏的地方多半藏着妖踪。",
        MapKind::Cave | MapKind::MoonEchoCorridor => "洞天灵阵听脚步声认人,别让妖气先替你报到。",
        MapKind::RiverTown | MapKind::RiverReedBed => {
            "江灯若逆水走,说明水底有人牵线,先看灯影再看路。"
        }
        MapKind::PlagueVillage => "病屋灯火忽明忽暗,药路和水脉要一起查。",
        MapKind::PlagueShrinePath => "荒寺铃声若停,瘴火便会贴着石阶往上爬。",
        MapKind::Capital | MapKind::CapitalMansion => "京城明处问不到实话,府门影子比正门更会说话。",
        MapKind::MansionMirrorGallery => "镜廊里别只信眼睛,脚下没有回声的倒影才是假路。",
        MapKind::SouthernRoad | MapKind::ThunderDrumPath => {
            "南疆雷鼓分风、云、誓三声,抢在鼓前走的人都会迷路。"
        }
        MapKind::FinalSanctum | MapKind::DreamWaterway => {
            "旧梦不会拦人,只会把最舍不得的那一刻放在路中间。"
        }
    }
}

fn route_commission_target(scene: &RunSceneState) -> Option<NodeKind> {
    const PRIORITY: [NodeKind; 5] = [
        NodeKind::Puzzle,
        NodeKind::Story,
        NodeKind::Event,
        NodeKind::Rest,
        NodeKind::Market,
    ];
    PRIORITY.into_iter().find(|kind| {
        scene
            .markers
            .iter()
            .any(|marker| !marker.cleared && marker.kind == *kind)
    })
}

fn route_commission_reward_preview(map: MapKind) -> &'static str {
    match map {
        MapKind::Village | MapKind::Bamboo | MapKind::Cave | MapKind::MoonEchoCorridor => {
            "包扎伤处，回复气血上限的12%（不超过缺损气血）"
        }
        MapKind::RiverTown | MapKind::RiverReedBed => "渡船盘缠25文",
        MapKind::PlagueVillage | MapKind::PlagueShrinePath => "应急药水1瓶",
        MapKind::Capital | MapKind::CapitalMansion | MapKind::MansionMirrorGallery => {
            "线索酬银35文"
        }
        MapKind::SouthernRoad | MapKind::ThunderDrumPath => "调息回复8点灵力（不超过缺损灵力）",
        MapKind::FinalSanctum | MapKind::DreamWaterway => {
            "回复气血上限的8%及6点灵力（不超过各自缺损）"
        }
    }
}

fn route_commission_reason(target: NodeKind) -> &'static str {
    match target {
        NodeKind::Puzzle => "破解路上的机关，让后来的人也能安全通过",
        NodeKind::Story => "听完当地人的心事，并作出你的回应",
        NodeKind::Event => "查明路上的异事，并作出你的选择",
        NodeKind::Rest => "在营地安顿队伍，选好下一战的营策",
        NodeKind::Market => "拜访集市了解补给，再向摊主告辞；不强求购买",
        _ => "查明前路情况",
    }
}

fn route_contact_dialogue(
    run: &RunState,
    map: MapKind,
    target: Option<NodeKind>,
) -> (String, Vec<String>, &'static str) {
    let beat = run.journey_beat();
    let name = route_contact_name(map);
    let title = format!("路人委托 · {name}");
    let mut lines = vec![
        format!("{name}在「{}」附近拦住你,递来一条地方线索。", beat.place),
        format!("「{}」", route_contact_clue(map)),
        format!(
            "【路人签】{} · {}",
            run.route_commission_receipt(),
            beat.title
        ),
    ];
    if let Some(target) = target {
        lines.push(format!(
            "【委托目标】{}：{}。",
            target.label(),
            route_commission_reason(target)
        ));
    } else {
        lines.push("【委托目标】这一路能托付的事已经差不多了,只剩一句地方提醒。".to_string());
    }
    lines.push(format!(
        "【约定酬谢】{}。办妥后发放；委托不挡主线界门。",
        route_commission_reward_preview(map)
    ));
    lines.push("【路况照应】每份回执还会改善本卷路况，并助你准备章末首领战。".into());
    if run.is_route_commission_completed() {
        lines.push("【已回执】这份委托已经办妥，酬谢已交付，不会再次发放。".into());
    } else if run.is_route_commission_active() {
        lines.push(format!(
            "【已接取】{}；办妥后自动回执。",
            run.route_commission_state_label()
        ));
    } else {
        lines.push("暂不接取也无妨；之后回到此处仍可领取。".into());
    }
    (title, lines, route_contact_asset(map))
}

fn grant_route_commission_reward(
    stats: &mut PlayerStats,
    map: MapKind,
    target: NodeKind,
) -> String {
    let target = target.label();
    match map {
        MapKind::Village | MapKind::Bamboo | MapKind::Cave | MapKind::MoonEchoCorridor => {
            let heal = (stats.max_hp * 12 / 100).max(1);
            let before = stats.hp;
            stats.hp = (stats.hp + heal).min(stats.max_hp);
            let healed = stats.hp - before;
            format!("【路人委托回执】{target} 已处理,对方替你包扎伤处,实际恢复 {healed} 点气血。")
        }
        MapKind::RiverTown | MapKind::RiverReedBed => {
            stats.gold += 25;
            format!("【路人委托回执】{target} 已处理,对方塞来一把船钱,钱财 +25。")
        }
        MapKind::PlagueVillage | MapKind::PlagueShrinePath => {
            stats.potions += 1;
            format!(
                "【路人委托回执】{target} 已处理,对方分出一瓶应急药水。药水 x{}",
                stats.potions
            )
        }
        MapKind::Capital | MapKind::CapitalMansion | MapKind::MansionMirrorGallery => {
            stats.gold += 35;
            format!("【路人委托回执】{target} 已处理,线人留下封口银,钱财 +35。")
        }
        MapKind::SouthernRoad | MapKind::ThunderDrumPath => {
            let gain = 8;
            let before = stats.mp;
            stats.mp = (stats.mp + gain).min(stats.max_mp);
            let restored = stats.mp - before;
            format!("【路人委托回执】{target} 已处理,鼓使替你稳住灵息,实际回复灵力 {restored}。")
        }
        MapKind::FinalSanctum | MapKind::DreamWaterway => {
            let heal = (stats.max_hp * 8 / 100).max(1);
            let mp = 6;
            let before_hp = stats.hp;
            let before_mp = stats.mp;
            stats.hp = (stats.hp + heal).min(stats.max_hp);
            stats.mp = (stats.mp + mp).min(stats.max_mp);
            let healed = stats.hp - before_hp;
            let restored = stats.mp - before_mp;
            format!(
                "【路人委托回执】{target} 已处理,旧梦微光护住心神,实际回复气血 {healed},灵力 {restored}。"
            )
        }
    }
}

fn route_commission_completion_line(
    stats: &mut PlayerStats,
    run: &mut RunState,
    map: MapKind,
    kind: NodeKind,
) -> Option<String> {
    if run.complete_route_commission(kind) {
        Some(grant_route_commission_reward(stats, map, kind))
    } else {
        None
    }
}

fn push_route_commission_completion_feedback(
    lines: &mut Vec<String>,
    stats: &mut PlayerStats,
    run: &mut RunState,
    map: MapKind,
    kind: NodeKind,
) {
    if let Some(line) = route_commission_completion_line(stats, run, map, kind) {
        lines.push(line);
        lines.push(format!("【路况照应】{}", run.route_guidance_summary()));
        lines.push(format!("【累计】路人回声 {}", run.route_contacts_helped));
    }
}

/// Called only after an event/story/rest choice succeeds or a market visit ends.
fn route_puzzle_enabled(map: MapKind) -> bool {
    matches!(
        map,
        MapKind::MoonEchoCorridor
            | MapKind::RiverReedBed
            | MapKind::PlagueShrinePath
            | MapKind::MansionMirrorGallery
            | MapKind::ThunderDrumPath
            | MapKind::DreamWaterway
    )
}

fn route_puzzle_asset(map: MapKind) -> &'static str {
    match map {
        MapKind::MoonEchoCorridor => "props/ai_cave_crystal.png",
        MapKind::RiverReedBed => "props/ai_spirit_lantern.png",
        MapKind::PlagueShrinePath => "props/ai_shrine_statue.png",
        MapKind::MansionMirrorGallery => "props/ai_spirit_lantern.png",
        MapKind::ThunderDrumPath => "props/ai_bamboo_gate.png",
        MapKind::DreamWaterway => "props/ai_spirit_lantern.png",
        _ => "props/ai_cave_crystal.png",
    }
}

fn grant_route_puzzle_reward(stats: &mut PlayerStats, run: &mut RunState, map: MapKind) -> String {
    run.record_route_puzzle();
    match map {
        MapKind::MoonEchoCorridor => {
            stats.max_mp += 3;
            stats.mp += 3;
            "【机关回响】月华入脉,灵力上限 +3。".to_string()
        }
        MapKind::RiverReedBed => {
            stats.gold += 18;
            "【机关回响】河灯带来渡钱,钱财 +18。".to_string()
        }
        MapKind::PlagueShrinePath => {
            stats.potions += 1;
            format!("【机关回响】铃心落下一瓶清瘴药。药水 x{}", stats.potions)
        }
        MapKind::MansionMirrorGallery => {
            run.daoxin += 1;
            "【机关回响】镜影退散,道心 +1。".to_string()
        }
        MapKind::ThunderDrumPath => {
            stats.atk += 1;
            "【机关回响】雷鼓淬剑,攻击 +1。".to_string()
        }
        MapKind::DreamWaterway => {
            run.qingyuan += 1;
            let heal = (stats.max_hp * 10 / 100).max(1);
            let before = stats.hp;
            stats.hp = (stats.hp + heal).min(stats.max_hp);
            format!(
                "【机关回响】旧梦灯暖了一瞬,情缘 +1,气血 +{}。",
                stats.hp - before
            )
        }
        _ => "【机关回响】道路重新安静下来。".to_string(),
    }
}

const RUN_VISIBLE_RADIUS: f32 = 4.25;
const RUN_SEEN_FOG_ALPHA: f32 = 0.30;
const RUN_UNSEEN_FOG_ALPHA: f32 = 0.60;

fn ensure_run_fog(scene: &mut RunSceneState) {
    let valid = scene.revealed.len() == MAP_H as usize
        && scene.revealed.iter().all(|row| row.len() == MAP_W as usize);
    if !valid {
        scene.revealed = vec![vec![false; MAP_W as usize]; MAP_H as usize];
    }
}

fn run_fog_visible(scene: &RunSceneState, col: i32, row: i32) -> bool {
    if scene.col < 0 || scene.row < 0 {
        return false;
    }
    let dx = (col - scene.col) as f32;
    let dy = (row - scene.row) as f32;
    dx * dx + dy * dy <= RUN_VISIBLE_RADIUS * RUN_VISIBLE_RADIUS
}

fn run_fog_revealed(scene: &RunSceneState, col: i32, row: i32) -> bool {
    if !(0..MAP_W).contains(&col) || !(0..MAP_H).contains(&row) {
        return false;
    }
    scene
        .revealed
        .get(row as usize)
        .and_then(|row| row.get(col as usize))
        .copied()
        .unwrap_or(false)
}

fn run_fog_alpha(scene: &RunSceneState, col: i32, row: i32) -> f32 {
    if run_fog_visible(scene, col, row) {
        0.0
    } else if run_fog_revealed(scene, col, row) {
        RUN_SEEN_FOG_ALPHA
    } else {
        RUN_UNSEEN_FOG_ALPHA
    }
}

fn run_fog_known(scene: &RunSceneState, col: i32, row: i32) -> bool {
    run_fog_visible(scene, col, row) || run_fog_revealed(scene, col, row)
}

fn run_fog_entity_visibility(scene: &RunSceneState, col: i32, row: i32) -> Visibility {
    if run_fog_known(scene, col, row) {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    }
}

fn reveal_run_fog(scene: &mut RunSceneState) -> bool {
    if scene.col < 0 || scene.row < 0 {
        return false;
    }
    ensure_run_fog(scene);
    let mut changed = false;
    let radius2 = RUN_VISIBLE_RADIUS * RUN_VISIBLE_RADIUS;
    let min_col = (scene.col - RUN_VISIBLE_RADIUS.ceil() as i32).max(0);
    let max_col = (scene.col + RUN_VISIBLE_RADIUS.ceil() as i32).min(MAP_W - 1);
    let min_row = (scene.row - RUN_VISIBLE_RADIUS.ceil() as i32).max(0);
    let max_row = (scene.row + RUN_VISIBLE_RADIUS.ceil() as i32).min(MAP_H - 1);

    for row in min_row..=max_row {
        for col in min_col..=max_col {
            let dx = (col - scene.col) as f32;
            let dy = (row - scene.row) as f32;
            if dx * dx + dy * dy > radius2 {
                continue;
            }
            let cell = &mut scene.revealed[row as usize][col as usize];
            if !*cell {
                *cell = true;
                changed = true;
            }
        }
    }
    changed
}

fn spawn_run_fog(commands: &mut Commands, scene: &RunSceneState) {
    let scope = || DespawnOnExit(AppState::RunScene);
    for row in 0..MAP_H {
        for col in 0..MAP_W {
            let p = tile_to_world(col, row);
            commands.spawn((
                RunFogTile { col, row },
                Sprite::from_color(
                    Color::srgba(0.01, 0.02, 0.035, run_fog_alpha(scene, col, row)),
                    Vec2::splat(TILE),
                ),
                Transform::from_xyz(p.x, p.y, 18.0),
                scope(),
            ));
        }
    }
}

pub fn update_run_fog(
    scene: Option<ResMut<RunSceneState>>,
    mut tiles: Query<(&RunFogTile, &mut Sprite)>,
    mut fogged: Query<(&RunFoggedEntity, &mut Visibility)>,
) {
    let Some(mut scene) = scene else { return };
    reveal_run_fog(&mut scene);
    for (tile, mut sprite) in &mut tiles {
        sprite.color = Color::srgba(0.01, 0.02, 0.035, run_fog_alpha(&scene, tile.col, tile.row));
    }
    for (fogged, mut visibility) in &mut fogged {
        *visibility = run_fog_entity_visibility(&scene, fogged.col, fogged.row);
    }
}

/// Taking the pending receipt makes repeated settlement harmless.
pub(super) fn settle_route_commission(
    dialogue: &mut RunDialogue,
    stats: &mut PlayerStats,
    run: &mut RunState,
) {
    if let Some((map, kind)) = dialogue.pending_commission.take() {
        push_route_commission_completion_feedback(&mut dialogue.lines, stats, run, map, kind);
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

#[path = "scene_layout.rs"]
mod layout;
#[path = "scene_places.rs"]
mod places;
use layout::generate_stage_tiles;
#[cfg(test)]
use layout::stage_exploration_span;

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

fn place_marker(
    markers: &mut Vec<SceneMarker>,
    candidates: &[(i32, i32)],
    rng: &mut Rng,
    kind: NodeKind,
    min_gap: i32,
) -> bool {
    if candidates.is_empty() {
        return false;
    }

    let gaps = [min_gap, min_gap.min(5), 3, 0];
    for gap in gaps {
        for _ in 0..400 {
            let pick = candidates[rng.range(0, candidates.len() as i32 - 1) as usize];
            let spread = markers
                .iter()
                .all(|m| (m.col - pick.0).abs() + (m.row - pick.1).abs() >= gap);
            if spread {
                markers.push(SceneMarker {
                    kind,
                    col: pick.0,
                    row: pick.1,
                    cleared: false,
                });
                return true;
            }
        }

        if let Some(&(col, row)) = candidates.iter().find(|&&(col, row)| {
            markers
                .iter()
                .all(|m| (m.col - col).abs() + (m.row - row).abs() >= gap)
        }) {
            markers.push(SceneMarker {
                kind,
                col,
                row,
                cleared: false,
            });
            return true;
        }
    }

    false
}

/// Build the next stage into `RunSceneState` and enter the scene.
pub fn advance_stage(
    mut commands: Commands,
    run: Option<ResMut<RunState>>,
    mut stats: ResMut<PlayerStats>,
    mut rng: ResMut<Rng>,
    mut next: ResMut<NextState<AppState>>,
) {
    let Some(mut run) = run else {
        next.set(AppState::Title);
        return;
    };
    // 贪泉纹的代价:每程入图先失血。
    let toll = run.hex_stage_hp_loss();
    if toll > 0 {
        stats.hp = (stats.hp - toll).max(1);
    }
    run.accept_journey_task();
    let map_kind = run.roll_map();
    let layout_variant = run.chapter * 16 + run.stage;
    let mut tiles = generate_stage_tiles(map_kind, layout_variant, &mut rng);

    // A recognizable road entrance is preferable to a random courtyard corner.
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
    if tiles[(MAP_H / 2) as usize][2].walkable() {
        spawn = (2, MAP_H / 2);
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
    let architecture = places::landmarks(map_kind, layout_variant);
    for r in 0..MAP_H {
        for c in 0..MAP_W {
            let d = from_spawn[r as usize][c as usize];
            let dp = (c - portal.0).abs() + (r - portal.1).abs();
            if d != u16::MAX
                && d >= 6
                && dp >= 4
                && tiles[r as usize][c as usize].walkable()
                && !architecture
                    .iter()
                    .any(|building| building.overhang_contains(c, r))
            {
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
        let planned_markers = run.journey_marker_plan().to_vec();
        for kind in planned_markers.iter().copied() {
            if kind == NodeKind::Boss {
                continue;
            }
            place_marker(&mut markers, &candidates, &mut rng, kind, 8);
        }

        // A small amount of surprise preserves replay variance without
        // overriding the authored journey pacing.
        if markers.len() < 4 && rng.chance(0.30) {
            let kind = roll_marker_kind(&mut rng, run.stage);
            place_marker(&mut markers, &candidates, &mut rng, kind, 6);
        }

        if route_puzzle_enabled(map_kind) {
            place_marker(&mut markers, &candidates, &mut rng, NodeKind::Puzzle, 7);
        }

        // Optional visible route life: one local contact plus loot / spring
        // detours. They add RPG texture and supplies, but never block the gate.
        place_marker(&mut markers, &candidates, &mut rng, NodeKind::Guide, 6);
        for (kind, chance) in [(NodeKind::Chest, 0.55), (NodeKind::Spring, 0.40)] {
            if !rng.chance(chance) {
                continue;
            }
            place_marker(&mut markers, &candidates, &mut rng, kind, 5);
        }
    }

    // The first story must be visible before a newcomer has learned the map.
    if run.chapter == 0 && run.stage == 0 {
        if let Some(index) = markers.iter().position(|m| m.kind == NodeKind::Story) {
            let nearby = (1..MAP_H - 1)
                .flat_map(|r| (1..MAP_W - 1).map(move |c| (c, r)))
                .find(|&(c, r)| {
                    let distance = from_spawn[r as usize][c as usize];
                    (2..=3).contains(&distance)
                        && tiles[r as usize][c as usize].walkable()
                        && !markers.iter().any(|m| (m.col, m.row) == (c, r))
                        && (c, r) != portal
                });
            if let Some((col, row)) = nearby {
                markers[index].col = col;
                markers[index].row = row;
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
        revealed: Vec::new(),
        col: spawn.0,
        row: spawn.1,
        facing_left: false,
        markers,
        portals,
        flow: Vec::new(),
        hazards,
        intro_shown: false,
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

pub(crate) fn spawn_run_scene(
    mut commands: Commands,
    font: Res<GameFont>,
    assets: Res<ExploreAssets>,
    anims: Res<AnimationAssets>,
    dolls: Res<PaperdollAssets>,
    lights: Res<LightingAssets>,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut terrain_materials: ResMut<Assets<TerrainMaterial>>,
    mut images: ResMut<Assets<Image>>,
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

    // Gameplay remains tile-based, but one map-level material samples all
    // terrain in world space and feathers wall/grass/water boundaries.
    spawn_terrain_map(
        &mut commands,
        &mut meshes,
        &mut terrain_materials,
        &mut images,
        &assets,
        &map,
        0.62,
        AppState::RunScene,
    );

    places::spawn(
        &mut commands,
        &asset_server,
        &scene,
        run.chapter * 16 + run.stage,
    );

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
    ensure_run_fog(&mut scene);
    reveal_run_fog(&mut scene);
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
    let shadow = theme::contact_shadow(
        &mut commands,
        &asset_server,
        Vec3::new(p.x, p.y - 29.0, 8.5),
        44.0,
        AppState::RunScene,
    );
    commands.entity(shadow).insert(theme::ShadowAnchor {
        actor: hero,
        offset: Vec2::new(0.0, -29.0),
    });
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
        if !marker.is_visitable() {
            continue;
        }
        spawn_marker_visual(
            &mut commands,
            &font,
            &dolls,
            &lights,
            &asset_server,
            &scene,
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
            RunFoggedEntity { col: c, row: r },
            run_fog_entity_visibility(&scene, c, r),
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
            RunFoggedEntity { col: c, row: r },
            run_fog_entity_visibility(&scene, c, r),
            Transform::from_xyz(p.x, p.y + 18.0, 9.0),
            scope(),
        ));
        let light = lighting::spawn_light(
            &mut commands,
            &lights,
            Vec3::new(p.x, p.y, 4.0),
            TILE * 3.2,
            Color::srgba(0.45, 0.95, 1.0, 0.42),
            AppState::RunScene,
        );
        commands.entity(light).insert((
            RunFoggedEntity { col: c, row: r },
            run_fog_entity_visibility(&scene, c, r),
        ));
    }

    // Fog of war: unknown cells sit above landmarks and props, while the
    // hero's local light keeps the playable area clear.
    spawn_run_fog(&mut commands, &scene);

    // Compact vitals, with one objective tracker; full receipts live in the inventory.
    commands
        .spawn((
            theme::panel(Node {
                position_type: PositionType::Absolute,
                top: Val::Px(18.0),
                left: Val::Px(18.0),
                width: Val::Px(310.0),
                padding: UiRect::all(Val::Px(14.0)),
                column_gap: Val::Px(12.0),
                align_items: AlignItems::Center,
                ..default()
            }),
            MapHudRoot,
            HudPanel::Vitals,
            scope(),
        ))
        .with_children(|hud| {
            hud.spawn((
                HudPortrait,
                ImageNode::new(asset_server.load("npcs/ai_hero.png")),
                Node {
                    width: Val::Px(56.0),
                    height: Val::Px(56.0),
                    flex_shrink: 0.0,
                    ..default()
                },
            ));
            hud.spawn(Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                row_gap: Val::Px(6.0),
                ..default()
            })
            .with_children(|col| {
                col.spawn((
                    HudVitalsText,
                    Text::new(""),
                    HudFont {
                        base: 14.0,
                        hint: false,
                    },
                    font.text_font(14.0),
                    TextColor(theme::TEXT),
                ));
                col.spawn(theme::bar_track(Val::Percent(100.0), 8.0))
                    .with_children(|b| {
                        b.spawn((HudHpFill, theme::bar_fill(theme::HEALTH)));
                    });
                col.spawn(theme::bar_track(Val::Percent(100.0), 6.0))
                    .with_children(|b| {
                        b.spawn((HudMpFill, theme::bar_fill(theme::SPIRIT)));
                    });
                col.spawn((
                    HudSubText,
                    Text::new(""),
                    HudFont {
                        base: 13.0,
                        hint: false,
                    },
                    font.text_font(13.0),
                    TextColor(theme::MUTED),
                ));
            });
        });
    let beat = run.journey_beat();
    commands
        .spawn((
            theme::panel(Node {
                position_type: PositionType::Absolute,
                top: Val::Px(18.0),
                right: Val::Px(18.0),
                max_width: Val::Percent(42.0),
                padding: UiRect::axes(Val::Px(18.0), Val::Px(12.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(5.0),
                ..default()
            }),
            MapHudRoot,
            HudPanel::Chapter,
            scope(),
        ))
        .with_children(|banner| {
            banner.spawn((
                Text::new(run.chapter_def().title),
                HudFont {
                    base: 19.0,
                    hint: false,
                },
                font.text_font(19.0),
                TextColor(theme::GOLD),
            ));
            banner.spawn((
                Text::new(format!(
                    "{} · {} · {}/{}",
                    beat.title,
                    beat.place,
                    run.stage + 1,
                    run.stage_count()
                )),
                HudFont {
                    base: 14.0,
                    hint: false,
                },
                font.text_font(14.0),
                TextColor(theme::MUTED),
            ));
        });
    commands
        .spawn((
            theme::panel(Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                left: Val::Px(18.0),
                max_width: Val::Percent(48.0),
                padding: UiRect::axes(Val::Px(18.0), Val::Px(12.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                ..default()
            }),
            MapHudRoot,
            HudPanel::Objective,
            scope(),
        ))
        .with_children(|p| {
            p.spawn((
                Text::new(beat.objective),
                HudFont {
                    base: 17.0,
                    hint: false,
                },
                font.text_font(17.0),
                TextColor(theme::TEXT),
            ));
            p.spawn((
                HudTaskText,
                Text::new(""),
                HudFont {
                    base: 14.0,
                    hint: false,
                },
                font.text_font(14.0),
                TextColor(theme::GOLD),
            ));
        });
    commands.spawn((
        Text::new("WASD / 方向键移动 · 空格 / 回车交互 · Esc 行囊"),
        HudFont {
            base: 14.0,
            hint: true,
        },
        font.text_font(14.0),
        TextColor(theme::TEXT),
        theme::panel(Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            right: Val::Px(18.0),
            padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
            ..default()
        }),
        MapHudRoot,
        HudPanel::Controls,
        scope(),
    ));

    // Dialogue overlay (events/story/rest/market/chapter card resolve here).
    dialogue_ui::spawn(&mut commands, &font, scope());

    // First stage of a chapter: show the chapter card over a full-screen
    // chapter painting.
    if !run.card_shown && run.stage == 0 {
        run.card_shown = true;
        let chapter = run.chapter;
        let card = chapter.min(super::content::CHAPTER_CARDS.len() - 1);
        dialogue.open_plain(
            run.chapter_def().title,
            &super::content::CHAPTER_CARDS[card][1..],
        );
        // 静态过场画兜底(动画图集加载失败时仍有画面)。
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
            ImageNode::new(asset_server.load(format!("ui/chapter{}_art.png", card + 1))),
            GlobalZIndex(40),
        ));
        // Wan2.2 i2v 生成的 32 帧过场动画(8×4 图集,640×352/帧,16fps 循环)。
        let layout = layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(640, 352),
            8,
            4,
            None,
            None,
        ));
        commands.spawn((
            ChapterArt,
            ChapterArtAnim::default(),
            scope(),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                ..default()
            },
            ImageNode::from_atlas_image(
                asset_server.load(format!("ui/anim/chapter{}_sheet.png", card + 1)),
                TextureAtlas { layout, index: 0 },
            ),
            GlobalZIndex(41),
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
    scene: &RunSceneState,
    index: usize,
    marker: &SceneMarker,
) {
    let scope = || DespawnOnExit(AppState::RunScene);
    let op = tile_to_world(marker.col, marker.row);
    let fogged = || {
        (
            RunFoggedEntity {
                col: marker.col,
                row: marker.row,
            },
            run_fog_entity_visibility(scene, marker.col, marker.row),
        )
    };

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
            fogged(),
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
            fogged(),
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
        commands.entity(light).insert((
            SceneMarkerVisual(index),
            RunFoggedEntity {
                col: marker.col,
                row: marker.row,
            },
            run_fog_entity_visibility(scene, marker.col, marker.row),
        ));
        return;
    }

    if marker.kind == NodeKind::Puzzle {
        commands.spawn((
            SceneMarkerVisual(index),
            Sprite {
                image: asset_server.load(route_puzzle_asset(scene.map)),
                color: Color::srgb(1.0, 0.94, 0.74),
                custom_size: Some(Vec2::new(58.0, 58.0)),
                ..default()
            },
            fogged(),
            Transform::from_xyz(op.x, op.y + 8.0, 9.0),
            scope(),
        ));
        commands.spawn((
            SceneMarkerVisual(index),
            SceneMarkerGlyph {
                base_y: op.y + 50.0,
            },
            Text2d::new("机"),
            font.text_font(23.0),
            TextColor(Color::srgb(0.96, 0.95, 0.70)),
            fogged(),
            Transform::from_xyz(op.x, op.y + 50.0, 11.0),
            scope(),
        ));
        let light = lighting::spawn_light(
            commands,
            lights,
            Vec3::new(op.x, op.y, 4.0),
            TILE * 3.0,
            Color::srgba(1.0, 0.88, 0.38, 0.44),
            AppState::RunScene,
        );
        commands.entity(light).insert((
            SceneMarkerVisual(index),
            RunFoggedEntity {
                col: marker.col,
                row: marker.row,
            },
            run_fog_entity_visibility(scene, marker.col, marker.row),
        ));
        return;
    }

    if marker.kind == NodeKind::Guide {
        commands.spawn((
            SceneMarkerVisual(index),
            Sprite {
                image: asset_server.load(route_contact_asset(scene.map)),
                custom_size: Some(Vec2::new(56.0, 68.0)),
                ..default()
            },
            fogged(),
            Transform::from_xyz(op.x, op.y + 14.0, 9.0),
            scope(),
        ));
        commands.spawn((
            SceneMarkerVisual(index),
            SceneMarkerGlyph {
                base_y: op.y + 56.0,
            },
            Text2d::new("人"),
            font.text_font(22.0),
            TextColor(Color::srgb(1.0, 0.88, 0.56)),
            fogged(),
            Transform::from_xyz(op.x, op.y + 56.0, 11.0),
            scope(),
        ));
        let light = lighting::spawn_light(
            commands,
            lights,
            Vec3::new(op.x, op.y, 4.0),
            TILE * 2.8,
            Color::srgba(1.0, 0.78, 0.36, 0.42),
            AppState::RunScene,
        );
        commands.entity(light).insert((
            SceneMarkerVisual(index),
            RunFoggedEntity {
                col: marker.col,
                row: marker.row,
            },
            run_fog_entity_visibility(scene, marker.col, marker.row),
        ));
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
            fogged(),
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
        commands.entity(light).insert((
            SceneMarkerVisual(index),
            RunFoggedEntity {
                col: marker.col,
                row: marker.row,
            },
            run_fog_entity_visibility(scene, marker.col, marker.row),
        ));
        return;
    }

    let mist = if marker.kind.optional() {
        Color::srgba(0.35, 0.78, 0.75, 0.60)
    } else {
        Color::srgba(0.88, 0.70, 0.35, 0.65)
    };
    let symbol = match marker.kind {
        NodeKind::Fight => "ui/icons/attack.png",
        NodeKind::Elite => "ui/icons/combo.png",
        NodeKind::Rest => "ui/icons/guard.png",
        NodeKind::Market => "ui/icons/item.png",
        _ => "ui/icons/spell.png",
    };
    commands.spawn((
        SceneMarkerVisual(index),
        Sprite {
            image: asset_server.load(symbol),
            color: mist.with_alpha(0.85),
            custom_size: Some(Vec2::splat(58.0)),
            ..default()
        },
        fogged(),
        Transform::from_xyz(op.x, op.y + 4.0, 9.0),
        scope(),
    ));
    commands.spawn((
        SceneMarkerVisual(index),
        SceneMarkerGlyph {
            base_y: op.y + 42.0,
        },
        Text2d::new(format!(
            "{} · {}",
            if marker.kind.optional() {
                "可选"
            } else {
                "目标"
            },
            marker.kind.label()
        )),
        font.text_font(15.0),
        TextColor(Color::srgb(0.90, 0.86, 1.0)),
        fogged(),
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
    commands.entity(light).insert((
        SceneMarkerVisual(index),
        RunFoggedEntity {
            col: marker.col,
            row: marker.row,
        },
        run_fog_entity_visibility(scene, marker.col, marker.row),
    ));
}

// ---------------------------------------------------------------------------
// Movement + payload dispatch
// ---------------------------------------------------------------------------

fn mandatory_marker_progress(scene: &RunSceneState) -> (usize, usize) {
    let total = scene.markers.iter().filter(|m| !m.kind.optional()).count();
    let cleared = scene
        .markers
        .iter()
        .filter(|m| !m.kind.optional() && m.cleared)
        .count();
    (cleared, total)
}

fn journey_task_condition(run: &RunState, scene: &RunSceneState) -> String {
    if run.is_journey_task_completed() {
        return "此地已完成，继续前往下一处。".into();
    }
    let (cleared, total) = mandatory_marker_progress(scene);
    if run.is_boss_stage() {
        return "靠近首领门，按空格迎战。".into();
    }
    if cleared >= total {
        return "此地目标已完成，走入青色界门继续。".into();
    }
    let mut kinds = Vec::new();
    for marker in scene
        .markers
        .iter()
        .filter(|m| !m.cleared && !m.kind.optional())
    {
        let name = marker.kind.label();
        if !kinds.contains(&name) {
            kinds.push(name);
        }
    }
    format!("已完成 {cleared}/{total} · 尚待{}", kinds.join("、"))
}

fn nearby_marker(scene: &RunSceneState) -> Option<usize> {
    scene
        .markers
        .iter()
        .enumerate()
        .filter(|(_, marker)| marker.is_visitable())
        .filter_map(|(index, marker)| {
            let distance = (scene.col - marker.col).abs() + (scene.row - marker.row).abs();
            (distance <= 1).then_some((index, distance))
        })
        .min_by_key(|(_, distance)| *distance)
        .map(|(index, _)| index)
}

fn journey_task_status_line(run: &RunState, scene: &RunSceneState) -> String {
    let action = nearby_marker(scene)
        .map(|index| format!("空格：{}", scene.markers[index].kind.label()))
        .unwrap_or_else(|| {
            let target = scene
                .markers
                .iter()
                .filter(|m| !m.cleared && !m.kind.optional())
                .min_by_key(|m| (m.col - scene.col).abs() + (m.row - scene.row).abs())
                .map(|m| (m.col, m.row, m.kind.label()))
                .or_else(|| scene.portals.first().map(|&(c, r)| (c, r, "界门")));
            target
                .map(|(c, r, name)| {
                    let north_south = match (r - scene.row).signum() {
                        -1 => "北",
                        1 => "南",
                        _ => "",
                    };
                    let east_west = match (c - scene.col).signum() {
                        -1 => "西",
                        1 => "东",
                        _ => "",
                    };
                    format!("{name}在{east_west}{north_south}方 · 靠近后按空格交互")
                })
                .unwrap_or_else(|| "靠近地标后按空格交互".into())
        });
    format!("{}\n{action}", journey_task_condition(run, scene))
}

pub fn open_journey_intro(
    run: Option<Res<RunState>>,
    scene: Option<ResMut<RunSceneState>>,
    mut dialogue: ResMut<RunDialogue>,
) {
    let (Some(run), Some(mut scene)) = (run, scene) else {
        return;
    };
    if dialogue.active || scene.intro_shown {
        return;
    }

    scene.intro_shown = true;
    let beat = run.journey_beat();
    let intro = run.journey_intro();
    dialogue.open_plain_owned(
        &beat.title,
        vec![
            intro.scene_line.into(),
            intro.party_line.into(),
            format!("此行：{}", beat.objective),
        ],
    );
}

pub fn run_scene_movement(
    mut commands: Commands,
    time: Res<Time>,
    font: Res<GameFont>,
    mut intent: ResMut<Intent>,
    mut blocked_hint_cooldown: Local<f32>,
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
    *blocked_hint_cooldown = (*blocked_hint_cooldown - time.delta_secs()).max(0.0);
    if let Some(index) = dialogue.puzzle_cleared_after.take() {
        if let Some(marker) = scene.markers.get_mut(index) {
            if marker.kind == NodeKind::Puzzle && !marker.cleared {
                marker.cleared = true;
                for (entity, visual) in &visuals {
                    if visual.0 == index {
                        commands.entity(entity).despawn();
                    }
                }
                let map_kind = scene.map;
                dialogue
                    .lines
                    .push(grant_route_puzzle_reward(&mut stats, &mut run, map_kind));
                dialogue.pending_commission = Some((map_kind, NodeKind::Puzzle));
                settle_route_commission(&mut dialogue, &mut stats, &mut run);
                recompute_flow(&mut scene, &map.0);
            }
        }
    }
    if dialogue.active || inventory.0 {
        return;
    }
    let interact = intent.confirm;
    let target = if interact {
        nearby_marker(&scene)
    } else {
        None
    };
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
    if let Some(dir) = intent.move_dir.filter(|_| target.is_none()) {
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
                if *blocked_hint_cooldown <= 0.0 {
                    let point = tile_to_world(scene.col, scene.row);
                    let hint = if map.0.at(nc, nr) == Tile::Water {
                        "水路不通，请寻找渡桥"
                    } else {
                        "前方有障碍，请沿路绕行"
                    };
                    commands.spawn((
                        SceneFloatText { age: 0.0 },
                        Text2d::new(hint),
                        font.text_font(17.0),
                        TextColor(theme::GOLD),
                        Transform::from_xyz(point.x, point.y + 36.0, 30.0),
                        DespawnOnExit(AppState::RunScene),
                    ));
                    *blocked_hint_cooldown = 1.4;
                }
            }
        }
    }
    intent.clear();
    if !moved && !interact {
        return;
    }

    if let Some(index) = target {
        let kind = scene.markers[index].kind;
        // Navigation records every first visit, including asking a guide for directions.
        // Guide interaction/visuals remain available without attracting the flow again.
        if kind != NodeKind::Puzzle {
            scene.markers[index].cleared = true;
        }
        if !matches!(kind, NodeKind::Guide | NodeKind::Puzzle) {
            for (entity, visual) in &visuals {
                if visual.0 == index {
                    commands.entity(entity).despawn();
                }
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
                dialogue.pending_commission = Some((scene.map, kind));
            }
            NodeKind::Story => {
                let roll = run.draw_story(&mut rng);
                dialogue.open_story(run.chapter, roll);
                dialogue.pending_commission = Some((scene.map, kind));
            }
            NodeKind::Rest => {
                dialogue.open_rest(&run);
                dialogue.pending_commission = Some((scene.map, kind));
            }
            NodeKind::Market => {
                dialogue.open_market();
                dialogue.pending_commission = Some((scene.map, kind));
            }
            NodeKind::Puzzle => {
                puzzle::open(&mut dialogue, index, scene.map, &mut rng);
            }
            NodeKind::Guide => {
                let target = if run.is_route_commission_active() {
                    run.active_route_commission
                        .map(|commission| commission.target)
                } else {
                    route_commission_target(&scene)
                };
                let (title, lines, portrait) = route_contact_dialogue(&run, scene.map, target);
                if !run.is_route_commission_active() && !run.is_route_commission_completed() {
                    if let Some(target) = target {
                        dialogue.open_route_commission(&title, lines, target, Some(portrait));
                    } else {
                        dialogue.open_plain_owned_with_portrait(&title, lines, Some(portrait));
                    }
                } else {
                    dialogue.open_plain_owned_with_portrait(&title, lines, Some(portrait));
                }
            }
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
                    let line = super::narrative::grant_random_relic(&mut stats, &mut run, &mut rng);
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
                let pct = if run.hex_spring_halved() { 17 } else { 35 };
                let heal = (stats.max_hp * pct / 100)
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
    if moved
        && !run.hex_hazard_immune()
        && scene
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
    let grass_base = if run.hex_grass_halved() { 0.04 } else { 0.08 };
    if moved
        && !(run.chapter == 0 && run.stage == 0)
        && map.0.at(scene.col, scene.row) == Tile::Grass
        && rng.chance(run.grass_encounter_chance(grass_base))
    {
        run.current_fight = Some(FightRank::Normal);
        commands.insert_resource(PendingEncounter {
            zone: map_zone(scene.map),
            kind: encounter_kind_for(&run, NodeKind::Fight),
        });
        commands.insert_resource(battle_mods_for(&run, FightRank::Normal));
        next.set(AppState::Battle);
        return;
    }

    // Leaving is a movement decision, with no extra administrative dialogue.
    if moved && map.0.at(scene.col, scene.row) == Tile::Portal {
        if scene.markers.iter().all(|m| m.cleared || m.kind.optional()) {
            run.complete_journey_task();
            run.stage += 1;
            next.set(AppState::NodeMap);
        } else {
            let position = tile_to_world(scene.col, scene.row);
            commands.spawn((
                SceneFloatText { age: 0.0 },
                Text2d::new("仍有金色目标未完成"),
                font.text_font(18.0),
                TextColor(theme::GOLD),
                Transform::from_xyz(position.x, position.y + 34.0, 30.0),
                DespawnOnExit(AppState::RunScene),
            ));
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

fn run_companion_tint(companion: Companion, pulse: f32) -> Color {
    match companion {
        Companion::Linger => Color::srgba(1.0, 0.96 + pulse * 0.04, 1.0, 1.0),
        Companion::SwordSister => Color::srgba(1.0, 0.93 + pulse * 0.05, 0.88 + pulse * 0.04, 1.0),
        Companion::SpiritWitch => Color::srgba(0.92 + pulse * 0.04, 1.0, 0.94 + pulse * 0.04, 1.0),
    }
}

fn run_follower_position(scene: &RunSceneState, slot: usize, moving: bool, phase: f32) -> Vec3 {
    let base = tile_to_world(scene.col, scene.row);
    let side = if scene.facing_left { 1.0 } else { -1.0 };
    let slot = slot as f32;
    let stride = if moving {
        (phase * 8.4 + slot * 0.9).sin()
    } else {
        (phase * 2.1 + slot * 0.7).sin() * 0.25
    };
    Vec3::new(
        base.x + side * (34.0 + slot * 22.0) + stride * 3.0,
        base.y - 18.0 - slot * 22.0 + stride.abs() * 2.0,
        9.0 - slot * 0.05,
    )
}

pub fn sync_run_party_followers(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    run: Option<Res<RunState>>,
    scene: Option<Res<RunSceneState>>,
    mut followers: Query<(Entity, &RunPartyFollower, &mut Transform, &mut Sprite)>,
) {
    let (Some(run), Some(scene)) = (run, scene) else {
        for (entity, _, _, _) in &mut followers {
            commands.entity(entity).despawn();
        }
        return;
    };
    if scene.col < 0 {
        return;
    }

    let desired = run.party_companions();
    let moving = scene.cooldown > 0.01;
    let phase = time.elapsed_secs();
    let mut present = vec![false; desired.len()];

    for (entity, follower, mut transform, mut sprite) in &mut followers {
        let Some(slot) = desired
            .iter()
            .position(|companion| *companion == follower.companion)
        else {
            commands.entity(entity).despawn();
            continue;
        };

        present[slot] = true;
        let pos = run_follower_position(&scene, slot, moving, phase);
        let sway = (phase * 3.0 + slot as f32 * 0.8).sin();
        transform.translation = pos;
        transform.rotation = Quat::from_rotation_z(sway * if moving { 0.045 } else { 0.020 });
        transform.scale = Vec3::new(1.0 - sway.abs() * 0.015, 1.0 + sway.abs() * 0.018, 1.0);
        sprite.flip_x = scene.facing_left;
        sprite.color = run_companion_tint(follower.companion, sway.abs());
    }

    for (slot, companion) in desired.iter().copied().enumerate() {
        if present.get(slot).copied().unwrap_or(false) {
            continue;
        }
        let follower = commands
            .spawn((
                Sprite {
                    image: asset_server.load(companion_cutout_path(companion)),
                    custom_size: Some(Vec2::splat(COMPANION_OVERWORLD_SIZE)),
                    ..default()
                },
                Transform::from_translation(run_follower_position(&scene, slot, moving, phase)),
                DespawnOnExit(AppState::RunScene),
            ))
            .id();
        commands
            .entity(follower)
            .insert(RunPartyFollower { companion });
        let position = run_follower_position(&scene, slot, moving, phase);
        let shadow = theme::contact_shadow(
            &mut commands,
            &asset_server,
            Vec3::new(position.x, position.y - 22.0, 8.4),
            34.0,
            AppState::RunScene,
        );
        commands.entity(shadow).insert(theme::ShadowAnchor {
            actor: follower,
            offset: Vec2::new(0.0, -22.0),
        });
    }
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

pub(super) fn update_run_hud(
    stats: Res<PlayerStats>,
    cameras: Query<&Camera, With<Camera2d>>,
    scale: Res<UiScale>,
    mut fonts: Query<(&HudFont, &mut TextFont)>,
    inventory: Res<InventoryOpen>,
    dialogue: Res<RunDialogue>,
    mut roots: Query<&mut Visibility, With<MapHudRoot>>,
    run: Option<Res<RunState>>,
    scene: Option<Res<RunSceneState>>,
    mut fills: ParamSet<(
        Query<&mut Node, With<HudHpFill>>,
        Query<&mut Node, With<HudMpFill>>,
        Query<(&HudPanel, &mut Node)>,
        Query<&mut Node, With<HudPortrait>>,
    )>,
    mut labels: ParamSet<(
        Query<&mut Text, With<HudSubText>>,
        Query<&mut Text, With<HudTaskText>>,
        Query<&mut Text, With<HudVitalsText>>,
    )>,
) {
    let portrait_units = cameras.iter().find_map(|camera| {
        let size = camera.physical_viewport_size()?;
        (size.y > size.x)
            .then(|| 1.0 / (scale.0 * camera.target_scaling_factor().unwrap_or(1.0)).max(0.1))
    });
    for (style, mut font) in &mut fonts {
        font.font_size = portrait_units
            .map_or(style.base, |units| {
                style.base.max(if style.hint { 12.0 } else { 15.0 }) * units
            })
            .into();
    }
    for (panel, mut node) in &mut fills.p2() {
        if let Some(units) = portrait_units {
            let pad = if matches!(panel, HudPanel::Controls) {
                8.0
            } else {
                10.0
            };
            node.padding = UiRect::all(Val::Px(pad * units));
            node.column_gap = Val::Px(8.0 * units);
            match panel {
                HudPanel::Vitals => {
                    node.width = Val::Percent(46.0);
                    node.max_width = Val::Auto;
                    node.left = Val::Px(14.0 * units);
                    node.top = Val::Px(14.0 * units);
                }
                HudPanel::Chapter => {
                    node.width = Val::Percent(36.0);
                    node.max_width = Val::Percent(36.0);
                    node.right = Val::Px(14.0 * units);
                    node.top = Val::Px(14.0 * units);
                }
                HudPanel::Objective => {
                    node.width = Val::Percent(44.0);
                    node.max_width = Val::Percent(44.0);
                    node.left = Val::Px(14.0 * units);
                    node.bottom = Val::Px(14.0 * units);
                }
                HudPanel::Controls => {
                    node.width = Val::Percent(44.0);
                    node.max_width = Val::Percent(44.0);
                    node.right = Val::Px(14.0 * units);
                    node.bottom = Val::Px(14.0 * units);
                }
            }
        } else {
            // Restore the original layouts after rotating or resizing back to landscape.
            match panel {
                HudPanel::Vitals => {
                    node.width = Val::Px(310.0);
                    node.max_width = Val::Auto;
                    node.left = Val::Px(18.0);
                    node.top = Val::Px(18.0);
                    node.padding = UiRect::all(Val::Px(14.0));
                    node.column_gap = Val::Px(12.0);
                }
                HudPanel::Chapter => {
                    node.width = Val::Auto;
                    node.max_width = Val::Percent(42.0);
                    node.right = Val::Px(18.0);
                    node.top = Val::Px(18.0);
                    node.padding = UiRect::axes(Val::Px(18.0), Val::Px(12.0));
                }
                HudPanel::Objective => {
                    node.width = Val::Auto;
                    node.max_width = Val::Percent(48.0);
                    node.left = Val::Px(18.0);
                    node.bottom = Val::Px(20.0);
                    node.padding = UiRect::axes(Val::Px(18.0), Val::Px(12.0));
                }
                HudPanel::Controls => {
                    node.width = Val::Auto;
                    node.max_width = Val::Auto;
                    node.right = Val::Px(18.0);
                    node.bottom = Val::Px(20.0);
                    node.padding = UiRect::axes(Val::Px(14.0), Val::Px(8.0));
                }
            }
        }
    }
    if let Ok(mut node) = fills.p3().single_mut() {
        let size = portrait_units.map_or(56.0, |units| 40.0 * units);
        node.width = Val::Px(size);
        node.height = Val::Px(size);
    }
    for mut visibility in &mut roots {
        *visibility = if inventory.0 || dialogue.active {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }

    if let Ok(mut node) = fills.p0().single_mut() {
        node.width = Val::Percent(
            (stats.hp.max(0) as f32 / stats.max_hp.max(1) as f32 * 100.0).clamp(0.0, 100.0),
        );
    }
    if let Ok(mut node) = fills.p1().single_mut() {
        node.width = Val::Percent(
            (stats.mp.max(0) as f32 / stats.max_mp.max(1) as f32 * 100.0).clamp(0.0, 100.0),
        );
    }
    if let Ok(mut text) = labels.p0().single_mut() {
        text.0 = format!("药水 {} · 钱财 {} 文", stats.potions, stats.gold);
    }
    if let Ok(mut text) = labels.p2().single_mut() {
        text.0 = format!("气血 {} · 灵力 {}", stats.hp.max(0), stats.mp.max(0));
    }
    if let (Some(run), Some(scene)) = (run.as_deref(), scene.as_deref()) {
        if let Ok(mut text) = labels.p1().single_mut() {
            text.0 = journey_task_status_line(&run, scene);
        }
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
    mut cameras: Query<(&Camera, &mut Transform), With<Camera2d>>,
) {
    let Some(scene) = scene else { return };
    if scene.col < 0 {
        return;
    }
    let target = tile_to_world(scene.col, scene.row);
    // Use the real viewport so tall windows keep the hero in view.
    for (camera, mut transform) in &mut cameras {
        let aspect = camera
            .logical_viewport_size()
            .map_or(16.0 / 9.0, |size| size.x / size.y.max(1.0));
        let range_x = (MAP_W as f32 * TILE * 0.5 - VIEW_HEIGHT * aspect * 0.5).max(0.0);
        let range_y = (MAP_H as f32 * TILE * 0.5 - VIEW_HEIGHT * 0.5).max(0.0);
        let cx = target.x.clamp(-range_x, range_x);
        let cy = target.y.clamp(-range_y, range_y);
        transform.translation.x += (cx - transform.translation.x) * 0.12;
        transform.translation.y += (cy - transform.translation.y) * 0.12;
    }
}

// ---------------------------------------------------------------------------
// Esc inventory (行囊 · 修行录)
// ---------------------------------------------------------------------------

pub fn inventory_toggle(
    mut commands: Commands,
    mut intent: ResMut<Intent>,
    mut inventory: ResMut<InventoryOpen>,
    dialogue: Res<RunDialogue>,
    font: Res<GameFont>,
    anims: Res<AnimationAssets>,
    stats: Res<PlayerStats>,
    run: Option<Res<RunState>>,
    scene: Option<Res<RunSceneState>>,
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

    // World-space dim + the same animated hero used in play, anchored to the camera view.
    let cam = cameras
        .iter()
        .next()
        .map(|t| t.translation)
        .unwrap_or(Vec3::ZERO);
    commands.spawn((
        InventoryUi,
        Sprite::from_color(
            Color::srgba(0.01, 0.02, 0.05, 0.96),
            Vec2::new(2400.0, 1400.0),
        ),
        Transform::from_xyz(cam.x, cam.y, 40.0),
        DespawnOnExit(AppState::RunScene),
    ));
    let doll = animation::spawn_animated_sprite(
        &mut commands,
        &anims,
        AnimationClip::HeroIdle,
        Vec3::new(cam.x - 235.0, cam.y - 10.0, 50.0),
        Vec2::splat(280.0),
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
    let task_status = scene
        .as_deref()
        .map(|scene| journey_task_status_line(&run, scene))
        .unwrap_or_else(|| run.journey_task_receipt());
    commands
        .spawn((
            InventoryUi,
            crate::game::ui_scroll::ScrollPanel,
            ScrollPosition::default(),
            DespawnOnExit(AppState::RunScene),
            theme::panel(Node {
                position_type: PositionType::Absolute,
                right: Val::Percent(6.0),
                top: Val::Percent(8.0),
                bottom: Val::Percent(8.0),
                width: Val::Percent(46.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                padding: UiRect::new(Val::Px(30.0), Val::Px(30.0), Val::Px(24.0), Val::Px(24.0)),
                overflow: Overflow::scroll_y(),
                ..default()
            }),
            GlobalZIndex(80),
        ))
        .with_children(|viewport| {
            viewport
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    flex_shrink: 0.0,
                    ..default()
                })
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("行囊 · 修行录"),
                        font.text_font(28.0),
                        TextColor(Color::srgb(0.95, 0.85, 0.55)),
                    ));
                    panel.spawn((
                        Text::new(format!(
                            "{} · {}\n{}",
                            run.chapter_def().title,
                            run.journey_beat().place,
                            run.journey_beat().objective
                        )),
                        font.text_font(16.0),
                        TextColor(theme::TEXT),
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
                    if run.relics.is_empty() {
                        panel.spawn((
                            Text::new("(尚未寻得法宝)"),
                            font.text_font(17.0),
                            TextColor(Color::srgba(0.75, 0.78, 0.85, 0.8)),
                        ));
                    } else {
                        // 按品级从高到低排,每件按品级着色(白/绿/蓝/紫/金)。
                        let mut sorted = run.relics.clone();
                        sorted.sort_by_key(|r| std::cmp::Reverse(r.grade()));
                        for r in sorted {
                            panel.spawn((
                                Text::new(format!(
                                    "〔{}〕【{}】{}",
                                    r.grade().name(),
                                    r.name(),
                                    r.desc()
                                )),
                                font.text_font(17.0),
                                TextColor(r.grade().color()),
                            ));
                        }
                    }
                    panel.spawn((
                        Text::new(format!(
                            "仙术秘卷({}/{}):",
                            run.skills.len(),
                            super::skill::SKILL_COUNT
                        )),
                        font.text_font(20.0),
                        TextColor(Color::srgb(0.62, 0.85, 1.0)),
                    ));
                    if run.skills.is_empty() {
                        panel.spawn((
                            Text::new("(尚未习得仙术)"),
                            font.text_font(16.0),
                            TextColor(Color::srgba(0.75, 0.78, 0.85, 0.8)),
                        ));
                    } else {
                        let mut sorted = run.skills.clone();
                        sorted.sort_by_key(|id| std::cmp::Reverse(super::skill::skill(*id).grade));
                        for id in sorted {
                            let def = super::skill::skill(id);
                            panel.spawn((
                                Text::new(format!(
                                    "〔{}〕「{}」灵{} · {}",
                                    def.grade.name(),
                                    def.name(),
                                    crate::game::battle::skill_cost(id, &run),
                                    def.desc()
                                )),
                                font.text_font(16.0),
                                TextColor(def.grade.color()),
                            ));
                        }
                    }
                    panel.spawn((
                        Text::new(format!("妖纹({}):", run.hexes.len())),
                        font.text_font(20.0),
                        TextColor(Color::srgb(0.90, 0.62, 1.0)),
                    ));
                    if run.hexes.is_empty() {
                        panel.spawn((
                            Text::new("(尚未染上妖纹)"),
                            font.text_font(16.0),
                            TextColor(Color::srgba(0.75, 0.78, 0.85, 0.8)),
                        ));
                    } else {
                        let mut sorted = run.hexes.clone();
                        sorted.sort_by_key(|h| std::cmp::Reverse(h.grade()));
                        for h in sorted {
                            panel.spawn((
                                Text::new(format!(
                                    "〔{}〕【{}】{}",
                                    h.grade().name(),
                                    h.name(),
                                    h.desc()
                                )),
                                font.text_font(16.0),
                                TextColor(h.grade().color()),
                            ));
                        }
                    }
                    panel.spawn((
                        Text::new(run.narrative_summary()),
                        font.text_font(15.0),
                        TextColor(theme::TEXT),
                    ));
                    panel.spawn((
                        Text::new("旅程记录"),
                        font.text_font(20.0),
                        TextColor(theme::GOLD),
                    ));
                    panel.spawn((
                        Text::new(format!(
                            "{}\n任务札：{}",
                            run.journey_summary().replace("实测用时", "此世历时"),
                            task_status
                        )),
                        font.text_font(15.0),
                        TextColor(theme::MUTED),
                    ));
                });
        });
    commands.spawn((
        InventoryUi,
        DespawnOnExit(AppState::RunScene),
        Text::new("滚轮或翻页键翻阅 · 退出键关闭"),
        font.text_font(14.0),
        TextColor(theme::MUTED),
        theme::panel(Node {
            position_type: PositionType::Absolute,
            right: Val::Percent(6.0),
            bottom: Val::Percent(3.0),
            padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
            ..default()
        }),
        GlobalZIndex(81),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commission_waits_for_resolution_and_settles_only_once() {
        let mut run = RunState::new(&mut Rng::default());
        let mut stats = PlayerStats::default();
        let before = stats.gold;
        assert!(run.accept_route_commission(NodeKind::Event));
        let mut dialogue = RunDialogue::default();
        dialogue.open_event(0);
        dialogue.pending_commission = Some((MapKind::RiverTown, NodeKind::Event));
        assert_eq!(stats.gold, before);
        assert!(!run.is_route_commission_completed());
        settle_route_commission(&mut dialogue, &mut stats, &mut run);
        assert_eq!(stats.gold, before + 25);
        assert_eq!(run.route_contacts_helped, 1);
        assert!(run.is_route_commission_completed());
        assert!(dialogue.pending_commission.is_none());
        assert!(dialogue.lines.iter().any(|s| s.contains("路况照应")));
        let line_count = dialogue.lines.len();
        settle_route_commission(&mut dialogue, &mut stats, &mut run);
        // Repeated callbacks and even a replayed pending receipt cannot grant again.
        dialogue.pending_commission = Some((MapKind::RiverTown, NodeKind::Event));
        settle_route_commission(&mut dialogue, &mut stats, &mut run);
        assert_eq!(stats.gold, before + 25);
        assert_eq!(run.route_contacts_helped, 1);
        assert_eq!(dialogue.lines.len(), line_count);
    }

    #[test]
    fn wrong_target_does_not_pay_or_complete_accepted_commission() {
        let mut run = RunState::new(&mut Rng::default());
        assert!(run.accept_route_commission(NodeKind::Story));
        let mut stats = PlayerStats::default();
        let before = stats.gold;
        let mut dialogue = RunDialogue::default();
        dialogue.pending_commission = Some((MapKind::Capital, NodeKind::Market));
        settle_route_commission(&mut dialogue, &mut stats, &mut run);
        assert_eq!(stats.gold, before);
        assert!(run.is_route_commission_active());
        assert_eq!(run.route_contacts_helped, 0);
        dialogue.pending_commission = Some((MapKind::Capital, NodeKind::Story));
        settle_route_commission(&mut dialogue, &mut stats, &mut run);
        assert_eq!(stats.gold, before + 35);
    }

    #[test]
    fn commission_recovery_receipts_report_actual_capped_gain() {
        let mut stats = PlayerStats::default();
        stats.hp = stats.max_hp - 1;
        let line = grant_route_commission_reward(&mut stats, MapKind::Bamboo, NodeKind::Puzzle);
        assert_eq!(stats.hp, stats.max_hp);
        assert!(line.contains("实际恢复 1 点气血"));
        stats.mp = stats.max_mp - 2;
        let line =
            grant_route_commission_reward(&mut stats, MapKind::SouthernRoad, NodeKind::Event);
        assert_eq!(stats.mp, stats.max_mp);
        assert!(line.contains("实际回复灵力 2"));
        let line =
            grant_route_commission_reward(&mut stats, MapKind::FinalSanctum, NodeKind::Story);
        assert!(line.contains("气血 0,灵力 0"));
    }

    #[test]
    fn guide_explains_work_payment_and_existing_contract_status() {
        let mut run = RunState::new(&mut Rng::default());
        let (_, offered, _) =
            route_contact_dialogue(&run, MapKind::RiverTown, Some(NodeKind::Market));
        assert!(offered.iter().any(|line| line.contains("不强求购买")));
        assert!(offered.iter().any(|line| line.contains("25文")));
        assert!(offered.iter().any(|line| line.contains("之后回到此处")));
        run.accept_route_commission(NodeKind::Market);
        let (_, accepted, _) =
            route_contact_dialogue(&run, MapKind::RiverTown, Some(NodeKind::Market));
        assert!(accepted.iter().any(|line| line.contains("已接取")));
        run.complete_route_commission(NodeKind::Market);
        let (_, completed, _) = route_contact_dialogue(&run, MapKind::RiverTown, None);
        assert!(completed.iter().any(|line| line.contains("不会再次发放")));
    }

    const ALL_ROUTE_MAPS: [MapKind; 15] = [
        MapKind::Village,
        MapKind::Bamboo,
        MapKind::Cave,
        MapKind::MoonEchoCorridor,
        MapKind::RiverTown,
        MapKind::RiverReedBed,
        MapKind::PlagueVillage,
        MapKind::PlagueShrinePath,
        MapKind::Capital,
        MapKind::CapitalMansion,
        MapKind::MansionMirrorGallery,
        MapKind::SouthernRoad,
        MapKind::ThunderDrumPath,
        MapKind::FinalSanctum,
        MapKind::DreamWaterway,
    ];

    fn blocking_signature(tiles: &[Vec<Tile>]) -> Vec<u8> {
        tiles
            .iter()
            .flatten()
            .map(|tile| match tile {
                Tile::Wall | Tile::Structure => 1,
                Tile::Water => 2,
                _ => 0,
            })
            .collect()
    }

    fn test_scene(markers: Vec<SceneMarker>) -> RunSceneState {
        RunSceneState {
            map: MapKind::Village,
            tiles: Vec::new(),
            revealed: Vec::new(),
            col: 0,
            row: 0,
            facing_left: false,
            markers,
            portals: Vec::new(),
            flow: Vec::new(),
            hazards: Vec::new(),
            intro_shown: false,
            cooldown: 0.0,
        }
    }

    #[test]
    fn visited_guide_remains_talkable_without_trapping_flow_or_portal_navigation() {
        let mut map = MapData::build(MapKind::Village);
        map.tiles = vec![vec![Tile::Wall; MAP_W as usize]; MAP_H as usize];
        // A real walkable corridor makes the BFS distances and downhill direction exact.
        for col in 1..=9 {
            map.tiles[2][col] = Tile::Path;
        }
        let mut scene = test_scene(vec![
            SceneMarker {
                kind: NodeKind::Guide,
                col: 2,
                row: 2,
                cleared: false,
            },
            SceneMarker {
                kind: NodeKind::Story,
                col: 6,
                row: 2,
                cleared: false,
            },
        ]);
        scene.portals = vec![(9, 2)];
        recompute_flow(&mut scene, &map);
        assert_eq!(scene.flow[2][2], 0);
        assert_eq!(scene.flow[2][6], 0);

        scene.markers[0].cleared = true;
        recompute_flow(&mut scene, &map);
        assert!(scene.markers[0].is_visitable());
        assert_eq!(scene.flow, distance_field(&map, &[(6, 2)]));
        assert_eq!(scene.flow[2][2], 4);
        assert!(scene.flow[2][3] < scene.flow[2][2]);

        scene.markers[1].cleared = true;
        recompute_flow(&mut scene, &map);
        assert!(!scene.markers[1].is_visitable());
        assert!(scene.markers[0].is_visitable());
        assert_eq!(scene.flow, distance_field(&map, &[(9, 2)]));
        assert_eq!(scene.flow[2][9], 0);
        assert_eq!(scene.flow[2][2], 7);
        // Returning to the guide preserves portal steering after another conversation.
        scene.markers[0].cleared = true;
        recompute_flow(&mut scene, &map);
        assert_eq!(scene.flow[2][2], 7);
        assert!(scene.flow[2][3] < scene.flow[2][2]);
    }

    #[test]
    fn map_topology_all_biomes_are_connected_spacious_and_distinct() {
        let mut signatures: Vec<(MapKind, Vec<u8>)> = Vec::new();

        for (index, map_kind) in ALL_ROUTE_MAPS.into_iter().enumerate() {
            let mut rng = Rng(0x5EED_CAFE_D00D_BAAD);
            let tiles = generate_stage_tiles(map_kind, index, &mut rng);
            assert_eq!(tiles.len(), MAP_H as usize, "{map_kind:?} height");
            assert!(
                tiles.iter().all(|row| row.len() == MAP_W as usize),
                "{map_kind:?} width"
            );
            assert!(
                tiles[0].iter().all(|tile| *tile == Tile::Wall)
                    && tiles[MAP_H as usize - 1]
                        .iter()
                        .all(|tile| *tile == Tile::Wall),
                "{map_kind:?} must keep a closed border"
            );
            assert!(
                (0..MAP_H as usize).all(|row| tiles[row][0] == Tile::Wall
                    && tiles[row][MAP_W as usize - 1] == Tile::Wall),
                "{map_kind:?} must keep a closed border"
            );

            let walkable = tiles
                .iter()
                .flatten()
                .filter(|tile| tile.walkable())
                .count();
            assert!(
                walkable >= 150,
                "{map_kind:?} has only {walkable} open cells"
            );
            assert!(
                stage_exploration_span(&tiles) >= 24,
                "{map_kind:?} lacks an exploration route"
            );

            let generated = MapData::generated(map_kind, tiles.clone());
            let start = (1..MAP_W - 1)
                .find_map(|col| {
                    (1..MAP_H - 1)
                        .find(|row| generated.at(col, *row).walkable())
                        .map(|row| (col, row))
                })
                .expect("quality-gated map should have a spawn");
            let reachable = distance_field(&generated, &[start])
                .iter()
                .flatten()
                .filter(|distance| **distance != u16::MAX)
                .count();
            assert_eq!(
                reachable, walkable,
                "{map_kind:?} left disconnected walkable pockets"
            );

            let signature = blocking_signature(&tiles);
            for (other_kind, other) in &signatures {
                assert!(
                    signature != *other,
                    "{map_kind:?} duplicated {other_kind:?} topology"
                );
            }
            signatures.push((map_kind, signature));
        }
    }

    #[test]
    fn map_topology_repeated_finale_biomes_get_distinct_variants() {
        for (map_kind, stages) in [
            (MapKind::FinalSanctum, vec![0usize, 2, 4, 6, 8, 10]),
            (MapKind::DreamWaterway, vec![1usize, 3, 5, 7, 9]),
        ] {
            let mut signatures = Vec::new();
            for stage in stages {
                let mut rng = Rng(0xA11C_E5EED);
                let tiles = generate_stage_tiles(map_kind, 6 * 16 + stage, &mut rng);
                let signature = blocking_signature(&tiles);
                assert!(
                    signatures.iter().all(|known| *known != signature),
                    "{map_kind:?} repeated its topology at stage {}",
                    stage + 1
                );
                signatures.push(signature);
            }
        }
    }

    #[test]
    fn map_topology_all_route_stages_fit_required_and_optional_objectives() {
        use crate::game::roguelike::{JOURNEY_MAPS, JOURNEY_MARKER_PLANS};

        for (chapter, maps) in JOURNEY_MAPS.iter().enumerate() {
            for (stage, map_kind) in maps.iter().copied().enumerate() {
                let variant = chapter * 16 + stage;
                let mut rng =
                    Rng(0xF17E_1D5E_5EED_0001 ^ ((chapter as u64) << 12) ^ ((stage as u64) << 4));
                let mut tiles = generate_stage_tiles(map_kind, variant, &mut rng);
                let map = MapData::generated(map_kind, tiles.clone());
                let spawn = (1..MAP_W - 1)
                    .find_map(|col| {
                        (1..MAP_H - 1)
                            .find(|row| map.at(col, *row).walkable())
                            .map(|row| (col, row))
                    })
                    .expect("quality-gated map should have a spawn");
                let from_spawn = distance_field(&map, &[spawn]);
                let portal = (0..MAP_H)
                    .flat_map(|row| (0..MAP_W).map(move |col| (col, row)))
                    .filter(|(col, row)| from_spawn[*row as usize][*col as usize] != u16::MAX)
                    .max_by_key(|(col, row)| from_spawn[*row as usize][*col as usize])
                    .expect("quality-gated map should have a portal target");
                tiles[portal.1 as usize][portal.0 as usize] = Tile::Portal;

                let candidates: Vec<(i32, i32)> = (0..MAP_H)
                    .flat_map(|row| (0..MAP_W).map(move |col| (col, row)))
                    .filter(|(col, row)| {
                        let distance = from_spawn[*row as usize][*col as usize];
                        let portal_distance = (col - portal.0).abs() + (row - portal.1).abs();
                        distance != u16::MAX
                            && distance >= 6
                            && portal_distance >= 4
                            && tiles[*row as usize][*col as usize].walkable()
                    })
                    .collect();
                let mut markers = Vec::new();
                for kind in JOURNEY_MARKER_PLANS[chapter][stage]
                    .iter()
                    .copied()
                    .filter(|kind| *kind != NodeKind::Boss)
                {
                    assert!(
                        place_marker(&mut markers, &candidates, &mut rng, kind, 8),
                        "chapter {} stage {} {:?} dropped required {}",
                        chapter + 1,
                        stage + 1,
                        map_kind,
                        kind.label()
                    );
                }
                if route_puzzle_enabled(map_kind)
                    && !JOURNEY_MARKER_PLANS[chapter][stage].contains(&NodeKind::Boss)
                {
                    assert!(
                        place_marker(&mut markers, &candidates, &mut rng, NodeKind::Puzzle, 7,),
                        "chapter {} stage {} {:?} dropped its mechanism",
                        chapter + 1,
                        stage + 1,
                        map_kind
                    );
                }
                if !JOURNEY_MARKER_PLANS[chapter][stage].contains(&NodeKind::Boss) {
                    assert!(
                        place_marker(&mut markers, &candidates, &mut rng, NodeKind::Guide, 6,),
                        "chapter {} stage {} {:?} dropped its route contact",
                        chapter + 1,
                        stage + 1,
                        map_kind
                    );
                }
            }
        }
    }

    #[test]
    fn journey_task_condition_tracks_only_required_markers() {
        let mut rng = Rng::default();
        let run = RunState::new(&mut rng);
        let scene = test_scene(vec![
            SceneMarker {
                kind: NodeKind::Story,
                col: 1,
                row: 1,
                cleared: true,
            },
            SceneMarker {
                kind: NodeKind::Fight,
                col: 2,
                row: 1,
                cleared: false,
            },
            SceneMarker {
                kind: NodeKind::Puzzle,
                col: 3,
                row: 1,
                cleared: false,
            },
            SceneMarker {
                kind: NodeKind::Chest,
                col: 4,
                row: 1,
                cleared: false,
            },
            SceneMarker {
                kind: NodeKind::Guide,
                col: 5,
                row: 1,
                cleared: false,
            },
        ]);

        assert_eq!(mandatory_marker_progress(&scene), (1, 3));
        assert!(journey_task_condition(&run, &scene).contains("1/3"));
        assert!(journey_task_condition(&run, &scene).contains("遭遇战"));
        assert!(journey_task_status_line(&run, &scene).contains("空格"));
        assert!(!journey_task_status_line(&run, &scene).contains("签收"));
    }

    #[test]
    fn journey_task_condition_reports_gate_ready_and_boss_stage() {
        let mut rng = Rng::default();
        let mut run = RunState::new(&mut rng);
        let ready = test_scene(vec![
            SceneMarker {
                kind: NodeKind::Story,
                col: 1,
                row: 1,
                cleared: true,
            },
            SceneMarker {
                kind: NodeKind::Market,
                col: 2,
                row: 1,
                cleared: true,
            },
        ]);
        assert!(journey_task_condition(&run, &ready).contains("界门"));

        run.stage = run.stage_count() - 1;
        let boss = test_scene(vec![SceneMarker {
            kind: NodeKind::Boss,
            col: 1,
            row: 1,
            cleared: false,
        }]);
        assert!(journey_task_condition(&run, &boss).contains("迎战"));
    }

    #[test]
    fn route_puzzle_dialogue_records_required_mechanism_rewards() {
        let mut rng = Rng::default();
        let mut run = RunState::new(&mut rng);
        run.stage = 2;
        let mut stats = PlayerStats::default();

        let lines = puzzle::riddle(MapKind::MoonEchoCorridor).clues;
        let reward = grant_route_puzzle_reward(&mut stats, &mut run, MapKind::MoonEchoCorridor);

        assert!(!NodeKind::Puzzle.optional());
        assert!(route_puzzle_enabled(MapKind::MoonEchoCorridor));
        assert!(!route_puzzle_enabled(MapKind::Village));
        assert_eq!(
            route_puzzle_asset(MapKind::MoonEchoCorridor),
            "props/ai_cave_crystal.png"
        );
        assert!(lines.iter().any(|line| line.contains("晶石")));
        assert_eq!(run.route_puzzles_solved, 1);
        assert!(stats.max_mp > PlayerStats::default().max_mp);
        assert!(reward.contains("灵力上限"));
        assert!(run.journey_summary().contains("机关破除：1"));
    }

    #[test]
    fn route_contact_dialogue_tracks_optional_errand_until_target_clears() {
        let mut rng = Rng::default();
        let mut run = RunState::new(&mut rng);
        run.stage = 1;
        let mut stats = PlayerStats {
            hp: 40,
            max_hp: 80,
            ..default()
        };
        let scene = test_scene(vec![
            SceneMarker {
                kind: NodeKind::Event,
                col: 1,
                row: 1,
                cleared: false,
            },
            SceneMarker {
                kind: NodeKind::Puzzle,
                col: 2,
                row: 1,
                cleared: false,
            },
        ]);

        let target = route_commission_target(&scene);
        let (title, lines, portrait) = route_contact_dialogue(&run, MapKind::Bamboo, target);

        assert!(NodeKind::Guide.optional());
        assert_eq!(target, Some(NodeKind::Puzzle));
        assert!(title.contains("竹林斥候"));
        assert!(lines.iter().any(|line| line.contains("十里坡旧道")));
        assert!(lines.iter().any(|line| line.contains("路人签 卷1-02")));
        assert!(lines.iter().any(|line| line.contains("机关")));
        assert_eq!(portrait, "npcs/ai_bamboo_scout.png");
        assert!(run.accept_route_commission(NodeKind::Puzzle));
        assert!(!run.complete_route_commission(NodeKind::Event));
        let reward = route_commission_completion_line(
            &mut stats,
            &mut run,
            MapKind::Bamboo,
            NodeKind::Puzzle,
        )
        .expect("matching target should complete commission");
        assert_eq!(run.route_contacts_helped, 1);
        assert!(stats.hp > 40);
        assert!(reward.contains("路人委托回执"));
        assert!(run.is_route_commission_completed());
        assert!(run.route_guidance_summary().contains("草地遇敌-8%"));
        assert!(run.journey_summary().contains("路人回声：1"));
        assert!(run.journey_summary().contains("路人签：已回执"));
        assert!(run.journey_summary().contains("路况：风声初稳"));
    }

    #[test]
    fn run_fog_reveals_current_vision_and_keeps_memory() {
        let mut scene = test_scene(Vec::new());
        scene.col = 5;
        scene.row = 5;

        assert!(reveal_run_fog(&mut scene));
        assert!(scene.revealed[5][5]);
        assert!(!scene.revealed[0][0]);
        assert_eq!(run_fog_alpha(&scene, 5, 5), 0.0);
        assert_eq!(run_fog_alpha(&scene, 0, 0), RUN_UNSEEN_FOG_ALPHA);
        assert!(matches!(
            run_fog_entity_visibility(&scene, 0, 0),
            Visibility::Hidden
        ));

        scene.col = 10;
        scene.row = 5;
        assert!(reveal_run_fog(&mut scene));
        assert!(scene.revealed[5][5]);
        assert_eq!(run_fog_alpha(&scene, 5, 5), RUN_SEEN_FOG_ALPHA);
        assert_eq!(run_fog_alpha(&scene, 10, 5), 0.0);
        assert!(matches!(
            run_fog_entity_visibility(&scene, 5, 5),
            Visibility::Inherited
        ));
    }
}

#[cfg(test)]
#[path = "scene_input_tests.rs"]
mod input_tests;

#[cfg(test)]
mod architecture_visibility_tests {
    use super::*;

    #[test]
    fn known_building_edge_reveals_architecture_and_preserves_correct_depth() {
        let mut app = App::new();
        let mut revealed = vec![vec![false; MAP_W as usize]; MAP_H as usize];
        revealed[7][5] = true; // Inn edge visible at spawn; its centre is not.
        app.insert_resource(RunSceneState {
            map: MapKind::Village,
            tiles: Vec::new(),
            revealed,
            col: 2,
            row: 8,
            facing_left: false,
            markers: Vec::new(),
            portals: Vec::new(),
            flow: Vec::new(),
            hazards: Vec::new(),
            intro_shown: true,
            cooldown: 0.0,
        })
        .add_systems(Update, update_architecture_depth);
        let building = SceneArchitecture {
            foot_row: 7,
            top_row: 5,
            left_col: 5,
            right_col: 9,
            walkover: false,
            shadow: false,
        };
        let body = app
            .world_mut()
            .spawn((
                building,
                Transform::default(),
                Visibility::Hidden,
                Sprite::default(),
            ))
            .id();
        let shadow = app
            .world_mut()
            .spawn((
                SceneArchitecture {
                    shadow: true,
                    ..building
                },
                Transform::default(),
                Visibility::Hidden,
                Sprite::default(),
            ))
            .id();
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(body).unwrap(),
            Visibility::Inherited
        );
        assert!(app.world().get::<Transform>(body).unwrap().translation.z < 10.0);
        assert_eq!(app.world().get::<Sprite>(body).unwrap().color.alpha(), 1.0);
        app.world_mut().resource_mut::<RunSceneState>().row = 4;
        app.world_mut().resource_mut::<RunSceneState>().col = 7;
        app.update();
        assert!(app.world().get::<Transform>(body).unwrap().translation.z > 10.0);
        assert!(app.world().get::<Transform>(shadow).unwrap().translation.z < 7.0);
        assert!(app.world().get::<Sprite>(body).unwrap().color.alpha() < 1.0);
        {
            let mut scene = app.world_mut().resource_mut::<RunSceneState>();
            scene.col = MAP_W - 2;
            scene.row = MAP_H - 2;
        }
        app.update();
        assert_eq!(app.world().get::<Sprite>(body).unwrap().color.alpha(), 1.0);
        assert_eq!(
            *app.world().get::<Visibility>(body).unwrap(),
            Visibility::Inherited
        );
        app.world_mut().resource_mut::<RunSceneState>().revealed[7][5] = false;
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(body).unwrap(),
            Visibility::Hidden
        );
        app.world_mut().resource_mut::<RunSceneState>().revealed[7][9] = true;
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(body).unwrap(),
            Visibility::Inherited
        );
    }
}

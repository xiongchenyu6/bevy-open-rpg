//! The chapter node-map screen: draws the random DAG, moves the player
//! between nodes, and dispatches node behaviour (battle / event / story /
//! rest / boss).

use bevy::prelude::*;

use super::super::battle::PendingEncounter;
use super::super::core::{GameFont, Intent, PlayerStats, Rng};
use super::event::{self, RunDialogue};
use super::graph::NodeKind;
use super::{FightRank, RunState, battle_mods_for, encounter_kind_for};
use crate::game::state::AppState;

// Layout constants (world units; the persistent Camera2d sits at the origin).
const LAYER_SPAN: f32 = 540.0;
const LAYER_BASE: f32 = -270.0;
const SLOT_STEP: f32 = 170.0;
const NODE_SIZE: f32 = 54.0;

fn node_pos(layer: usize, layer_count: usize, slot: usize, width: usize) -> Vec2 {
    let step = LAYER_SPAN / (layer_count.max(2) - 1) as f32;
    let y = LAYER_BASE + layer as f32 * step;
    let x = (slot as f32 - (width as f32 - 1.0) / 2.0) * SLOT_STEP;
    Vec2::new(x, y)
}

// ---------------------------------------------------------------------------
// Components / resources
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct MapNodeSprite(pub usize);

#[derive(Component)]
pub struct MapCursorSprite;

#[derive(Component)]
pub struct RunHudText;

/// Which of the currently-reachable nodes the cursor points at.
#[derive(Resource, Default)]
pub struct MapCursor(pub usize);

fn kind_color(kind: NodeKind, cleared: bool) -> Color {
    let c = match kind {
        NodeKind::Fight => Color::srgb(0.62, 0.32, 0.28),
        NodeKind::Elite => Color::srgb(0.72, 0.24, 0.46),
        NodeKind::Event => Color::srgb(0.28, 0.48, 0.66),
        NodeKind::Story => Color::srgb(0.78, 0.58, 0.24),
        NodeKind::Rest => Color::srgb(0.30, 0.56, 0.36),
        NodeKind::Market => Color::srgb(0.62, 0.48, 0.20),
        NodeKind::Boss => Color::srgb(0.55, 0.20, 0.62),
    };
    if cleared { c.with_alpha(0.35) } else { c }
}

// ---------------------------------------------------------------------------
// Scene setup
// ---------------------------------------------------------------------------

pub fn spawn_node_map(
    mut commands: Commands,
    font: Res<GameFont>,
    asset_server: Res<AssetServer>,
    run: Option<ResMut<RunState>>,
    mut dialogue: ResMut<RunDialogue>,
    mut cursor: ResMut<MapCursor>,
    mut next: ResMut<NextState<AppState>>,
) {
    let Some(mut run) = run else {
        // No active run (e.g. state set directly) — fall back to the title.
        next.set(AppState::Title);
        return;
    };
    cursor.0 = 0;
    let scope = || DespawnOnExit(AppState::NodeMap);

    // Ink-wash mountain backdrop, dimmed so the node graph stays readable.
    let mut backdrop = Sprite::from_image(asset_server.load("ui/map_bg.png"));
    backdrop.custom_size = Some(Vec2::new(1280.0, 720.0));
    backdrop.color = Color::srgba(0.62, 0.66, 0.74, 1.0);
    commands.spawn((backdrop, Transform::from_xyz(0.0, 0.0, -10.0), scope()));

    // Chapter heading.
    commands.spawn((
        Text::new(run.chapter_def().title),
        font.text_font(30.0),
        TextColor(Color::srgb(0.95, 0.85, 0.55)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(14.0),
            justify_self: JustifySelf::Center,
            ..default()
        },
        scope(),
    ));

    // HUD (top-left), refreshed every frame.
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

    // Controls hint (bottom-right).
    commands.spawn((
        Text::new("上/下 选路 · 空格 前进"),
        font.text_font(16.0),
        TextColor(Color::srgba(0.85, 0.87, 0.9, 0.7)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            right: Val::Px(14.0),
            ..default()
        },
        scope(),
    ));

    let graph = &run.graph;

    // Edges first (behind nodes).
    for (i, node) in graph.nodes.iter().enumerate() {
        let a = node_pos(node.layer, graph.layer_count, node.slot, node.layer_width);
        for &j in &node.next {
            let target = &graph.nodes[j];
            let b = node_pos(
                target.layer,
                graph.layer_count,
                target.slot,
                target.layer_width,
            );
            let mid = (a + b) / 2.0;
            let delta = b - a;
            let taken = node.cleared && target.cleared;
            commands.spawn((
                Sprite::from_color(
                    if taken {
                        Color::srgba(0.9, 0.8, 0.5, 0.55)
                    } else {
                        Color::srgba(0.55, 0.6, 0.7, 0.35)
                    },
                    Vec2::new(delta.length() - NODE_SIZE, 3.0),
                ),
                Transform::from_translation(mid.extend(-5.0))
                    .with_rotation(Quat::from_rotation_z(delta.y.atan2(delta.x))),
                scope(),
            ));
        }
        let _ = i;
    }

    // Nodes + glyphs.
    for (i, node) in graph.nodes.iter().enumerate() {
        let pos = node_pos(node.layer, graph.layer_count, node.slot, node.layer_width);
        let here = run.position == i;
        commands
            .spawn((
                MapNodeSprite(i),
                Sprite::from_color(
                    kind_color(node.kind, node.cleared && !here),
                    Vec2::splat(NODE_SIZE),
                ),
                Transform::from_translation(pos.extend(0.0)),
                scope(),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text2d::new(node.kind.glyph()),
                    font.text_font(26.0),
                    TextColor(Color::srgb(0.97, 0.95, 0.9)),
                    Transform::from_xyz(0.0, 0.0, 1.0),
                ));
                // Dark rim so nodes read as tokens against the painting.
                parent.spawn((
                    Sprite::from_color(
                        Color::srgba(0.03, 0.04, 0.08, 0.9),
                        Vec2::splat(NODE_SIZE + 6.0),
                    ),
                    Transform::from_xyz(0.0, 0.0, -0.5),
                ));
                if here {
                    parent.spawn((
                        Sprite::from_color(
                            Color::srgba(0.95, 0.85, 0.5, 0.9),
                            Vec2::splat(NODE_SIZE + 12.0),
                        ),
                        Transform::from_xyz(0.0, 0.0, -1.0),
                    ));
                }
            });
    }

    // Cursor ring (moved onto the selected reachable node every frame).
    commands.spawn((
        MapCursorSprite,
        Sprite::from_color(
            Color::srgba(0.4, 0.9, 1.0, 0.85),
            Vec2::splat(NODE_SIZE + 18.0),
        ),
        Transform::from_xyz(0.0, 0.0, -2.0),
        Visibility::Hidden,
        scope(),
    ));

    // Dialogue overlay box (hidden until opened).
    event::spawn_run_dialogue_ui(&mut commands, &font, scope());

    // First visit of this chapter: show the chapter card.
    if !run.card_shown {
        run.card_shown = true;
        let chapter = run.chapter;
        dialogue.open_plain(
            run.chapter_def().title,
            super::content::CHAPTER_CARDS[chapter.min(3)],
        );
    }
}

// ---------------------------------------------------------------------------
// Input / cursor / HUD
// ---------------------------------------------------------------------------

pub fn node_map_input(
    mut commands: Commands,
    mut intent: ResMut<Intent>,
    mut cursor: ResMut<MapCursor>,
    run: Option<ResMut<RunState>>,
    mut rng: ResMut<Rng>,
    mut dialogue: ResMut<RunDialogue>,
    mut next: ResMut<NextState<AppState>>,
    mut nodes: Query<(&MapNodeSprite, &mut Sprite)>,
) {
    let Some(mut run) = run else {
        return;
    };
    if dialogue.active {
        return; // run_dialogue_input already consumed the intent
    }

    let reachable = run.reachable();
    if reachable.is_empty() {
        return;
    }
    cursor.0 = cursor.0.min(reachable.len() - 1);

    if intent.up && cursor.0 + 1 < reachable.len() {
        cursor.0 += 1;
    }
    if intent.down && cursor.0 > 0 {
        cursor.0 -= 1;
    }

    if intent.confirm {
        let target = reachable[cursor.0];
        run.position = target;
        run.graph.nodes[target].cleared = true;
        cursor.0 = 0;
        let kind = run.graph.nodes[target].kind;

        // Repaint the travelled node immediately (scene isn't respawned for
        // overlay nodes).
        for (marker, mut sprite) in &mut nodes {
            if marker.0 == target {
                sprite.color = Color::srgb(0.9, 0.8, 0.5);
            }
        }

        match kind {
            NodeKind::Fight | NodeKind::Elite | NodeKind::Boss => {
                let rank = match kind {
                    NodeKind::Elite => FightRank::Elite,
                    NodeKind::Boss => FightRank::Boss,
                    _ => FightRank::Normal,
                };
                let zone = run.roll_zone(&mut rng);
                run.current_fight = Some(rank);
                commands.insert_resource(PendingEncounter {
                    zone,
                    kind: encounter_kind_for(&run, run.graph.nodes[target].kind),
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
            NodeKind::Rest => {
                dialogue.open_rest();
            }
            NodeKind::Market => {
                dialogue.open_market();
            }
        }
    }
    intent.clear();
}

pub fn update_node_cursor(
    time: Res<Time>,
    cursor: Res<MapCursor>,
    run: Option<Res<RunState>>,
    dialogue: Res<RunDialogue>,
    mut ring: Query<(&mut Transform, &mut Visibility), With<MapCursorSprite>>,
) {
    let Some(run) = run else { return };
    let Ok((mut transform, mut visibility)) = ring.single_mut() else {
        return;
    };
    let reachable = run.reachable();
    if reachable.is_empty() || dialogue.active {
        *visibility = Visibility::Hidden;
        return;
    }
    *visibility = Visibility::Visible;
    let node = &run.graph.nodes[reachable[cursor.0.min(reachable.len() - 1)]];
    let pos = node_pos(
        node.layer,
        run.graph.layer_count,
        node.slot,
        node.layer_width,
    );
    transform.translation = pos.extend(-2.0);
    let pulse = 1.0 + (time.elapsed_secs() * 4.0).sin() * 0.06;
    transform.scale = Vec3::splat(pulse);
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

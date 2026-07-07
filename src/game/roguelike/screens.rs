//! Title, post-battle Reward, and Ending screens for the roguelike run.

use bevy::prelude::*;
use bevy::ui::widget::NodeImageMode;

use super::super::core::{GameFont, Intent, PlayerStats, Rng};
use super::super::quest::QuestLog;
use super::content;
use super::event;
use super::{
    BREAKTHROUGH_ATK, BREAKTHROUGH_DEF, BREAKTHROUGH_HP, BREAKTHROUGH_MP, CHAPTER_COUNT, FightRank,
    Relic, RunOutcome, RunState,
};
use crate::game::state::AppState;

// ---------------------------------------------------------------------------
// Title
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct TitleUi;

/// Full-screen backdrop image plus a dark letterbox band so overlay text
/// stays readable on any art.
fn spawn_screen_backdrop(
    commands: &mut Commands,
    asset_server: &AssetServer,
    path: &'static str,
    scope: impl Bundle,
) {
    let mut sprite = Sprite::from_image(asset_server.load(path));
    sprite.custom_size = Some(Vec2::new(1280.0, 720.0));
    commands.spawn((sprite, Transform::from_xyz(0.0, 0.0, -10.0), scope));
}

pub fn spawn_title(mut commands: Commands, font: Res<GameFont>, asset_server: Res<AssetServer>) {
    let scope = || DespawnOnExit(AppState::Title);
    spawn_screen_backdrop(&mut commands, &asset_server, "ui/title_bg.png", scope());
    commands
        .spawn((
            TitleUi,
            scope(),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexEnd,
                row_gap: Val::Px(14.0),
                padding: UiRect::bottom(Val::Px(46.0)),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(14.0),
                        padding: UiRect::axes(Val::Px(46.0), Val::Px(22.0)),
                        ..default()
                    },
                    ImageNode {
                        image: asset_server.load("ui/panel_frame.png"),
                        image_mode: NodeImageMode::Sliced(super::event::panel_slicer()),
                        ..default()
                    },
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("御剑行 · 轮回"),
                        font.text_font(64.0),
                        TextColor(Color::srgb(0.97, 0.88, 0.62)),
                    ));
                    panel.spawn((
                        Text::new("仙侠肉鸽 · 一局一世 · 随机路途 · 剧情织缘"),
                        font.text_font(22.0),
                        TextColor(Color::srgb(0.82, 0.87, 0.95)),
                    ));
                    panel.spawn((
                        Text::new("没有练级,只有抉择:法宝、奇遇与誓言,决定这一世的结局。"),
                        font.text_font(18.0),
                        TextColor(Color::srgba(0.82, 0.87, 0.95, 0.8)),
                    ));
                    panel.spawn((
                        Text::new("—— 按 空格 踏入轮回 ——"),
                        font.text_font(26.0),
                        TextColor(Color::srgb(0.55, 0.92, 1.0)),
                    ));
                });
        });
}

pub fn title_input(
    mut commands: Commands,
    time: Res<Time>,
    mut intent: ResMut<Intent>,
    mut stats: ResMut<PlayerStats>,
    mut quest: ResMut<QuestLog>,
    mut rng: ResMut<Rng>,
    mut next: ResMut<NextState<AppState>>,
) {
    if !intent.confirm {
        return;
    }
    intent.clear();

    // Fresh hero, fresh fate. Mix wall-clock entropy into the RNG so each
    // desktop launch rolls a different world (capture presets re-seed after
    // this for determinism).
    *stats = PlayerStats::default();
    // Run-mode baseline: a touch sturdier than the levelled campaign's Lv.1,
    // since there is no grinding to fall back on.
    stats.atk += 2;
    stats.def += 1;
    stats.potions += 1;
    *quest = QuestLog::default();
    // Mix real-time entropy into the seed. wasm32-unknown-unknown has no
    // SystemTime, so the web build stirs in Bevy's boot-relative clock
    // (frame-exact press timing) instead of the wall clock.
    #[cfg(target_arch = "wasm32")]
    let wall_nanos = (time.elapsed_secs_f64() * 1.0e9) as u64;
    #[cfg(not(target_arch = "wasm32"))]
    let wall_nanos = {
        let _ = &time;
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos() as u64)
            .unwrap_or(0)
    };
    rng.0 ^= (wall_nanos << 16) | 1;
    let run = RunState::new(&mut rng);
    commands.insert_resource(run);
    next.set(AppState::NodeMap);
}

// ---------------------------------------------------------------------------
// Reward (three-choice loot after a won run battle)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub enum RewardOption {
    Relic(Relic),
    HealHalf,
    MaxHp(i32),
    Atk(i32),
    Potions(u32),
    Gold(u32),
}

impl RewardOption {
    fn label(&self) -> String {
        match self {
            RewardOption::Relic(r) => format!("法宝【{}】—— {}", r.name(), r.desc()),
            RewardOption::HealHalf => "疗伤调息 —— 回复五成气血".to_string(),
            RewardOption::MaxHp(n) => format!("淬体丹 —— 气血上限 +{n}"),
            RewardOption::Atk(n) => format!("砺剑石 —— 攻击 +{n}"),
            RewardOption::Potions(n) => format!("行囊补给 —— 药水 ×{n}"),
            RewardOption::Gold(n) => format!("妖丹换钱 —— {n} 文"),
        }
    }
}

#[derive(Resource, Default)]
pub struct RewardChoices {
    pub options: Vec<RewardOption>,
    pub selected: usize,
}

#[derive(Component)]
pub struct RewardListText;

fn roll_relic_option(
    run: &RunState,
    rng: &mut Rng,
    taken: &[RewardOption],
) -> Option<RewardOption> {
    let owned_or_offered = |r: Relic| {
        run.has_relic(r)
            || taken
                .iter()
                .any(|o| matches!(o, RewardOption::Relic(x) if *x == r))
    };
    let pool: Vec<Relic> = super::ALL_RELICS
        .iter()
        .copied()
        .filter(|r| !owned_or_offered(*r))
        .collect();
    if pool.is_empty() {
        return None;
    }
    Some(RewardOption::Relic(
        pool[rng.range(0, pool.len() as i32 - 1) as usize],
    ))
}

fn roll_rewards(run: &RunState, rng: &mut Rng) -> Vec<RewardOption> {
    let elite_or_boss = !matches!(run.current_fight, Some(FightRank::Normal) | None);
    let mut options: Vec<RewardOption> = Vec::new();

    // Slot 1: a relic (guaranteed for elites/bosses, a coin flip otherwise).
    if elite_or_boss || rng.chance(0.45) {
        if let Some(option) = roll_relic_option(run, rng, &options) {
            options.push(option);
        }
    }
    if options.is_empty() {
        options.push(RewardOption::Atk(3));
    }
    // Elites/bosses may show a second relic.
    if elite_or_boss && rng.chance(0.5) {
        if let Some(option) = roll_relic_option(run, rng, &options) {
            options.push(option);
        }
    }
    // Fill remaining slots with distinct consumable/stat picks.
    let mut fillers: Vec<RewardOption> = vec![
        RewardOption::HealHalf,
        RewardOption::Potions(2),
        RewardOption::MaxHp(12),
        RewardOption::Gold(45),
        RewardOption::Atk(2),
    ];
    while options.len() < 3 && !fillers.is_empty() {
        let index = rng.range(0, fillers.len() as i32 - 1) as usize;
        options.push(fillers.swap_remove(index));
    }
    options
}

pub fn spawn_reward(
    mut commands: Commands,
    font: Res<GameFont>,
    asset_server: Res<AssetServer>,
    run: Option<Res<RunState>>,
    mut choices: ResMut<RewardChoices>,
    mut rng: ResMut<Rng>,
    mut next: ResMut<NextState<AppState>>,
) {
    let Some(run) = run else {
        next.set(AppState::Title);
        return;
    };
    choices.options = roll_rewards(&run, &mut rng);
    choices.selected = 0;

    let scope = || DespawnOnExit(AppState::Reward);
    spawn_screen_backdrop(&mut commands, &asset_server, "ui/reward_bg.png", scope());
    commands
        .spawn((
            scope(),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(20.0),
                padding: UiRect::axes(Val::Px(60.0), Val::Px(30.0)),
                ..default()
            },
        ))
        .with_children(|parent| {
            let title = match run.current_fight {
                Some(FightRank::Boss) => "章末大胜 · 择一战利",
                Some(FightRank::Elite) => "力克精英 · 择一战利",
                _ => "小胜一场 · 择一战利",
            };
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(18.0),
                        padding: UiRect::axes(Val::Px(40.0), Val::Px(24.0)),
                        ..default()
                    },
                    ImageNode {
                        image: asset_server.load("ui/panel_frame.png"),
                        image_mode: NodeImageMode::Sliced(super::event::panel_slicer()),
                        ..default()
                    },
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new(title),
                        font.text_font(34.0),
                        TextColor(Color::srgb(0.95, 0.85, 0.55)),
                    ));
                    panel.spawn((
                        RewardListText,
                        Text::new(""),
                        font.text_font(24.0),
                        TextColor(Color::srgb(0.9, 0.92, 0.95)),
                    ));
                    panel.spawn((
                        Text::new("上/下 选择 · 空格 拿取"),
                        font.text_font(16.0),
                        TextColor(Color::srgba(0.85, 0.87, 0.9, 0.7)),
                    ));
                });
        });
}

pub fn reward_input(
    mut commands: Commands,
    mut intent: ResMut<Intent>,
    mut choices: ResMut<RewardChoices>,
    run: Option<ResMut<RunState>>,
    mut stats: ResMut<PlayerStats>,
    mut rng: ResMut<Rng>,
    mut next: ResMut<NextState<AppState>>,
    mut list: Query<&mut Text, With<RewardListText>>,
) {
    let Some(mut run) = run else {
        next.set(AppState::Title);
        return;
    };

    if intent.up && choices.selected > 0 {
        choices.selected -= 1;
    }
    if intent.down && choices.selected + 1 < choices.options.len() {
        choices.selected += 1;
    }

    if intent.confirm && !choices.options.is_empty() {
        let picked = choices.options[choices.selected].clone();
        match picked {
            RewardOption::Relic(relic) => {
                event::grant_relic(relic, &mut stats, &mut run);
            }
            RewardOption::HealHalf => {
                stats.hp = (stats.hp + stats.max_hp / 2).min(stats.max_hp);
            }
            RewardOption::MaxHp(n) => {
                stats.max_hp += n;
                stats.hp += n;
            }
            RewardOption::Atk(n) => stats.atk += n,
            RewardOption::Potions(n) => stats.potions += n,
            RewardOption::Gold(n) => stats.gold += n,
        }

        // Route onwards: boss victories advance the chapter (or end the run).
        let was_boss = matches!(run.current_fight, Some(FightRank::Boss));
        run.current_fight = None;
        if was_boss {
            if run.chapter + 1 >= CHAPTER_COUNT {
                // 道心情缘双高解锁隐藏结局。
                run.outcome = Some(if run.daoxin >= 5 && run.qingyuan >= 5 {
                    RunOutcome::VictoryBoth
                } else if run.qingyuan >= run.daoxin {
                    RunOutcome::VictoryLove
                } else {
                    RunOutcome::VictoryResolve
                });
                next.set(AppState::Ending);
            } else {
                // 境界突破 — the story-driven power curve.
                stats.max_hp += BREAKTHROUGH_HP;
                stats.max_mp += BREAKTHROUGH_MP;
                stats.atk += BREAKTHROUGH_ATK;
                stats.def += BREAKTHROUGH_DEF;
                stats.full_restore();
                run.next_chapter(&mut rng);
                // Fresh chapter: drop the old stage so the hop rebuilds it.
                commands.remove_resource::<super::scene::RunSceneState>();
                next.set(AppState::NodeMap);
            }
        } else {
            // Back to the same map, standing where the fight started.
            next.set(AppState::RunScene);
        }
        commands.remove_resource::<super::RunBattleMods>();
    }
    intent.clear();

    if let Ok(mut text) = list.single_mut() {
        text.0 = choices
            .options
            .iter()
            .enumerate()
            .map(|(i, option)| {
                if i == choices.selected {
                    format!("▶ {}", option.label())
                } else {
                    format!("　 {}", option.label())
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
    }
}

// ---------------------------------------------------------------------------
// Ending
// ---------------------------------------------------------------------------

pub fn spawn_ending(
    mut commands: Commands,
    font: Res<GameFont>,
    asset_server: Res<AssetServer>,
    run: Option<Res<RunState>>,
    mut next: ResMut<NextState<AppState>>,
) {
    let Some(run) = run else {
        next.set(AppState::Title);
        return;
    };
    let lines: &[&str] = match run.outcome {
        Some(RunOutcome::VictoryLove) => content::ENDING_LOVE,
        Some(RunOutcome::VictoryResolve) => content::ENDING_RESOLVE,
        Some(RunOutcome::VictoryBoth) => content::ENDING_BOTH,
        _ => content::ENDING_DEFEAT,
    };
    let scope = || DespawnOnExit(AppState::Ending);
    spawn_screen_backdrop(&mut commands, &asset_server, "ui/ending_bg.png", scope());
    commands
        .spawn((
            scope(),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::horizontal(Val::Px(80.0)),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(12.0),
                        padding: UiRect::axes(Val::Px(44.0), Val::Px(26.0)),
                        ..default()
                    },
                    ImageNode {
                        image: asset_server.load("ui/panel_frame.png"),
                        image_mode: NodeImageMode::Sliced(super::event::panel_slicer()),
                        ..default()
                    },
                ))
                .with_children(|panel| {
                    for (i, line) in lines.iter().enumerate() {
                        let (size, color) = if i == 0 {
                            (34.0, Color::srgb(0.95, 0.85, 0.55))
                        } else {
                            (21.0, Color::srgb(0.9, 0.92, 0.95))
                        };
                        panel.spawn((Text::new(*line), font.text_font(size), TextColor(color)));
                    }
                    // 此世回响:链式奇遇的抉择在结局被记起。
                    let mut echoes: Vec<&str> = Vec::new();
                    match (run.fox_stage, run.fox_kind) {
                        (3, k) if k > 0 => echoes.push("山径上救下的那只白狐,最终以九尾月华相送。"),
                        (3, _) => echoes.push("你偶尔会想起山径上那一眼——凉薄,原来有价。"),
                        (s, k) if s > 0 && k > 0 => echoes.push("桃溪山径的白狐,不知伤好了没有。"),
                        _ => {}
                    }
                    if run.qin_stage >= 2 {
                        echoes.push("宫墙外那支《雾散》,如今真的应验了。");
                    } else if run.qin_stage == 1 {
                        echoes.push("疫城渡口的琴声,后来再没听见过。");
                    }
                    for echo in echoes {
                        panel.spawn((
                            Text::new(echo),
                            font.text_font(18.0),
                            TextColor(Color::srgba(0.85, 0.78, 0.95, 0.9)),
                        ));
                    }
                    panel.spawn((
                        Text::new(format!(
                            "\n此世战绩:胜 {} 场 · 法宝 {} 件 · 道心 {} · 情缘 {}",
                            run.fights_won,
                            run.relics.len(),
                            run.daoxin,
                            run.qingyuan,
                        )),
                        font.text_font(18.0),
                        TextColor(Color::srgba(0.75, 0.8, 0.9, 0.85)),
                    ));
                    panel.spawn((
                        Text::new("—— 按 空格 回到轮回之初 ——"),
                        font.text_font(22.0),
                        TextColor(Color::srgb(0.5, 0.9, 1.0)),
                    ));
                });
        });
}

pub fn ending_input(
    mut commands: Commands,
    mut intent: ResMut<Intent>,
    mut next: ResMut<NextState<AppState>>,
) {
    if !intent.confirm {
        return;
    }
    intent.clear();
    commands.remove_resource::<RunState>();
    next.set(AppState::Title);
}

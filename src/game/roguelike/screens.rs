//! Title, post-battle Reward, and Ending screens for the roguelike run.

use crate::game::ui as theme;
use bevy::prelude::*;

use super::super::core::{GameFont, Intent, PlayerStats, Rng};
use super::super::quest::QuestLog;
use super::content;
use super::narrative;
use super::{CHAPTER_COUNT, FightRank, Relic, RunOutcome, RunState};
use crate::game::state::AppState;

// ---------------------------------------------------------------------------
// Title
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct TitleUi;

#[derive(Component)]
pub(super) struct StartButton;

pub fn title_pointer_input(
    mut intent: ResMut<Intent>,
    mut buttons: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (With<StartButton>, Changed<Interaction>),
    >,
) {
    for (interaction, mut background, mut border) in &mut buttons {
        let hovered = *interaction != Interaction::None;
        background.0 = if hovered {
            Color::srgb(0.17, 0.19, 0.16)
        } else {
            theme::SURFACE
        };
        *border = BorderColor::all(if hovered { theme::GOLD } else { theme::EDGE });
        if *interaction == Interaction::Pressed {
            intent.confirm = true;
        }
    }
}

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
                .spawn((theme::panel(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(14.0),
                    padding: UiRect::axes(Val::Px(46.0), Val::Px(22.0)),
                    ..default()
                }),))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("御剑行 · 轮回"),
                        font.text_font(64.0),
                        TextColor(Color::srgb(0.97, 0.88, 0.62)),
                    ));
                    panel.spawn((
                        Text::new("仙侠肉鸽 · 一局一世 · 随机路途 · 剧情织缘"),
                        font.text_font(22.0),
                        TextColor(theme::MUTED),
                    ));
                    panel.spawn((
                        Text::new("没有练级,只有抉择:法宝、奇遇与誓言,决定这一世的结局。"),
                        font.text_font(18.0),
                        TextColor(Color::srgba(0.82, 0.87, 0.95, 0.8)),
                    ));
                    panel
                        .spawn((
                            Button,
                            StartButton,
                            Node {
                                padding: UiRect::axes(Val::Px(30.0), Val::Px(12.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                border_radius: BorderRadius::all(Val::Px(7.0)),
                                ..default()
                            },
                            BackgroundColor(theme::SURFACE),
                            BorderColor::all(theme::EDGE),
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("踏入轮回"),
                                font.text_font(26.0),
                                TextColor(theme::GOLD),
                            ));
                        });
                    panel.spawn((
                        Text::new("WASD / 方向键移动 · 空格 / 回车交互 · Esc 行囊"),
                        font.text_font(15.0),
                        TextColor(theme::MUTED),
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
    let mut run = RunState::new(&mut rng);
    // 开局技能:御剑·流光保底输出,再随机拓一式(每局起手不同)。
    run.skills.push(super::skill::SkillId::SwordGleam);
    if let Some(id) = super::skill::roll_skill(&run.skills, &mut rng) {
        run.skills.push(id);
    }
    commands.insert_resource(run);
    next.set(AppState::NodeMap);
}

// ---------------------------------------------------------------------------
// Reward (three-choice loot after a won run battle)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub enum RewardOption {
    Relic(Relic),
    Hex(super::hex::HexMark),
    Skill(super::skill::SkillId),
    HealHalf,
    MaxHp(i32),
    Atk(i32),
    Potions(u32),
    Gold(u32),
}

impl RewardOption {
    fn label(&self) -> String {
        match self {
            RewardOption::Relic(r) => format!(
                "〔{}〕法宝【{}】—— {}",
                r.grade().name(),
                r.name(),
                r.desc()
            ),
            RewardOption::Hex(h) => format!(
                "〔{}〕妖纹【{}】—— {}",
                h.grade().name(),
                h.name(),
                h.desc()
            ),
            RewardOption::Skill(id) => {
                let def = super::skill::skill(*id);
                format!(
                    "〔{}〕仙术「{}」—— {}(灵{})",
                    def.grade.name(),
                    def.name(),
                    def.desc(),
                    def.cost
                )
            }
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
pub(crate) struct RewardCard(usize);

#[derive(Component)]
pub(crate) struct RewardCardText(usize);

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
    // 按品级权重抽取:凡品常见,仙品一局难遇。
    let total: u32 = pool.iter().map(|r| r.grade().weight()).sum();
    let mut roll = rng.range(0, total as i32 - 1) as u32;
    for r in &pool {
        let w = r.grade().weight();
        if roll < w {
            return Some(RewardOption::Relic(*r));
        }
        roll -= w;
    }
    pool.last().copied().map(RewardOption::Relic)
}

fn roll_rewards(run: &RunState, rng: &mut Rng) -> Vec<RewardOption> {
    let elite_or_boss = !matches!(run.current_fight, Some(FightRank::Normal) | None);
    let mut options: Vec<RewardOption> = Vec::new();

    // Slot 1: a relic (guaranteed for elites/bosses, a coin flip otherwise;
    // 星孤纹把普通掉率减半)。
    let relic_chance = if run.hex_relic_drop_halved() {
        0.22
    } else {
        0.45
    };
    if elite_or_boss || rng.chance(relic_chance) {
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
    // 妖纹槽:约三成战利里混入一枚有代价的随机词条(海克斯)。
    if options.len() < 3 && rng.chance(0.35) {
        if let Some(h) = super::hex::roll_hex(&run.hexes, rng) {
            options.push(RewardOption::Hex(h));
        }
    }
    // 技能槽:约四成战利里可拓一式仙术秘卷。
    if options.len() < 3 && rng.chance(0.40) {
        if let Some(id) = super::skill::roll_skill(&run.skills, rng) {
            options.push(RewardOption::Skill(id));
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
) {
    let Some(run) = run else {
        return;
    };
    choices.options = roll_rewards(&run, &mut rng);
    choices.selected = 0;
    let scope = || DespawnOnExit(AppState::Reward);
    spawn_screen_backdrop(&mut commands, &asset_server, "ui/reward_bg.png", scope());
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            scope(),
        ))
        .with_children(|root| {
            root.spawn(theme::panel(Node {
                width: Val::Percent(84.0),
                max_width: Val::Px(1080.0),
                padding: UiRect::all(Val::Px(28.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(22.0),
                ..default()
            }))
            .with_children(|p| {
                let title = match run.current_fight {
                    Some(FightRank::Boss) => "章末大胜",
                    Some(FightRank::Elite) => "力克精英",
                    _ => "小胜一场",
                };
                p.spawn((
                    Text::new(title),
                    font.text_font(34.0),
                    TextColor(theme::GOLD),
                ));
                if matches!(run.current_fight, Some(FightRank::Boss)) {
                    let clear = run.chapter_clear_reward();
                    p.spawn((
                        Text::new(format!(
                            "章印 · {}\n{}\n境界突破：气血 +{} · 灵力 +{} · 攻击 +{} · 防御 +{}\n仙术修习：第四至七章的效力加成依次为 +2、+4、+8、+12。",
                            clear.seal, clear.line, clear.hp, clear.mp, clear.atk, clear.def
                        )),
                        font.text_font(17.0),
                        TextColor(theme::TEXT),
                    ));
                }
                p.spawn((
                    Text::new("择一带上，继续此世旅途"),
                    font.text_font(17.0),
                    TextColor(theme::MUTED),
                ));
                p.spawn(Node {
                    width: Val::Percent(100.0),
                    column_gap: Val::Px(16.0),
                    ..default()
                })
                .with_children(|row| {
                    for (index, option) in choices.options.iter().enumerate() {
                        let icon = match option {
                            RewardOption::Relic(_) => "guard",
                            RewardOption::Hex(_) => "combo",
                            RewardOption::Skill(_) => "spell",
                            RewardOption::Atk(_) => "attack",
                            _ => "item",
                        };
                        row.spawn((
                            Button,
                            RewardCard(index),
                            Node {
                                flex_grow: 1.0,
                                flex_basis: Val::Px(0.0),
                                min_height: Val::Px(240.0),
                                padding: UiRect::all(Val::Px(22.0)),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(18.0),
                                border: UiRect::all(Val::Px(1.0)),
                                border_radius: BorderRadius::all(Val::Px(8.0)),
                                ..default()
                            },
                            BackgroundColor(theme::SURFACE),
                            BorderColor::all(theme::EDGE),
                        ))
                        .with_children(|card| {
                            card.spawn((
                                ImageNode::new(asset_server.load(format!("ui/icons/{icon}.png"))),
                                Node {
                                    width: Val::Px(48.0),
                                    height: Val::Px(48.0),
                                    ..default()
                                },
                            ));
                            card.spawn((
                                RewardCardText(index),
                                Text::new(option.label().replace("——", "\n\n")),
                                font.text_font(18.0),
                                TextColor(theme::TEXT),
                            ));
                        });
                    }
                });
                p.spawn((
                    Text::new("↑ ↓ 选择 · 空格拿取 · 点击战利品"),
                    font.text_font(14.0),
                    TextColor(theme::MUTED),
                ));
            });
        });
}

pub(crate) fn reward_input(
    mut commands: Commands,
    mut intent: ResMut<Intent>,
    mut choices: ResMut<RewardChoices>,
    run: Option<ResMut<RunState>>,
    mut stats: ResMut<PlayerStats>,
    mut rng: ResMut<Rng>,
    mut next: ResMut<NextState<AppState>>,
    interactions: Query<(&RewardCard, &Interaction), Changed<Interaction>>,
    mut cards: Query<(&RewardCard, &mut BackgroundColor, &mut BorderColor)>,
    mut labels: Query<(&RewardCardText, &mut TextColor)>,
) {
    let Some(mut run) = run else {
        next.set(AppState::Title);
        return;
    };

    for (card, interaction) in &interactions {
        if *interaction != Interaction::None {
            choices.selected = card.0;
        }
        if *interaction == Interaction::Pressed {
            intent.confirm = true;
        }
    }
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
                narrative::grant_relic(relic, &mut stats, &mut run);
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
            RewardOption::Hex(h) => {
                h.on_pickup(&mut stats);
                run.hexes.push(h);
            }
            RewardOption::Skill(id) => run.skills.push(id),
        }

        // Route onwards: boss victories advance the chapter (or end the run).
        let was_boss = matches!(run.current_fight, Some(FightRank::Boss));
        run.current_fight = None;
        if was_boss {
            let clear = *run.chapter_clear_reward();
            run.record_chapter_clear();
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
                stats.max_hp += clear.hp;
                stats.max_mp += clear.mp;
                stats.atk += clear.atk;
                stats.def += clear.def;
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

    for (card, mut background, mut border) in &mut cards {
        let selected = card.0 == choices.selected;
        background.0 = if selected {
            Color::srgb(0.16, 0.18, 0.15)
        } else {
            theme::SURFACE
        };
        *border = BorderColor::all(if selected { theme::GOLD } else { theme::EDGE });
    }
    for (label, mut color) in &mut labels {
        let grade_color = choices
            .options
            .get(label.0)
            .map(|option| match option {
                RewardOption::Relic(r) => r.grade().color(),
                RewardOption::Hex(h) => h.grade().color(),
                RewardOption::Skill(id) => super::skill::skill(*id).grade.color(),
                _ => theme::TEXT,
            })
            .unwrap_or(theme::TEXT);
        *color = TextColor(grade_color);
    }
}

// ---------------------------------------------------------------------------
// Ending
// ---------------------------------------------------------------------------

/// Three actual recorded decisions explain how this hero grew; never fabricate
/// unseen choices or repeat the whole inventory ledger on the ending screen.
fn ending_choice_recap(run: &RunState) -> Vec<String> {
    run.narrative_log
        .iter()
        .rev()
        .take(3)
        .map(|record| {
            let consequence = record
                .receipt
                .lines()
                .filter(|line| line.starts_with("【实得回礼】"))
                .map(|line| line.trim_start_matches("【实得回礼】"))
                .collect::<Vec<_>>()
                .join("；");
            format!(
                "{}：{}
这次选择带来：{}",
                record.title, record.choice, consequence
            )
        })
        .collect()
}

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
                    crate::game::ui_scroll::ScrollPanel,
                    ScrollPosition::default(),
                    theme::panel(Node {
                        width: Val::Percent(100.0),
                        max_width: Val::Px(1120.0),
                        max_height: Val::Percent(86.0),
                        overflow: Overflow::scroll_y(),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(12.0),
                        padding: UiRect::axes(Val::Px(44.0), Val::Px(26.0)),
                        ..default()
                    }),
                ))
                .with_children(|viewport| {
                    viewport
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            flex_shrink: 0.0,
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(12.0),
                            ..default()
                        })
                        .with_children(|panel| {
                            for (i, line) in lines.iter().enumerate() {
                                let (size, color) = if i == 0 {
                                    (34.0, Color::srgb(0.95, 0.85, 0.55))
                                } else {
                                    (21.0, Color::srgb(0.9, 0.92, 0.95))
                                };
                                panel.spawn((
                                    Text::new(*line),
                                    font.text_font(size),
                                    TextColor(color),
                                ));
                            }
                            // 此世回响:链式奇遇的抉择在结局被记起。
                            let echoes = run.side_story_echoes(matches!(
                                run.outcome,
                                Some(
                                    RunOutcome::VictoryLove
                                        | RunOutcome::VictoryResolve
                                        | RunOutcome::VictoryBoth
                                )
                            ));
                            for echo in echoes {
                                panel.spawn((
                                    Text::new(echo),
                                    font.text_font(18.0),
                                    TextColor(Color::srgba(0.85, 0.78, 0.95, 0.9)),
                                ));
                            }
                            for recap in ending_choice_recap(&run) {
                                panel.spawn((
                                    Text::new(recap),
                                    font.text_font(16.0),
                                    TextColor(theme::MUTED),
                                    Node {
                                        width: Val::Percent(100.0),
                                        ..default()
                                    },
                                ));
                            }
                            panel.spawn((
                                Text::new(format!("章印：{}", run.chapter_seal_summary())),
                                font.text_font(18.0),
                                TextColor(Color::srgba(0.78, 0.92, 1.0, 0.9)),
                            ));
                            panel.spawn((
                                Text::new(run.chapter_vow_archive_summary()),
                                font.text_font(18.0),
                                TextColor(Color::srgba(0.88, 0.82, 1.0, 0.9)),
                            ));
                            panel.spawn((
                                Text::new(format!(
                                    "此世历时：{}\n{}",
                                    run.play_time_summary(),
                                    run.chapter_play_time_archive_summary()
                                )),
                                font.text_font(16.0),
                                TextColor(Color::srgba(0.72, 0.9, 0.82, 0.9)),
                            ));
                            panel.spawn((
                        Text::new(format!(
                            "\n此世战绩:胜 {} 场 · 法宝 {} 件 · 缘忆 {} 段 · 道心 {} · 情缘 {}",
                            run.fights_won,
                            run.relics.len(),
                            run.seen_story_scenes.len(),
                            run.daoxin,
                            run.qingyuan,
                        )),
                        font.text_font(18.0),
                        TextColor(Color::srgba(0.75, 0.8, 0.9, 0.85)),
                    ));
                            panel.spawn((
                                Text::new("—— 按 空格 回到轮回之初 ——"),
                                font.text_font(22.0),
                                TextColor(theme::GOLD),
                            ));
                        });
                });
            parent.spawn((
                Text::new("滚轮或翻页键翻阅 · 空格重新开始"),
                font.text_font(14.0),
                TextColor(theme::TEXT),
                Node {
                    margin: UiRect::top(Val::Px(12.0)),
                    ..default()
                },
            ));
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

#[cfg(test)]
mod ui_tests {
    use super::*;
    #[test]
    fn actual_start_button_hover_does_not_start_but_press_confirms() {
        let mut app = App::new();
        app.insert_resource(Intent::default())
            .add_systems(Update, title_pointer_input);
        let button = app
            .world_mut()
            .spawn((
                Button,
                StartButton,
                Interaction::Hovered,
                BackgroundColor(theme::SURFACE),
                BorderColor::all(theme::EDGE),
            ))
            .id();
        app.update();
        assert!(!app.world().resource::<Intent>().confirm);
        app.world_mut()
            .entity_mut(button)
            .insert(Interaction::Pressed);
        app.update();
        assert!(app.world().resource::<Intent>().confirm);
    }
    #[test]
    fn ending_recap_uses_only_actual_records_and_keeps_three() {
        use super::super::narrative::{NarrativeKind, NarrativeRecord};
        let mut run = RunState::new(&mut Rng::default());
        assert!(ending_choice_recap(&run).is_empty());
        for index in 0..5 {
            run.narrative_log.push(NarrativeRecord {
                kind: NarrativeKind::Event(index),
                chapter: 0,
                title: format!("事件{index}"),
                choice: format!("选择{index}"),
                receipt: format!("【已付代价】钱财 -10 文\n【实得回礼】灵力 +{index}"),
            });
        }
        let recaps = ending_choice_recap(&run);
        assert_eq!(recaps.len(), 3);
        assert!(recaps[0].contains("事件4：选择4"));
        assert!(recaps[0].contains("灵力 +4"));
        assert!(!recaps.join("\n").contains("事件1"));
    }
}

//! Battle presentation; combat rules and timing stay in the parent module.
use super::*;
use crate::game::ui as theme;

#[derive(Component)]
pub(super) struct EnemyHpFill;
#[derive(Component)]
pub(super) struct PlayerHpFill;
#[derive(Component)]
pub(super) struct PlayerMpFill;
#[derive(Component)]
pub(super) struct PressureFill;
#[derive(Component)]
pub(super) struct MenuCard(usize);
#[derive(Component)]
pub(super) struct MenuIcon(usize);
#[derive(Resource, Default)]
pub(super) struct SkillPage(pub usize);
#[derive(Component)]
pub(super) struct PageControls;
#[derive(Component)]
pub(super) struct PageButton(bool);
#[derive(Component)]
pub(super) struct PageText;
#[derive(Component, Clone, Copy)]
pub(super) enum BattleLabel {
    Enemy,
    Intent,
    Tactics,
    Party,
    Vitals,
    Momentum,
    Message,
    DetailTitle,
    Detail,
    Hint,
    Command(usize),
}

pub(super) fn spawn(commands: &mut Commands, font: &GameFont, assets: &AssetServer) {
    commands.insert_resource(SkillPage::default());
    let scope = || DespawnOnExit(AppState::Battle);
    commands
        .spawn((
            theme::panel(Node {
                position_type: PositionType::Absolute,
                top: Val::Px(18.0),
                left: Val::Percent(29.0),
                right: Val::Percent(29.0),
                padding: UiRect::axes(Val::Px(22.0), Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            }),
            scope(),
        ))
        .with_children(|p| {
            p.spawn((
                BattleLabel::Enemy,
                Text::new(""),
                font.text_font(22.0),
                TextColor(theme::TEXT),
            ));
            p.spawn(theme::bar_track(Val::Percent(100.0), 8.0))
                .with_children(|b| {
                    b.spawn((EnemyHpFill, theme::bar_fill(theme::HEALTH)));
                });
            p.spawn((
                BattleLabel::Intent,
                Text::new(""),
                font.text_font(16.0),
                TextColor(theme::GOLD),
            ));
            p.spawn(theme::bar_track(Val::Percent(100.0), 5.0))
                .with_children(|b| {
                    b.spawn((PressureFill, theme::bar_fill(theme::GOLD)));
                });
            p.spawn((
                BattleLabel::Tactics,
                Text::new(""),
                font.text_font(13.0),
                TextColor(theme::GOLD),
            ));
        });
    commands
        .spawn((
            theme::panel(Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(248.0),
                left: Val::Px(24.0),
                right: Val::Px(24.0),
                padding: UiRect::axes(Val::Px(18.0), Val::Px(10.0)),
                ..default()
            }),
            scope(),
        ))
        .with_children(|p| {
            p.spawn((
                BattleLabel::Message,
                Text::new(""),
                font.text_font(17.0),
                TextColor(theme::TEXT),
            ));
        });
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(22.0),
                left: Val::Px(24.0),
                right: Val::Px(24.0),
                height: Val::Px(212.0),
                column_gap: Val::Px(12.0),
                ..default()
            },
            scope(),
        ))
        .with_children(|row| {
            row.spawn(theme::panel(Node {
                width: Val::Percent(27.0),
                padding: UiRect::all(Val::Px(18.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(9.0),
                ..default()
            }))
            .with_children(|p| {
                p.spawn((
                    BattleLabel::Party,
                    Text::new(""),
                    font.text_font(19.0),
                    TextColor(theme::TEXT),
                ));
                p.spawn((
                    BattleLabel::Vitals,
                    Text::new(""),
                    font.text_font(15.0),
                    TextColor(theme::MUTED),
                ));
                p.spawn(theme::bar_track(Val::Percent(100.0), 10.0))
                    .with_children(|b| {
                        b.spawn((PlayerHpFill, theme::bar_fill(theme::HEALTH)));
                    });
                p.spawn(theme::bar_track(Val::Percent(100.0), 7.0))
                    .with_children(|b| {
                        b.spawn((PlayerMpFill, theme::bar_fill(theme::SPIRIT)));
                    });
                p.spawn((
                    BattleLabel::Momentum,
                    Text::new(""),
                    font.text_font(16.0),
                    TextColor(theme::GOLD),
                ));
            });
            row.spawn(theme::panel(Node {
                width: Val::Percent(36.0),
                padding: UiRect::all(Val::Px(18.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            }))
            .with_children(|p| {
                p.spawn((
                    BattleLabel::DetailTitle,
                    Text::new(""),
                    font.text_font(22.0),
                    TextColor(theme::GOLD),
                ));
                p.spawn((
                    BattleLabel::Detail,
                    Text::new(""),
                    font.text_font(16.0),
                    TextColor(theme::TEXT),
                    Node {
                        flex_grow: 1.0,
                        ..default()
                    },
                ));
                p.spawn((
                    BattleLabel::Hint,
                    Text::new(""),
                    font.text_font(13.0),
                    TextColor(theme::MUTED),
                ));
            });
            row.spawn(theme::panel(Node {
                flex_grow: 1.0,
                flex_basis: Val::Px(0.0),
                padding: UiRect::all(Val::Px(12.0)),
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Percent(3.0),
                row_gap: Val::Px(8.0),
                align_content: AlignContent::FlexStart,
                ..default()
            }))
            .with_children(|p| {
                for (i, icon) in ["attack", "guard", "spell", "combo", "item", "flee"]
                    .iter()
                    .enumerate()
                {
                    p.spawn((
                        Button,
                        MenuCard(i),
                        Node {
                            width: Val::Percent(48.5),
                            height: Val::Px(55.0),
                            padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                            column_gap: Val::Px(8.0),
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            ..default()
                        },
                        BackgroundColor(theme::SURFACE),
                        BorderColor::all(Color::NONE),
                    ))
                    .with_children(|b| {
                        b.spawn((
                            MenuIcon(i),
                            ImageNode::new(assets.load(format!("ui/icons/{icon}.png"))),
                            Node {
                                width: Val::Px(32.0),
                                height: Val::Px(32.0),
                                flex_shrink: 0.0,
                                ..default()
                            },
                        ));
                        b.spawn((
                            BattleLabel::Command(i),
                            Text::new(""),
                            font.text_font(16.0),
                            TextColor(theme::TEXT),
                        ));
                    });
                }
                p.spawn((
                    PageControls,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(20.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        display: Display::None,
                        ..default()
                    },
                ))
                .with_children(|row| {
                    for (next, label) in [(false, "‹ 上页"), (true, "下页 ›")] {
                        row.spawn((
                            Button,
                            PageButton(next),
                            Node {
                                padding: UiRect::horizontal(Val::Px(8.0)),
                                ..default()
                            },
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new(label),
                                font.text_font(13.0),
                                TextColor(theme::GOLD),
                            ));
                        });
                        if !next {
                            row.spawn((
                                PageText,
                                Text::new(""),
                                font.text_font(12.0),
                                TextColor(theme::MUTED),
                            ));
                        }
                    }
                });
            });
        });
}

pub(super) fn menu_pointer_input(
    mut state: ResMut<BattleState>,
    mut intent: ResMut<Intent>,
    run: Option<Res<RunState>>,
    cards: Query<(&MenuCard, &Interaction), Changed<Interaction>>,
    pages: Query<(&PageButton, &Interaction), Changed<Interaction>>,
    mut page: ResMut<SkillPage>,
) {
    if state.phase != Phase::Menu {
        return;
    }
    if state.skill_menu {
        let count = run.as_ref().map_or(0, |r| r.skills.len());
        if state.skill_cursor < count {
            page.0 = state.skill_cursor / 5;
        }
        for (button, interaction) in &pages {
            if *interaction == Interaction::Pressed {
                let last = count.saturating_sub(1) / 5;
                page.0 = if button.0 {
                    (page.0 + 1).min(last)
                } else {
                    page.0.saturating_sub(1)
                };
                state.skill_cursor = page.0 * 5;
            }
        }
    }
    for (card, interaction) in &cards {
        if *interaction == Interaction::None {
            continue;
        }
        if state.skill_menu {
            let count = run.as_ref().map_or(0, |r| r.skills.len());
            let index = if card.0 == 5 {
                count
            } else {
                page.0 * 5 + card.0
            };
            if index >= count && card.0 != 5 {
                continue;
            }
            state.skill_cursor = index;
        } else {
            state.menu_index = card.0;
        }
        if *interaction == Interaction::Pressed {
            intent.confirm = true;
        }
    }
}

fn command_detail(
    state: &BattleState,
    quest: &QuestLog,
    run: Option<&RunState>,
    stats: &PlayerStats,
) -> (String, String) {
    if state.skill_menu {
        if let Some(id) = run.and_then(|r| r.skills.get(state.skill_cursor)) {
            let def = super::super::roguelike::skill::skill(*id);
            let cost = run.map_or(def.cost, |r| skill_cost(*id, r));
            return (
                def.name().into(),
                format!(
                    "{} · 灵力 {}{}\n{}{}",
                    def.grade.name(),
                    cost,
                    run.filter(|run| skill_mastery_bonus(run) > 0)
                        .map(|run| format!(" · 修习威力 +{}", skill_mastery_bonus(run)))
                        .unwrap_or_default(),
                    format!(
                        "{}\n{}",
                        def.desc(),
                        if matches!(
                            def.kind,
                            super::super::roguelike::skill::SkillKind::Ward
                                | super::super::roguelike::skill::SkillKind::Mend
                        ) {
                            "调息类：不施压，会放弃反击/追击机会。"
                        } else if matches!(
                            def.kind,
                            super::super::roguelike::skill::SkillKind::Bind
                                | super::super::roguelike::skill::SkillKind::Flurry
                                | super::super::roguelike::skill::SkillKind::Stun
                        ) {
                            "破甲类：施压 2，可穿透架甲。"
                        } else {
                            "进攻类：施压 1，可穿透架甲。"
                        }
                    ),
                    if stats.mp < cost {
                        "\n灵力不足，先御守调息。"
                    } else {
                        ""
                    }
                ),
            );
        }
        return ("返回指令".into(), "合上秘卷，选择其他行动。".into());
    }
    match state.menu_index {
        0 => (
            "挥剑 · 攻击".into(),
            if state.tactics.riposte || state.tactics.exposed {
                "机会已至！下一击伤害 +50%。\n普攻施压 2；反击额外施压 3。".into()
            } else if state.intent == EnemyIntent::Brace {
                "敌人架甲：普攻伤害减半，无法施压。\n改用仙术穿透，或御守回灵。".into()
            } else {
                "普攻施压 2，积累气势。\n架势条填满可打断敌人，打开追击机会。".into()
            },
        ),
        1 => (
            "调息 · 御守".into(),
            if matches!(state.intent, EnemyIntent::Heavy | EnemyIntent::Special) {
                "卸去重击，恢复灵力，气势 +1。\n借势反击：下一击 +50%，额外施压 3。".into()
            } else {
                "降低受击伤害，恢复灵力。\n蓄力或凝气回合不会触发反击。".into()
            },
        ),
        2 if run.is_some_and(|r| !r.skills.is_empty()) => (
            "秘卷 · 仙术".into(),
            "仙术穿透架甲，通常施压 1。\n缠缚、连击、震荡类施压 2；疗伤/护罩不施压。".into(),
        ),
        2 => (
            battle_spell_name(quest, run).into(),
            format!(
                "消耗灵力 {}，施放本章仙术。",
                fallback_spell_cost(quest, run)
            ),
        ),
        3 if run.is_some() => (
            "绝技 · 剑气爆发".into(),
            format!(
                "需要气势 {}，释放剑气。\n先用攻击积势，再寻找出手机会。",
                run.map_or(MOMENTUM_MAX, |r| r.hex_burst_cost())
            ),
        ),
        3 => (
            "同心 · 合击".into(),
            if combo_unlocked(quest) {
                format!("消耗灵力 {COMBO_COST}，与同伴共同出招。")
            } else {
                "尚未解锁，继续旅途结识同伴。".into()
            },
        ),
        4 => (
            "补给 · 药水".into(),
            format!("使用一瓶药水恢复气血。\n现有药水 {} 瓶。", stats.potions),
        ),
        _ => (
            "退避 · 逃跑".into(),
            if run.is_some_and(|r| matches!(r.current_fight, Some(FightRank::Boss))) {
                "退路被首领封住，必须迎战。".into()
            } else {
                "逃跑成功率 50%；失败消耗回合，敌人继续出招。".into()
            },
        ),
    }
}

/// Keep the main result and one important consequence visible.
pub(super) fn latest_message(message: &str) -> String {
    let lines: Vec<&str> = message
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    let Some(primary) = lines
        .iter()
        .copied()
        .find(|line| !line.starts_with("【中毒】") && !line.starts_with("【灼烧】"))
        .or_else(|| lines.first().copied())
    else {
        return String::new();
    };
    let consequence = lines
        .iter()
        .filter(|line| **line != primary)
        .rev()
        .find(|line| {
            [
                "战斗提示",
                "破势",
                "反击",
                "护罩挡下",
                "追击",
                "冰封",
                "震慑",
                "倒下",
                "失去",
                "失败",
            ]
            .iter()
            .any(|key| line.contains(key))
        })
        .copied()
        .or_else(|| lines.iter().copied().find(|line| *line != primary));
    let shorten = |line: &str| {
        let mut text: String = line.chars().take(66).collect();
        if line.chars().count() > 66 {
            text.push('…');
        }
        text
    };
    let mut result = shorten(primary);
    if let Some(consequence) = consequence {
        result.push('\n');
        result.push_str(&shorten(consequence));
    }
    result
}

pub(super) fn update(
    state: Res<BattleState>,
    stats: Res<PlayerStats>,
    quest: Res<QuestLog>,
    run: Option<Res<RunState>>,
    mut bars: ParamSet<(
        Query<&mut Node, With<EnemyHpFill>>,
        Query<&mut Node, With<PlayerHpFill>>,
        Query<&mut Node, With<PlayerMpFill>>,
        Query<&mut Node, With<PressureFill>>,
    )>,
    mut labels: Query<(&BattleLabel, &mut Text, &mut TextColor), Without<PageText>>,
    mut cards: Query<
        (&MenuCard, &mut BackgroundColor, &mut BorderColor, &mut Node),
        (
            Without<EnemyHpFill>,
            Without<PlayerHpFill>,
            Without<PlayerMpFill>,
            Without<PressureFill>,
            Without<PageControls>,
        ),
    >,
    mut page: ResMut<SkillPage>,
    mut controls: Query<
        &mut Node,
        (
            With<PageControls>,
            Without<MenuCard>,
            Without<EnemyHpFill>,
            Without<PlayerHpFill>,
            Without<PlayerMpFill>,
            Without<PressureFill>,
        ),
    >,
    mut page_text: Query<&mut Text, (With<PageText>, Without<BattleLabel>)>,
    mut icons: Query<(&MenuIcon, &mut ImageNode)>,
    assets: Res<AssetServer>,
) {
    let ratio = |value: i32, max: i32| {
        Val::Percent((value.max(0) as f32 / max.max(1) as f32 * 100.0).clamp(0.0, 100.0))
    };
    if let Ok(mut n) = bars.p0().single_mut() {
        n.width = ratio(state.enemy.hp, state.enemy.max_hp);
    }
    if let Ok(mut n) = bars.p1().single_mut() {
        n.width = ratio(stats.hp, stats.max_hp);
    }
    if let Ok(mut n) = bars.p2().single_mut() {
        n.width = ratio(stats.mp, stats.max_mp);
    }
    if let Ok(mut n) = bars.p3().single_mut() {
        n.width = ratio(state.tactics.pressure as i32, state.tactics.limit as i32);
    }
    let run = run.as_deref();
    let skills = run.map(|r| r.skills.as_slice()).unwrap_or_default();
    if state.skill_menu && state.skill_cursor < skills.len() {
        page.0 = state.skill_cursor / 5;
    }
    if let Ok(mut node) = controls.single_mut() {
        node.display = if state.skill_menu {
            Display::Flex
        } else {
            Display::None
        };
    }
    if let Ok(mut text) = page_text.single_mut() {
        text.0 = format!("{} / {}", page.0 + 1, skills.len().div_ceil(5).max(1));
    }
    for (icon, mut image) in &mut icons {
        let name = if state.skill_menu {
            if icon.0 == 5 { "flee" } else { "spell" }
        } else {
            ["attack", "guard", "spell", "combo", "item", "flee"][icon.0]
        };
        let handle = assets.load(format!("ui/icons/{name}.png"));
        if image.image != handle {
            image.image = handle;
        }
    }
    let selected_slot = if state.skill_menu {
        if state.skill_cursor >= skills.len() {
            5
        } else {
            state.skill_cursor % 5
        }
    } else {
        state.menu_index
    };
    let (title, mut detail) = command_detail(&state, &quest, run, &stats);
    if state.player_shield > 0 {
        detail.push_str(&format!("\n护罩 {}", state.player_shield));
    }
    let mut bonuses = Vec::new();
    if state.bond_bonus.is_some() {
        bonuses.push(battle_bond_summary(state.bond_bonus).to_string());
    }
    if state.blessing.is_some() {
        bonuses.push(battle_blessing_summary(state.blessing).to_string());
    }
    if state.camp_bonus.is_some() || state.run_camp_tactic.is_some() {
        bonuses.push(battle_camp_summary(state.camp_bonus, state.run_camp_tactic));
    }
    if !bonuses.is_empty() {
        detail.push_str(&format!("\n{}", bonuses.join(" · ")));
    }
    for (label, mut text, mut color) in &mut labels {
        text.0 = match *label {
            BattleLabel::Enemy => format!(
                "{}{}    {}/{}",
                state.enemy.name,
                if state.boss_phase2 { " · 真身" } else { "" },
                state.enemy.hp.max(0),
                state.enemy.max_hp
            ),
            BattleLabel::Intent => {
                let mut s = if state.phase == Phase::Won {
                    "胜利 · 战利结算中".into()
                } else if state.phase == Phase::EnemyActing {
                    if !state.enemy_attacked {
                        "出手被打断或蓄力中 · 下一手见架势预告".into()
                    } else {
                        state.executing_intent.describe().to_string()
                    }
                } else {
                    state.intent.describe().to_string()
                };
                if state.statuses.poison.turns > 0 {
                    s.push_str(&format!(" · 毒{}", state.statuses.poison.stacks));
                }
                if state.statuses.burn.turns > 0 {
                    s.push_str(&format!(" · 灼烧{}", state.statuses.burn.turns));
                }
                if state.statuses.frozen {
                    s.push_str(" · 冰冻");
                }
                if state.statuses.stunned {
                    s.push_str(" · 震慑");
                }
                if state.statuses.freeze_immunity > 0 {
                    s.push_str(" · 抗冻");
                }
                s
            }
            BattleLabel::Tactics => {
                if state.tactics.broken {
                    "破势 · 即将打断敌方行动".into()
                } else if state.tactics.riposte {
                    "反击就绪 · 下一击 +50%，额外施压 3".into()
                } else if state.tactics.exposed {
                    "破绽已开 · 下一次进攻伤害 +50%".into()
                } else {
                    format!(
                        "{} · 施压 {}/{} · 普攻 2 / 仙术 1 / 破甲术 2",
                        state.tactics.style.name(),
                        state.tactics.pressure,
                        state.tactics.limit
                    )
                }
            }
            BattleLabel::Party => battle_party_summary(&quest, run),
            BattleLabel::Vitals => format!(
                "气血 {}/{}   灵力 {}/{}",
                stats.hp.max(0),
                stats.max_hp,
                stats.mp.max(0),
                stats.max_mp
            ),
            BattleLabel::Momentum => format!(
                "气势 {}   药水 {}",
                (0..MOMENTUM_MAX)
                    .map(|i| if i < state.momentum { '◆' } else { '◇' })
                    .collect::<String>(),
                stats.potions
            ),
            BattleLabel::Message => latest_message(&state.message),
            BattleLabel::DetailTitle => title.clone(),
            BattleLabel::Detail => detail.clone(),
            BattleLabel::Hint => {
                if state.phase == Phase::Menu {
                    "↑ ↓ 选择 · 空格确认 · 点击指令".into()
                } else {
                    "招式演出中…".into()
                }
            }
            BattleLabel::Command(slot) => {
                let selected = state.phase == Phase::Menu && slot == selected_slot;
                *color = TextColor(if selected { theme::GOLD } else { theme::TEXT });
                if state.skill_menu {
                    if slot == 5 {
                        "返回".into()
                    } else {
                        let index = page.0 * 5 + slot;
                        skills
                            .get(index)
                            .map(|id| {
                                let def = super::super::roguelike::skill::skill(*id);
                                let cost = run.map_or(def.cost, |r| skill_cost(*id, r));
                                if !selected {
                                    *color = TextColor(def.grade.color());
                                }
                                format!("{} · {}", def.name(), cost)
                            })
                            .unwrap_or_else(|| "—".into())
                    }
                } else {
                    match slot {
                        2 => "仙术".into(),
                        3 if run.is_some() => "绝技".into(),
                        _ => MENU[slot].into(),
                    }
                }
            }
        };
    }
    for (card, mut background, mut border, mut node) in &mut cards {
        node.height = Val::Px(if state.skill_menu { 47.0 } else { 55.0 });
        let selected = state.phase == Phase::Menu && card.0 == selected_slot;
        background.0 = if selected {
            Color::srgb(0.17, 0.19, 0.16)
        } else {
            theme::SURFACE
        };
        *border = BorderColor::all(if selected { theme::GOLD } else { theme::EDGE });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn battle_notice_keeps_main_result_and_key_consequence_and_bounds_lines() {
        assert_eq!(latest_message("开战\n伤害 12\n\n"), "开战\n伤害 12");
        assert_eq!(
            latest_message("造成 12 点伤害\n护罩挡下 6 点伤害\n法宝伤害 +9"),
            "造成 12 点伤害\n护罩挡下 6 点伤害"
        );
        assert_eq!(latest_message(&"字".repeat(90)).chars().count(), 67);
        assert_eq!(
            latest_message("【中毒】造成 3 点伤害\n敌人造成 10 点伤害\n护罩挡下 6 点伤害"),
            "敌人造成 10 点伤害\n护罩挡下 6 点伤害"
        );
    }

    #[test]
    fn fallback_command_text_includes_hex_and_relic_price_modifiers() {
        let mut state = super::super::tests::status_test_state(EncounterKind::Random);
        state.phase = Phase::Menu;
        state.menu_index = 2;
        let quest = QuestLog::default();
        let stats = PlayerStats::default();
        for (hex, relic, expected) in [
            (false, false, 5),
            (true, false, 7),
            (false, true, 3),
            (true, true, 5),
        ] {
            let mut run = RunState::new(&mut Rng::default());
            if hex {
                run.hexes
                    .push(crate::game::roguelike::hex::HexMark::MoonBite);
            }
            if relic {
                run.relics
                    .push(crate::game::roguelike::Relic::SpiritPendant);
            }
            let (_, text) = command_detail(&state, &quest, Some(&run), &stats);
            assert!(text.contains(&format!("消耗灵力 {expected}，")), "{text}");
        }
    }

    fn pointer_app(skill_count: usize, selected: usize) -> App {
        let mut app = App::new();
        let mut state = super::super::tests::status_test_state(EncounterKind::Random);
        state.phase = Phase::Menu;
        state.skill_menu = true;
        state.skill_cursor = selected;
        let mut run = RunState::new(&mut Rng::default());
        run.skills = vec![crate::game::roguelike::skill::SkillId::SwordGleam; skill_count];
        app.insert_resource(state)
            .insert_resource(run)
            .insert_resource(Intent::default())
            .insert_resource(SkillPage(selected / 5))
            .add_systems(Update, menu_pointer_input);
        app
    }

    #[test]
    fn hovering_return_with_exactly_five_skills_retains_visible_page() {
        let mut app = pointer_app(5, 4);
        let return_card = app
            .world_mut()
            .spawn((MenuCard(5), Interaction::Hovered))
            .id();
        app.update();
        assert_eq!(app.world().resource::<BattleState>().skill_cursor, 5);
        assert_eq!(app.world().resource::<SkillPage>().0, 0);
        assert!(!app.world().resource::<Intent>().confirm);
        // Subsequent frames must retain the page when Return remains selected.
        app.update();
        assert_eq!(app.world().resource::<SkillPage>().0, 0);
        app.world_mut().despawn(return_card);
        app.world_mut().spawn((MenuCard(2), Interaction::Hovered));
        app.update();
        assert_eq!(app.world().resource::<BattleState>().skill_cursor, 2);
    }

    #[test]
    fn hovering_return_retains_first_and_last_skill_pages() {
        for selected in [1, 6] {
            let mut app = pointer_app(7, selected);
            app.world_mut().spawn((MenuCard(5), Interaction::Hovered));
            app.update();
            assert_eq!(app.world().resource::<BattleState>().skill_cursor, 7);
            assert_eq!(app.world().resource::<SkillPage>().0, selected / 5);
            app.update();
            assert_eq!(app.world().resource::<SkillPage>().0, selected / 5);
        }
    }

    #[test]
    fn pointer_next_and_previous_reach_other_skill_pages_and_clamp() {
        let mut app = pointer_app(12, 0);
        let next = app
            .world_mut()
            .spawn((PageButton(true), Interaction::Pressed))
            .id();
        app.update();
        assert_eq!(app.world().resource::<SkillPage>().0, 1);
        assert_eq!(app.world().resource::<BattleState>().skill_cursor, 5);
        assert!(!app.world().resource::<Intent>().confirm);
        app.world_mut().despawn(next);
        let next = app
            .world_mut()
            .spawn((PageButton(true), Interaction::Pressed))
            .id();
        app.update();
        assert_eq!(app.world().resource::<SkillPage>().0, 2);
        assert_eq!(app.world().resource::<BattleState>().skill_cursor, 10);
        app.world_mut().despawn(next);
        let next = app
            .world_mut()
            .spawn((PageButton(true), Interaction::Pressed))
            .id();
        app.update();
        assert_eq!(app.world().resource::<SkillPage>().0, 2);
        app.world_mut().despawn(next);
        app.world_mut()
            .spawn((PageButton(false), Interaction::Pressed));
        app.update();
        assert_eq!(app.world().resource::<SkillPage>().0, 1);
        assert_eq!(app.world().resource::<BattleState>().skill_cursor, 5);
    }

    #[test]
    fn pressing_skill_selects_page_relative_index_and_confirms() {
        let mut app = pointer_app(7, 5);
        app.world_mut().spawn((MenuCard(1), Interaction::Pressed));
        app.update();
        assert_eq!(app.world().resource::<BattleState>().skill_cursor, 6);
        assert_eq!(app.world().resource::<SkillPage>().0, 1);
        assert!(app.world().resource::<Intent>().confirm);
    }

    #[test]
    fn pointer_ignores_unavailable_slots_and_nonmenu_combat_phase() {
        let mut app = pointer_app(7, 5);
        let missing = app
            .world_mut()
            .spawn((MenuCard(4), Interaction::Pressed))
            .id();
        app.update();
        assert_eq!(app.world().resource::<BattleState>().skill_cursor, 5);
        assert!(!app.world().resource::<Intent>().confirm);
        app.world_mut().despawn(missing);
        app.world_mut().resource_mut::<BattleState>().phase = Phase::PlayerActing;
        app.world_mut().spawn((MenuCard(1), Interaction::Pressed));
        app.world_mut()
            .spawn((PageButton(false), Interaction::Pressed));
        app.update();
        assert_eq!(app.world().resource::<BattleState>().skill_cursor, 5);
        assert_eq!(app.world().resource::<SkillPage>().0, 1);
        assert!(!app.world().resource::<Intent>().confirm);
    }
}

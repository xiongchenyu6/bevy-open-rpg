//! Run-mode dialogue/event overlay.
//!
//! One overlay serves chapter cards, story scenes (「缘」), random events
//! (「遇」) and rests (「歇」). It lives inside the `NodeMap` state as a
//! resource-driven UI box, so the map stays visible behind the text.

use bevy::prelude::*;

use super::super::core::{GameFont, Intent, PlayerStats, Rng};
use super::content::{self, Effect, Outcome};
use super::{ALL_RELICS, RunState};

// ---------------------------------------------------------------------------
// Overlay state
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum DialogueSource {
    #[default]
    Plain,
    /// Index into [`content::EVENTS`].
    Event(usize),
    /// Story scene of the current chapter (roll picks from the pool).
    Story {
        chapter: usize,
        roll: usize,
    },
    Rest,
    Market,
}

#[derive(Resource, Default)]
pub struct RunDialogue {
    pub active: bool,
    pub title: String,
    pub lines: Vec<String>,
    pub idx: usize,
    /// Option labels; shown once `idx` has walked past `lines`.
    pub options: Vec<String>,
    pub selected: usize,
    pub source: DialogueSource,
    /// Options already resolved (so the tail lines just advance and close).
    pub resolved: bool,
}

impl RunDialogue {
    pub fn open_plain(&mut self, title: &str, lines: &[&str]) {
        *self = Self {
            active: true,
            title: title.to_string(),
            lines: lines.iter().map(|s| s.to_string()).collect(),
            source: DialogueSource::Plain,
            resolved: true,
            ..default()
        };
    }

    pub fn open_event(&mut self, index: usize) {
        let ev = &content::EVENTS[index];
        *self = Self {
            active: true,
            title: format!("奇遇 · {}", ev.title),
            lines: ev.lines.iter().map(|s| s.to_string()).collect(),
            options: ev.options.iter().map(|o| o.label.to_string()).collect(),
            source: DialogueSource::Event(index),
            ..default()
        };
    }

    pub fn open_story(&mut self, chapter: usize, roll: usize) {
        let scene = content::pick_story(chapter, roll);
        let mut lines: Vec<String> = scene.lines.iter().map(|s| s.to_string()).collect();
        lines.push(scene.prompt.to_string());
        *self = Self {
            active: true,
            title: "剧情 · 缘".to_string(),
            lines,
            options: scene.options.iter().map(|o| o.label.to_string()).collect(),
            source: DialogueSource::Story { chapter, roll },
            ..default()
        };
    }

    pub fn open_rest(&mut self) {
        *self = Self {
            active: true,
            title: "歇脚".to_string(),
            lines: content::REST_INTRO.iter().map(|s| s.to_string()).collect(),
            options: vec![
                content::REST_MEDITATE.to_string(),
                content::REST_SPAR.to_string(),
                content::REST_TALK.to_string(),
            ],
            source: DialogueSource::Rest,
            ..default()
        };
    }

    pub fn open_market(&mut self) {
        *self = Self {
            active: true,
            title: "集市".to_string(),
            lines: content::MARKET_INTRO
                .iter()
                .map(|s| s.to_string())
                .collect(),
            options: vec![
                content::MARKET_POTION.to_string(),
                content::MARKET_TONIC.to_string(),
                content::MARKET_WHETSTONE.to_string(),
                content::MARKET_LEAVE.to_string(),
            ],
            source: DialogueSource::Market,
            ..default()
        };
    }

    /// The overlay is waiting on an option pick (all lines shown, unresolved).
    fn in_choice(&self) -> bool {
        !self.resolved && self.idx + 1 >= self.lines.len() && !self.options.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Effects
// ---------------------------------------------------------------------------

/// Apply an outcome's mechanical effect. Returns an extra feedback line for
/// effects whose result is rolled at runtime (e.g. which relic dropped).
pub fn apply_outcome(
    outcome: &Outcome,
    stats: &mut PlayerStats,
    run: &mut RunState,
    rng: &mut Rng,
) -> Option<String> {
    run.daoxin += outcome.daoxin;
    run.qingyuan += outcome.qingyuan;
    apply_effect(outcome.effect, stats, run, rng)
}

pub fn apply_effect(
    effect: Effect,
    stats: &mut PlayerStats,
    run: &mut RunState,
    rng: &mut Rng,
) -> Option<String> {
    match effect {
        Effect::None => None,
        Effect::HealPct(pct) => {
            let amount = (stats.max_hp * pct / 100).max(1);
            stats.hp = (stats.hp + amount).min(stats.max_hp);
            None
        }
        Effect::DamagePct(pct) => {
            let amount = (stats.max_hp * pct / 100).max(1);
            stats.hp = (stats.hp - amount).max(1);
            None
        }
        Effect::GainPotions(n) => {
            stats.potions += n;
            None
        }
        Effect::GainGold(n) => {
            stats.gold += n;
            None
        }
        Effect::LoseGold(n) => {
            stats.gold = stats.gold.saturating_sub(n);
            None
        }
        Effect::GainAtk(n) => {
            stats.atk += n;
            None
        }
        Effect::GainDef(n) => {
            stats.def += n;
            None
        }
        Effect::GainMaxHp(n) => {
            stats.max_hp += n;
            stats.hp += n;
            None
        }
        Effect::GainMaxMp(n) => {
            stats.max_mp += n;
            stats.mp += n;
            None
        }
        Effect::RandomRelic => Some(grant_random_relic(stats, run, rng)),
    }
}

/// Grant a random unowned relic (falls back to gold when the collection is
/// complete). Returns the feedback line.
pub fn grant_random_relic(stats: &mut PlayerStats, run: &mut RunState, rng: &mut Rng) -> String {
    let unowned: Vec<_> = ALL_RELICS
        .iter()
        .copied()
        .filter(|r| !run.has_relic(*r))
        .collect();
    if unowned.is_empty() {
        stats.gold += 50;
        return "法宝已集齐,化作 50 文钱财。".to_string();
    }
    let relic = unowned[rng.range(0, unowned.len() as i32 - 1) as usize];
    grant_relic(relic, stats, run)
}

/// Add a relic and apply its pick-up effects. Returns the feedback line.
pub fn grant_relic(relic: super::Relic, stats: &mut PlayerStats, run: &mut RunState) -> String {
    use super::Relic;
    run.relics.push(relic);
    match relic {
        Relic::PeachHairpin => {
            run.qingyuan += 2;
            stats.max_hp += 15;
            stats.hp += 15;
        }
        Relic::HeavenScroll => {
            run.daoxin += 2;
            stats.max_mp += 10;
            stats.mp += 10;
        }
        Relic::StarSand => {
            stats.max_mp += 12;
            stats.mp += 12;
        }
        Relic::GinsengRoot => {
            stats.max_hp += 20;
            stats.hp += 20;
        }
        _ => {}
    }
    format!("获得法宝【{}】:{}", relic.name(), relic.desc())
}

// ---------------------------------------------------------------------------
// Input: advance lines / pick options
// ---------------------------------------------------------------------------

pub fn run_dialogue_input(
    mut dialogue: ResMut<RunDialogue>,
    mut intent: ResMut<Intent>,
    mut stats: ResMut<PlayerStats>,
    run: Option<ResMut<RunState>>,
    mut rng: ResMut<Rng>,
) {
    if !dialogue.active {
        return;
    }
    let Some(mut run) = run else {
        dialogue.active = false;
        return;
    };

    if dialogue.in_choice() {
        if intent.up && dialogue.selected > 0 {
            dialogue.selected -= 1;
        }
        if intent.down && dialogue.selected + 1 < dialogue.options.len() {
            dialogue.selected += 1;
        }
        if intent.confirm {
            let pick = dialogue.selected;
            resolve_choice(&mut dialogue, pick, &mut stats, &mut run, &mut rng);
        }
        // Dialogue swallows all input while open.
        intent.clear();
        return;
    }

    if intent.confirm {
        if dialogue.idx + 1 < dialogue.lines.len() {
            dialogue.idx += 1;
        } else {
            dialogue.active = false;
        }
    }
    intent.clear();
}

fn resolve_choice(
    dialogue: &mut RunDialogue,
    pick: usize,
    stats: &mut PlayerStats,
    run: &mut RunState,
    rng: &mut Rng,
) {
    let outcome: Outcome = match dialogue.source {
        DialogueSource::Event(index) => {
            let option = &content::EVENTS[index].options[pick];
            if rng.chance(option.chance) {
                option.success
            } else {
                option.failure.unwrap_or(option.success)
            }
        }
        DialogueSource::Story { chapter, roll } => {
            content::pick_story(chapter, roll).options[pick].outcome
        }
        DialogueSource::Rest => {
            // Resolved fully in code: 0 = meditate, 1 = spar, 2 = heart-to-heart.
            let mult = run.rest_multiplier();
            match pick {
                0 => {
                    let amount = stats.max_hp * 50 * mult / 100;
                    stats.hp = (stats.hp + amount).min(stats.max_hp);
                    dialogue
                        .lines
                        .push(format!("一夜吐纳,气血回复了 {amount} 点。"));
                }
                1 => {
                    stats.atk += 2;
                    let amount = stats.max_hp * 10 * mult / 100;
                    stats.hp = (stats.hp + amount).min(stats.max_hp);
                    dialogue
                        .lines
                        .push("以火光为敌手拆招至深夜,剑势又利了几分。(攻击 +2)".to_string());
                }
                _ => {
                    run.qingyuan += 1;
                    let amount = stats.max_hp * 30 * mult / 100;
                    stats.hp = (stats.hp + amount).min(stats.max_hp);
                    dialogue.lines.push(
                        "灵儿讲起小时候偷摘桃子被追着跑的糗事,火堆边的夜忽然就不冷了。(情缘 +1)"
                            .to_string(),
                    );
                }
            }
            dialogue.resolved = true;
            dialogue.options.clear();
            dialogue.idx += 1;
            return;
        }
        DialogueSource::Market => {
            // Purchases keep the stall open; leaving closes it.
            let line = match pick {
                0 => {
                    if stats.spend_gold(40) {
                        stats.potions += 1;
                        format!(
                            "摊主麻利地包好一瓶药水。(药水 ×{},余 {} 文)",
                            stats.potions, stats.gold
                        )
                    } else {
                        "钱袋一抖,铜板不够。摊主笑而不语。".to_string()
                    }
                }
                1 => {
                    if stats.spend_gold(70) {
                        stats.max_hp += 12;
                        stats.hp += 12;
                        format!("淬体丹入腹,筋骨微热。(气血上限 +12,余 {} 文)", stats.gold)
                    } else {
                        "淬体丹好是好,就是买不起。".to_string()
                    }
                }
                2 => {
                    if stats.spend_gold(60) {
                        stats.atk += 2;
                        format!(
                            "就着摊边的水槽把剑磨利了三分。(攻击 +2,余 {} 文)",
                            stats.gold
                        )
                    } else {
                        "砺剑石沉手,钱袋更轻,还是算了。".to_string()
                    }
                }
                _ => {
                    dialogue
                        .lines
                        .push("摊主们拱手相送:「剑客慢走,前路顺风。」".to_string());
                    dialogue.resolved = true;
                    dialogue.options.clear();
                    dialogue.idx += 1;
                    return;
                }
            };
            // Stay in choice mode: append feedback and keep the options up.
            dialogue.lines.push(line);
            dialogue.idx = dialogue.lines.len() - 1;
            return;
        }
        DialogueSource::Plain => return,
    };

    // Append outcome lines and any rolled-effect feedback, then fall into
    // plain line-advancing mode.
    for line in outcome.lines {
        dialogue.lines.push(line.to_string());
    }
    if let Some(extra) = apply_outcome(&outcome, stats, run, rng) {
        dialogue.lines.push(extra);
    }
    dialogue.resolved = true;
    dialogue.options.clear();
    dialogue.idx += 1;
}

// ---------------------------------------------------------------------------
// UI
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct RunDialogueRoot;

#[derive(Component)]
pub struct RunDialogueTitle;

#[derive(Component)]
pub struct RunDialogueLine;

#[derive(Component)]
pub struct RunDialogueOptions;

/// Spawn the (hidden) overlay box. Called from the node-map scene setup so it
/// carries the same `DespawnOnExit` scope.
pub fn spawn_run_dialogue_ui(commands: &mut Commands, font: &GameFont, scope: impl Bundle) {
    commands
        .spawn((
            RunDialogueRoot,
            scope,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(8.0),
                right: Val::Percent(8.0),
                bottom: Val::Px(28.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                padding: UiRect::axes(Val::Px(18.0), Val::Px(14.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.05, 0.10, 0.92)),
            Visibility::Hidden,
            GlobalZIndex(50),
        ))
        .with_children(|parent| {
            parent.spawn((
                RunDialogueTitle,
                Text::new(""),
                font.text_font(20.0),
                TextColor(Color::srgb(0.95, 0.83, 0.52)),
            ));
            parent.spawn((
                RunDialogueLine,
                Text::new(""),
                font.text_font(22.0),
                TextColor(Color::srgb(0.94, 0.94, 0.90)),
            ));
            parent.spawn((
                RunDialogueOptions,
                Text::new(""),
                font.text_font(20.0),
                TextColor(Color::srgb(0.75, 0.88, 1.0)),
            ));
        });
}

pub fn update_run_dialogue_ui(
    dialogue: Res<RunDialogue>,
    mut root: Query<&mut Visibility, With<RunDialogueRoot>>,
    mut texts: ParamSet<(
        Query<&mut Text, With<RunDialogueTitle>>,
        Query<&mut Text, With<RunDialogueLine>>,
        Query<&mut Text, With<RunDialogueOptions>>,
    )>,
) {
    let Ok(mut visibility) = root.single_mut() else {
        return;
    };
    *visibility = if dialogue.active {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    if !dialogue.active {
        return;
    }

    if let Ok(mut text) = texts.p0().single_mut() {
        text.0 = dialogue.title.clone();
    }
    if let Ok(mut text) = texts.p1().single_mut() {
        text.0 = dialogue
            .lines
            .get(dialogue.idx)
            .cloned()
            .unwrap_or_default();
    }
    if let Ok(mut text) = texts.p2().single_mut() {
        text.0 = if dialogue.in_choice() {
            dialogue
                .options
                .iter()
                .enumerate()
                .map(|(i, label)| {
                    if i == dialogue.selected {
                        format!("▶ {label}")
                    } else {
                        format!("　 {label}")
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            "(空格继续)".to_string()
        };
    }
}

//! Run-mode dialogue/event overlay.
//!
//! One overlay serves chapter cards, story scenes (「缘」), random events
//! (「遇」) and rests (「歇」). It lives inside the `NodeMap` state as a
//! resource-driven UI box, so the map stays visible behind the text.

use bevy::prelude::*;
use bevy::ui::widget::NodeImageMode;

use super::super::core::{GameFont, Intent, PlayerStats, Rng};
use super::super::quest::Companion;
use super::content::{self, Effect, Outcome};
use super::graph::NodeKind;
use super::{ALL_RELICS, RunCampTactic, RunChapterVow, RunState};
use crate::game::state::AppState;

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
    /// Explicit route-task contract shown at the start of each run map.
    RouteTask,
    /// Optional local errand offered by a visible Guide marker.
    RouteCommission {
        target: NodeKind,
    },
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
    /// Asset path of the speaker/scene portrait card, if any.
    pub portrait: Option<&'static str>,
    /// 对峙对话结束后立即开打(章末魔门的先礼后兵)。
    pub boss_battle_after: bool,
    /// 任务交付对话关闭后推进下一程。
    pub advance_stage_after: bool,
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

    pub fn open_plain_owned(&mut self, title: &str, lines: Vec<String>) {
        *self = Self {
            active: true,
            title: title.to_string(),
            lines,
            source: DialogueSource::Plain,
            resolved: true,
            ..default()
        };
    }

    pub fn open_plain_owned_with_portrait(
        &mut self,
        title: &str,
        lines: Vec<String>,
        portrait: Option<&'static str>,
    ) {
        *self = Self {
            active: true,
            title: title.to_string(),
            lines,
            source: DialogueSource::Plain,
            resolved: true,
            portrait,
            ..default()
        };
    }

    pub fn open_route_task(&mut self, title: &str, lines: Vec<String>) {
        *self = Self {
            active: true,
            title: title.to_string(),
            lines,
            options: vec!["领取并追踪".to_string(), "先看地图".to_string()],
            source: DialogueSource::RouteTask,
            resolved: false,
            ..default()
        };
    }

    pub fn open_route_task_turn_in(&mut self, title: &str, lines: Vec<String>) {
        *self = Self {
            active: true,
            title: title.to_string(),
            lines,
            source: DialogueSource::Plain,
            resolved: true,
            advance_stage_after: true,
            ..default()
        };
    }

    pub fn open_route_commission(
        &mut self,
        title: &str,
        lines: Vec<String>,
        target: NodeKind,
        portrait: Option<&'static str>,
    ) {
        *self = Self {
            active: true,
            title: title.to_string(),
            lines,
            options: vec!["领取并追踪".to_string(), "只问路线".to_string()],
            source: DialogueSource::RouteCommission { target },
            resolved: false,
            portrait,
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
            portrait: content::event_portrait(index),
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
            portrait: Some(content::story_portrait(chapter, roll)),
            ..default()
        };
    }

    pub fn open_rest(&mut self, run: &RunState) {
        let mut lines = content::REST_INTRO
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        lines.push(run.camp_scene_line());
        lines.push(run.camp_party_line().to_string());
        lines.push("选择今晚的营策；它会写入行囊，并在下一场战斗开场消耗。".to_string());
        let options = run
            .available_camp_tactics()
            .into_iter()
            .map(|tactic| run.camp_tactic_label(tactic).to_string())
            .collect();
        let portrait = if run.party_companions().contains(&Companion::SpiritWitch) {
            Some(content::PORTRAIT_SPIRIT_WITCH)
        } else if run.party_companions().contains(&Companion::SwordSister) {
            Some(content::PORTRAIT_SWORD_SISTER)
        } else {
            Some(content::PORTRAIT_LINGER)
        };
        *self = Self {
            active: true,
            title: "营地 · 歇脚".to_string(),
            lines,
            options,
            source: DialogueSource::Rest,
            portrait,
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
            portrait: Some(content::PORTRAIT_MERCHANT),
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
        Effect::LosePotions(n) => {
            stats.potions = stats.potions.saturating_sub(n);
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
    mut next: ResMut<NextState<AppState>>,
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
            let advance_stage = dialogue.advance_stage_after;
            dialogue.advance_stage_after = false;
            dialogue.active = false;
            if advance_stage {
                run.stage += 1;
                next.set(AppState::NodeMap);
            }
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
            // 白狐初遇:救或不救,写进这一世的因果。
            if index == content::EV_FOX1 {
                run.fox_kind = if pick == 0 { 1 } else { -1 };
            }
            let option = &content::EVENTS[index].options[pick];
            if rng.chance(option.chance) {
                option.success
            } else {
                option.failure.unwrap_or(option.success)
            }
        }
        DialogueSource::Story { chapter, roll } => {
            let outcome = content::pick_story(chapter, roll).options[pick].outcome;
            let vow = RunChapterVow::from_scores(outcome.daoxin, outcome.qingyuan);
            let fresh = run.record_chapter_vow(chapter, vow);
            let state = if fresh { "已立下" } else { "已记录过" };
            dialogue.lines.push(format!(
                "【本卷誓记】{} {state}:{}",
                vow.name(),
                vow.record_line()
            ));
            outcome
        }
        DialogueSource::Rest => {
            let tactics = run.available_camp_tactics();
            let tactic = tactics.get(pick).copied().unwrap_or(RunCampTactic::Breath);
            let first_visit = run.record_camp_scene();
            run.set_camp_tactic(tactic);
            let mult = run.rest_multiplier();
            let mut result_line = match tactic {
                RunCampTactic::Breath => {
                    let amount = stats.max_hp * 50 * mult / 100;
                    stats.hp = (stats.hp + amount).min(stats.max_hp);
                    let mana = (stats.max_mp * 30 / 100).max(1);
                    stats.mp = (stats.mp + mana).min(stats.max_mp);
                    format!("一夜吐纳,气血回复了 {amount} 点,灵力回稳 {mana} 点。")
                }
                RunCampTactic::SwordGuard => {
                    stats.atk += 2;
                    let amount = stats.max_hp * 10 * mult / 100;
                    stats.hp = (stats.hp + amount).min(stats.max_hp);
                    "以火光为敌手拆招至深夜,剑势又利了几分。(攻击 +2)".to_string()
                }
                RunCampTactic::LingerWard => {
                    run.qingyuan += 1;
                    let amount = stats.max_hp * 30 * mult / 100;
                    stats.hp = (stats.hp + amount).min(stats.max_hp);
                    "灵儿讲起小时候偷摘桃子被追着跑的糗事,火堆边的夜忽然就不冷了。(情缘 +1)"
                        .to_string()
                }
                RunCampTactic::SpiritFocus => {
                    run.daoxin += 1;
                    let mana = (stats.max_mp * 60 / 100).max(1);
                    stats.mp = (stats.mp + mana).min(stats.max_mp);
                    "南瑶以铜铃压住杂乱灵纹,队伍心口都清明了一线。(道心 +1)".to_string()
                }
            };
            if first_visit {
                result_line.push_str(&format!(
                    "\n【营火照应】本世已记下 {} 处营地。",
                    run.camp_scenes_seen.len()
                ));
            } else {
                result_line.push_str("\n【营火照应】这处营地已记录过,本次只更新下一战营策。");
            }
            result_line.push_str(&format!(
                "\n【营策已备】{}:{}",
                tactic.name(),
                tactic.battle_line()
            ));
            dialogue.lines.push(result_line);
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
        DialogueSource::RouteTask => {
            if pick == 0 {
                let receipt = run.journey_task_receipt();
                let already = !run.accept_journey_task();
                let state = if already {
                    "已经在任务札中"
                } else {
                    "已写入任务札"
                };
                dialogue.lines.push(format!(
                    "【主线已接取】{receipt} {state}。HUD 与行囊会持续追踪完成条件。"
                ));
            } else {
                dialogue.lines.push(format!(
                    "【暂未签收】{} 仍待确认；靠近主线标记或界门前会再次展开任务契约。",
                    run.journey_task_receipt()
                ));
            }
            dialogue.resolved = true;
            dialogue.options.clear();
            dialogue.idx += 1;
            return;
        }
        DialogueSource::RouteCommission { target } => {
            if pick == 0 {
                let receipt = run.route_commission_receipt();
                let already = !run.accept_route_commission(target);
                let state = if already {
                    "已经在行囊中追踪"
                } else {
                    "已写入行囊"
                };
                dialogue.lines.push(format!(
                    "【路人委托已接取】{receipt} {state}。目标：清理本程的「{}」标记；完成后自动回执。",
                    target.label()
                ));
            } else {
                dialogue.lines.push(format!(
                    "【只问路线】{} 暂不接取；本程仍可继续主线。",
                    run.route_commission_receipt()
                ));
            }
            dialogue.resolved = true;
            dialogue.options.clear();
            dialogue.idx += 1;
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

#[cfg(test)]
mod tests {
    use super::super::super::core::{PlayerStats, Rng};
    use super::*;

    #[test]
    fn route_task_choice_accepts_current_journey_receipt() {
        let mut rng = Rng::default();
        let mut run = RunState::new(&mut rng);
        let mut stats = PlayerStats::default();
        let mut dialogue = RunDialogue::default();

        dialogue.open_route_task("任务札", vec!["【主线契约】主线签 卷1-01".to_string()]);
        assert!(!run.is_journey_task_accepted());

        resolve_choice(&mut dialogue, 0, &mut stats, &mut run, &mut rng);

        assert!(run.is_journey_task_accepted());
        assert!(dialogue.resolved);
        assert!(dialogue.options.is_empty());
        assert!(
            dialogue
                .lines
                .iter()
                .any(|line| line.contains("【主线已接取】主线签 卷1-01"))
        );
    }

    #[test]
    fn route_task_cancel_leaves_receipt_unsigned() {
        let mut rng = Rng::default();
        let mut run = RunState::new(&mut rng);
        let mut stats = PlayerStats::default();
        let mut dialogue = RunDialogue::default();

        dialogue.open_route_task("任务札", vec!["【主线契约】主线签 卷1-01".to_string()]);
        resolve_choice(&mut dialogue, 1, &mut stats, &mut run, &mut rng);

        assert!(!run.is_journey_task_accepted());
        assert!(
            dialogue
                .lines
                .iter()
                .any(|line| line.contains("【暂未签收】主线签 卷1-01"))
        );
    }

    #[test]
    fn route_task_turn_in_dialogue_marks_deferred_stage_advance() {
        let mut dialogue = RunDialogue::default();

        dialogue.open_route_task_turn_in("任务归档", vec!["【交付任务】主线签 卷1-01".to_string()]);

        assert!(dialogue.active);
        assert!(dialogue.resolved);
        assert!(dialogue.options.is_empty());
        assert!(dialogue.advance_stage_after);
    }

    #[test]
    fn route_commission_choice_tracks_local_errand_target() {
        let mut rng = Rng::default();
        let mut run = RunState::new(&mut rng);
        let mut stats = PlayerStats::default();
        let mut dialogue = RunDialogue::default();

        dialogue.open_route_commission(
            "路人委托",
            vec!["【路人签】路人签 卷1-01".to_string()],
            NodeKind::Event,
            None,
        );
        resolve_choice(&mut dialogue, 0, &mut stats, &mut run, &mut rng);

        assert!(run.is_route_commission_active());
        assert_eq!(
            run.route_commission_hud_label(),
            Some("路人签 卷1-01 -> 奇遇".to_string())
        );
        assert!(
            dialogue
                .lines
                .iter()
                .any(|line| line.contains("【路人委托已接取】路人签 卷1-01"))
        );
    }

    #[test]
    fn story_choice_records_current_chapter_vow() {
        let mut rng = Rng::default();
        let mut run = RunState::new(&mut rng);
        let mut stats = PlayerStats::default();
        let mut dialogue = RunDialogue::default();

        dialogue.open_story(0, 0);
        resolve_choice(&mut dialogue, 0, &mut stats, &mut run, &mut rng);

        assert_eq!(run.current_chapter_vow(), Some(RunChapterVow::Heart));
        assert_eq!(run.chapter_vow_summary(), "护心誓");
        assert!(
            dialogue
                .lines
                .iter()
                .any(|line| line.contains("【本卷誓记】护心誓"))
        );
    }

    #[test]
    fn rest_choice_records_camp_scene_and_next_battle_tactic() {
        let mut rng = Rng::default();
        let mut run = RunState::new(&mut rng);
        run.stage = 3;
        let mut stats = PlayerStats {
            hp: 40,
            ..default()
        };
        let mut dialogue = RunDialogue::default();

        dialogue.open_rest(&run);
        assert!(dialogue.title.contains("营地"));
        assert!(
            dialogue
                .options
                .iter()
                .any(|option| option.contains("灵儿护念"))
        );

        resolve_choice(&mut dialogue, 2, &mut stats, &mut run, &mut rng);

        assert_eq!(run.active_camp_tactic, Some(RunCampTactic::LingerWard));
        assert_eq!(run.qingyuan, 1);
        assert_eq!(run.camp_scenes_seen.len(), 1);
        assert!(stats.hp > 40);
        assert!(
            dialogue
                .lines
                .iter()
                .any(|line| line.contains("【营策已备】灵护"))
        );
    }

    #[test]
    fn late_rest_unlocks_spirit_focus_tactic() {
        let mut rng = Rng::default();
        let mut run = RunState::new(&mut rng);
        run.chapter = 5;
        let mut stats = PlayerStats { mp: 1, ..default() };
        let mut dialogue = RunDialogue::default();

        dialogue.open_rest(&run);
        assert!(
            dialogue
                .options
                .iter()
                .any(|option| option.contains("南瑶凝灵"))
        );

        resolve_choice(&mut dialogue, 3, &mut stats, &mut run, &mut rng);

        assert_eq!(run.active_camp_tactic, Some(RunCampTactic::SpiritFocus));
        assert_eq!(run.daoxin, 1);
        assert!(stats.mp > 1);
    }
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

#[derive(Component)]
pub struct RunDialoguePortrait;

/// Nine-slice profile for the gold-trimmed ink panel (`ui/panel_frame.png`,
/// 768×512): corners stay crisp, the near-black centre stretches.
pub fn panel_slicer() -> TextureSlicer {
    TextureSlicer {
        border: BorderRect::axes(110.0, 58.0),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 0.6,
    }
}

/// Spawn the (hidden) overlay box — a portrait card on the left, text column
/// on the right. Called from the node-map scene setup so it carries the same
/// `DespawnOnExit` scope.
pub fn spawn_run_dialogue_ui(
    commands: &mut Commands,
    font: &GameFont,
    panel: Handle<Image>,
    scope: impl Bundle,
) {
    commands
        .spawn((
            RunDialogueRoot,
            scope,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(8.0),
                right: Val::Percent(8.0),
                bottom: Val::Px(28.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(16.0),
                align_items: AlignItems::FlexStart,
                padding: UiRect::new(Val::Px(32.0), Val::Px(32.0), Val::Px(46.0), Val::Px(20.0)),
                ..default()
            },
            ImageNode {
                image: panel,
                image_mode: NodeImageMode::Sliced(panel_slicer()),
                ..default()
            },
            Visibility::Hidden,
            GlobalZIndex(50),
        ))
        .with_children(|parent| {
            parent.spawn((
                RunDialoguePortrait,
                ImageNode::default(),
                Node {
                    width: Val::Px(132.0),
                    height: Val::Px(176.0),
                    flex_shrink: 0.0,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)),
                Visibility::Hidden,
            ));
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    flex_grow: 1.0,
                    ..default()
                })
                .with_children(|column| {
                    column.spawn((
                        RunDialogueTitle,
                        Text::new(""),
                        font.text_font(20.0),
                        TextColor(Color::srgb(0.95, 0.83, 0.52)),
                    ));
                    column.spawn((
                        RunDialogueLine,
                        Text::new(""),
                        font.text_font(22.0),
                        TextColor(Color::srgb(0.94, 0.94, 0.90)),
                    ));
                    column.spawn((
                        RunDialogueOptions,
                        Text::new(""),
                        font.text_font(20.0),
                        TextColor(Color::srgb(0.75, 0.88, 1.0)),
                    ));
                });
        });
}

pub fn update_run_dialogue_ui(
    dialogue: Res<RunDialogue>,
    asset_server: Res<AssetServer>,
    mut root: Query<&mut Visibility, (With<RunDialogueRoot>, Without<RunDialoguePortrait>)>,
    mut portrait: Query<
        (&mut ImageNode, &mut Visibility),
        (With<RunDialoguePortrait>, Without<RunDialogueRoot>),
    >,
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

    if let Ok((mut image, mut portrait_visibility)) = portrait.single_mut() {
        match dialogue.portrait {
            Some(path) => {
                if dialogue.is_changed() {
                    image.image = asset_server.load(path);
                }
                *portrait_visibility = Visibility::Inherited;
            }
            None => {
                *portrait_visibility = Visibility::Hidden;
            }
        }
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

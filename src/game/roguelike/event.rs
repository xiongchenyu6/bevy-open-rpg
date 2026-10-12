//! Run-mode dialogue/event overlay.
//!
//! One overlay serves chapter cards, story scenes (「缘」), random events
//! (「遇」) and rests (「歇」). It lives inside the `NodeMap` state as a
//! resource-driven UI box, so the map stays visible behind the text.

use bevy::prelude::*;

use super::super::core::{Intent, PlayerStats, Rng};
use super::super::quest::Companion;
use super::content::{self, Effect, Outcome};
use super::graph::NodeKind;
use super::narrative::{self, NarrativeKind, NarrativeRecord};
use super::{RunCampTactic, RunChapterVow, RunState};
use crate::game::explore::MapKind;

// ---------------------------------------------------------------------------
// Overlay state
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum DialogueSource {
    #[default]
    Plain,
    /// Index into [`content::EVENTS`].
    Event(usize),
    /// Story scene in the current chapter’s authored sequence.
    Story {
        chapter: usize,
        roll: usize,
    },
    Rest,
    Market,
    /// A route mechanism with an authored decision.
    Puzzle {
        index: usize,
    },
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
    /// The local errand completes after this overlay is actually resolved.
    pub pending_commission: Option<(MapKind, NodeKind)>,
    pub choice_error: Option<String>,
    pub puzzle_answer: Option<usize>,
    pub puzzle_hint: Option<&'static str>,
    pub puzzle_solved_line: Option<&'static str>,
    pub puzzle_cleared_after: Option<usize>,
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
            title: format!("剧情 · {}", scene.title),
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
        } else if run.party_companions().contains(&Companion::Linger) {
            Some(content::PORTRAIT_LINGER)
        } else {
            None
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

    pub fn choice_preview(&self, run: Option<&RunState>) -> String {
        if let Some(error) = &self.choice_error {
            return error.clone();
        }
        if let DialogueSource::Puzzle { index } = self.source {
            return super::puzzle::choice_preview(index, self.selected);
        }
        let kind = match self.source {
            DialogueSource::Event(index) => Some(NarrativeKind::Event(index)),
            DialogueSource::Story { chapter, roll } => Some(NarrativeKind::Story(chapter, roll)),
            _ => None,
        };
        if kind.is_some_and(|k| run.is_some_and(|r| r.has_narrative_receipt(k))) {
            return "此事已结：可回顾，无消耗，不会重复发放回礼。".into();
        }
        match self.source {
            DialogueSource::Event(index) => {
                let option = &content::EVENTS[index].options[self.selected];
                let mut outcome = option.success;
                if index == content::EV_QIN2 && self.selected == 0 {
                    outcome.effect = if run.is_some_and(|r| r.qin_kind > 0) {
                        Effect::Bundle(&[
                            Effect::LearnSkill(super::skill::SkillId::SoulLantern),
                            Effect::GainMaxMp(6),
                        ])
                    } else {
                        Effect::Bundle(&[
                            Effect::LearnSkill(super::skill::SkillId::MoonHalo),
                            Effect::RestoreMpPct(25),
                        ])
                    };
                }
                let mut text = narrative::choice_preview(
                    option.cost,
                    &outcome,
                    option.chance,
                    option.failure.as_ref(),
                );
                if index == content::EV_FOX2_COLD && self.selected == 0 {
                    text.push_str("\n后续：修复与阿狸的关系，终遇转为善缘。");
                }
                if index == content::EV_QIN1 {
                    text.push_str(if self.selected == 0 {
                        "\n后续：琴师记下你的资助，重逢时传授引魂之术。"
                    } else {
                        "\n后续：琴师记下你的倾听，重逢时传授护身之术。"
                    });
                }
                text
            }
            DialogueSource::Story { chapter, roll } => {
                let option = &content::pick_story(chapter, roll).options[self.selected];
                narrative::choice_preview(option.cost, &option.outcome, 1.0, None)
            }
            _ => String::new(),
        }
    }

    /// The overlay is waiting on an option pick (all lines shown, unresolved).
    pub(super) fn in_choice(&self) -> bool {
        !self.resolved && self.idx + 1 >= self.lines.len() && !self.options.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Effects
// ---------------------------------------------------------------------------

/// Apply an outcome's mechanical effect. Returns an extra feedback line for
/// effects whose result is rolled at runtime (e.g. which relic dropped).
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
            dialogue.choice_error = None;
        }
        if intent.down && dialogue.selected + 1 < dialogue.options.len() {
            dialogue.selected += 1;
            dialogue.choice_error = None;
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

pub(crate) fn resolve_choice(
    dialogue: &mut RunDialogue,
    pick: usize,
    stats: &mut PlayerStats,
    run: &mut RunState,
    rng: &mut Rng,
) {
    if matches!(dialogue.source, DialogueSource::Puzzle { .. }) {
        super::puzzle::resolve_choice(dialogue, pick, stats, run, rng);
        return;
    }
    let kind_and_cost = match dialogue.source {
        DialogueSource::Event(index) => Some((
            NarrativeKind::Event(index),
            content::EVENTS[index].options[pick].cost,
        )),
        DialogueSource::Story { chapter, roll } => Some((
            NarrativeKind::Story(chapter, roll),
            content::pick_story(chapter, roll).options[pick].cost,
        )),
        _ => None,
    };
    let mut paid_receipt = String::new();
    if let Some((kind, cost)) = kind_and_cost {
        if run.has_narrative_receipt(kind) {
            dialogue
                .lines
                .push("【缘事回顾】此事已结，不再扣除代价或发放回礼。".into());
            dialogue.resolved = true;
            dialogue.options.clear();
            dialogue.idx += 1;
            super::scene::settle_route_commission(dialogue, stats, run);
            return;
        }
        match cost.pay(stats) {
            Ok(receipt) => paid_receipt = receipt,
            Err(reason) => {
                dialogue.choice_error = Some(reason);
                return;
            }
        }
    }
    dialogue.choice_error = None;
    let mut event_succeeded = true;
    let outcome: Outcome = match dialogue.source {
        DialogueSource::Event(index) => {
            let option = &content::EVENTS[index].options[pick];
            event_succeeded = rng.chance(option.chance);
            if index == content::EV_QIN2 && pick == 0 {
                let mut gift = option.success;
                gift.effect = if run.qin_kind > 0 {
                    Effect::Bundle(&[
                        Effect::LearnSkill(super::skill::SkillId::SoulLantern),
                        Effect::GainMaxMp(6),
                    ])
                } else {
                    Effect::Bundle(&[
                        Effect::LearnSkill(super::skill::SkillId::MoonHalo),
                        Effect::RestoreMpPct(25),
                    ])
                };
                gift
            } else if event_succeeded {
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
            let before = (stats.hp, stats.mp, stats.atk, run.daoxin, run.qingyuan);
            let mut result_line = match tactic {
                RunCampTactic::Breath => {
                    let amount = stats.max_hp * 50 * mult / 100;
                    stats.hp = (stats.hp + amount).min(stats.max_hp);
                    let mana = (stats.max_mp * 30 / 100).max(1);
                    stats.mp = (stats.mp + mana).min(stats.max_mp);
                    "一夜吐纳，你轮流替同伴守夜，天亮时又能继续赶路。".into()
                }
                RunCampTactic::SwordGuard => {
                    if first_visit {
                        stats.atk += 1;
                    }
                    let amount = stats.max_hp * 10 * mult / 100;
                    stats.hp = (stats.hp + amount).min(stats.max_hp);
                    format!(
                        "以火光为敌手拆招至深夜，剑守已备。(攻击 +{})",
                        if first_visit { 1 } else { 0 }
                    )
                }
                RunCampTactic::LingerWard => {
                    if first_visit {
                        run.qingyuan += 1;
                    }
                    let amount = stats.max_hp * 30 * mult / 100;
                    stats.hp = (stats.hp + amount).min(stats.max_hp);
                    "灵儿讲起小时候偷摘桃子被追着跑的糗事,火堆边的夜忽然就不冷了。".to_string()
                }
                RunCampTactic::SpiritFocus => {
                    if first_visit {
                        run.daoxin += 1;
                    }
                    let mana = (stats.max_mp * 60 / 100).max(1);
                    stats.mp = (stats.mp + mana).min(stats.max_mp);
                    "南瑶以铜铃压住杂乱灵纹,队伍心口都清明了一线。".to_string()
                }
            };
            result_line.push_str(&format!(
                "\n【实得照应】气血 {:+} · 灵力 {:+} · 攻击 {:+} · 道心 {:+} · 情缘 {:+}",
                stats.hp - before.0,
                stats.mp - before.1,
                stats.atk - before.2,
                run.daoxin - before.3,
                run.qingyuan - before.4
            ));
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
            super::scene::settle_route_commission(dialogue, stats, run);
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
                    super::scene::settle_route_commission(dialogue, stats, run);
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
        DialogueSource::Plain | DialogueSource::Puzzle { .. } => return,
    };

    // Narrative prose and mechanical receipts are separate; receipts come
    // from actual mutations, never from manually maintained text promises.
    for line in outcome.lines {
        if !line.is_empty() {
            dialogue.lines.push(line.to_string());
        }
    }
    if matches!(dialogue.source, DialogueSource::Event(content::EV_QIN2)) && pick == 0 {
        dialogue.lines.push(
            if run.qin_kind > 0 {
                "苏弦递来写着指法的曲谱：你资助的盘缠让她带难民走到这里，如今她将引魂之术教给你。"
            } else {
                "苏弦将护身的指法记在纸上：你那晚安静听完的曲子，正是凝成月轮的灵诀。"
            }
            .into(),
        );
    }
    if !paid_receipt.is_empty() {
        dialogue.lines.push(paid_receipt.clone());
    }
    let reward_receipt = narrative::apply_outcome(&outcome, stats, run, rng).unwrap_or_default();
    dialogue.lines.push(reward_receipt.clone());
    if let Some((kind, _)) = kind_and_cost {
        let title = match kind {
            NarrativeKind::Event(index) => content::EVENTS[index].title,
            NarrativeKind::Story(chapter, roll) => content::pick_story(chapter, roll).title,
        };
        run.narrative_log.push(NarrativeRecord {
            kind,
            chapter: run.chapter,
            title: title.into(),
            choice: dialogue.options[pick].clone(),
            receipt: [paid_receipt, reward_receipt]
                .into_iter()
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("\n"),
        });
        if let NarrativeKind::Event(index) = kind {
            run.resolve_chain(index, pick, event_succeeded);
            if index == content::EV_FOX2_COLD && run.fox_kind == 2 {
                dialogue
                    .lines
                    .push("【缘事续记】阿狸接受了补救；下一次相逢会以善缘相待。".into());
            }
        }
    }
    super::scene::settle_route_commission(dialogue, stats, run);
    dialogue.resolved = true;
    dialogue.options.clear();
    dialogue.idx += 1;
}

#[cfg(test)]
#[path = "narrative_tests.rs"]
mod narrative_tests;

#[cfg(test)]
mod tests {
    use super::super::super::core::{PlayerStats, Rng};
    use super::*;

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

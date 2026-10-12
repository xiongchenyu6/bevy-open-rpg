//! Authored choice contracts, reward previews, and this life's story receipts.
use super::ALL_RELICS;
use super::content::{Effect, Outcome};
use super::{RunState, skill};
use crate::game::core::{PlayerStats, Rng};
use crate::game::explore::MapKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cost {
    Free,
    Gold(u32),
    Potion(u32),
    HealthPct(i32),
}

impl Cost {
    pub fn label(self) -> String {
        match self {
            Self::Free => "无消耗".into(),
            Self::Gold(n) => format!("钱财 -{n} 文"),
            Self::Potion(n) => format!("药水 -{n}"),
            Self::HealthPct(n) => format!("气血消耗上限的 {n}%"),
        }
    }

    /// Validate before mutating anything. Rejected choices consume no RNG,
    /// relationship progress, or reward, and leave the choice open.
    pub fn pay(self, stats: &mut PlayerStats) -> Result<String, String> {
        let paid = match self {
            Self::Free => return Ok(String::new()),
            Self::Gold(n) => {
                if stats.gold < n {
                    return Err(format!("还缺 {} 文，选择尚未结算。", n - stats.gold));
                }
                stats.gold -= n;
                format!("钱财 -{n} 文")
            }
            Self::Potion(n) => {
                if stats.potions < n {
                    return Err(format!("还缺 {} 瓶药水，选择尚未结算。", n - stats.potions));
                }
                stats.potions -= n;
                format!("药水 -{n}")
            }
            Self::HealthPct(pct) => {
                let n = (stats.max_hp * pct / 100).max(1);
                if stats.hp <= n {
                    return Err(format!("需至少 {} 点气血，不能以濒死之身尝试。", n + 1));
                }
                stats.hp -= n;
                format!("气血 -{n}")
            }
        };
        Ok(format!("【已付代价】{paid}"))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NarrativeKind {
    Event(usize),
    Story(usize, usize),
}

#[derive(Clone, Debug)]
pub struct NarrativeRecord {
    pub kind: NarrativeKind,
    pub chapter: usize,
    pub title: String,
    pub choice: String,
    pub receipt: String,
}

pub fn preview(effect: Effect) -> String {
    match effect {
        Effect::None => "无物资变化".into(),
        Effect::HealPct(n) => format!("恢复 {n}% 气血"),
        Effect::RestoreMpPct(n) => format!("恢复 {n}% 灵力"),
        Effect::DamagePct(n) => format!("损失 {n}% 气血，保留至少一点"),
        Effect::GainPotions(n) => format!("药水 +{n}"),
        Effect::GainGold(n) => format!("钱财 +{n} 文"),
        Effect::LoseGold(n) => format!("钱财损失最多 {n} 文"),
        Effect::GainAtk(n) => format!("攻击 +{n}"),
        Effect::GainDef(n) => format!("防御 +{n}"),
        Effect::GainMaxHp(n) => format!("气血上限 +{n}"),
        Effect::GainMaxMp(n) => format!("灵力上限 +{n}"),
        Effect::RandomRelic => "随机未持有法宝".into(),
        Effect::Relic(relic) => format!(
            "法宝【{}】：{}（已有则获 40 文）",
            relic.name(),
            relic.desc()
        ),
        Effect::LearnSkill(id) => {
            let def = skill::skill(id);
            format!(
                "习得【{}】（基础灵力 {}；已会则灵力上限 +2 并回灵 25%）\n用途：{}",
                def.name,
                def.cost,
                def.desc()
            )
        }
        Effect::Bundle(effects) => effects
            .iter()
            .map(|e| preview(*e))
            .collect::<Vec<_>>()
            .join("；"),
    }
}

pub fn choice_preview(
    cost: Cost,
    outcome: &Outcome,
    chance: f32,
    failure: Option<&Outcome>,
) -> String {
    let mut text = format!("代价：{}\n收获：{}", cost.label(), preview(outcome.effect));
    if outcome.daoxin != 0 {
        text.push_str(&format!("；道心 {:+}", outcome.daoxin));
    }
    if outcome.qingyuan != 0 {
        text.push_str(&format!("；情缘 {:+}", outcome.qingyuan));
    }
    if let Some(failure) = failure {
        text.push_str(&format!(
            "\n成功率 {:.0}% · 失手：{}（代价仍会扣除）",
            chance * 100.0,
            preview(failure.effect)
        ));
    }

    text
}

/// Geography is authored. A city notice or river crossing cannot randomly
/// materialize deep inside a cave, and the final waterway has its own pool.
pub fn event_eligible(index: usize, map: MapKind) -> bool {
    let allowed: &[usize] = match map {
        MapKind::Village | MapKind::Bamboo => &[0, 1, 2, 6, 7, 11, 12, 13, 15, 16, 17, 18, 21],
        MapKind::Cave | MapKind::MoonEchoCorridor => &[1, 3, 5, 6, 7, 8, 11, 15, 17, 18, 19],
        MapKind::RiverTown | MapKind::RiverReedBed => &[0, 2, 4, 5, 6, 9, 10, 12, 13, 14, 19, 21],
        MapKind::PlagueVillage | MapKind::PlagueShrinePath => {
            &[2, 7, 8, 9, 11, 12, 13, 14, 16, 17, 18, 19, 21]
        }
        MapKind::Capital | MapKind::CapitalMansion | MapKind::MansionMirrorGallery => {
            &[0, 2, 3, 4, 6, 9, 12, 13, 14, 19, 20]
        }
        MapKind::SouthernRoad | MapKind::ThunderDrumPath => {
            &[0, 1, 5, 7, 8, 9, 11, 15, 16, 17, 18, 21]
        }
        MapKind::FinalSanctum | MapKind::DreamWaterway => &[1, 3, 5, 8, 15, 18, 19],
    };
    allowed.contains(&index)
}

impl RunState {
    pub fn has_narrative_receipt(&self, kind: NarrativeKind) -> bool {
        self.narrative_log.iter().any(|r| r.kind == kind)
    }

    pub fn narrative_summary(&self) -> String {
        if self.narrative_log.is_empty() {
            return "缘事簿：尚无回执".into();
        }
        let mut lines = vec!["缘事簿 · 此世选择与实得回礼".into()];
        for record in &self.narrative_log {
            lines.push(format!(
                "卷{} · {}\n选择：{}\n{}",
                record.chapter + 1,
                record.title,
                record.choice,
                record.receipt
            ));
        }
        lines.join("\n\n")
    }

    /// Chain completion means a paid and resolved choice, not merely seeing
    /// an event. The repaired fox relationship changes its final encounter.
    pub fn resolve_chain(&mut self, index: usize, pick: usize, success: bool) {
        use super::content::*;
        match index {
            EV_FOX1 => {
                self.fox_stage = 1;
                self.fox_kind = if pick == 0 { 1 } else { -1 };
            }
            EV_FOX2_WARM => self.fox_stage = 2,
            EV_FOX2_COLD => {
                self.fox_stage = 2;
                if pick == 0 && success {
                    self.fox_kind = 2;
                }
            }
            EV_FOX3_WARM | EV_FOX3_COLD => self.fox_stage = 3,
            EV_QIN1 => {
                self.qin_stage = 1;
                self.qin_kind = if pick == 0 { 1 } else { -1 };
            }
            EV_QIN2 => self.qin_stage = 2,
            _ => {}
        }
    }

    pub fn side_story_echoes(&self, victory: bool) -> Vec<&'static str> {
        let mut echoes = Vec::new();
        match (self.fox_stage, self.fox_kind) {
            (3, 2) => echoes.push("阿狸收下了你后来送去的伤药。初次错过的善意，终于有了补救。"),
            (3, k) if k > 0 => echoes.push("山径上的一瓶伤药，换来了阿狸在险路上的护持。"),
            (3, _) => echoes.push("与阿狸的旧怨在路上了结，你们仍没有成为朋友。"),
            (s, k) if s > 0 && k > 0 => echoes.push("你救过的白狐仍在路上，这段缘尚未走完。"),
            (s, _) if s > 0 => echoes.push("山径上那只受伤的白狐，始终没有等到你的回头。"),
            _ => {}
        }
        if self.qin_stage >= 2 {
            echoes.push(if victory {
                "守门棚里的《雾散》终于不再只是祝愿，琴师带着新曲继续赶路。"
            } else {
                "琴师的《雾散》仍留在记忆里；这一世尚未让它应验。"
            });
        } else if self.qin_stage == 1 {
            echoes.push("你还记得渡口那支曲子，但未在此世听到它的下半阕。");
        }
        echoes
    }
}

pub fn apply_outcome(
    outcome: &Outcome,
    stats: &mut PlayerStats,
    run: &mut RunState,
    rng: &mut Rng,
) -> Option<String> {
    run.daoxin += outcome.daoxin;
    run.qingyuan += outcome.qingyuan;
    let mut receipt = apply_effect(outcome.effect, stats, run, rng).unwrap_or_default();
    if outcome.daoxin != 0 {
        receipt.push_str(&format!("；道心 {:+}", outcome.daoxin));
    }
    if outcome.qingyuan != 0 {
        receipt.push_str(&format!("；情缘 {:+}", outcome.qingyuan));
    }
    Some(format!("【实得回礼】{}", receipt.trim_start_matches('；')))
}

pub fn apply_effect(
    effect: Effect,
    stats: &mut PlayerStats,
    run: &mut RunState,
    rng: &mut Rng,
) -> Option<String> {
    let receipt = match effect {
        Effect::None => "无物资变化".to_string(),
        Effect::HealPct(pct) => {
            let before = stats.hp;
            stats.hp = (stats.hp + (stats.max_hp * pct / 100).max(1)).min(stats.max_hp);
            format!("气血 +{}", stats.hp - before)
        }
        Effect::RestoreMpPct(pct) => {
            let before = stats.mp;
            stats.mp = (stats.mp + (stats.max_mp * pct / 100).max(1)).min(stats.max_mp);
            format!("灵力 +{}", stats.mp - before)
        }
        Effect::DamagePct(pct) => {
            let before = stats.hp;
            stats.hp = (stats.hp - (stats.max_hp * pct / 100).max(1)).max(1);
            format!("气血 -{}", before - stats.hp)
        }
        Effect::GainPotions(n) => {
            stats.potions += n;
            format!("药水 +{n}")
        }
        Effect::GainGold(n) => {
            stats.gold += n;
            format!("钱财 +{n} 文")
        }
        Effect::LoseGold(n) => {
            let before = stats.gold;
            stats.gold = stats.gold.saturating_sub(n);
            format!("钱财 -{} 文", before - stats.gold)
        }
        Effect::GainAtk(n) => {
            stats.atk += n;
            format!("攻击 +{n}")
        }
        Effect::GainDef(n) => {
            stats.def += n;
            format!("防御 +{n}")
        }
        Effect::GainMaxHp(n) => {
            stats.max_hp += n;
            stats.hp += n;
            format!("气血上限 +{n}，气血 +{n}")
        }
        Effect::GainMaxMp(n) => {
            stats.max_mp += n;
            stats.mp += n;
            format!("灵力上限 +{n}，灵力 +{n}")
        }
        Effect::RandomRelic => grant_random_relic(stats, run, rng),
        Effect::Relic(relic) => grant_relic(relic, stats, run),
        Effect::LearnSkill(id) => {
            if run.skills.contains(&id) {
                let before = stats.mp;
                stats.max_mp += 2;
                stats.mp = (stats.mp + 2 + (stats.max_mp / 4).max(1)).min(stats.max_mp);
                format!(
                    "已会【{}】，研习使灵力上限 +2，灵力 +{}",
                    skill::skill(id).name,
                    stats.mp - before
                )
            } else {
                run.skills.push(id);
                format!("习得【{}】（战斗仙术中可用）", skill::skill(id).name)
            }
        }
        Effect::Bundle(effects) => effects
            .iter()
            .filter_map(|effect| apply_effect(*effect, stats, run, rng))
            .collect::<Vec<_>>()
            .join("；"),
    };
    Some(receipt)
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
    if run.has_relic(relic) {
        stats.gold += 40;
        return format!("已持有【{}】，回礼改为钱财 +40 文。", relic.name());
    }
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

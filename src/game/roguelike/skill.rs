//! 百技谱 —— 十系元素 × 十种形态 = 100 种仙术技能。
//!
//! 技能是数据驱动生成的:每系元素带一种主色与词根,每种形态带一套
//! 数值基底、结算逻辑与特效形状;两两组合出 100 种名称、描述、数值、
//! 品级与视觉都不同的技能。玩家开局习得两式,之后从战利三选一、
//! 章末大胜中继续拓谱——一局能凑出什么技能组,也是 build 随机性的
//! 一部分。

use super::super::core::Rng;
use super::Grade;
use bevy::prelude::Color;

/// 十系元素:决定技能配色、词根与微调倾向。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Element {
    Fire,    // 炎
    Ice,     // 冰
    Thunder, // 雷
    Wind,    // 风
    Earth,   // 岩
    Water,   // 水
    Wood,    // 木
    Light,   // 光
    Dark,    // 冥
    Sword,   // 剑
}

pub const ALL_ELEMENTS: [Element; 10] = [
    Element::Fire,
    Element::Ice,
    Element::Thunder,
    Element::Wind,
    Element::Earth,
    Element::Water,
    Element::Wood,
    Element::Light,
    Element::Dark,
    Element::Sword,
];

impl Element {
    pub fn word(self) -> &'static str {
        match self {
            Element::Fire => "烈焰",
            Element::Ice => "玄冰",
            Element::Thunder => "奔雷",
            Element::Wind => "灵风",
            Element::Earth => "崩岳",
            Element::Water => "沧浪",
            Element::Wood => "青藤",
            Element::Light => "皓月",
            Element::Dark => "幽冥",
            Element::Sword => "御剑",
        }
    }

    /// 特效主色。
    pub fn color(self) -> Color {
        match self {
            Element::Fire => Color::srgb(1.0, 0.45, 0.15),
            Element::Ice => Color::srgb(0.55, 0.85, 1.0),
            Element::Thunder => Color::srgb(0.55, 0.55, 1.0),
            Element::Wind => Color::srgb(0.60, 1.0, 0.75),
            Element::Earth => Color::srgb(0.85, 0.65, 0.35),
            Element::Water => Color::srgb(0.30, 0.60, 1.0),
            Element::Wood => Color::srgb(0.45, 0.90, 0.40),
            Element::Light => Color::srgb(1.0, 0.95, 0.65),
            Element::Dark => Color::srgb(0.70, 0.35, 0.95),
            Element::Sword => Color::srgb(0.90, 0.95, 1.0),
        }
    }

    /// 元素微调:每系在同形态上各有偏性(伤害/费用小幅增减)。
    fn power_bias(self) -> i32 {
        match self {
            Element::Fire | Element::Thunder => 2,
            Element::Dark | Element::Sword => 1,
            Element::Ice | Element::Earth | Element::Water => 0,
            Element::Wind | Element::Wood | Element::Light => -1,
        }
    }

    fn cost_bias(self) -> i32 {
        match self {
            Element::Fire | Element::Thunder | Element::Dark => 1,
            Element::Wind | Element::Wood | Element::Light => -1,
            _ => 0,
        }
    }
}

/// 十种形态:决定结算逻辑、数值基底与特效形状。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkillKind {
    /// 斩(单体重击)。
    Slash,
    /// 爆(高伤高费)。
    Burst,
    /// 蚀(挂 3 回合流失)。
    Corrode,
    /// 缚(破敌防御)。
    Bind,
    /// 摄(伤害并吸血)。
    Drain,
    /// 连(2~3 段乱舞)。
    Flurry,
    /// 护(护盾抵伤)。
    Ward,
    /// 愈(疗伤)。
    Mend,
    /// 震(小伤+概率眩晕)。
    Stun,
    /// 诛(残血处决,低血翻倍)。
    Execute,
}

pub const ALL_KINDS: [SkillKind; 10] = [
    SkillKind::Slash,
    SkillKind::Burst,
    SkillKind::Corrode,
    SkillKind::Bind,
    SkillKind::Drain,
    SkillKind::Flurry,
    SkillKind::Ward,
    SkillKind::Mend,
    SkillKind::Stun,
    SkillKind::Execute,
];

impl SkillKind {
    pub fn word(self) -> &'static str {
        match self {
            SkillKind::Slash => "斩",
            SkillKind::Burst => "爆",
            SkillKind::Corrode => "蚀",
            SkillKind::Bind => "缚",
            SkillKind::Drain => "摄",
            SkillKind::Flurry => "连",
            SkillKind::Ward => "护",
            SkillKind::Mend => "愈",
            SkillKind::Stun => "震",
            SkillKind::Execute => "诛",
        }
    }

    /// (基础威力, 基础费用, 品级)。
    fn base(self) -> (i32, i32, Grade) {
        match self {
            SkillKind::Slash => (14, 4, Grade::Fine),
            SkillKind::Burst => (24, 8, Grade::Epic),
            SkillKind::Corrode => (6, 5, Grade::Superior),
            SkillKind::Bind => (8, 5, Grade::Fine),
            SkillKind::Drain => (11, 6, Grade::Superior),
            SkillKind::Flurry => (7, 6, Grade::Superior),
            SkillKind::Ward => (14, 5, Grade::Fine),
            SkillKind::Mend => (20, 6, Grade::Common),
            SkillKind::Stun => (9, 6, Grade::Epic),
            SkillKind::Execute => (12, 7, Grade::Celestial),
        }
    }
}

/// 一式技能 = 元素 × 形态。`SkillId` 即 `element_idx * 10 + kind_idx`。
pub type SkillId = usize;

#[derive(Clone, Copy, Debug)]
pub struct SkillDef {
    pub element: Element,
    pub kind: SkillKind,
    pub power: i32,
    pub cost: i32,
    pub grade: Grade,
}

pub const SKILL_COUNT: usize = 100;

pub fn skill(id: SkillId) -> SkillDef {
    let element = ALL_ELEMENTS[(id / 10) % 10];
    let kind = ALL_KINDS[id % 10];
    let (power, cost, grade) = kind.base();
    SkillDef {
        element,
        kind,
        power: (power + element.power_bias()).max(1),
        cost: (cost + element.cost_bias()).max(2),
        grade,
    }
}

pub fn skill_id(element: Element, kind: SkillKind) -> SkillId {
    let e = ALL_ELEMENTS.iter().position(|x| *x == element).unwrap_or(0);
    let k = ALL_KINDS.iter().position(|x| *x == kind).unwrap_or(0);
    e * 10 + k
}

impl SkillDef {
    /// 技名:元素词 + 形态字(如「烈焰·斩」「幽冥·诛」)。
    pub fn name(&self) -> String {
        format!("{}·{}", self.element.word(), self.kind.word())
    }

    /// 一行效果说明(战斗子菜单与行囊用)。
    pub fn desc(&self) -> String {
        let p = self.power;
        match self.kind {
            SkillKind::Slash => format!("单体 {p} 点元素伤害"),
            SkillKind::Burst => format!("高爆 {p} 点元素伤害"),
            SkillKind::Corrode => format!("{p} 点伤害并侵蚀 3 回合(每回合再失 {p})"),
            SkillKind::Bind => format!("{p} 点伤害并削去敌防 3 点"),
            SkillKind::Drain => format!("{p} 点伤害并吸取等量五成气血"),
            SkillKind::Flurry => format!("连击 2~3 段,每段 {p} 点"),
            SkillKind::Ward => format!("结 {p} 点护罩,先于气血抵伤"),
            SkillKind::Mend => format!("回复 {p} 点气血"),
            SkillKind::Stun => format!("{p} 点伤害,五成概率震慑敌人一回合"),
            SkillKind::Execute => format!("{p} 点伤害,敌血三成以下翻倍"),
        }
    }
}

/// 从未习得的技里按品级权重抽一式。
pub fn roll_skill(known: &[SkillId], rng: &mut Rng) -> Option<SkillId> {
    let pool: Vec<SkillId> = (0..SKILL_COUNT).filter(|id| !known.contains(id)).collect();
    if pool.is_empty() {
        return None;
    }
    let total: u32 = pool.iter().map(|id| skill(*id).grade.weight()).sum();
    let mut roll = rng.range(0, total as i32 - 1) as u32;
    for id in &pool {
        let w = skill(*id).grade.weight();
        if roll < w {
            return Some(*id);
        }
        roll -= w;
    }
    pool.last().copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 100 技名称唯一、数值合法、品级覆盖全部五档。
    #[test]
    fn hundred_skills_are_distinct_and_sane() {
        let mut names = std::collections::HashSet::new();
        let mut grades = std::collections::HashSet::new();
        for id in 0..SKILL_COUNT {
            let s = skill(id);
            assert!(names.insert(s.name()), "duplicate name {}", s.name());
            assert!(s.power >= 1 && s.cost >= 2);
            grades.insert(s.grade.name());
        }
        assert_eq!(names.len(), 100);
        assert_eq!(grades.len(), 5);
    }

    /// 抽技不重复,抽干后返回 None。
    #[test]
    fn roll_skill_is_unique_until_exhausted() {
        let mut rng = super::super::super::core::Rng::default();
        let mut known: Vec<SkillId> = Vec::new();
        for _ in 0..SKILL_COUNT {
            let id = roll_skill(&known, &mut rng).expect("pool not empty");
            assert!(!known.contains(&id));
            known.push(id);
        }
        assert!(roll_skill(&known, &mut rng).is_none());
    }
}

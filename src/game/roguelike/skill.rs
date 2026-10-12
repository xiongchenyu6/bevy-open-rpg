//! 二十式仙术谱。
//!
//! 旧版用十种颜色乘十种形态凑出一百个技能,名字和画面都高度重复。
//! 现在只保留二十个手工编排的技能:结算规则仍然数据驱动,但每一式
//! 都有独立名称、数值、状态与 [`VfxStyle`]。

use super::super::core::Rng;
use super::Grade;
use bevy::prelude::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Element {
    Fire,
    Ice,
    Thunder,
    Wind,
    Earth,
    Water,
    Wood,
    Light,
    Dark,
    Sword,
}

impl Element {
    pub fn color(self) -> Color {
        match self {
            Element::Fire => Color::srgb(1.0, 0.28, 0.08),
            Element::Ice => Color::srgb(0.40, 0.88, 1.0),
            Element::Thunder => Color::srgb(0.56, 0.48, 1.0),
            Element::Wind => Color::srgb(0.42, 1.0, 0.72),
            Element::Earth => Color::srgb(0.92, 0.62, 0.24),
            Element::Water => Color::srgb(0.16, 0.64, 1.0),
            Element::Wood => Color::srgb(0.36, 0.92, 0.38),
            Element::Light => Color::srgb(1.0, 0.90, 0.44),
            Element::Dark => Color::srgb(0.76, 0.24, 0.92),
            Element::Sword => Color::srgb(0.86, 0.95, 1.0),
        }
    }
}

/// 战斗结算形态。视觉表现由 [`VfxStyle`] 单独决定。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkillKind {
    Slash,
    Burst,
    Corrode,
    Bind,
    Drain,
    Flurry,
    Ward,
    Mend,
    Stun,
    Execute,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SkillStatus {
    Burn { damage: i32, turns: u8 },
    Poison { damage: i32, turns: u8 },
    Freeze { chance: f32 },
    Stun { chance: f32 },
}

/// 二十套独立的技能演出脚本。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VfxStyle {
    SwordGleam,
    CinderMeteor,
    FrostLance,
    StormChain,
    WindSpiral,
    EarthRift,
    TidalDragon,
    ThornCage,
    MoonHalo,
    SoulLantern,
    KarmaLotus,
    GlacierCrown,
    ThunderPrison,
    BladeStorm,
    MountainAegis,
    WhaleBreath,
    WorldTreeBloom,
    Starfall,
    Eclipse,
    SwordArray,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SkillId {
    SwordGleam,
    CinderMeteor,
    FrostLance,
    StormChain,
    WindSpiral,
    EarthRift,
    TidalDragon,
    ThornCage,
    MoonHalo,
    SoulLantern,
    KarmaLotus,
    GlacierCrown,
    ThunderPrison,
    BladeStorm,
    MountainAegis,
    WhaleBreath,
    WorldTreeBloom,
    Starfall,
    Eclipse,
    SwordArray,
}

pub const ALL_SKILLS: [SkillId; 20] = [
    SkillId::SwordGleam,
    SkillId::CinderMeteor,
    SkillId::FrostLance,
    SkillId::StormChain,
    SkillId::WindSpiral,
    SkillId::EarthRift,
    SkillId::TidalDragon,
    SkillId::ThornCage,
    SkillId::MoonHalo,
    SkillId::SoulLantern,
    SkillId::KarmaLotus,
    SkillId::GlacierCrown,
    SkillId::ThunderPrison,
    SkillId::BladeStorm,
    SkillId::MountainAegis,
    SkillId::WhaleBreath,
    SkillId::WorldTreeBloom,
    SkillId::Starfall,
    SkillId::Eclipse,
    SkillId::SwordArray,
];

pub const SKILL_COUNT: usize = ALL_SKILLS.len();

#[derive(Clone, Copy, Debug)]
pub struct SkillDef {
    pub name: &'static str,
    pub element: Element,
    pub kind: SkillKind,
    pub status: Option<SkillStatus>,
    pub vfx: VfxStyle,
    pub power: i32,
    pub cost: i32,
    pub grade: Grade,
}

#[allow(clippy::too_many_arguments)]
const fn def(
    name: &'static str,
    element: Element,
    kind: SkillKind,
    status: Option<SkillStatus>,
    vfx: VfxStyle,
    power: i32,
    cost: i32,
    grade: Grade,
) -> SkillDef {
    SkillDef {
        name,
        element,
        kind,
        status,
        vfx,
        power,
        cost,
        grade,
    }
}

pub fn skill(id: SkillId) -> SkillDef {
    match id {
        SkillId::SwordGleam => def(
            "御剑·流光",
            Element::Sword,
            SkillKind::Slash,
            None,
            VfxStyle::SwordGleam,
            15,
            4,
            Grade::Fine,
        ),
        SkillId::CinderMeteor => def(
            "烈焰·陨星",
            Element::Fire,
            SkillKind::Burst,
            Some(SkillStatus::Burn {
                damage: 5,
                turns: 3,
            }),
            VfxStyle::CinderMeteor,
            25,
            8,
            Grade::Epic,
        ),
        SkillId::FrostLance => def(
            "玄冰·霜枪",
            Element::Ice,
            SkillKind::Stun,
            Some(SkillStatus::Freeze { chance: 0.55 }),
            VfxStyle::FrostLance,
            10,
            6,
            Grade::Epic,
        ),
        SkillId::StormChain => def(
            "奔雷·天链",
            Element::Thunder,
            SkillKind::Flurry,
            None,
            VfxStyle::StormChain,
            8,
            7,
            Grade::Superior,
        ),
        SkillId::WindSpiral => def(
            "灵风·回岚",
            Element::Wind,
            SkillKind::Mend,
            None,
            VfxStyle::WindSpiral,
            18,
            5,
            Grade::Common,
        ),
        SkillId::EarthRift => def(
            "崩岳·地脉",
            Element::Earth,
            SkillKind::Bind,
            None,
            VfxStyle::EarthRift,
            10,
            5,
            Grade::Fine,
        ),
        SkillId::TidalDragon => def(
            "沧浪·龙吟",
            Element::Water,
            SkillKind::Execute,
            None,
            VfxStyle::TidalDragon,
            13,
            7,
            Grade::Epic,
        ),
        SkillId::ThornCage => def(
            "青藤·森罗缚",
            Element::Wood,
            SkillKind::Bind,
            Some(SkillStatus::Poison {
                damage: 3,
                turns: 3,
            }),
            VfxStyle::ThornCage,
            9,
            5,
            Grade::Superior,
        ),
        SkillId::MoonHalo => def(
            "皓月·清辉轮",
            Element::Light,
            SkillKind::Ward,
            None,
            VfxStyle::MoonHalo,
            16,
            5,
            Grade::Fine,
        ),
        SkillId::SoulLantern => def(
            "幽冥·噬魂灯",
            Element::Dark,
            SkillKind::Drain,
            None,
            VfxStyle::SoulLantern,
            12,
            6,
            Grade::Superior,
        ),
        SkillId::KarmaLotus => def(
            "烈焰·业火莲",
            Element::Fire,
            SkillKind::Corrode,
            Some(SkillStatus::Burn {
                damage: 4,
                turns: 4,
            }),
            VfxStyle::KarmaLotus,
            8,
            6,
            Grade::Superior,
        ),
        SkillId::GlacierCrown => def(
            "玄冰·冰棘林",
            Element::Ice,
            SkillKind::Burst,
            Some(SkillStatus::Freeze { chance: 0.35 }),
            VfxStyle::GlacierCrown,
            23,
            8,
            Grade::Epic,
        ),
        SkillId::ThunderPrison => def(
            "奔雷·九霄狱",
            Element::Thunder,
            SkillKind::Stun,
            Some(SkillStatus::Stun { chance: 0.60 }),
            VfxStyle::ThunderPrison,
            11,
            7,
            Grade::Epic,
        ),
        SkillId::BladeStorm => def(
            "灵风·千刃岚",
            Element::Wind,
            SkillKind::Flurry,
            None,
            VfxStyle::BladeStorm,
            8,
            6,
            Grade::Superior,
        ),
        SkillId::MountainAegis => def(
            "崩岳·玄甲山",
            Element::Earth,
            SkillKind::Ward,
            None,
            VfxStyle::MountainAegis,
            19,
            6,
            Grade::Fine,
        ),
        SkillId::WhaleBreath => def(
            "沧浪·鲸息潮",
            Element::Water,
            SkillKind::Drain,
            None,
            VfxStyle::WhaleBreath,
            14,
            7,
            Grade::Superior,
        ),
        SkillId::WorldTreeBloom => def(
            "青藤·扶桑生",
            Element::Wood,
            SkillKind::Mend,
            None,
            VfxStyle::WorldTreeBloom,
            24,
            6,
            Grade::Common,
        ),
        SkillId::Starfall => def(
            "皓月·星河坠",
            Element::Light,
            SkillKind::Burst,
            None,
            VfxStyle::Starfall,
            29,
            10,
            Grade::Celestial,
        ),
        SkillId::Eclipse => def(
            "幽冥·蚀月劫",
            Element::Dark,
            SkillKind::Corrode,
            Some(SkillStatus::Poison {
                damage: 6,
                turns: 3,
            }),
            VfxStyle::Eclipse,
            10,
            8,
            Grade::Epic,
        ),
        SkillId::SwordArray => def(
            "御剑·万剑归宗",
            Element::Sword,
            SkillKind::Execute,
            None,
            VfxStyle::SwordArray,
            16,
            9,
            Grade::Celestial,
        ),
    }
}

impl SkillDef {
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn desc(&self) -> String {
        let base = match self.kind {
            SkillKind::Slash => format!("单体 {} 点元素伤害", self.power),
            SkillKind::Burst => format!("高爆 {} 点元素伤害", self.power),
            SkillKind::Corrode => format!("{} 点伤害并留下持续侵蚀", self.power),
            SkillKind::Bind => format!("{} 点伤害并削去敌防 3 点", self.power),
            SkillKind::Drain => format!("{} 点伤害并吸取等量五成气血", self.power),
            SkillKind::Flurry => format!("连击 2~3 段,每段 {} 点", self.power),
            SkillKind::Ward => format!("结 {} 点护罩,先于气血抵伤", self.power),
            SkillKind::Mend => format!("回复 {} 点气血", self.power),
            SkillKind::Stun => format!("{} 点伤害并尝试控制敌人", self.power),
            SkillKind::Execute => format!("{} 点伤害,敌血三成以下翻倍", self.power),
        };
        match self.status {
            Some(SkillStatus::Burn { damage, turns }) => {
                format!("{base};灼烧 {damage}×{turns}回合")
            }
            Some(SkillStatus::Poison { damage, turns }) => {
                format!("{base};中毒 {damage}×{turns}回合,最多3层")
            }
            Some(SkillStatus::Freeze { chance }) => {
                format!("{base};{}%冰冻", (chance * 100.0).round() as i32)
            }
            Some(SkillStatus::Stun { chance }) => {
                format!("{base};{}%震慑", (chance * 100.0).round() as i32)
            }
            None => base,
        }
    }
}

/// 从未习得的技能里按品级权重抽取一式。
pub fn roll_skill(known: &[SkillId], rng: &mut Rng) -> Option<SkillId> {
    let pool: Vec<SkillId> = ALL_SKILLS
        .iter()
        .copied()
        .filter(|id| !known.contains(id))
        .collect();
    if pool.is_empty() {
        return None;
    }
    let total: u32 = pool.iter().map(|id| skill(*id).grade.weight()).sum();
    let mut roll = rng.range(0, total as i32 - 1) as u32;
    for id in &pool {
        let weight = skill(*id).grade.weight();
        if roll < weight {
            return Some(*id);
        }
        roll -= weight;
    }
    pool.last().copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn twenty_curated_skills_are_distinct_and_sane() {
        let mut names = std::collections::HashSet::new();
        let mut styles = std::collections::HashSet::new();
        let mut grades = std::collections::HashSet::new();
        for id in ALL_SKILLS {
            let definition = skill(id);
            assert!(names.insert(definition.name()), "duplicate skill name");
            assert!(styles.insert(definition.vfx), "duplicate VFX style");
            assert!(definition.power >= 1 && definition.cost >= 2);
            grades.insert(definition.grade.name());
        }
        assert_eq!(names.len(), SKILL_COUNT);
        assert_eq!(styles.len(), SKILL_COUNT);
        assert_eq!(grades.len(), 5);
    }

    #[test]
    fn roll_skill_is_unique_until_exhausted() {
        let mut rng = super::super::super::core::Rng::default();
        let mut known = Vec::new();
        for _ in 0..SKILL_COUNT {
            let id = roll_skill(&known, &mut rng).expect("pool not empty");
            assert!(!known.contains(&id));
            known.push(id);
        }
        assert!(roll_skill(&known, &mut rng).is_none());
    }
}

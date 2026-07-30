//! 妖纹(海克斯)—— 杀戮尖塔式的有代价随机词条。
//!
//! 战利三选一里会随机混入「妖纹」:一枚强力增益绑着一条明确的代价,
//! 可无限叠加。每一局抽到什么、敢不敢拿、怎么围绕它组 build,
//! 构成 run 与 run 之间最大的随机差异。
//! 效果全部通过 `RunState` 的聚合查询进入战斗/地图钩子。

use super::super::core::Rng;

/// 一枚妖纹:增益 + 代价。文案里「◆」是增益,「◇」是代价。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HexMark {
    /// 血怒:气血低于一半时攻击 +5;◇ 气血上限 -8。
    BloodRage,
    /// 贪泉:每胜一场额外 +18 文;◇ 每程入图失 3 气血。
    GreedSpring,
    /// 疾风:御守当回合也叠 1 层气势;◇ 御守回灵 -2。
    GaleStep,
    /// 妖契:绝技只需 2 层气势;◇ 绝技后自伤 4。
    DemonPact,
    /// 磐心:御守额外减免 8 点;◇ 攻击 -2。
    StoneHeart,
    /// 血偿:击杀回复 6 气血;◇ 开战先失 2 灵力。
    BloodTithe,
    /// 雾行:草丛遇敌率减半;◇ 战利钱财 -30%。
    MistWalk,
    /// 雷引:每战首次攻击伤害 +50%;◇ 敌人首击伤害 +3。
    ThunderBrand,
    /// 星孤:法宝少于 3 件时攻击 +6;◇ 法宝掉落概率减半。
    LoneStar,
    /// 蚀月:仙术伤害 +6;◇ 仙术费用 +2。
    MoonBite,
    /// 韧藤:每次敌人回合后回 2 气血;◇ 药水回复 -10。
    Vinegrip,
    /// 燃魂:攻击与仙术 +3;◇ 每场战斗结束失 4 气血。
    SoulBurn,
    /// 空杯:灵力上限 +8 且开战满灵;◇ 气血上限 -10。
    EmptyCup,
    /// 断因:瘴气毒格不再伤你;◇ 灵泉治疗减半。
    SeveredFate,
}

pub const ALL_HEXES: [HexMark; 14] = [
    HexMark::BloodRage,
    HexMark::GreedSpring,
    HexMark::GaleStep,
    HexMark::DemonPact,
    HexMark::StoneHeart,
    HexMark::BloodTithe,
    HexMark::MistWalk,
    HexMark::ThunderBrand,
    HexMark::LoneStar,
    HexMark::MoonBite,
    HexMark::Vinegrip,
    HexMark::SoulBurn,
    HexMark::EmptyCup,
    HexMark::SeveredFate,
];

impl HexMark {
    /// 妖纹品级(增益与代价越极端,品级越高)。
    pub fn grade(self) -> super::Grade {
        use super::Grade;
        match self {
            HexMark::MistWalk | HexMark::Vinegrip | HexMark::StoneHeart | HexMark::GreedSpring => {
                Grade::Fine
            }
            HexMark::BloodRage
            | HexMark::GaleStep
            | HexMark::BloodTithe
            | HexMark::MoonBite
            | HexMark::EmptyCup
            | HexMark::SeveredFate
            | HexMark::LoneStar => Grade::Superior,
            HexMark::ThunderBrand | HexMark::SoulBurn => Grade::Epic,
            HexMark::DemonPact => Grade::Celestial,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            HexMark::BloodRage => "血怒纹",
            HexMark::GreedSpring => "贪泉纹",
            HexMark::GaleStep => "疾风纹",
            HexMark::DemonPact => "妖契纹",
            HexMark::StoneHeart => "磐心纹",
            HexMark::BloodTithe => "血偿纹",
            HexMark::MistWalk => "雾行纹",
            HexMark::ThunderBrand => "雷引纹",
            HexMark::LoneStar => "星孤纹",
            HexMark::MoonBite => "蚀月纹",
            HexMark::Vinegrip => "韧藤纹",
            HexMark::SoulBurn => "燃魂纹",
            HexMark::EmptyCup => "空杯纹",
            HexMark::SeveredFate => "断因纹",
        }
    }

    pub fn desc(self) -> &'static str {
        match self {
            HexMark::BloodRage => "◆气血过半以下攻+5 ◇气血上限-8",
            HexMark::GreedSpring => "◆胜场+18文 ◇每程入图失3血",
            HexMark::GaleStep => "◆御守也叠气势 ◇御守回灵-2",
            HexMark::DemonPact => "◆绝技只需2层气势 ◇绝技后自伤4",
            HexMark::StoneHeart => "◆御守再减伤8点 ◇攻击-2",
            HexMark::BloodTithe => "◆击杀回血6 ◇开战失2灵",
            HexMark::MistWalk => "◆草丛遇敌减半 ◇战利钱财-30%",
            HexMark::ThunderBrand => "◆每战首击伤害+50% ◇敌首击+3",
            HexMark::LoneStar => "◆法宝<3件时攻+6 ◇法宝掉率减半",
            HexMark::MoonBite => "◆仙术伤害+6 ◇仙术费用+2",
            HexMark::Vinegrip => "◆敌回合后回2血 ◇药水回复-10",
            HexMark::SoulBurn => "◆攻与仙术+3 ◇战后失4血",
            HexMark::EmptyCup => "◆灵上限+8开战满灵 ◇气血上限-10",
            HexMark::SeveredFate => "◆瘴毒不侵 ◇灵泉治疗减半",
        }
    }

    /// 拾取时的一次性属性调整(上限增减立即生效)。
    pub fn on_pickup(self, stats: &mut super::super::core::PlayerStats) {
        match self {
            HexMark::BloodRage => {
                stats.max_hp = (stats.max_hp - 8).max(30);
                stats.hp = stats.hp.min(stats.max_hp);
            }
            HexMark::EmptyCup => {
                stats.max_mp += 8;
                stats.mp = stats.max_mp;
                stats.max_hp = (stats.max_hp - 10).max(30);
                stats.hp = stats.hp.min(stats.max_hp);
            }
            _ => {}
        }
    }
}

/// 从未持有的妖纹里按品级权重抽一枚(全部持有时返回 None)。
/// 品级越高权重越低——仙品妖契一局难遇。
pub fn roll_hex(owned: &[HexMark], rng: &mut Rng) -> Option<HexMark> {
    let pool: Vec<HexMark> = ALL_HEXES
        .iter()
        .copied()
        .filter(|h| !owned.contains(h))
        .collect();
    if pool.is_empty() {
        return None;
    }
    let total: u32 = pool.iter().map(|h| h.grade().weight()).sum();
    let mut roll = rng.range(0, total as i32 - 1) as u32;
    for h in &pool {
        let w = h.grade().weight();
        if roll < w {
            return Some(*h);
        }
        roll -= w;
    }
    pool.last().copied()
}

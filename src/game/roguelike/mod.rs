//! Roguelike run mode (御剑行·轮回).
//!
//! One "run" is: Title → authored chapter journey beats on generated maps →
//! battles / events / story scenes / rests → chapter boss → next chapter →
//! final boss → ending. No experience or levels: all growth comes from 法宝
//! (relics), 丹药 (consumables), stat boons, and per-chapter breakthroughs.
//!
//! The legacy `Explore` overworld is untouched; `Battle` is shared. Battle
//! detects run mode via the presence of the [`RunState`] resource.

use bevy::prelude::*;

pub mod content;
pub mod event;
pub mod graph;
pub mod hex;
pub mod scene;
pub mod screens;
pub mod skill;

use super::battle::{EncounterKind, EncounterZone};
use super::core::Rng;
use super::explore::MapKind;
use super::quest::{BossKind, Companion};
use super::state::AppState;
use graph::NodeKind;

// ---------------------------------------------------------------------------
// Relics (法宝) — persistent passive items collected during a run
// ---------------------------------------------------------------------------

/// 品级:法宝与妖纹的稀有度。决定掉落权重与 UI 用色。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Grade {
    /// 凡品(白):随处可得的实用小物。
    Common,
    /// 良品(绿):有明确战力的常备之选。
    Fine,
    /// 上品(蓝):足以改变打法的利器。
    Superior,
    /// 极品(紫):一件成型的核心。
    Epic,
    /// 仙品(金):一局难遇的传说。
    Celestial,
}

impl Grade {
    pub fn name(self) -> &'static str {
        match self {
            Grade::Common => "凡品",
            Grade::Fine => "良品",
            Grade::Superior => "上品",
            Grade::Epic => "极品",
            Grade::Celestial => "仙品",
        }
    }

    /// UI 用色(白/绿/蓝/紫/金)。
    pub fn color(self) -> bevy::prelude::Color {
        use bevy::prelude::Color;
        match self {
            Grade::Common => Color::srgb(0.90, 0.90, 0.92),
            Grade::Fine => Color::srgb(0.55, 0.95, 0.55),
            Grade::Superior => Color::srgb(0.48, 0.76, 1.0),
            Grade::Epic => Color::srgb(0.85, 0.55, 1.0),
            Grade::Celestial => Color::srgb(1.0, 0.84, 0.35),
        }
    }

    /// 掉落权重:品级越高越难遇。
    pub fn weight(self) -> u32 {
        match self {
            Grade::Common => 40,
            Grade::Fine => 30,
            Grade::Superior => 20,
            Grade::Epic => 8,
            Grade::Celestial => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Relic {
    SwordTassel,   // 青锋剑穗:普攻伤害 +5
    SwordSutra,    // 御剑心诀:仙术伤害 +9
    JadeVial,      // 玉清瓶:药水回复 +25
    SpiritPendant, // 灵犀佩:仙术耗蓝 -2
    TortoiseArmor, // 龟灵甲:受到的伤害 -3
    CloudSleeves,  // 云袖:逃跑必定成功
    SandalCharm,   // 檀木符:阵亡时原地复活一次(半血)
    BloodBead,     // 嗜血珠:击杀敌人回复 12 点气血
    ThunderDrum,   // 雷泽鼓:开战即对敌造成 15 点雷伤
    PixiuPouch,    // 貔貅囊:战利钱财 +50%
    KunlunMirror,  // 昆仑镜:对精英与首领伤害 +25%
    PeachHairpin,  // 桃木钗:拾取时情缘 +2、气血上限 +15
    HeavenScroll,  // 无字天书:拾取时道心 +2、术法上限 +10
    DrunkenBrew,   // 醉仙酿:休息时回复翻倍
    ChixiaoCore,   // 赤霄剑胆:普攻伤害 +4(可与剑穗叠加)
    StarSand,      // 星辰砂:仙术伤害 +5,拾取时术法上限 +12
    VajraPestle,   // 韦陀杵:受到的伤害 -2(可与龟灵甲叠加)
    BreathSoil,    // 息壤袋:每场战斗开始回复 10 点气血
    YinYangMirror, // 阴阳镜:开战时敌人攻击 -3
    SoulLantern,   // 引魂灯:击杀敌人回复 4 点灵力
    GinsengRoot,   // 千年参:拾取时气血上限 +20
    TigerTalisman, // 虎啸符:敌人凶猛强击的伤害额外 -5
}

pub const ALL_RELICS: [Relic; 22] = [
    Relic::SwordTassel,
    Relic::SwordSutra,
    Relic::JadeVial,
    Relic::SpiritPendant,
    Relic::TortoiseArmor,
    Relic::CloudSleeves,
    Relic::SandalCharm,
    Relic::BloodBead,
    Relic::ThunderDrum,
    Relic::PixiuPouch,
    Relic::KunlunMirror,
    Relic::PeachHairpin,
    Relic::HeavenScroll,
    Relic::DrunkenBrew,
    Relic::ChixiaoCore,
    Relic::StarSand,
    Relic::VajraPestle,
    Relic::BreathSoil,
    Relic::YinYangMirror,
    Relic::SoulLantern,
    Relic::GinsengRoot,
    Relic::TigerTalisman,
];

impl Relic {
    /// 法宝品级。
    pub fn grade(self) -> Grade {
        match self {
            Relic::JadeVial | Relic::DrunkenBrew | Relic::GinsengRoot => Grade::Common,
            Relic::SwordTassel
            | Relic::ChixiaoCore
            | Relic::VajraPestle
            | Relic::SpiritPendant
            | Relic::BreathSoil
            | Relic::SoulLantern => Grade::Fine,
            Relic::SwordSutra
            | Relic::TortoiseArmor
            | Relic::BloodBead
            | Relic::StarSand
            | Relic::YinYangMirror
            | Relic::TigerTalisman
            | Relic::PeachHairpin
            | Relic::HeavenScroll => Grade::Superior,
            Relic::ThunderDrum | Relic::PixiuPouch | Relic::KunlunMirror | Relic::CloudSleeves => {
                Grade::Epic
            }
            Relic::SandalCharm => Grade::Celestial,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Relic::SwordTassel => "青锋剑穗",
            Relic::SwordSutra => "御剑心诀",
            Relic::JadeVial => "玉清瓶",
            Relic::SpiritPendant => "灵犀佩",
            Relic::TortoiseArmor => "龟灵甲",
            Relic::CloudSleeves => "云袖",
            Relic::SandalCharm => "檀木符",
            Relic::BloodBead => "嗜血珠",
            Relic::ThunderDrum => "雷泽鼓",
            Relic::PixiuPouch => "貔貅囊",
            Relic::KunlunMirror => "昆仑镜",
            Relic::PeachHairpin => "桃木钗",
            Relic::HeavenScroll => "无字天书",
            Relic::DrunkenBrew => "醉仙酿",
            Relic::ChixiaoCore => "赤霄剑胆",
            Relic::StarSand => "星辰砂",
            Relic::VajraPestle => "韦陀杵",
            Relic::BreathSoil => "息壤袋",
            Relic::YinYangMirror => "阴阳镜",
            Relic::SoulLantern => "引魂灯",
            Relic::GinsengRoot => "千年参",
            Relic::TigerTalisman => "虎啸符",
        }
    }

    pub fn desc(self) -> &'static str {
        match self {
            Relic::SwordTassel => "普攻伤害 +5",
            Relic::SwordSutra => "仙术伤害 +9",
            Relic::JadeVial => "药水回复 +25",
            Relic::SpiritPendant => "仙术耗蓝 -2",
            Relic::TortoiseArmor => "受到的伤害 -3",
            Relic::CloudSleeves => "逃跑必定成功",
            Relic::SandalCharm => "阵亡时复活一次(半血)",
            Relic::BloodBead => "击杀敌人回血 12",
            Relic::ThunderDrum => "开战即造成 15 点雷伤",
            Relic::PixiuPouch => "战利钱财 +50%",
            Relic::KunlunMirror => "对精英与首领伤害 +25%",
            Relic::PeachHairpin => "情缘 +2,气血上限 +15",
            Relic::HeavenScroll => "道心 +2,术法上限 +10",
            Relic::DrunkenBrew => "休息回复翻倍",
            Relic::ChixiaoCore => "普攻伤害 +4",
            Relic::StarSand => "仙术伤害 +5,术法上限 +12",
            Relic::VajraPestle => "受到的伤害 -2",
            Relic::BreathSoil => "开战回复 10 点气血",
            Relic::YinYangMirror => "开战时敌人攻击 -3",
            Relic::SoulLantern => "击杀敌人回复 4 点灵力",
            Relic::GinsengRoot => "气血上限 +20",
            Relic::TigerTalisman => "敌人强击伤害额外 -5",
        }
    }
}

// ---------------------------------------------------------------------------
// Chapters
// ---------------------------------------------------------------------------

/// Seven PAL-like story arcs: village, moon cave, river, plague, capital,
/// southern thunder, and finale.
pub const CHAPTER_COUNT: usize = 7;
/// Authored route length target for a long-form run.
pub const JOURNEY_ROUTE_STAGES: usize = 41;
const RUN_PARTY_SOLO: [Companion; 0] = [];
const RUN_PARTY_LINGER: [Companion; 1] = [Companion::Linger];
const RUN_PARTY_SWORD: [Companion; 2] = [Companion::Linger, Companion::SwordSister];
const RUN_PARTY_FULL: [Companion; 3] = [
    Companion::Linger,
    Companion::SwordSister,
    Companion::SpiritWitch,
];

pub struct ChapterDef {
    pub title: &'static str,
    /// Walkable tile maps this chapter's nodes play out on.
    pub maps: &'static [MapKind],
    /// Encounter pools for normal / elite fights in this chapter.
    pub zones: &'static [EncounterZone],
    /// Boss candidates — one is rolled per run (randomness across runs).
    pub bosses: &'static [BossKind],
    /// Stat multipliers applied to non-boss enemies in this chapter.
    pub enemy_hp_mul: f32,
    pub enemy_atk_mul: f32,
    /// Number of random layers between the entry layer and the boss
    /// (story layer + rest layer are added on top of this).
    pub depth: usize,
}

pub struct JourneyBeat {
    pub title: &'static str,
    pub place: &'static str,
    pub objective: &'static str,
    pub gate_line: &'static str,
}

pub struct ChapterPacing {
    pub minutes: u32,
    pub focus: &'static str,
}

pub struct JourneyIntro {
    pub scene_line: &'static str,
    pub party_line: &'static str,
}

pub const TARGET_RUNTIME_MINUTES: u32 = 600;

pub const CHAPTER_PACING: [ChapterPacing; CHAPTER_COUNT] = [
    ChapterPacing {
        minutes: 65,
        focus: "余杭开局、初遇、十里坡和仙岛誓约",
    },
    ChapterPacing {
        minutes: 70,
        focus: "水月洞天、灵泉整备和月影追妖",
    },
    ChapterPacing {
        minutes: 75,
        focus: "苏州江岸、河灯、人间烟火与河魇",
    },
    ChapterPacing {
        minutes: 80,
        focus: "白河疫雨、荒寺、黑水镇与瘴根",
    },
    ChapterPacing {
        minutes: 85,
        focus: "扬州夜市、京城府影和锁妖镜阵",
    },
    ChapterPacing {
        minutes: 90,
        focus: "苗岭驿路、南诏旧誓和雷鼓祭台",
    },
    ChapterPacing {
        minutes: 135,
        focus: "南诏终局、回梦水道、众愿照心和终战",
    },
];

pub const CHAPTERS: [ChapterDef; CHAPTER_COUNT] = [
    ChapterDef {
        title: "第一卷 · 余杭夜雨",
        maps: &[
            MapKind::Village,
            MapKind::Bamboo,
            MapKind::Cave,
            MapKind::MoonEchoCorridor,
        ],
        zones: &[EncounterZone::Village, EncounterZone::Bamboo],
        bosses: &[BossKind::MountainFiend],
        enemy_hp_mul: 1.35,
        enemy_atk_mul: 0.70,
        depth: 5,
    },
    ChapterDef {
        title: "第二卷 · 水月洞天",
        maps: &[
            MapKind::Village,
            MapKind::Cave,
            MapKind::MoonEchoCorridor,
            MapKind::RiverTown,
        ],
        zones: &[EncounterZone::Village, EncounterZone::Cave],
        bosses: &[BossKind::MoonWraith],
        enemy_hp_mul: 1.50,
        enemy_atk_mul: 0.85,
        depth: 5,
    },
    ChapterDef {
        title: "第三卷 · 苏州江灯",
        maps: &[MapKind::RiverTown, MapKind::RiverReedBed],
        zones: &[EncounterZone::RiverTown],
        bosses: &[BossKind::RiverDemon],
        enemy_hp_mul: 1.65,
        enemy_atk_mul: 1.00,
        depth: 5,
    },
    ChapterDef {
        title: "第四卷 · 白河疫雨",
        maps: &[
            MapKind::PlagueVillage,
            MapKind::PlagueShrinePath,
            MapKind::RiverReedBed,
        ],
        zones: &[EncounterZone::PlagueVillage, EncounterZone::RiverTown],
        bosses: &[BossKind::MiasmaRoot],
        enemy_hp_mul: 1.85,
        enemy_atk_mul: 1.15,
        depth: 5,
    },
    ChapterDef {
        title: "第五卷 · 京华镜影",
        maps: &[
            MapKind::Capital,
            MapKind::CapitalMansion,
            MapKind::MansionMirrorGallery,
        ],
        zones: &[EncounterZone::Capital],
        bosses: &[BossKind::MirrorMinister],
        enemy_hp_mul: 2.05,
        enemy_atk_mul: 1.30,
        depth: 5,
    },
    ChapterDef {
        title: "第六卷 · 南疆雷誓",
        maps: &[MapKind::SouthernRoad, MapKind::ThunderDrumPath],
        zones: &[EncounterZone::SouthernRoad],
        bosses: &[BossKind::ThunderQilin],
        enemy_hp_mul: 2.25,
        enemy_atk_mul: 1.40,
        depth: 5,
    },
    ChapterDef {
        title: "终卷 · 心渊照影",
        maps: &[MapKind::FinalSanctum, MapKind::DreamWaterway],
        zones: &[EncounterZone::FinalSanctum],
        bosses: &[BossKind::DreamEclipse],
        enemy_hp_mul: 2.50,
        enemy_atk_mul: 1.55,
        depth: 11,
    },
];

pub const JOURNEY_MAPS: [&[MapKind]; CHAPTER_COUNT] = [
    &[
        MapKind::Village,
        MapKind::Bamboo,
        MapKind::MoonEchoCorridor,
        MapKind::Cave,
        MapKind::MoonEchoCorridor,
    ],
    &[
        MapKind::Village,
        MapKind::RiverTown,
        MapKind::MoonEchoCorridor,
        MapKind::Cave,
        MapKind::MoonEchoCorridor,
    ],
    &[
        MapKind::RiverTown,
        MapKind::RiverReedBed,
        MapKind::RiverTown,
        MapKind::RiverReedBed,
        MapKind::RiverTown,
    ],
    &[
        MapKind::PlagueShrinePath,
        MapKind::PlagueVillage,
        MapKind::PlagueShrinePath,
        MapKind::RiverReedBed,
        MapKind::PlagueShrinePath,
    ],
    &[
        MapKind::Capital,
        MapKind::CapitalMansion,
        MapKind::MansionMirrorGallery,
        MapKind::CapitalMansion,
        MapKind::MansionMirrorGallery,
    ],
    &[
        MapKind::SouthernRoad,
        MapKind::ThunderDrumPath,
        MapKind::SouthernRoad,
        MapKind::ThunderDrumPath,
        MapKind::SouthernRoad,
    ],
    &[
        MapKind::FinalSanctum,
        MapKind::DreamWaterway,
        MapKind::FinalSanctum,
        MapKind::DreamWaterway,
        MapKind::FinalSanctum,
        MapKind::DreamWaterway,
        MapKind::FinalSanctum,
        MapKind::DreamWaterway,
        MapKind::FinalSanctum,
        MapKind::DreamWaterway,
        MapKind::FinalSanctum,
    ],
];

pub const JOURNEY_BEATS: [&[JourneyBeat]; CHAPTER_COUNT] = [
    &[
        JourneyBeat {
            title: "客栈夜雨",
            place: "余杭客栈后院",
            objective: "查清红衣剑姊留下的妖讯,带着灵儿的线索离开村口。",
            gate_line: "客栈灯火还照着回廊,先压住院外妖气才敢出村。",
        },
        JourneyBeat {
            title: "十里坡旧道",
            place: "竹影山路",
            objective: "沿竹坡寻找仙岛渡口,顺手清掉挡路的小妖。",
            gate_line: "旧道尽头风声发紧,少一处标记都会引来追妖。",
        },
        JourneyBeat {
            title: "仙汀问影",
            place: "水月洞天外汀",
            objective: "循水声靠近洞天,确认月光晶尘中的灵儿踪迹。",
            gate_line: "汀边水影未平,界门不会把你送进洞天深处。",
        },
        JourneyBeat {
            title: "洞天灵誓",
            place: "水月洞天",
            objective: "守住洞天灵阵,在妖影压来前寻得同行誓言。",
            gate_line: "灵阵仍有裂光,先把洞中异动逐一照明。",
        },
        JourneyBeat {
            title: "破庙妖门",
            place: "月回廊与破庙之间",
            objective: "穿过破庙夜路,迎战追到村外的赤鬼山妖。",
            gate_line: "破庙妖门已经喷出赤火,此处没有退路。",
        },
    ],
    &[
        JourneyBeat {
            title: "药圃晨雾",
            place: "竹篱药圃",
            objective: "替灵儿寻齐压惊草药,把村人受妖气牵连的痕迹记下。",
            gate_line: "药圃里还有乱窜妖息,先把可疑光点逐一查清。",
        },
        JourneyBeat {
            title: "渔港风灯",
            place: "余杭小港",
            objective: "护住准备离港的渔灯,问出夜里看见月影的人证。",
            gate_line: "港口风灯仍在乱摆,不稳住路标就会错过渡船。",
        },
        JourneyBeat {
            title: "月回暗阶",
            place: "月回廊下层",
            objective: "沿暗阶绕回洞天深处,截断追妖留下的第二条路。",
            gate_line: "暗阶下还有回声,先探完标记再踏进界门。",
        },
        JourneyBeat {
            title: "灵泉归潮",
            place: "洞天回湾",
            objective: "让灵泉归潮稳住阵心,为月影首领一战做最后整备。",
            gate_line: "灵泉潮线未合,界门不会让人贸然进深门。",
        },
        JourneyBeat {
            title: "月影妖门",
            place: "洞天深门",
            objective: "面对月影首领,护住刚结下的誓言。",
            gate_line: "月影魔门已经现形,此处没有退路。",
        },
    ],
    &[
        JourneyBeat {
            title: "江岸迷雾",
            place: "苏州渡口",
            objective: "顺着江雾追查疫火源头,找出河灯倒流的原因。",
            gate_line: "渡口雾灯未稳,先把江边异响清明。",
        },
        JourneyBeat {
            title: "赤水渡魂",
            place: "赤水渡",
            objective: "护送残魂过渡,换取通往河魇深处的水路记号。",
            gate_line: "渡魂灯还没齐,界门不会替你开水路。",
        },
        JourneyBeat {
            title: "河灯逆流",
            place: "芦苇浅湾",
            objective: "扶正逆流河灯,逼出拖船入雾的黑鳞妖影。",
            gate_line: "河灯还在倒走,先把水面主线标记稳住。",
        },
        JourneyBeat {
            title: "江心伏潮",
            place: "苏州江心",
            objective: "沿伏潮追到河魇蛟巢口,护住摆渡人留下的水符。",
            gate_line: "江心潮声未定,少一处标记都会翻船。",
        },
        JourneyBeat {
            title: "江雾妖门",
            place: "黑鳞漩口",
            objective: "迎战河魇蛟,替江岸城镇夺回水路。",
            gate_line: "江雾妖门已经卷开,此处只剩一战。",
        },
    ],
    &[
        JourneyBeat {
            title: "白河药路",
            place: "白河村外",
            objective: "护送药路消息入村,让病屋重新点起灯。",
            gate_line: "药铃还在发抖,妖气未退前村路不会放行。",
        },
        JourneyBeat {
            title: "玉佛荒阶",
            place: "荒寺石阶",
            objective: "查荒寺封印,分辨佛火与妖火哪一处先乱。",
            gate_line: "荒阶残火未收,界门只会把人绕回原地。",
        },
        JourneyBeat {
            title: "黑水残镇",
            place: "黑水镇外湾",
            objective: "沿黑水残街追到疫火根须,救出被困村人。",
            gate_line: "黑水还在倒卷,先探明镇口的发光妖讯。",
        },
        JourneyBeat {
            title: "药王井口",
            place: "白河旧井",
            objective: "从旧井水脉确认疫毒走向,把还能用的药引带回路上。",
            gate_line: "井口瘴纹还未退,先把主线标记照亮。",
        },
        JourneyBeat {
            title: "疫火妖门",
            place: "瘴根深潭",
            objective: "烧断瘴母根,替白河村和黑水镇打开活路。",
            gate_line: "疫火妖门已经点燃,此处只剩一战。",
        },
    ],
    &[
        JourneyBeat {
            title: "扬州夜市",
            place: "灯市暗巷",
            objective: "在人声最盛处查镜阵线索,不要让暗帖散入城中。",
            gate_line: "夜市人潮未散,先截住会发光的妖讯。",
        },
        JourneyBeat {
            title: "尚书府影",
            place: "京城府门",
            objective: "潜入府门阴影,找出镜仆替身的出入口。",
            gate_line: "府门影子还缺一角,界门不认这条暗路。",
        },
        JourneyBeat {
            title: "锁妖镜廊",
            place: "镜阵偏院",
            objective: "在镜廊中辨认真身与倒影,保住队伍退路。",
            gate_line: "镜面仍在反照人心,先破掉剩余标记。",
        },
        JourneyBeat {
            title: "京畿追缉",
            place: "城郊栈道",
            objective: "甩开镜仆追缉,护住能证明尚书府阴谋的暗帖。",
            gate_line: "追缉影子还在路上,先处理发亮的埋伏点。",
        },
        JourneyBeat {
            title: "照影妖门",
            place: "镜阵中庭",
            objective: "破开照影国师的镜阵,夺回南下道路。",
            gate_line: "照影妖门已开,退后只会被镜仆合围。",
        },
    ],
    &[
        JourneyBeat {
            title: "苗岭驿路",
            place: "南疆前哨",
            objective: "离开京华入苗岭,寻找雷纹与旧鼓的源头。",
            gate_line: "驿路雷火未散,界门不会让外人深入。",
        },
        JourneyBeat {
            title: "南诏鼓道",
            place: "百越灵道",
            objective: "听鼓点穿过灵道,让南瑶的旧誓浮出水面。",
            gate_line: "鼓道仍有乱雷,先稳住图上的每一处光。",
        },
        JourneyBeat {
            title: "灵蛇旧寨",
            place: "苗岭旧寨",
            objective: "在旧寨听取南瑶族人的证词,找出雷纹为何失控。",
            gate_line: "寨门符绳还没落稳,界门不会放外人进鼓道。",
        },
        JourneyBeat {
            title: "雷鼓祭台",
            place: "雷鼓祭台",
            objective: "按风、云、誓三路鼓点稳住祭台,补齐南下誓印。",
            gate_line: "祭台鼓纹还缺一角,先把每处主线标记踩实。",
        },
        JourneyBeat {
            title: "雷麟妖门",
            place: "灵道终坡",
            objective: "迎战雷麟,让南疆灵道重新承认队伍的誓印。",
            gate_line: "雷麟妖门已开,退后只会让旧鼓失控。",
        },
    ],
    &[
        JourneyBeat {
            title: "南诏残垣",
            place: "王城旧墙",
            objective: "循旧墙入终局,把前路欠下的人情逐一照见。",
            gate_line: "残垣回声未息,先确认所有发光记号。",
        },
        JourneyBeat {
            title: "拜月水殿",
            place: "黑水祭台",
            objective: "越过水殿祭台,让道心与情缘同时稳住。",
            gate_line: "水殿还在吞光,少一步都会被旧梦拖回。",
        },
        JourneyBeat {
            title: "回梦水道",
            place: "旧梦归潮",
            objective: "穿过回梦水道,把白狐、琴师与同伴回响带到终门。",
            gate_line: "归潮灯签未稳,界门暂不肯开。",
        },
        JourneyBeat {
            title: "女娲灵台",
            place: "心渊外台",
            objective: "在灵台前定下最后回应,选择以情守人或以剑破局。",
            gate_line: "灵台光影还在问心,先探完这段水路。",
        },
        JourneyBeat {
            title: "旧宫灯引",
            place: "王城旧宫",
            objective: "点亮旧宫残灯,把南诏城里未说出口的愿望串起来。",
            gate_line: "旧宫灯引还少几盏,界门不会把愿望留在身后。",
        },
        JourneyBeat {
            title: "水魔鳞潮",
            place: "黑水鳞湾",
            objective: "穿过鳞潮压迫的水湾,逼近拜月术式的真源。",
            gate_line: "鳞潮尚未退下,先照出水面下的主线标记。",
        },
        JourneyBeat {
            title: "梦里客栈",
            place: "回梦客栈",
            objective: "在梦中客栈辨认最初的夜雨,带回没有说完的告别。",
            gate_line: "梦中灯火还乱,少一盏都会把路绕回开端。",
        },
        JourneyBeat {
            title: "圣姑残阵",
            place: "圣姑旧阵",
            objective: "修补残阵里的护命线,让终战前的队伍状态稳住。",
            gate_line: "残阵符纹还缺回应,先处理所有发光节点。",
        },
        JourneyBeat {
            title: "众愿照心",
            place: "心渊回廊",
            objective: "让途中救过、错过、记住的人化为照心灯,回答终门的问句。",
            gate_line: "照心灯还没聚齐,终门不会承认这一路。",
        },
        JourneyBeat {
            title: "天蛇祭阶",
            place: "终门祭阶",
            objective: "沿祭阶压住最后一轮水影,把道心与情缘送到门前。",
            gate_line: "祭阶水影还在试探,先清完主线路标。",
        },
        JourneyBeat {
            title: "心渊终门",
            place: "宿命水影",
            objective: "面对终章水影,让这一世的选择得到结局。",
            gate_line: "终门已亮,这一世的账只能在门内结清。",
        },
    ],
];

pub const JOURNEY_MARKER_PLANS: [&[&[NodeKind]]; CHAPTER_COUNT] = [
    &[
        &[NodeKind::Story, NodeKind::Market, NodeKind::Fight],
        &[NodeKind::Fight, NodeKind::Event, NodeKind::Rest],
        &[NodeKind::Story, NodeKind::Event, NodeKind::Fight],
        &[NodeKind::Story, NodeKind::Rest, NodeKind::Elite],
        &[NodeKind::Boss],
    ],
    &[
        &[NodeKind::Story, NodeKind::Rest, NodeKind::Fight],
        &[NodeKind::Elite, NodeKind::Event, NodeKind::Fight],
        &[NodeKind::Market, NodeKind::Story, NodeKind::Elite],
        &[NodeKind::Fight, NodeKind::Rest, NodeKind::Event],
        &[NodeKind::Boss],
    ],
    &[
        &[NodeKind::Story, NodeKind::Market, NodeKind::Event],
        &[NodeKind::Rest, NodeKind::Fight, NodeKind::Event],
        &[NodeKind::Market, NodeKind::Story, NodeKind::Elite],
        &[NodeKind::Fight, NodeKind::Rest, NodeKind::Event],
        &[NodeKind::Boss],
    ],
    &[
        &[NodeKind::Story, NodeKind::Fight, NodeKind::Elite],
        &[NodeKind::Fight, NodeKind::Fight, NodeKind::Event],
        &[NodeKind::Elite, NodeKind::Fight, NodeKind::Rest],
        &[NodeKind::Story, NodeKind::Rest, NodeKind::Fight],
        &[NodeKind::Boss],
    ],
    &[
        &[NodeKind::Market, NodeKind::Story, NodeKind::Event],
        &[NodeKind::Fight, NodeKind::Event, NodeKind::Elite],
        &[NodeKind::Story, NodeKind::Elite, NodeKind::Event],
        &[NodeKind::Fight, NodeKind::Market, NodeKind::Event],
        &[NodeKind::Boss],
    ],
    &[
        &[NodeKind::Rest, NodeKind::Fight, NodeKind::Market],
        &[NodeKind::Fight, NodeKind::Elite, NodeKind::Story],
        &[NodeKind::Story, NodeKind::Fight, NodeKind::Elite],
        &[NodeKind::Rest, NodeKind::Elite, NodeKind::Story],
        &[NodeKind::Boss],
    ],
    &[
        &[NodeKind::Story, NodeKind::Fight, NodeKind::Rest],
        &[NodeKind::Elite, NodeKind::Event, NodeKind::Fight],
        &[NodeKind::Story, NodeKind::Event, NodeKind::Rest],
        &[NodeKind::Story, NodeKind::Market, NodeKind::Elite],
        &[NodeKind::Story, NodeKind::Fight, NodeKind::Event],
        &[NodeKind::Elite, NodeKind::Rest, NodeKind::Fight],
        &[NodeKind::Story, NodeKind::Event, NodeKind::Fight],
        &[NodeKind::Market, NodeKind::Elite, NodeKind::Rest],
        &[NodeKind::Fight, NodeKind::Story, NodeKind::Event],
        &[NodeKind::Elite, NodeKind::Fight, NodeKind::Story],
        &[NodeKind::Boss],
    ],
];

pub const JOURNEY_INTROS: [&[JourneyIntro]; CHAPTER_COUNT] = [
    &[
        JourneyIntro {
            scene_line: "雨丝从客栈檐角垂下来,后院竹篱外有一串湿脚印停在灯影边。",
            party_line: "李逍遥：这村口夜路我熟,可今晚连风都像在催人出门。",
        },
        JourneyIntro {
            scene_line: "十里坡的竹叶压得很低,旧路上每一截泥痕都像被妖爪翻过。",
            party_line: "赵灵儿：山气里有水声,再往前或许就能找到渡口。",
        },
        JourneyIntro {
            scene_line: "浅汀月光碎在水面,远处洞门一亮一暗,像有人隔水回望。",
            party_line: "赵灵儿：这里的灵息和我梦里一样,别让妖气先碰到晶尘。",
        },
        JourneyIntro {
            scene_line: "洞天水雾沿石阶漫开,灵阵中央的微光被黑影压得只剩一线。",
            party_line: "李逍遥：既然走到这里,就把这条路守到底。",
        },
        JourneyIntro {
            scene_line: "破庙火盆忽明忽暗,赤鬼山妖的脚印在庙门前烧成一串黑坑。",
            party_line: "赵灵儿：追兵快到了,我们先把庙外妖影引开。",
        },
    ],
    &[
        JourneyIntro {
            scene_line: "竹篱药圃蒙着晨雾,被踏断的草叶下还残留一丝黑气。",
            party_line: "赵灵儿：这些草药能救人,妖气也会顺着药香找来。",
        },
        JourneyIntro {
            scene_line: "小港风灯一盏盏偏向水面,渔船缆绳像被谁从水下扯紧。",
            party_line: "李逍遥：先别急着上船,港口的人证比风还乱。",
        },
        JourneyIntro {
            scene_line: "月回暗阶贴着潮湿石壁往下折,每一级都映出半个脚印。",
            party_line: "赵灵儿：追兵还有第二条路,不能让他们从背后进洞天。",
        },
        JourneyIntro {
            scene_line: "洞天回湾的灵泉涨落不定,阵心在水声里一明一灭。",
            party_line: "李逍遥：把能准备的都准备好,门后那东西快等不住了。",
        },
        JourneyIntro {
            scene_line: "洞天深门前月色凝成紫影,魔门像一只睁开的眼。",
            party_line: "李逍遥：不管门后是什么,我先拔剑。",
        },
    ],
    &[
        JourneyIntro {
            scene_line: "苏州渡口雾重,河灯不顺水走,反而一盏盏朝黑处倒退。",
            party_line: "赵灵儿：灯走反了,多半有人在水底牵着怨气。",
        },
        JourneyIntro {
            scene_line: "赤水渡上残魂排成细线,每盏渡魂灯都映着不同的病容。",
            party_line: "赵灵儿：送他们过水,他们会告诉我们哪条路还活着。",
        },
        JourneyIntro {
            scene_line: "芦苇浅湾里河灯一明一灭,水下黑影顺着灯线缓缓游动。",
            party_line: "李逍遥：河灯不是在漂,是在给我们画路。",
        },
        JourneyIntro {
            scene_line: "江心伏潮把船影揉成碎片,每一道浪纹都像黑鳞擦过。",
            party_line: "林月衡：蛟巢就在前面,别让它把渡口拖进雾里。",
        },
        JourneyIntro {
            scene_line: "黑鳞漩口轰然张开,江雾卷成一道直落水底的妖门。",
            party_line: "李逍遥：这一路的水债,今日一并讨回来。",
        },
    ],
    &[
        JourneyIntro {
            scene_line: "白河村外药铃摇得急,病屋灯火隔着雾像快要熄灭。",
            party_line: "李逍遥：先把药路护住,人还在等救命消息。",
        },
        JourneyIntro {
            scene_line: "荒寺石阶覆着冷灰,佛火和妖火混在一起,分不出哪边更烫。",
            party_line: "赵灵儿：封印没有坏透,找准裂口还能补回去。",
        },
        JourneyIntro {
            scene_line: "黑水残镇的门窗都朝内闭着,水声却在街角一遍遍敲门。",
            party_line: "李逍遥：有人被困在里面,我们不能只从镇口绕过去。",
        },
        JourneyIntro {
            scene_line: "白河旧井的井绳没有人碰,却一寸寸往黑水里沉。",
            party_line: "赵灵儿：水脉染得太深了,药引和疫毒都在井下。",
        },
        JourneyIntro {
            scene_line: "瘴根深潭翻着红泡,潭底像有许多细根缠住月影。",
            party_line: "林月衡：首领不肯露面,那就把它藏身的水先逼亮。",
        },
    ],
    &[
        JourneyIntro {
            scene_line: "扬州灯市人声如潮,暗巷里却有镜片一样的寒光贴墙游走。",
            party_line: "林月衡：热闹处最适合藏线人,也最适合藏杀机。",
        },
        JourneyIntro {
            scene_line: "尚书府门影子比门还深,石狮眼里映出两个不一样的月亮。",
            party_line: "林月衡：镜仆出入口就在府门影下,别只看正门。",
        },
        JourneyIntro {
            scene_line: "镜廊一层照出身形,一层照出心念,再深处连脚步声都会回头。",
            party_line: "赵灵儿：若看见另一个自己,先听队友的声音。",
        },
        JourneyIntro {
            scene_line: "城郊栈道尘土未落,镜仆追缉的影子已经贴上木栏。",
            party_line: "林月衡：暗帖在手,就会有人想让我们永远走不到南边。",
        },
        JourneyIntro {
            scene_line: "镜阵中庭百面铜镜同时转向队伍,每一面都映着不同的退路。",
            party_line: "林月衡：不破这阵,南边的路永远只是镜中影。",
        },
    ],
    &[
        JourneyIntro {
            scene_line: "离京后的驿路忽然空旷,远山雷纹一跳,像有人在云底点鼓。",
            party_line: "南瑶：那不是天雷,是族里的旧鼓在认路。",
        },
        JourneyIntro {
            scene_line: "百越灵道两侧挂着旧符,每走一步,鼓声就从脚底往心口撞。",
            party_line: "南瑶：跟着鼓点走,不要抢在雷声前面。",
        },
        JourneyIntro {
            scene_line: "苗岭旧寨门口挂着断开的符绳,屋檐下的铃只朝一个方向响。",
            party_line: "南瑶：这里认得我的鼓纹,也认得族里失控的雷。",
        },
        JourneyIntro {
            scene_line: "雷鼓祭台三面开口,风声、云声、誓声从不同石鼓里滚出来。",
            party_line: "南瑶：三路鼓点都要稳住,否则南下的誓印不会成形。",
        },
        JourneyIntro {
            scene_line: "终坡上雷光把云底撕开,雷麟的影子踏着旧鼓声逼近。",
            party_line: "南瑶：接住这道雷,南疆才会把最后的门交给我们。",
        },
    ],
    &[
        JourneyIntro {
            scene_line: "南诏残垣被水汽泡得发白,每一块旧砖都像记得一个名字。",
            party_line: "赵灵儿：前路欠下的人情,会在这里一起回声。",
        },
        JourneyIntro {
            scene_line: "拜月水殿没有风,黑水却自己起纹,把灯影拖成细长的蛇。",
            party_line: "李逍遥：道心也好,情缘也好,别在这水里散了。",
        },
        JourneyIntro {
            scene_line: "回梦水道映出旧日岔路,白狐影、琴声和同伴脚步一层层浮上来。",
            party_line: "南瑶：听见回声也别回头,让它跟着我们去终门。",
        },
        JourneyIntro {
            scene_line: "女娲灵台前光影静得像一面水镜,所有选择都在镜中等回答。",
            party_line: "赵灵儿：最后这一步,不要只问剑,也问问自己。",
        },
        JourneyIntro {
            scene_line: "王城旧宫的残灯一盏接一盏亮起,灯芯里都是没有说完的名字。",
            party_line: "赵灵儿：他们不是拦路,是在等有人替他们记得。",
        },
        JourneyIntro {
            scene_line: "黑水鳞湾涌起细密鳞光,水面像披上会呼吸的甲。",
            party_line: "南瑶：水魔的影子离得很近,别让它把阵脚搅散。",
        },
        JourneyIntro {
            scene_line: "回梦客栈重新下起第一夜的雨,柜台后却没有掌灯的人。",
            party_line: "李逍遥：原来走了这么远,梦还是会把人带回起点。",
        },
        JourneyIntro {
            scene_line: "圣姑旧阵只剩半圈护命线,每一道断纹都还留着热度。",
            party_line: "南瑶：把护命线接上,我们才有力气走到最后。",
        },
        JourneyIntro {
            scene_line: "心渊回廊里亮起许多小灯,每盏都映出旅途中一个选择。",
            party_line: "赵灵儿：这些光会问我们,有没有认真听过他们的愿望。",
        },
        JourneyIntro {
            scene_line: "终门祭阶被水影一层层盖住,台阶尽头只剩一线白光。",
            party_line: "林月衡：最后的水影交给剑,最后的选择交给你们。",
        },
        JourneyIntro {
            scene_line: "心渊终门亮起时,一路遇见的人与妖都化作水面细光。",
            party_line: "李逍遥：这一世走到这里,结局就由我们亲手接住。",
        },
    ],
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChapterClearReward {
    pub seal: &'static str,
    pub line: &'static str,
    pub hp: i32,
    pub mp: i32,
    pub atk: i32,
    pub def: i32,
}

/// Chapter boss clears award a visible 章印 plus uneven breakthrough growth, so
/// chapter endings read like story milestones instead of generic loot.
pub const CHAPTER_CLEAR_REWARDS: [ChapterClearReward; CHAPTER_COUNT] = [
    ChapterClearReward {
        seal: "余杭赤火印",
        line: "赤鬼山妖退散,余杭夜路终于能听见人声。",
        hp: 16,
        mp: 6,
        atk: 4,
        def: 2,
    },
    ChapterClearReward {
        seal: "水月灵誓印",
        line: "月魄妖退散,水月洞天重新映出同伴身影。",
        hp: 18,
        mp: 8,
        atk: 4,
        def: 2,
    },
    ChapterClearReward {
        seal: "苏州河灯印",
        line: "河魇蛟沉入江心,倒流河灯顺水归城。",
        hp: 20,
        mp: 8,
        atk: 5,
        def: 2,
    },
    ChapterClearReward {
        seal: "白河清瘴印",
        line: "瘴母根断裂,白河雨声第一次像雨。",
        hp: 22,
        mp: 9,
        atk: 5,
        def: 3,
    },
    ChapterClearReward {
        seal: "京华破镜印",
        line: "照影国师镜阵碎裂,暗帖有了能见天日的路。",
        hp: 24,
        mp: 9,
        atk: 6,
        def: 3,
    },
    ChapterClearReward {
        seal: "南疆雷誓印",
        line: "雷麟收起天雷,南疆灵道承认队伍的誓印。",
        hp: 26,
        mp: 10,
        atk: 6,
        def: 4,
    },
    ChapterClearReward {
        seal: "心渊照影印",
        line: "宿命水影散入终门,这一世的选择有了回声。",
        hp: 0,
        mp: 0,
        atk: 0,
        def: 0,
    },
];

// ---------------------------------------------------------------------------
// Run state
// ---------------------------------------------------------------------------

/// Why the current battle started — decides what the Reward screen offers and
/// where control returns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FightRank {
    Normal,
    Elite,
    Boss,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RunRouteCommission {
    pub chapter: usize,
    pub stage: usize,
    pub target: NodeKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunCampTactic {
    Breath,
    SwordGuard,
    LingerWard,
    SpiritFocus,
}

impl RunCampTactic {
    pub fn name(self) -> &'static str {
        match self {
            RunCampTactic::Breath => "调息",
            RunCampTactic::SwordGuard => "剑守",
            RunCampTactic::LingerWard => "灵护",
            RunCampTactic::SpiritFocus => "凝灵",
        }
    }

    pub fn battle_line(self) -> &'static str {
        match self {
            RunCampTactic::Breath => "吐纳守夜,下一战开场回稳气血与灵力,并给出少量威力。",
            RunCampTactic::SwordGuard => "拆招到深夜,下一战剑式与绝技威力提高。",
            RunCampTactic::LingerWard => "灵儿护念成阵,下一战开场护住气血,受击时额外挡伤。",
            RunCampTactic::SpiritFocus => "南瑶凝住灵纹,下一战仙术与绝技更稳,御守时额外回灵。",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunChapterVow {
    Heart,
    Resolve,
    Twin,
}

impl RunChapterVow {
    pub fn from_scores(daoxin: i32, qingyuan: i32) -> Self {
        if qingyuan > daoxin {
            Self::Heart
        } else if daoxin > qingyuan {
            Self::Resolve
        } else {
            Self::Twin
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Heart => "护心誓",
            Self::Resolve => "破势誓",
            Self::Twin => "同心誓",
        }
    }

    pub fn record_line(self) -> &'static str {
        match self {
            Self::Heart => "把这一卷的牵挂记成护心誓,章末对峙时会压低首领攻势。",
            Self::Resolve => "把这一卷的决断记成破势誓,章末对峙时会先破首领气血。",
            Self::Twin => "把这一卷的取舍记成同心誓,章末对峙时会同时稳住攻守。",
        }
    }

    pub fn boss_line(self) -> &'static str {
        match self {
            Self::Heart => "同行牵挂先一步护住心脉,首领攻势被压低。",
            Self::Resolve => "本卷决断化成破势剑意,首领气血先被削开。",
            Self::Twin => "道心与情缘同时回应,首领气血与攻势都被压住一线。",
        }
    }

    pub fn boss_hp_multiplier(self) -> f32 {
        match self {
            Self::Heart => 1.0,
            Self::Resolve => 0.92,
            Self::Twin => 0.95,
        }
    }

    pub fn boss_atk_multiplier(self) -> f32 {
        match self {
            Self::Heart => 0.90,
            Self::Resolve => 1.0,
            Self::Twin => 0.95,
        }
    }
}

/// How the run ended; consumed by the Ending screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunOutcome {
    Defeat,
    VictoryLove,    // 情缘 ending
    VictoryResolve, // 道心 ending
    VictoryBoth,    // 道心情缘双全的隐藏结局
}

/// The whole roguelike run. Present ⇔ a run is active (battle systems use
/// this to tell run-mode battles from legacy Explore battles).
#[derive(Resource)]
pub struct RunState {
    pub chapter: usize,
    /// Map index within the chapter: 0..depth-1 are normal stages, the last
    /// stage is the boss map.
    pub stage: usize,
    /// Active play time measured by the real run states. Large frame gaps are
    /// clamped by the tracking system so suspended-window time is not counted.
    pub play_seconds: f32,
    pub chapter_play_seconds: [f32; CHAPTER_COUNT],
    /// Map used for the previous stage (avoid immediate repeats).
    pub last_map: Option<MapKind>,
    /// Boss rolled for this chapter.
    pub boss: BossKind,
    pub relics: Vec<Relic>,
    /// Ending counters, pushed by story/event choices.
    pub daoxin: i32,
    pub qingyuan: i32,
    /// Rank of the fight currently in progress (set when entering Battle).
    pub current_fight: Option<FightRank>,
    /// The chapter card has been shown for the current chapter.
    pub card_shown: bool,
    /// Event-pool indices already drawn this run (no repeats until exhausted).
    pub seen_events: Vec<usize>,
    /// Chapter/story indices already viewed this run. Required story nodes draw
    /// from unseen entries first so one playthrough does not repeat a scene.
    pub seen_story_scenes: Vec<(usize, usize)>,
    /// Route stages whose main task contract has been explicitly signed.
    pub accepted_journey_tasks: Vec<(usize, usize)>,
    /// Route stages whose signed main task has been turned in at the gate.
    pub completed_journey_tasks: Vec<(usize, usize)>,
    /// Chapter boss seals earned this run, indexed by chapter.
    pub chapter_seals: Vec<usize>,
    /// Optional per-stage local errand currently signed from a Guide marker.
    pub active_route_commission: Option<RunRouteCommission>,
    /// Local guide errands that were completed and rewarded.
    pub completed_route_commissions: Vec<(usize, usize)>,
    /// One story vow recorded by each chapter's 「缘」 choice.
    pub chapter_vows: Vec<(usize, RunChapterVow)>,
    /// Current route-stage rests seen. Used to make camp scenes count in long-run
    /// progress rather than behave like anonymous heal wells.
    pub camp_scenes_seen: Vec<(usize, usize)>,
    /// One prepared camp tactic for the next battle.
    pub active_camp_tactic: Option<RunCampTactic>,
    /// Optional route NPCs / small errands helped during this run.
    pub route_contacts_helped: u32,
    /// Required route mechanisms solved during this run.
    pub route_puzzles_solved: u32,
    /// Set when the run ends; read by the Ending screen.
    pub outcome: Option<RunOutcome>,
    /// Totals for the ending screen.
    pub fights_won: u32,
    /// One-shot revive from 檀木符 has been consumed.
    pub revive_used: bool,
    /// 白狐三遇链:0 未遇,1/2/3 已推进到第几遇。
    /// 妖纹(海克斯):战利里随机出现的有代价词条,可叠加成 build。
    pub hexes: Vec<hex::HexMark>,
    /// 已习得的技能(百技谱下标),战斗「仙术」子菜单从这里列出。
    pub skills: Vec<skill::SkillId>,
    pub fox_stage: u8,
    /// 白狐链善恶记号:+1 救过,-1 无视(初遇选择写入)。
    pub fox_kind: i8,
    /// 盲女琴师链:0 未遇,1 听过曲,2 已重逢。
    pub qin_stage: u8,
}

impl RunState {
    pub fn new(rng: &mut Rng) -> Self {
        let chapter = 0;
        let def = &CHAPTERS[chapter];
        let boss = def.bosses[rng.range(0, def.bosses.len() as i32 - 1) as usize];
        Self {
            chapter,
            stage: 0,
            play_seconds: 0.0,
            chapter_play_seconds: [0.0; CHAPTER_COUNT],
            last_map: None,
            boss,
            relics: Vec::new(),
            daoxin: 0,
            qingyuan: 0,
            current_fight: None,
            card_shown: false,
            seen_events: Vec::new(),
            seen_story_scenes: Vec::new(),
            accepted_journey_tasks: Vec::new(),
            completed_journey_tasks: Vec::new(),
            chapter_seals: Vec::new(),
            active_route_commission: None,
            completed_route_commissions: Vec::new(),
            chapter_vows: Vec::new(),
            camp_scenes_seen: Vec::new(),
            active_camp_tactic: None,
            route_contacts_helped: 0,
            route_puzzles_solved: 0,
            outcome: None,
            fights_won: 0,
            revive_used: false,
            hexes: Vec::new(),
            skills: Vec::new(),
            fox_stage: 0,
            fox_kind: 0,
            qin_stage: 0,
        }
    }

    /// Advance to the next chapter.
    pub fn next_chapter(&mut self, rng: &mut Rng) {
        self.chapter += 1;
        let def = &CHAPTERS[self.chapter];
        self.boss = def.bosses[rng.range(0, def.bosses.len() as i32 - 1) as usize];
        self.stage = 0;
        self.last_map = None;
        self.card_shown = false;
    }

    /// Total maps in the current chapter (the last one is the boss map).
    pub fn stage_count(&self) -> usize {
        self.chapter_def().depth
    }

    pub fn is_boss_stage(&self) -> bool {
        self.stage + 1 >= self.stage_count()
    }

    pub fn chapter_def(&self) -> &'static ChapterDef {
        &CHAPTERS[self.chapter]
    }

    pub fn journey_beat(&self) -> &'static JourneyBeat {
        let chapter = self.chapter.min(JOURNEY_BEATS.len() - 1);
        let beats = JOURNEY_BEATS[chapter];
        &beats[self.stage.min(beats.len() - 1)]
    }

    pub fn journey_summary(&self) -> String {
        let beat = self.journey_beat();
        format!(
            "{} · {}\n任务：{} [{}]    {}    章印：{}/{}    营策：{}    誓记：{}\n节奏：{}    {}\n实测用时：{} · 本卷 {}\n队伍：{}    缘忆：{}\n营火照应：{}    机关破除：{}    路人回声：{}    路人签：{}    路况：{}    {}\n地点：{}\n目标：{}",
            self.chapter_def().title,
            beat.title,
            self.journey_task_receipt(),
            self.journey_task_state_label(),
            self.journey_task_archive_label(),
            self.chapter_seals.len(),
            CHAPTER_COUNT,
            self.camp_summary(),
            self.chapter_vow_summary(),
            self.journey_stage_pacing_summary(),
            self.chapter_pacing_summary(),
            self.play_time_summary(),
            self.chapter_play_time_summary(self.chapter),
            self.party_summary(),
            self.current_story_memory_summary(),
            self.camp_scenes_seen.len(),
            self.route_puzzles_solved,
            self.route_contacts_helped,
            self.route_commission_state_label(),
            self.route_guidance_summary(),
            self.route_boss_preparation_summary(),
            beat.place,
            beat.objective
        )
    }

    pub fn chapter_pacing(&self) -> &'static ChapterPacing {
        &CHAPTER_PACING[self.chapter.min(CHAPTER_PACING.len() - 1)]
    }

    pub fn record_play_time(&mut self, seconds: f32) {
        if !seconds.is_finite() || seconds <= 0.0 {
            return;
        }
        self.play_seconds += seconds;
        let chapter = self.chapter.min(CHAPTER_COUNT - 1);
        self.chapter_play_seconds[chapter] += seconds;
    }

    pub fn play_time_summary(&self) -> String {
        format_play_time(self.play_seconds)
    }

    pub fn chapter_play_time_summary(&self, chapter: usize) -> String {
        format_play_time(self.chapter_play_seconds[chapter.min(CHAPTER_COUNT - 1)])
    }

    pub fn chapter_play_time_archive_summary(&self) -> String {
        let parts = self
            .chapter_play_seconds
            .iter()
            .enumerate()
            .map(|(chapter, seconds)| format!("卷{} {}", chapter + 1, format_play_time(*seconds)))
            .collect::<Vec<_>>();
        parts
            .chunks(4)
            .map(|chunk| chunk.join(" · "))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn chapter_pacing_summary(&self) -> String {
        let pacing = self.chapter_pacing();
        format!(
            "本卷约{}分 · {} · 全旅程目标 10小时",
            pacing.minutes, pacing.focus
        )
    }

    pub fn journey_stage_minutes(&self) -> u32 {
        let chapter = self.chapter.min(JOURNEY_MARKER_PLANS.len() - 1);
        let plans = JOURNEY_MARKER_PLANS[chapter];
        let total_weight: u32 = plans
            .iter()
            .map(|plan| journey_marker_plan_weight(plan))
            .sum();
        let stage_weight = journey_marker_plan_weight(self.journey_marker_plan());
        ((self.chapter_pacing().minutes * stage_weight) + (total_weight / 2)) / total_weight
    }

    pub fn journey_required_content_summary(&self) -> String {
        summarize_journey_marker_plan(self.journey_marker_plan())
    }

    pub fn journey_stage_pacing_summary(&self) -> String {
        format!(
            "本程约{}分 · 必做：{}",
            self.journey_stage_minutes(),
            self.journey_required_content_summary()
        )
    }

    pub fn record_route_contact(&mut self) {
        self.route_contacts_helped += 1;
    }

    pub fn route_commission_id(&self) -> (usize, usize) {
        (self.chapter, self.stage)
    }

    pub fn route_commission_receipt(&self) -> String {
        format!("路人签 卷{}-{:02}", self.chapter + 1, self.stage + 1)
    }

    pub fn is_route_commission_active(&self) -> bool {
        self.active_route_commission.is_some_and(|commission| {
            (commission.chapter, commission.stage) == self.route_commission_id()
        })
    }

    pub fn is_route_commission_completed(&self) -> bool {
        self.completed_route_commissions
            .contains(&self.route_commission_id())
    }

    pub fn accept_route_commission(&mut self, target: NodeKind) -> bool {
        if self.is_route_commission_completed() {
            return false;
        }
        let commission = RunRouteCommission {
            chapter: self.chapter,
            stage: self.stage,
            target,
        };
        if self.active_route_commission == Some(commission) {
            false
        } else {
            self.active_route_commission = Some(commission);
            true
        }
    }

    pub fn complete_route_commission(&mut self, kind: NodeKind) -> bool {
        let Some(commission) = self.active_route_commission else {
            return false;
        };
        if (commission.chapter, commission.stage) != self.route_commission_id()
            || commission.target != kind
            || self
                .completed_route_commissions
                .contains(&self.route_commission_id())
        {
            return false;
        }
        self.completed_route_commissions
            .push(self.route_commission_id());
        self.active_route_commission = None;
        self.record_route_contact();
        true
    }

    pub fn route_commission_state_label(&self) -> String {
        if self.is_route_commission_completed() {
            return "已回执".to_string();
        }
        if let Some(commission) = self.active_route_commission {
            if (commission.chapter, commission.stage) == self.route_commission_id() {
                return format!("已追踪 {}", commission.target.label());
            }
        }
        "未接取".to_string()
    }

    pub fn route_commission_hud_label(&self) -> Option<String> {
        let commission = self.active_route_commission?;
        if (commission.chapter, commission.stage) == self.route_commission_id() {
            Some(format!(
                "{} -> {}",
                self.route_commission_receipt(),
                commission.target.label()
            ))
        } else {
            None
        }
    }

    pub fn chapter_route_contacts(&self) -> usize {
        self.completed_route_commissions
            .iter()
            .filter(|(chapter, _)| *chapter == self.chapter)
            .count()
    }

    pub fn route_guidance_rank(&self) -> usize {
        self.chapter_route_contacts().min(3)
    }

    pub fn route_guidance_multiplier(&self) -> f32 {
        match self.route_guidance_rank() {
            0 => 1.0,
            1 => 0.92,
            2 => 0.84,
            _ => 0.76,
        }
    }

    pub fn grass_encounter_chance(&self, base: f32) -> f32 {
        (base * self.route_guidance_multiplier()).clamp(0.0, 1.0)
    }

    pub fn route_guidance_label(&self) -> &'static str {
        match self.route_guidance_rank() {
            0 => "未稳",
            1 => "风声初稳",
            2 => "乡路半稳",
            _ => "熟路照应",
        }
    }

    pub fn route_guidance_summary(&self) -> String {
        let contacts = self.chapter_route_contacts();
        let reduction = ((1.0 - self.route_guidance_multiplier()) * 100.0).round() as i32;
        if reduction <= 0 {
            format!("{} 本卷路人签 {}", self.route_guidance_label(), contacts)
        } else {
            format!(
                "{} 本卷路人签 {} · 草地遇敌-{}%",
                self.route_guidance_label(),
                contacts,
                reduction
            )
        }
    }

    pub fn route_guidance_hud_label(&self) -> Option<String> {
        if self.route_guidance_rank() == 0 {
            None
        } else {
            Some(self.route_guidance_summary())
        }
    }

    pub fn route_boss_preparation_rank(&self) -> usize {
        match self.chapter_route_contacts() {
            0 | 1 => 0,
            2 => 1,
            _ => 2,
        }
    }

    pub fn route_boss_preparation_label(&self) -> &'static str {
        match self.route_boss_preparation_rank() {
            0 => "未备",
            1 => "乡路照应",
            _ => "熟路照应",
        }
    }

    pub fn route_boss_hp_multiplier(&self) -> f32 {
        match self.route_boss_preparation_rank() {
            0 | 1 => 1.0,
            _ => 0.96,
        }
    }

    pub fn route_boss_atk_multiplier(&self) -> f32 {
        match self.route_boss_preparation_rank() {
            0 => 1.0,
            1 => 0.96,
            _ => 0.92,
        }
    }

    pub fn route_boss_restore(&self) -> (i32, i32) {
        match self.route_boss_preparation_rank() {
            0 => (0, 0),
            1 => (8, 4),
            _ => (12, 8),
        }
    }

    pub fn route_boss_preparation_summary(&self) -> String {
        let contacts = self.chapter_route_contacts();
        match self.route_boss_preparation_rank() {
            0 => format!("首领照应 未备 本卷路人签 {contacts}/3"),
            1 => format!("首领照应 乡路照应 本卷路人签 {contacts}/3 · 首领攻势-4%"),
            _ => format!("首领照应 熟路照应 本卷路人签 {contacts}/3 · 首领气血-4% 攻势-8%"),
        }
    }

    pub fn record_route_puzzle(&mut self) {
        self.route_puzzles_solved += 1;
    }

    pub fn record_chapter_vow(&mut self, chapter: usize, vow: RunChapterVow) -> bool {
        if self
            .chapter_vows
            .iter()
            .any(|(recorded, _)| *recorded == chapter)
        {
            false
        } else {
            self.chapter_vows.push((chapter, vow));
            true
        }
    }

    pub fn chapter_vow(&self, chapter: usize) -> Option<RunChapterVow> {
        self.chapter_vows
            .iter()
            .find_map(|(recorded, vow)| (*recorded == chapter).then_some(*vow))
    }

    pub fn current_chapter_vow(&self) -> Option<RunChapterVow> {
        self.chapter_vow(self.chapter)
    }

    pub fn chapter_vow_summary(&self) -> String {
        self.current_chapter_vow()
            .map(|vow| vow.name().to_string())
            .unwrap_or_else(|| "未立".to_string())
    }

    pub fn chapter_vow_archive_summary(&self) -> String {
        if self.chapter_vows.is_empty() {
            return "誓记：未立".to_string();
        }
        let heart = self
            .chapter_vows
            .iter()
            .filter(|(_, vow)| *vow == RunChapterVow::Heart)
            .count();
        let resolve = self
            .chapter_vows
            .iter()
            .filter(|(_, vow)| *vow == RunChapterVow::Resolve)
            .count();
        let twin = self
            .chapter_vows
            .iter()
            .filter(|(_, vow)| *vow == RunChapterVow::Twin)
            .count();
        let mut parts = Vec::new();
        if heart > 0 {
            parts.push(format!("护心{heart}"));
        }
        if resolve > 0 {
            parts.push(format!("破势{resolve}"));
        }
        if twin > 0 {
            parts.push(format!("同心{twin}"));
        }
        format!("誓记：{}", parts.join(" · "))
    }

    pub fn journey_task_receipt(&self) -> String {
        format!("主线签 卷{}-{:02}", self.chapter + 1, self.stage + 1)
    }

    pub fn journey_task_id(&self) -> (usize, usize) {
        (self.chapter, self.stage)
    }

    pub fn is_journey_task_accepted(&self) -> bool {
        self.accepted_journey_tasks
            .iter()
            .any(|task| *task == self.journey_task_id())
    }

    pub fn is_journey_task_completed(&self) -> bool {
        self.completed_journey_tasks
            .iter()
            .any(|task| *task == self.journey_task_id())
    }

    pub fn accept_journey_task(&mut self) -> bool {
        let task = self.journey_task_id();
        if self.accepted_journey_tasks.contains(&task) {
            false
        } else {
            self.accepted_journey_tasks.push(task);
            true
        }
    }

    pub fn complete_journey_task(&mut self) -> bool {
        let task = self.journey_task_id();
        if !self.accepted_journey_tasks.contains(&task) {
            self.accepted_journey_tasks.push(task);
        }
        if self.completed_journey_tasks.contains(&task) {
            false
        } else {
            self.completed_journey_tasks.push(task);
            true
        }
    }

    pub fn journey_task_archive_label(&self) -> String {
        format!(
            "主线归档 {}/{}",
            self.completed_journey_tasks.len(),
            JOURNEY_ROUTE_STAGES
        )
    }

    pub fn journey_task_state_label(&self) -> &'static str {
        if self.is_journey_task_completed() {
            "已归档"
        } else if self.is_journey_task_accepted() {
            "已追踪"
        } else {
            "待签收"
        }
    }

    pub fn chapter_clear_reward(&self) -> &'static ChapterClearReward {
        &CHAPTER_CLEAR_REWARDS[self.chapter.min(CHAPTER_CLEAR_REWARDS.len() - 1)]
    }

    pub fn record_chapter_clear(&mut self) -> bool {
        // A chapter seal is the boss-stage receipt. Keep both ledgers atomic so
        // chapter transitions cannot leave the main task permanently open.
        self.complete_journey_task();
        let chapter = self.chapter.min(CHAPTER_CLEAR_REWARDS.len() - 1);
        if self.chapter_seals.contains(&chapter) {
            false
        } else {
            self.chapter_seals.push(chapter);
            true
        }
    }

    pub fn chapter_seal_summary(&self) -> String {
        if self.chapter_seals.is_empty() {
            return "未得".to_string();
        }
        self.chapter_seals
            .iter()
            .filter_map(|chapter| CHAPTER_CLEAR_REWARDS.get(*chapter))
            .map(|reward| reward.seal)
            .collect::<Vec<_>>()
            .join("、")
    }

    pub fn camp_scene_id(&self) -> (usize, usize) {
        (self.chapter, self.stage)
    }

    pub fn record_camp_scene(&mut self) -> bool {
        let scene = self.camp_scene_id();
        if self.camp_scenes_seen.contains(&scene) {
            false
        } else {
            self.camp_scenes_seen.push(scene);
            true
        }
    }

    pub fn set_camp_tactic(&mut self, tactic: RunCampTactic) {
        self.active_camp_tactic = Some(tactic);
    }

    pub fn take_camp_tactic(&mut self) -> Option<RunCampTactic> {
        self.active_camp_tactic.take()
    }

    pub fn camp_summary(&self) -> &'static str {
        self.active_camp_tactic
            .map(RunCampTactic::name)
            .unwrap_or("未备")
    }

    pub fn available_camp_tactics(&self) -> Vec<RunCampTactic> {
        let mut tactics = vec![RunCampTactic::Breath, RunCampTactic::SwordGuard];
        if self.party_companions().contains(&Companion::Linger) {
            tactics.push(RunCampTactic::LingerWard);
        }
        if self.party_companions().contains(&Companion::SpiritWitch) {
            tactics.push(RunCampTactic::SpiritFocus);
        }
        tactics
    }

    pub fn camp_tactic_label(&self, tactic: RunCampTactic) -> &'static str {
        match tactic {
            RunCampTactic::Breath => "调息守夜(回复气血/灵力,备调息)",
            RunCampTactic::SwordGuard
                if self.party_companions().contains(&Companion::SwordSister) =>
            {
                "林月衡试招(攻击 +2,备剑守)"
            }
            RunCampTactic::SwordGuard => "温酒论剑(攻击 +2,备剑守)",
            RunCampTactic::LingerWard => "灵儿护念(情缘 +1,备灵护)",
            RunCampTactic::SpiritFocus => "南瑶凝灵(道心 +1,备凝灵)",
        }
    }

    pub fn camp_scene_line(&self) -> String {
        let beat = self.journey_beat();
        let chapter_line = match self.chapter {
            0 => "夜雨在篝火外慢慢小了,远处客栈灯影像还在等人回去。",
            1 => "洞天石壁映出月纹,水声一圈圈绕过队伍脚边。",
            2 => "江风吹得河灯低低摇晃,水下黑影暂时不敢靠近火光。",
            3 => "白河湿雾压在营地外,药香和瘴气隔着火堆互不相让。",
            4 => "京华更鼓从远墙后传来,镜片般的月光落在剑鞘上。",
            5 => "南疆雷云低垂,旧鼓声在地脉里替队伍守着方向。",
            _ => "心渊水光映出一路来的人影,每一次停步都像在回答终门。",
        };
        format!("【{}】{} {}", beat.place, chapter_line, beat.objective)
    }

    pub fn camp_party_line(&self) -> &'static str {
        let party = self.party_companions();
        if party.contains(&Companion::SpiritWitch) {
            "南瑶把铜铃埋进灰里:「今晚定一条路,明早雷声就不会乱。」"
        } else if party.contains(&Companion::SwordSister) {
            "林月衡把剑横在膝上:「别只想着睡,把下一战先拆一遍。」"
        } else if party.contains(&Companion::Linger) {
            "赵灵儿把灵息拢成小小灯罩:「歇一歇,明天才护得住更多人。」"
        } else {
            "李逍遥把剑靠在篝火边,第一次觉得夜路比白日更会说话。"
        }
    }

    pub fn party_companions(&self) -> &'static [Companion] {
        match (self.chapter, self.stage) {
            (0, 0) => &RUN_PARTY_SOLO,
            (0..=2, _) => &RUN_PARTY_LINGER,
            (3 | 4, _) => &RUN_PARTY_SWORD,
            _ => &RUN_PARTY_FULL,
        }
    }

    pub fn party_summary(&self) -> String {
        let mut names = vec!["李逍遥"];
        for companion in self.party_companions() {
            names.push(match companion {
                Companion::Linger => "赵灵儿",
                Companion::SwordSister => "林月衡",
                Companion::SpiritWitch => "南瑶",
            });
        }
        names.join("、")
    }

    pub fn journey_marker_plan(&self) -> &'static [NodeKind] {
        let chapter = self.chapter.min(JOURNEY_MARKER_PLANS.len() - 1);
        let plans = JOURNEY_MARKER_PLANS[chapter];
        plans[self.stage.min(plans.len() - 1)]
    }

    pub fn journey_intro(&self) -> &'static JourneyIntro {
        let chapter = self.chapter.min(JOURNEY_INTROS.len() - 1);
        let intros = JOURNEY_INTROS[chapter];
        &intros[self.stage.min(intros.len() - 1)]
    }

    pub fn journey_map(&self) -> MapKind {
        let chapter = self.chapter.min(JOURNEY_MAPS.len() - 1);
        let maps = JOURNEY_MAPS[chapter];
        maps[self.stage.min(maps.len() - 1)]
    }

    pub fn has_relic(&self, relic: Relic) -> bool {
        self.relics.contains(&relic)
    }

    /// Pick the authored route map for this stage. Its topology blueprint and
    /// terrain profile both follow the journey beat.
    pub fn roll_map(&mut self) -> MapKind {
        let pick = self.journey_map();
        self.last_map = Some(pick);
        pick
    }

    /// Pick a random encounter zone for this chapter.
    pub fn roll_zone(&self, rng: &mut Rng) -> EncounterZone {
        let zones = self.chapter_def().zones;
        zones[rng.range(0, zones.len() as i32 - 1) as usize]
    }

    /// 「遇」节点的抽取入口:链式奇遇按进度优先触发,否则落回随机池。
    /// 链事件跨章推进——初遇的选择决定后续与结局回响。
    pub fn draw_chain_or_event(&mut self, rng: &mut Rng) -> usize {
        use content as c;
        if self.fox_stage == 0 && rng.chance(0.6) {
            self.fox_stage = 1;
            return c::EV_FOX1;
        }
        if self.fox_stage == 1 && self.chapter >= 2 && rng.chance(0.6) {
            self.fox_stage = 2;
            return if self.fox_kind > 0 {
                c::EV_FOX2_WARM
            } else {
                c::EV_FOX2_COLD
            };
        }
        if self.fox_stage == 2 && self.chapter >= 5 && rng.chance(0.7) {
            self.fox_stage = 3;
            return if self.fox_kind > 0 {
                c::EV_FOX3_WARM
            } else {
                c::EV_FOX3_COLD
            };
        }
        if self.qin_stage == 0 && self.chapter >= 2 && rng.chance(0.5) {
            self.qin_stage = 1;
            return c::EV_QIN1;
        }
        if self.qin_stage == 1 && self.chapter >= 6 && rng.chance(0.6) {
            self.qin_stage = 2;
            return c::EV_QIN2;
        }
        self.draw_event(rng)
    }

    /// Draw a random event index the player has not seen this run; the pool
    /// resets once exhausted (chain events live past `CHAIN_START` and never
    /// enter this pool).
    pub fn draw_event(&mut self, rng: &mut Rng) -> usize {
        let unseen: Vec<usize> = (0..content::CHAIN_START)
            .filter(|i| !self.seen_events.contains(i))
            .collect();
        let pool = if unseen.is_empty() {
            self.seen_events.clear();
            (0..content::CHAIN_START).collect()
        } else {
            unseen
        };
        let index = pool[rng.range(0, pool.len() as i32 - 1) as usize];
        self.seen_events.push(index);
        index
    }

    /// Draw an unseen 「缘」 scene from the current chapter. The pool only
    /// resets after every authored scene in that chapter has been viewed.
    pub fn draw_story(&mut self, rng: &mut Rng) -> usize {
        let chapter = self.chapter.min(CHAPTER_COUNT - 1);
        let count = content::story_count(chapter);
        let unseen: Vec<usize> = (0..count)
            .filter(|index| !self.seen_story_scenes.contains(&(chapter, *index)))
            .collect();
        let pool = if unseen.is_empty() {
            self.seen_story_scenes
                .retain(|(seen_chapter, _)| *seen_chapter != chapter);
            (0..count).collect()
        } else {
            unseen
        };
        let index = pool[rng.range(0, pool.len() as i32 - 1) as usize];
        self.seen_story_scenes.push((chapter, index));
        index
    }

    pub fn current_story_memory_summary(&self) -> String {
        let chapter = self.chapter.min(CHAPTER_COUNT - 1);
        let seen = self
            .seen_story_scenes
            .iter()
            .filter(|(seen_chapter, _)| *seen_chapter == chapter)
            .count();
        format!("{seen}/{}", content::story_count(chapter))
    }

    // --- 妖纹(海克斯)聚合查询:战斗与地图钩子从这里读效果 ---

    pub fn has_hex(&self, h: hex::HexMark) -> bool {
        self.hexes.contains(&h)
    }

    /// 攻击加成合计(磐心-2 / 燃魂+3 / 星孤:法宝<3 时+6 / 血怒:半血下+5)。
    pub fn hex_atk_delta(&self, hp: i32, max_hp: i32) -> i32 {
        use hex::HexMark as H;
        let mut d = 0;
        if self.has_hex(H::StoneHeart) {
            d -= 2;
        }
        if self.has_hex(H::SoulBurn) {
            d += 3;
        }
        if self.has_hex(H::LoneStar) && self.relics.len() < 3 {
            d += 6;
        }
        if self.has_hex(H::BloodRage) && hp * 2 < max_hp {
            d += 5;
        }
        d
    }

    /// 仙术伤害加成(蚀月+6 / 燃魂+3)。
    pub fn hex_spell_bonus(&self) -> i32 {
        use hex::HexMark as H;
        let mut d = 0;
        if self.has_hex(H::MoonBite) {
            d += 6;
        }
        if self.has_hex(H::SoulBurn) {
            d += 3;
        }
        d
    }

    /// 仙术费用增量(蚀月+2)。
    pub fn hex_spell_cost_delta(&self) -> i32 {
        if self.has_hex(hex::HexMark::MoonBite) {
            2
        } else {
            0
        }
    }

    /// 绝技所需气势(妖契 2,否则默认 3)。
    pub fn hex_burst_cost(&self) -> u32 {
        if self.has_hex(hex::HexMark::DemonPact) {
            2
        } else {
            3
        }
    }

    /// 绝技后的自伤(妖契 4)。
    pub fn hex_burst_self_hurt(&self) -> i32 {
        if self.has_hex(hex::HexMark::DemonPact) {
            4
        } else {
            0
        }
    }

    /// 御守追加减免(磐心 8)。
    pub fn hex_guard_block(&self) -> i32 {
        if self.has_hex(hex::HexMark::StoneHeart) {
            8
        } else {
            0
        }
    }

    /// 御守回灵增减(疾风 -2)。
    pub fn hex_guard_mp_delta(&self) -> i32 {
        if self.has_hex(hex::HexMark::GaleStep) {
            -2
        } else {
            0
        }
    }

    /// 御守是否叠气势(疾风)。
    pub fn hex_guard_momentum(&self) -> bool {
        self.has_hex(hex::HexMark::GaleStep)
    }

    /// 每战首次攻击的伤害倍率(雷引 1.5)。
    pub fn hex_first_strike_mul(&self) -> f32 {
        if self.has_hex(hex::HexMark::ThunderBrand) {
            1.5
        } else {
            1.0
        }
    }

    /// 敌人本战首击附加伤害(雷引 3)。
    pub fn hex_enemy_first_hit_bonus(&self) -> i32 {
        if self.has_hex(hex::HexMark::ThunderBrand) {
            3
        } else {
            0
        }
    }

    /// 击杀回血(血偿 6)。
    pub fn hex_kill_heal(&self) -> i32 {
        if self.has_hex(hex::HexMark::BloodTithe) {
            6
        } else {
            0
        }
    }

    /// 开战失灵(血偿 2)。
    pub fn hex_battle_start_mp_loss(&self) -> i32 {
        if self.has_hex(hex::HexMark::BloodTithe) {
            2
        } else {
            0
        }
    }

    /// 敌人回合后的回血(韧藤 2)。
    pub fn hex_enemy_turn_regen(&self) -> i32 {
        if self.has_hex(hex::HexMark::Vinegrip) {
            2
        } else {
            0
        }
    }

    /// 药水回复增减(韧藤 -10)。
    pub fn hex_potion_delta(&self) -> i32 {
        if self.has_hex(hex::HexMark::Vinegrip) {
            -10
        } else {
            0
        }
    }

    /// 战斗结束的失血(燃魂 4)。
    pub fn hex_battle_end_hp_loss(&self) -> i32 {
        if self.has_hex(hex::HexMark::SoulBurn) {
            4
        } else {
            0
        }
    }

    /// 每胜一场的额外钱财(贪泉 18)。
    pub fn hex_win_gold(&self) -> u32 {
        if self.has_hex(hex::HexMark::GreedSpring) {
            18
        } else {
            0
        }
    }

    /// 战利钱财倍率(雾行 ×0.7)。
    pub fn hex_gold_mul(&self) -> f32 {
        if self.has_hex(hex::HexMark::MistWalk) {
            0.7
        } else {
            1.0
        }
    }

    /// 每程入图的失血(贪泉 3)。
    pub fn hex_stage_hp_loss(&self) -> i32 {
        if self.has_hex(hex::HexMark::GreedSpring) {
            3
        } else {
            0
        }
    }

    /// 草丛遇敌是否减半(雾行)。
    pub fn hex_grass_halved(&self) -> bool {
        self.has_hex(hex::HexMark::MistWalk)
    }

    /// 法宝掉率是否减半(星孤)。
    pub fn hex_relic_drop_halved(&self) -> bool {
        self.has_hex(hex::HexMark::LoneStar)
    }

    /// 瘴气毒格免疫(断因)。
    pub fn hex_hazard_immune(&self) -> bool {
        self.has_hex(hex::HexMark::SeveredFate)
    }

    /// 灵泉治疗是否减半(断因)。
    pub fn hex_spring_halved(&self) -> bool {
        self.has_hex(hex::HexMark::SeveredFate)
    }

    /// 行囊妖纹清单。
    pub fn hex_summary(&self) -> String {
        if self.hexes.is_empty() {
            return "(尚未染上妖纹)".to_string();
        }
        self.hexes
            .iter()
            .map(|h| format!("【{}】{}", h.name(), h.desc()))
            .collect::<Vec<_>>()
            .join("\n")
    }

    // --- 道心/情缘里程碑:剧情计数直接反哺战斗 ---

    /// 情缘≥4:灵儿灵息缠身,敌人每回合行动后为你回血。
    pub fn bond_regen(&self) -> i32 {
        if self.qingyuan >= 4 { 2 } else { 0 }
    }

    /// 情缘≥7:开战灵儿先布灵息护罩(开战回血)。
    pub fn bond_shield(&self) -> i32 {
        if self.qingyuan >= 7 { 8 } else { 0 }
    }

    /// 道心≥4:绝技·剑气爆发威力 ×1.3。
    pub fn resolve_burst_mul(&self) -> f32 {
        if self.daoxin >= 4 { 1.3 } else { 1.0 }
    }

    /// 御守卸力后保留的伤害百分比:道心≥7 时 35%→20%。
    pub fn guard_keep_pct(&self) -> i32 {
        if self.daoxin >= 7 { 20 } else { 35 }
    }

    /// 道心≥7:御守回灵 4→6。
    pub fn guard_mp_restore(&self) -> i32 {
        if self.daoxin >= 7 { 6 } else { 4 }
    }

    /// 行囊里程碑一览(已激活的以「◆」标出)。
    pub fn milestone_summary(&self) -> String {
        let mark = |on: bool| if on { "◆" } else { "◇" };
        format!(
            "{} 情缘4 灵息缠身(敌回合后回血)  {} 情缘7 开战灵息罩
{} 道心4 绝技威力+30%  {} 道心7 御守精进(减伤85%·回灵+)",
            mark(self.qingyuan >= 4),
            mark(self.qingyuan >= 7),
            mark(self.daoxin >= 4),
            mark(self.daoxin >= 7),
        )
    }

    // --- relic-driven battle modifiers, summed over owned relics ---

    fn sum_relics(&self, f: impl Fn(Relic) -> i32) -> i32 {
        self.relics.iter().copied().map(f).sum()
    }

    pub fn attack_bonus(&self) -> i32 {
        self.sum_relics(|r| match r {
            Relic::SwordTassel => 5,
            Relic::ChixiaoCore => 4,
            _ => 0,
        })
    }

    pub fn spell_bonus(&self) -> i32 {
        self.sum_relics(|r| match r {
            Relic::SwordSutra => 9,
            Relic::StarSand => 5,
            _ => 0,
        })
    }

    pub fn spell_cost_delta(&self) -> i32 {
        self.sum_relics(|r| match r {
            Relic::SpiritPendant => -2,
            _ => 0,
        })
    }

    pub fn potion_bonus(&self) -> i32 {
        self.sum_relics(|r| match r {
            Relic::JadeVial => 25,
            _ => 0,
        })
    }

    pub fn incoming_reduction(&self) -> i32 {
        self.sum_relics(|r| match r {
            Relic::TortoiseArmor => 3,
            Relic::VajraPestle => 2,
            _ => 0,
        })
    }

    /// Extra reduction that only applies to an enemy's凶猛强击.
    pub fn strong_hit_guard(&self) -> i32 {
        self.sum_relics(|r| match r {
            Relic::TigerTalisman => 5,
            _ => 0,
        })
    }

    pub fn flee_always(&self) -> bool {
        self.has_relic(Relic::CloudSleeves)
    }

    pub fn on_kill_heal(&self) -> i32 {
        self.sum_relics(|r| match r {
            Relic::BloodBead => 12,
            _ => 0,
        })
    }

    pub fn on_kill_mana(&self) -> i32 {
        self.sum_relics(|r| match r {
            Relic::SoulLantern => 4,
            _ => 0,
        })
    }

    pub fn opening_strike(&self) -> i32 {
        self.sum_relics(|r| match r {
            Relic::ThunderDrum => 15,
            _ => 0,
        })
    }

    /// Heal applied to the player at the start of every battle.
    pub fn battle_start_heal(&self) -> i32 {
        self.sum_relics(|r| match r {
            Relic::BreathSoil => 10,
            _ => 0,
        })
    }

    /// Flat debuff applied to the enemy's attack at spawn.
    pub fn enemy_atk_debuff(&self) -> i32 {
        self.sum_relics(|r| match r {
            Relic::YinYangMirror => 3,
            _ => 0,
        })
    }

    pub fn gold_multiplier(&self) -> f32 {
        if self.has_relic(Relic::PixiuPouch) {
            1.5
        } else {
            1.0
        }
    }

    /// Extra damage multiplier vs elites and bosses.
    pub fn hunter_multiplier(&self, rank: FightRank) -> f32 {
        if rank != FightRank::Normal && self.has_relic(Relic::KunlunMirror) {
            1.25
        } else {
            1.0
        }
    }

    pub fn rest_multiplier(&self) -> i32 {
        if self.has_relic(Relic::DrunkenBrew) {
            2
        } else {
            1
        }
    }

    /// 檀木符 revive, if available. Returns true and consumes it.
    pub fn try_revive(&mut self) -> bool {
        if self.has_relic(Relic::SandalCharm) && !self.revive_used {
            self.revive_used = true;
            true
        } else {
            false
        }
    }
}

/// Stat multipliers for the battle being entered (normal/elite scaling).
/// Inserted next to `PendingEncounter` by the node map; read by
/// `spawn_battle`.
#[derive(Resource, Clone, Copy)]
pub struct RunBattleMods {
    pub hp_mul: f32,
    pub atk_mul: f32,
    pub rank: FightRank,
}

/// Build the encounter parameters for a node fight and remember its rank.
pub fn battle_mods_for(run: &RunState, rank: FightRank) -> RunBattleMods {
    let def = run.chapter_def();
    match rank {
        // Boss defs were tuned for the levelled legacy campaign; the run mode
        // has no levels, so bosses fight at a flat discount instead.
        FightRank::Boss => RunBattleMods {
            hp_mul: 0.72,
            atk_mul: 0.68,
            rank,
        },
        FightRank::Elite => RunBattleMods {
            hp_mul: def.enemy_hp_mul * 1.40,
            atk_mul: def.enemy_atk_mul * 1.15,
            rank,
        },
        FightRank::Normal => RunBattleMods {
            hp_mul: def.enemy_hp_mul,
            atk_mul: def.enemy_atk_mul,
            rank,
        },
    }
}

/// Encounter kind for a node.
pub fn encounter_kind_for(run: &RunState, kind: NodeKind) -> EncounterKind {
    match kind {
        NodeKind::Boss => EncounterKind::Boss(run.boss),
        _ => EncounterKind::Random,
    }
}

fn journey_marker_weight(kind: NodeKind) -> u32 {
    match kind {
        NodeKind::Fight => 9,
        NodeKind::Elite => 12,
        NodeKind::Event => 9,
        NodeKind::Story => 10,
        NodeKind::Rest => 7,
        NodeKind::Market => 7,
        NodeKind::Puzzle => 11,
        NodeKind::Boss => 18,
        NodeKind::Chest | NodeKind::Spring | NodeKind::Guide => 4,
    }
}

fn journey_marker_plan_weight(plan: &[NodeKind]) -> u32 {
    let required: u32 = plan.iter().copied().map(journey_marker_weight).sum();
    let optional_overhead = if plan.contains(&NodeKind::Boss) { 4 } else { 8 };
    required + optional_overhead
}

fn summarize_journey_marker_plan(plan: &[NodeKind]) -> String {
    let order = [
        NodeKind::Story,
        NodeKind::Fight,
        NodeKind::Elite,
        NodeKind::Event,
        NodeKind::Rest,
        NodeKind::Market,
        NodeKind::Puzzle,
        NodeKind::Boss,
    ];
    let mut parts = Vec::new();
    for kind in order {
        let count = plan.iter().filter(|node| **node == kind).count();
        if count > 0 {
            parts.push(format!("{}x{}", kind.label(), count));
        }
    }
    if parts.is_empty() {
        "自由探索".to_string()
    } else {
        parts.join(" · ")
    }
}

fn format_play_time(seconds: f32) -> String {
    let total = seconds.max(0.0).round() as u64;
    let hours = total / 3600;
    let minutes = (total % 3600) / 60;
    let seconds = total % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

fn track_run_play_time(
    time: Res<Time>,
    state: Res<State<AppState>>,
    run: Option<ResMut<RunState>>,
) {
    let Some(mut run) = run else { return };
    if matches!(
        state.get(),
        AppState::NodeMap | AppState::RunScene | AppState::Battle | AppState::Reward
    ) {
        run.record_play_time(time.delta_secs().min(0.25));
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct RoguelikePlugin;

impl Plugin for RoguelikePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<event::RunDialogue>()
            .init_resource::<screens::RewardChoices>()
            .init_resource::<scene::InventoryOpen>()
            .add_systems(Update, track_run_play_time)
            .add_systems(OnEnter(AppState::Title), screens::spawn_title)
            .add_systems(
                Update,
                screens::title_input.run_if(in_state(AppState::Title)),
            )
            // `NodeMap` is a zero-frame hop: it rolls the next stage's map
            // and markers into `RunSceneState`, then enters the scene.
            .add_systems(OnEnter(AppState::NodeMap), scene::advance_stage)
            .add_systems(
                OnEnter(AppState::RunScene),
                (scene::zoom_camera_in, scene::spawn_run_scene).chain(),
            )
            .add_systems(OnExit(AppState::RunScene), scene::zoom_camera_out)
            .add_systems(
                Update,
                (
                    event::run_dialogue_input,
                    scene::clear_chapter_art,
                    scene::open_journey_intro,
                    scene::inventory_toggle,
                    scene::run_scene_movement,
                    scene::update_run_fog,
                    scene::animate_chapter_art,
                    scene::animate_hero,
                    scene::sync_run_party_followers,
                    scene::camera_follow,
                    scene::animate_marker_glyph,
                    scene::animate_scene_float_text,
                    scene::update_run_hud,
                    event::update_run_dialogue_ui,
                )
                    .chain()
                    .run_if(in_state(AppState::RunScene)),
            )
            .add_systems(OnEnter(AppState::Reward), screens::spawn_reward)
            .add_systems(
                Update,
                screens::reward_input.run_if(in_state(AppState::Reward)),
            )
            .add_systems(OnEnter(AppState::Ending), screens::spawn_ending)
            .add_systems(
                Update,
                screens::ending_input.run_if(in_state(AppState::Ending)),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::super::core::Rng;
    use super::*;

    #[test]
    fn journey_beats_match_chapter_depths() {
        assert_eq!(JOURNEY_BEATS.len(), CHAPTER_COUNT);
        assert_eq!(JOURNEY_MARKER_PLANS.len(), CHAPTER_COUNT);
        assert_eq!(JOURNEY_INTROS.len(), CHAPTER_COUNT);
        assert_eq!(JOURNEY_MAPS.len(), CHAPTER_COUNT);
        assert_eq!(CHAPTER_PACING.len(), CHAPTER_COUNT);
        assert_eq!(CHAPTER_CLEAR_REWARDS.len(), CHAPTER_COUNT);
        assert_eq!(content::CHAPTER_CARDS.len(), CHAPTER_COUNT);
        assert_eq!(content::STORY_SCENES.len(), CHAPTER_COUNT);
        assert_eq!(content::STORY_EXTRA.len(), CHAPTER_COUNT);
        assert_eq!(CHAPTER_COUNT, 7);
        assert_eq!(
            CHAPTER_PACING
                .iter()
                .map(|pacing| pacing.minutes)
                .sum::<u32>(),
            TARGET_RUNTIME_MINUTES
        );
        assert_eq!(TARGET_RUNTIME_MINUTES, 600);
        assert_eq!(CHAPTERS[0].bosses, &[BossKind::MountainFiend]);
        assert_eq!(CHAPTERS[6].bosses, &[BossKind::DreamEclipse]);
        assert_eq!(CHAPTER_CLEAR_REWARDS[0].seal, "余杭赤火印");
        assert_eq!(CHAPTER_CLEAR_REWARDS[6].seal, "心渊照影印");
        assert_eq!(CHAPTER_CLEAR_REWARDS[6].hp, 0);
        let total_stages: usize = CHAPTERS.iter().map(|chapter| chapter.depth).sum();
        assert_eq!(total_stages, JOURNEY_ROUTE_STAGES);
        let mut unique_maps = Vec::new();
        for (chapter, beats) in JOURNEY_BEATS.iter().enumerate() {
            assert_eq!(
                beats.len(),
                CHAPTERS[chapter].depth,
                "{} should have one journey beat per map stage",
                CHAPTERS[chapter].title
            );
            let intros = JOURNEY_INTROS[chapter];
            assert_eq!(
                intros.len(),
                CHAPTERS[chapter].depth,
                "{} should have one authored intro per map stage",
                CHAPTERS[chapter].title
            );
            let route_maps = JOURNEY_MAPS[chapter];
            assert_eq!(
                route_maps.len(),
                CHAPTERS[chapter].depth,
                "{} should have one authored map kind per route beat",
                CHAPTERS[chapter].title
            );
            for (stage, map) in route_maps.iter().copied().enumerate() {
                assert!(
                    CHAPTERS[chapter].maps.contains(&map),
                    "{} stage {} uses a map outside the chapter pool: {:?}",
                    CHAPTERS[chapter].title,
                    stage + 1,
                    map
                );
                if stage > 0 {
                    assert_ne!(
                        route_maps[stage - 1],
                        map,
                        "{} stages {} and {} should not reuse the same map back-to-back",
                        CHAPTERS[chapter].title,
                        stage,
                        stage + 1
                    );
                }
                if !unique_maps.contains(&map) {
                    unique_maps.push(map);
                }
            }
            let plans = JOURNEY_MARKER_PLANS[chapter];
            assert_eq!(
                plans.len(),
                CHAPTERS[chapter].depth,
                "{} should have one marker plan per map stage",
                CHAPTERS[chapter].title
            );
            for (stage, plan) in plans.iter().enumerate() {
                if stage + 1 == CHAPTERS[chapter].depth {
                    assert_eq!(*plan, &[NodeKind::Boss]);
                } else {
                    assert!(
                        plan.len() >= 3,
                        "{} stage {} needs enough required content",
                        CHAPTERS[chapter].title,
                        stage + 1
                    );
                    assert!(!plan.contains(&NodeKind::Boss));
                    assert!(plan.iter().all(|kind| !kind.optional()));
                }
                assert!(
                    journey_marker_plan_weight(plan) > 0,
                    "{} stage {} needs pacing weight",
                    CHAPTERS[chapter].title,
                    stage + 1
                );
            }
            assert!(
                CHAPTER_PACING[chapter].minutes >= 60,
                "{} needs enough runtime budget for a chapter arc",
                CHAPTERS[chapter].title
            );
            assert!(!CHAPTER_PACING[chapter].focus.is_empty());
            assert!(
                beats.last().is_some_and(|beat| beat.title.contains("门")),
                "{} should end at a boss gate beat",
                CHAPTERS[chapter].title
            );
        }
        assert_eq!(unique_maps.len(), 15);
    }

    #[test]
    fn story_content_capacity_covers_required_story_nodes() {
        for chapter in 0..CHAPTER_COUNT {
            let required = JOURNEY_MARKER_PLANS[chapter]
                .iter()
                .flat_map(|plan| plan.iter().copied())
                .filter(|kind| *kind == NodeKind::Story)
                .count();
            let available = content::story_count(chapter);
            assert!(
                required > 0,
                "{} needs a planned story node instead of a random fallback",
                CHAPTERS[chapter].title
            );
            assert!(
                available >= required,
                "{} requires {required} story nodes but only has {available} authored scenes",
                CHAPTERS[chapter].title
            );
        }
    }

    #[test]
    fn story_content_draws_do_not_repeat_before_chapter_pool_exhausts() {
        for chapter in 0..CHAPTER_COUNT {
            let mut rng = Rng(0x5707_1E5E_ED00_0001 ^ chapter as u64);
            let mut run = RunState::new(&mut rng);
            run.chapter = chapter;
            let count = content::story_count(chapter);
            let mut drawn = Vec::new();
            for _ in 0..count {
                let index = run.draw_story(&mut rng);
                assert!(index < count);
                assert!(
                    !drawn.contains(&index),
                    "{} repeated story scene {index} before exhausting its pool",
                    CHAPTERS[chapter].title
                );
                drawn.push(index);
            }
            assert_eq!(
                run.current_story_memory_summary(),
                format!("{count}/{count}")
            );

            let repeated = run.draw_story(&mut rng);
            assert!(repeated < count);
            assert_eq!(
                run.current_story_memory_summary(),
                format!("1/{count}"),
                "{} should reset only after exhausting its pool",
                CHAPTERS[chapter].title
            );
        }
    }

    #[test]
    fn story_content_all_scenes_have_choices_and_resolved_outcomes() {
        for chapter in 0..CHAPTER_COUNT {
            for index in 0..content::story_count(chapter) {
                let scene = content::pick_story(chapter, index);
                assert!(
                    scene.lines.len() >= 3,
                    "{} story {index} needs a complete setup",
                    CHAPTERS[chapter].title
                );
                assert!(!scene.prompt.is_empty());
                assert!(scene.options.len() >= 2);
                for option in scene.options {
                    assert!(!option.label.is_empty());
                    assert!(
                        option.outcome.lines.len() >= 2,
                        "{} story {index} has an unresolved option",
                        CHAPTERS[chapter].title
                    );
                }
            }
        }
    }

    #[test]
    fn story_content_portraits_match_scene_speakers() {
        assert_eq!(
            content::story_portrait(5, 2),
            content::PORTRAIT_SPIRIT_WITCH
        );
        assert_eq!(
            content::story_portrait(6, 5),
            content::PORTRAIT_SWORD_SISTER
        );
        assert_eq!(content::story_portrait(6, 6), content::PORTRAIT_LINGER);
    }

    #[test]
    fn journey_summary_tracks_current_stage() {
        let mut rng = Rng::default();
        let mut run = RunState::new(&mut rng);

        assert_eq!(run.journey_beat().title, "客栈夜雨");
        assert_eq!(run.journey_map(), MapKind::Village);
        assert_eq!(run.journey_task_receipt(), "主线签 卷1-01");
        assert_eq!(run.journey_task_state_label(), "待签收");
        assert!(!run.is_journey_task_accepted());
        assert!(!run.is_journey_task_completed());
        assert_eq!(run.route_commission_receipt(), "路人签 卷1-01");
        assert_eq!(run.route_commission_state_label(), "未接取");
        assert!(!run.is_route_commission_active());
        assert!(!run.is_route_commission_completed());
        assert_eq!(run.journey_task_archive_label(), "主线归档 0/41");
        assert_eq!(run.chapter_seal_summary(), "未得");
        assert_eq!(run.camp_summary(), "未备");
        assert_eq!(
            run.available_camp_tactics(),
            vec![RunCampTactic::Breath, RunCampTactic::SwordGuard]
        );
        assert_eq!(run.party_companions(), &[]);
        assert_eq!(run.party_summary(), "李逍遥");
        assert!(run.journey_intro().scene_line.contains("雨丝"));
        assert_eq!(
            run.journey_marker_plan(),
            &[NodeKind::Story, NodeKind::Market, NodeKind::Fight]
        );
        assert_eq!(
            run.journey_required_content_summary(),
            "剧情x1 · 遭遇战x1 · 集市x1"
        );
        assert!(run.journey_stage_minutes() > 0);
        assert!(run.journey_stage_pacing_summary().contains("本程约"));
        assert!(run.chapter_pacing_summary().contains("本卷约65分"));
        assert!(run.chapter_pacing_summary().contains("全旅程目标 10小时"));
        assert!(
            run.journey_summary()
                .contains("任务：主线签 卷1-01 [待签收]")
        );
        assert!(run.journey_summary().contains("节奏：本程约"));
        assert!(run.journey_summary().contains("必做：剧情x1"));
        assert!(run.journey_summary().contains("余杭客栈后院"));
        assert!(run.journey_summary().contains("章印：0/7"));
        assert!(run.journey_summary().contains("实测用时：00:00:00"));
        assert!(run.journey_summary().contains("营策：未备"));
        assert!(run.journey_summary().contains("誓记：未立"));
        assert!(run.journey_summary().contains("路人签：未接取"));
        assert!(run.journey_summary().contains("路况：未稳 本卷路人签 0"));
        assert!(run.accept_route_commission(NodeKind::Story));
        assert!(!run.accept_route_commission(NodeKind::Story));
        assert!(run.is_route_commission_active());
        assert!(run.route_commission_state_label().contains("剧情"));
        assert_eq!(
            run.route_commission_hud_label(),
            Some("路人签 卷1-01 -> 剧情".to_string())
        );
        assert!(!run.complete_route_commission(NodeKind::Rest));
        assert!(run.complete_route_commission(NodeKind::Story));
        assert!(!run.complete_route_commission(NodeKind::Story));
        assert!(run.is_route_commission_completed());
        assert_eq!(run.route_contacts_helped, 1);
        assert_eq!(run.route_commission_state_label(), "已回执");
        assert_eq!(run.chapter_route_contacts(), 1);
        assert!(run.route_guidance_summary().contains("草地遇敌-8%"));
        assert!(run.record_camp_scene());
        assert!(!run.record_camp_scene());
        run.set_camp_tactic(RunCampTactic::SwordGuard);
        assert_eq!(run.camp_summary(), "剑守");
        assert_eq!(run.take_camp_tactic(), Some(RunCampTactic::SwordGuard));
        assert_eq!(run.camp_summary(), "未备");
        assert!(run.accept_journey_task());
        assert!(!run.accept_journey_task());
        assert_eq!(run.journey_task_state_label(), "已追踪");
        assert!(
            run.journey_summary()
                .contains("任务：主线签 卷1-01 [已追踪]")
        );
        assert!(run.complete_journey_task());
        assert!(!run.complete_journey_task());
        assert!(run.is_journey_task_completed());
        assert_eq!(run.journey_task_state_label(), "已归档");
        assert_eq!(run.journey_task_archive_label(), "主线归档 1/41");
        assert!(run.journey_summary().contains("主线归档 1/41"));
        assert!(run.record_chapter_clear());
        assert!(!run.record_chapter_clear());
        assert_eq!(run.chapter_clear_reward().seal, "余杭赤火印");
        assert_eq!(run.chapter_seal_summary(), "余杭赤火印");
        assert!(run.journey_summary().contains("章印：1/7"));

        run.stage = 3;
        assert_eq!(run.journey_beat().title, "洞天灵誓");
        assert_eq!(run.journey_map(), MapKind::Cave);
        assert_eq!(run.journey_task_receipt(), "主线签 卷1-04");
        assert_eq!(run.journey_task_state_label(), "待签收");
        assert_eq!(run.party_companions(), &[Companion::Linger]);
        assert!(
            run.available_camp_tactics()
                .contains(&RunCampTactic::LingerWard)
        );
        assert!(run.party_summary().contains("赵灵儿"));
        assert!(run.journey_intro().scene_line.contains("洞天水雾"));
        assert_eq!(
            run.journey_marker_plan(),
            &[NodeKind::Story, NodeKind::Rest, NodeKind::Elite]
        );
        assert!(run.journey_stage_pacing_summary().contains("精英强袭x1"));
        assert!(run.journey_summary().contains("守住洞天灵阵"));

        run.chapter = CHAPTER_COUNT - 1;
        run.stage = run.stage_count() - 1;
        assert_eq!(run.journey_beat().title, "心渊终门");
        assert_eq!(run.journey_map(), MapKind::FinalSanctum);
        assert_eq!(run.journey_task_receipt(), "主线签 卷7-11");
        assert_eq!(
            run.party_companions(),
            &[
                Companion::Linger,
                Companion::SwordSister,
                Companion::SpiritWitch
            ]
        );
        assert!(run.party_summary().contains("南瑶"));
        assert!(
            run.available_camp_tactics()
                .contains(&RunCampTactic::SpiritFocus)
        );
        assert_eq!(run.journey_marker_plan(), &[NodeKind::Boss]);
        assert_eq!(run.journey_required_content_summary(), "章末首领x1");
        assert!(run.chapter_pacing_summary().contains("本卷约135分"));
        assert!(run.journey_intro().party_line.contains("结局"));
        assert!(run.journey_beat().objective.contains("结局"));
    }

    #[test]
    fn chapter_clear_archives_unclaimed_boss_task_once() {
        let mut rng = Rng::default();
        let mut run = RunState::new(&mut rng);
        run.stage = run.stage_count() - 1;

        assert!(!run.is_journey_task_accepted());
        assert!(!run.is_journey_task_completed());
        assert!(run.record_chapter_clear());
        assert!(run.is_journey_task_accepted());
        assert!(run.is_journey_task_completed());
        assert_eq!(run.accepted_journey_tasks.len(), 1);
        assert_eq!(run.completed_journey_tasks.len(), 1);
        assert!(!run.record_chapter_clear());
        assert_eq!(run.accepted_journey_tasks.len(), 1);
        assert_eq!(run.completed_journey_tasks.len(), 1);
    }

    #[test]
    fn run_play_time_tracks_total_and_chapter_ledgers() {
        let mut rng = Rng::default();
        let mut run = RunState::new(&mut rng);
        run.record_play_time(65.0);
        run.record_play_time(f32::NAN);
        run.record_play_time(-10.0);
        run.chapter = 1;
        run.record_play_time(3600.0);

        assert_eq!(run.play_time_summary(), "01:01:05");
        assert_eq!(run.chapter_play_time_summary(0), "00:01:05");
        assert_eq!(run.chapter_play_time_summary(1), "01:00:00");
        let archive = run.chapter_play_time_archive_summary();
        assert!(archive.contains("卷1 00:01:05"));
        assert!(archive.contains("卷2 01:00:00"));
        assert!(archive.contains('\n'));
    }

    #[test]
    fn chapter_vows_record_story_choices_and_archive_counts() {
        let mut rng = Rng::default();
        let mut run = RunState::new(&mut rng);

        assert_eq!(run.chapter_vow_summary(), "未立");
        assert!(run.record_chapter_vow(0, RunChapterVow::Heart));
        assert!(!run.record_chapter_vow(0, RunChapterVow::Resolve));
        assert_eq!(run.current_chapter_vow(), Some(RunChapterVow::Heart));
        assert_eq!(run.chapter_vow_summary(), "护心誓");
        assert_eq!(RunChapterVow::Heart.boss_atk_multiplier(), 0.90);
        assert_eq!(RunChapterVow::Resolve.boss_hp_multiplier(), 0.92);

        run.next_chapter(&mut rng);
        assert_eq!(run.chapter_vow_summary(), "未立");
        assert!(run.record_chapter_vow(1, RunChapterVow::Resolve));
        assert_eq!(run.chapter_vow_archive_summary(), "誓记：护心1 · 破势1");
        assert_eq!(RunChapterVow::from_scores(2, 2), RunChapterVow::Twin);
    }

    #[test]
    fn route_commission_receipts_reduce_current_chapter_grass_pressure() {
        let mut rng = Rng::default();
        let mut run = RunState::new(&mut rng);

        let base = 0.08;
        assert_eq!(run.chapter_route_contacts(), 0);
        assert_eq!(run.route_guidance_label(), "未稳");
        assert_eq!(run.route_guidance_hud_label(), None);
        assert!((run.grass_encounter_chance(base) - base).abs() < 0.0001);

        assert!(run.accept_route_commission(NodeKind::Story));
        assert!(run.complete_route_commission(NodeKind::Story));
        assert_eq!(run.chapter_route_contacts(), 1);
        assert_eq!(run.route_guidance_label(), "风声初稳");
        assert!(
            run.route_guidance_hud_label()
                .is_some_and(|line| line.contains("草地遇敌-8%"))
        );
        assert!((run.grass_encounter_chance(base) - 0.0736).abs() < 0.0001);

        run.stage = 1;
        assert!(run.accept_route_commission(NodeKind::Event));
        assert!(run.complete_route_commission(NodeKind::Event));
        assert_eq!(run.chapter_route_contacts(), 2);
        assert_eq!(run.route_guidance_label(), "乡路半稳");
        assert!((run.grass_encounter_chance(base) - 0.0672).abs() < 0.0001);

        run.stage = 2;
        assert!(run.accept_route_commission(NodeKind::Rest));
        assert!(run.complete_route_commission(NodeKind::Rest));
        assert_eq!(run.chapter_route_contacts(), 3);
        assert_eq!(run.route_guidance_label(), "熟路照应");
        assert!((run.grass_encounter_chance(base) - 0.0608).abs() < 0.0001);

        run.next_chapter(&mut rng);
        assert_eq!(run.chapter_route_contacts(), 0);
        assert_eq!(run.route_guidance_label(), "未稳");
        assert!((run.grass_encounter_chance(base) - base).abs() < 0.0001);
    }

    #[test]
    fn route_commission_receipts_prepare_chapter_boss_opening() {
        let mut rng = Rng::default();
        let mut run = RunState::new(&mut rng);

        assert_eq!(run.route_boss_preparation_rank(), 0);
        assert_eq!(run.route_boss_preparation_label(), "未备");
        assert_eq!(run.route_boss_hp_multiplier(), 1.0);
        assert_eq!(run.route_boss_atk_multiplier(), 1.0);
        assert_eq!(run.route_boss_restore(), (0, 0));
        assert!(
            run.route_boss_preparation_summary()
                .contains("本卷路人签 0/3")
        );

        assert!(run.accept_route_commission(NodeKind::Story));
        assert!(run.complete_route_commission(NodeKind::Story));
        assert_eq!(run.route_boss_preparation_rank(), 0);

        run.stage = 1;
        assert!(run.accept_route_commission(NodeKind::Event));
        assert!(run.complete_route_commission(NodeKind::Event));
        assert_eq!(run.route_boss_preparation_rank(), 1);
        assert_eq!(run.route_boss_preparation_label(), "乡路照应");
        assert_eq!(run.route_boss_hp_multiplier(), 1.0);
        assert_eq!(run.route_boss_atk_multiplier(), 0.96);
        assert_eq!(run.route_boss_restore(), (8, 4));
        assert!(run.journey_summary().contains("首领照应 乡路照应"));

        run.stage = 2;
        assert!(run.accept_route_commission(NodeKind::Rest));
        assert!(run.complete_route_commission(NodeKind::Rest));
        assert_eq!(run.route_boss_preparation_rank(), 2);
        assert_eq!(run.route_boss_preparation_label(), "熟路照应");
        assert_eq!(run.route_boss_hp_multiplier(), 0.96);
        assert_eq!(run.route_boss_atk_multiplier(), 0.92);
        assert_eq!(run.route_boss_restore(), (12, 8));
        assert!(run.route_boss_preparation_summary().contains("首领气血-4%"));

        run.next_chapter(&mut rng);
        assert_eq!(run.route_boss_preparation_rank(), 0);
        assert_eq!(run.route_boss_preparation_label(), "未备");
    }

    /// 白狐链按章推进,善恶分歧走向不同事件;随机池永不吐出链下标。
    #[test]
    fn fox_chain_progresses_and_branches_by_kindness() {
        let mut rng = Rng::default();
        let mut run = RunState::new(&mut rng);

        // 第一卷:反复抽直到白狐初遇触发(60% 概率,重试足够多次)。
        let mut first = None;
        for _ in 0..64 {
            let idx = run.draw_chain_or_event(&mut rng);
            if idx >= content::CHAIN_START {
                first = Some(idx);
                break;
            }
        }
        assert_eq!(first, Some(content::EV_FOX1));
        assert_eq!(run.fox_stage, 1);

        // 救了白狐 → 善缘;第三卷江路应触发「白衣回礼」。
        run.fox_kind = 1;
        run.chapter = 2;
        let mut second = None;
        for _ in 0..64 {
            let idx = run.draw_chain_or_event(&mut rng);
            if idx >= content::CHAIN_START {
                second = Some(idx);
                break;
            }
        }
        assert_eq!(second, Some(content::EV_FOX2_WARM));

        // 第六卷善缘收束于「狐仙赠丹」。
        run.chapter = 5;
        let mut third = None;
        for _ in 0..64 {
            let idx = run.draw_chain_or_event(&mut rng);
            if idx >= content::CHAIN_START && idx != content::EV_QIN1 && idx != content::EV_QIN2 {
                third = Some(idx);
                break;
            }
        }
        assert_eq!(third, Some(content::EV_FOX3_WARM));
        assert_eq!(run.fox_stage, 3);

        // 链走完后,随机池不会再吐出链事件下标。
        for _ in 0..64 {
            let idx = run.draw_event(&mut rng);
            assert!(idx < content::CHAIN_START);
        }
    }

    /// 妖纹抽取不重复,效果聚合正确叠加。
    #[test]
    fn hexes_roll_unique_and_aggregate() {
        let mut rng = Rng::default();
        let mut run = RunState::new(&mut rng);
        for _ in 0..hex::ALL_HEXES.len() {
            let h = hex::roll_hex(&run.hexes, &mut rng).expect("pool not empty");
            assert!(!run.hexes.contains(&h), "no duplicates");
            run.hexes.push(h);
        }
        assert!(hex::roll_hex(&run.hexes, &mut rng).is_none());
        // 全部持有时的聚合:磐心-2 + 燃魂+3 + 星孤(法宝<3)+6 + 血怒(半血)+5。
        assert_eq!(run.hex_atk_delta(10, 100), 12);
        assert_eq!(run.hex_burst_cost(), 2);
        assert!(run.hex_grass_halved());
        assert!(run.hex_hazard_immune());
    }

    /// 无视白狐的一世,走向「白影避走」与「妖狐拦路」。
    #[test]
    fn fox_chain_cold_branch() {
        let mut rng = Rng::default();
        let mut run = RunState::new(&mut rng);
        run.fox_stage = 1;
        run.fox_kind = -1;
        run.chapter = 2;
        let mut second = None;
        for _ in 0..64 {
            let idx = run.draw_chain_or_event(&mut rng);
            if idx >= content::CHAIN_START && idx != content::EV_QIN1 && idx != content::EV_QIN2 {
                second = Some(idx);
                break;
            }
        }
        assert_eq!(second, Some(content::EV_FOX2_COLD));

        run.chapter = 5;
        let mut third = None;
        for _ in 0..64 {
            let idx = run.draw_chain_or_event(&mut rng);
            if idx >= content::CHAIN_START && idx != content::EV_QIN1 && idx != content::EV_QIN2 {
                third = Some(idx);
                break;
            }
        }
        assert_eq!(third, Some(content::EV_FOX3_COLD));
    }
}

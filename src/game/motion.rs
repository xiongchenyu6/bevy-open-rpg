//! 2D Character Motion Catalog (112 named actions), critically-damped procedural
//! pose engine (`Pose2d` / `MotionState2d`), and lightweight 2D combat impact
//! juice particles (razor hit-sparks, flash star, crit cross-glint, kill finisher
//! ring, tapered slash arc smears, and footstep/skid dust puffs).
//!
//! All VFX use pure `Sprite::from_color` primitives so they work identically on
//! native Vulkan/Wayland, WebGPU (`wasm32-unknown-unknown`), and CPU-only tests.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use super::battle::BattleFeel;
use super::state::AppState;

// ---------------------------------------------------------------------------
// Motion Catalog (112 distinct named 2D RPG character actions)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MotionCategory {
    Locomotion,
    ChargeStance,
    MeleeAttack,
    SpellCast,
    DefenseDodge,
    HitReaction,
    StatusAilment,
    ConsumableInteract,
    VictoryDeath,
    EnemyBoss,
    PartyEmote,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MotionMeta {
    pub id: MotionId,
    pub zh_name: &'static str,
    pub en_name: &'static str,
    pub category: MotionCategory,
    pub priority: u8,
    pub duration: f32,
    pub looping: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MotionId {
    // --- Locomotion (14) ---
    IdleCalm,
    IdleCombat,
    IdleExhausted,
    IdleInjured,
    WalkCasual,
    WalkCautious,
    RunStriding,
    SprintDash,
    SkidStop,
    TurnPivot,
    JumpAscend,
    LandCushion,
    StepForward,
    StepBack,

    // --- Charge, Windup & Channel Stances (12) ---
    ChargeHeavyWeapon,
    ChargeSwordQi,
    ChannelSpell,
    ChannelSpirit,
    DrawBow,
    GuardBrace,
    ParryReady,
    UltimateAwaken,
    BossTelegraph,
    FocusBreath,
    GatherQi,
    TalismanAim,

    // --- Multi-Hit Combo & Melee Attacks (14) ---
    SlashCombo1,
    SlashCombo2,
    SlashCombo3,
    LungeThrust,
    SpinCleave,
    UppercutLift,
    DownwardSlam,
    FlurryStrikes,
    CounterRiposte,
    BreakerStrike,
    PursuitFinisher,
    CrossCut,
    SwordArrayBurst,
    DualCoopSlash,

    // --- Spell Cast, Ranged & Support (12) ---
    CastBolt,
    CastMeteor,
    CastFrostLance,
    CastLightning,
    CastEarthRift,
    CastTidalWave,
    CastThornBind,
    VolleyRelease,
    CastHeal,
    CastWardShield,
    CastBuffAura,
    CastCleanse,

    // --- Dodge, Backstep & Recovery (8) ---
    DodgeBackstep,
    DodgeSidestep,
    DodgeRoll,
    AirEvade,
    GuardImpact,
    PerfectParry,
    RecoverStance,
    FleeDash,

    // --- Directional Hitstun, Stagger & Knockback (10) ---
    HitstunFront,
    HitstunBack,
    HitstunUp,
    HitstunHeavy,
    HitstopFreeze,
    StaggerWobble,
    StaggerKneel,
    KnockbackSlide,
    KnockdownFlat,
    GetupRise,

    // --- Status Ailments (12) ---
    AilmentBurning,
    AilmentChilled,
    AilmentFrozen,
    AilmentRooted,
    AilmentBleeding,
    AilmentVulnerable,
    AilmentPoisoned,
    AilmentCursed,
    AilmentFeared,
    AilmentSilenced,
    AilmentExhausted,
    AilmentConfused,

    // --- Consumables, Interactions, Level-Up, Victory & Death (10) ---
    UsePotion,
    UseTalisman,
    OpenChest,
    InspectShrine,
    CampRest,
    TalkGesture,
    LevelUpAscend,
    VictoryFlourish,
    DefeatCollapse,
    ReviveAwaken,

    // --- Enemy & Boss Motions (10) ---
    BossEntranceRoar,
    BossPhaseShift,
    EnemyHeavyWindup,
    EnemyLungeBite,
    EnemyDrainHex,
    EnemyIronBrace,
    EnemySpecialUnleash,
    EnemyStaggerCollapse,
    EnemyEnragePulse,
    EnemyDissolve,

    // --- Party, Co-op & NPC Emotes (10) ---
    EmoteRally,
    EmoteSalute,
    EmoteCheer,
    EmotePoint,
    EmoteBeckon,
    EmoteBow,
    EmoteHealAssist,
    EmoteFollowupReady,
    EmoteGuardAlly,
    EmoteNodAgree,
}

impl MotionId {
    pub const ALL: &'static [MotionId] = &[
        // Locomotion (14)
        Self::IdleCalm,
        Self::IdleCombat,
        Self::IdleExhausted,
        Self::IdleInjured,
        Self::WalkCasual,
        Self::WalkCautious,
        Self::RunStriding,
        Self::SprintDash,
        Self::SkidStop,
        Self::TurnPivot,
        Self::JumpAscend,
        Self::LandCushion,
        Self::StepForward,
        Self::StepBack,
        // Charge, Windup & Channel Stances (12)
        Self::ChargeHeavyWeapon,
        Self::ChargeSwordQi,
        Self::ChannelSpell,
        Self::ChannelSpirit,
        Self::DrawBow,
        Self::GuardBrace,
        Self::ParryReady,
        Self::UltimateAwaken,
        Self::BossTelegraph,
        Self::FocusBreath,
        Self::GatherQi,
        Self::TalismanAim,
        // Melee Attacks (14)
        Self::SlashCombo1,
        Self::SlashCombo2,
        Self::SlashCombo3,
        Self::LungeThrust,
        Self::SpinCleave,
        Self::UppercutLift,
        Self::DownwardSlam,
        Self::FlurryStrikes,
        Self::CounterRiposte,
        Self::BreakerStrike,
        Self::PursuitFinisher,
        Self::CrossCut,
        Self::SwordArrayBurst,
        Self::DualCoopSlash,
        // Spell Cast, Ranged & Support (12)
        Self::CastBolt,
        Self::CastMeteor,
        Self::CastFrostLance,
        Self::CastLightning,
        Self::CastEarthRift,
        Self::CastTidalWave,
        Self::CastThornBind,
        Self::VolleyRelease,
        Self::CastHeal,
        Self::CastWardShield,
        Self::CastBuffAura,
        Self::CastCleanse,
        // Dodge, Backstep & Recovery (8)
        Self::DodgeBackstep,
        Self::DodgeSidestep,
        Self::DodgeRoll,
        Self::AirEvade,
        Self::GuardImpact,
        Self::PerfectParry,
        Self::RecoverStance,
        Self::FleeDash,
        // Hitstun, Stagger & Knockback (10)
        Self::HitstunFront,
        Self::HitstunBack,
        Self::HitstunUp,
        Self::HitstunHeavy,
        Self::HitstopFreeze,
        Self::StaggerWobble,
        Self::StaggerKneel,
        Self::KnockbackSlide,
        Self::KnockdownFlat,
        Self::GetupRise,
        // Status Ailments (12)
        Self::AilmentBurning,
        Self::AilmentChilled,
        Self::AilmentFrozen,
        Self::AilmentRooted,
        Self::AilmentBleeding,
        Self::AilmentVulnerable,
        Self::AilmentPoisoned,
        Self::AilmentCursed,
        Self::AilmentFeared,
        Self::AilmentSilenced,
        Self::AilmentExhausted,
        Self::AilmentConfused,
        // Consumables, Interactions, Level-Up, Victory & Death (10)
        Self::UsePotion,
        Self::UseTalisman,
        Self::OpenChest,
        Self::InspectShrine,
        Self::CampRest,
        Self::TalkGesture,
        Self::LevelUpAscend,
        Self::VictoryFlourish,
        Self::DefeatCollapse,
        Self::ReviveAwaken,
        // Enemy & Boss Motions (10)
        Self::BossEntranceRoar,
        Self::BossPhaseShift,
        Self::EnemyHeavyWindup,
        Self::EnemyLungeBite,
        Self::EnemyDrainHex,
        Self::EnemyIronBrace,
        Self::EnemySpecialUnleash,
        Self::EnemyStaggerCollapse,
        Self::EnemyEnragePulse,
        Self::EnemyDissolve,
        // Party, Co-op & NPC Emotes (10)
        Self::EmoteRally,
        Self::EmoteSalute,
        Self::EmoteCheer,
        Self::EmotePoint,
        Self::EmoteBeckon,
        Self::EmoteBow,
        Self::EmoteHealAssist,
        Self::EmoteFollowupReady,
        Self::EmoteGuardAlly,
        Self::EmoteNodAgree,
    ];

    pub const fn meta(self) -> MotionMeta {
        use MotionCategory as C;
        let (zh, en, cat, prio, dur, looping) = match self {
            // Locomotion
            Self::IdleCalm => ("静立调息", "Calm Idle", C::Locomotion, 1, 2.2, true),
            Self::IdleCombat => ("持剑戒备", "Combat Ready", C::Locomotion, 2, 1.6, true),
            Self::IdleExhausted => ("气喘乏力", "Exhausted Stance", C::Locomotion, 3, 1.5, true),
            Self::IdleInjured => ("负伤扶剑", "Injured Stance", C::Locomotion, 3, 1.7, true),
            Self::WalkCasual => ("缓步漫行", "Casual Walk", C::Locomotion, 4, 0.56, true),
            Self::WalkCautious => ("潜行探步", "Cautious Step", C::Locomotion, 4, 0.68, true),
            Self::RunStriding => ("疾步奔行", "Striding Run", C::Locomotion, 5, 0.42, true),
            Self::SprintDash => ("御风疾驰", "Wind Sprint", C::Locomotion, 6, 0.34, true),
            Self::SkidStop => ("急刹顿步", "Skid Stop", C::Locomotion, 7, 0.28, false),
            Self::TurnPivot => ("回身折转", "Pivot Turn", C::Locomotion, 6, 0.22, false),
            Self::JumpAscend => ("轻功起跃", "Qinggong Leap", C::Locomotion, 7, 0.40, false),
            Self::LandCushion => ("落地卸力", "Cushion Land", C::Locomotion, 7, 0.24, false),
            Self::StepForward => ("踏步进逼", "Advance Step", C::Locomotion, 5, 0.28, false),
            Self::StepBack => ("撤步拉开", "Retreat Step", C::Locomotion, 5, 0.28, false),
            // Charge, Windup & Channel
            Self::ChargeHeavyWeapon => {
                ("沉剑蓄力", "Heavy Charge", C::ChargeStance, 14, 0.90, true)
            }
            Self::ChargeSwordQi => (
                "凝气聚剑",
                "Sword Qi Charge",
                C::ChargeStance,
                14,
                0.85,
                true,
            ),
            Self::ChannelSpell => ("掐诀咏咒", "Spell Channel", C::ChargeStance, 15, 0.78, true),
            Self::ChannelSpirit => (
                "聚灵凝脉",
                "Spirit Channel",
                C::ChargeStance,
                14,
                0.84,
                true,
            ),
            Self::DrawBow => ("拉弓满月", "Bow Full Draw", C::ChargeStance, 14, 0.72, true),
            Self::GuardBrace => ("沉身御守", "Guard Brace", C::ChargeStance, 16, 0.80, true),
            Self::ParryReady => ("借势待反", "Riposte Ready", C::ChargeStance, 16, 0.75, true),
            Self::UltimateAwaken => (
                "绝技觉醒",
                "Ultimate Awaken",
                C::ChargeStance,
                18,
                0.95,
                true,
            ),
            Self::BossTelegraph => (
                "首领蓄威",
                "Boss Telegraph",
                C::ChargeStance,
                18,
                0.88,
                true,
            ),
            Self::FocusBreath => ("凝神归元", "Focus Breath", C::ChargeStance, 12, 0.92, true),
            Self::GatherQi => ("吸纳灵气", "Gather Qi", C::ChargeStance, 13, 0.88, true),
            Self::TalismanAim => ("符箓锁定", "Talisman Aim", C::ChargeStance, 13, 0.65, true),
            // Melee Attacks
            Self::SlashCombo1 => ("惊鸿初斩", "Slash Combo 1", C::MeleeAttack, 20, 0.52, false),
            Self::SlashCombo2 => ("回风二斩", "Slash Combo 2", C::MeleeAttack, 20, 0.54, false),
            Self::SlashCombo3 => ("断浪三斩", "Slash Combo 3", C::MeleeAttack, 22, 0.62, false),
            Self::LungeThrust => ("长虹贯日", "Lunge Thrust", C::MeleeAttack, 21, 0.56, false),
            Self::SpinCleave => ("旋风横扫", "Spin Cleave", C::MeleeAttack, 22, 0.60, false),
            Self::UppercutLift => ("挑剑破空", "Rising Slash", C::MeleeAttack, 21, 0.54, false),
            Self::DownwardSlam => ("力劈华山", "Overhead Slam", C::MeleeAttack, 23, 0.66, false),
            Self::FlurryStrikes => (
                "疾风连斩",
                "Flurry Strikes",
                C::MeleeAttack,
                23,
                0.68,
                false,
            ),
            Self::CounterRiposte => (
                "借势反击",
                "Counter Riposte",
                C::MeleeAttack,
                25,
                0.58,
                false,
            ),
            Self::BreakerStrike => (
                "破势重击",
                "Stance Breaker",
                C::MeleeAttack,
                25,
                0.64,
                false,
            ),
            Self::PursuitFinisher => (
                "追击绝杀",
                "Pursuit Finisher",
                C::MeleeAttack,
                26,
                0.66,
                false,
            ),
            Self::CrossCut => ("十字剑芒", "Cross Cut", C::MeleeAttack, 22, 0.56, false),
            Self::SwordArrayBurst => (
                "万剑齐发",
                "Sword Array Burst",
                C::MeleeAttack,
                26,
                0.82,
                false,
            ),
            Self::DualCoopSlash => (
                "同心合击",
                "Co-op Dual Slash",
                C::MeleeAttack,
                26,
                0.74,
                false,
            ),
            // Spell Cast, Ranged & Support
            Self::CastBolt => (
                "灵弹疾射",
                "Cast Spirit Bolt",
                C::SpellCast,
                20,
                0.58,
                false,
            ),
            Self::CastMeteor => (
                "陨炎天降",
                "Cast Cinder Meteor",
                C::SpellCast,
                23,
                0.72,
                false,
            ),
            Self::CastFrostLance => (
                "寒冰凝枪",
                "Cast Frost Lance",
                C::SpellCast,
                22,
                0.66,
                false,
            ),
            Self::CastLightning => (
                "九霄引雷",
                "Cast Storm Chain",
                C::SpellCast,
                23,
                0.64,
                false,
            ),
            Self::CastEarthRift => ("裂地震脉", "Cast Earth Rift", C::SpellCast, 23, 0.70, false),
            Self::CastTidalWave => (
                "沧澜化龙",
                "Cast Tidal Dragon",
                C::SpellCast,
                23,
                0.72,
                false,
            ),
            Self::CastThornBind => ("荆棘缠缚", "Cast Thorn Cage", C::SpellCast, 21, 0.64, false),
            Self::VolleyRelease => ("漫天箭雨", "Volley Release", C::SpellCast, 21, 0.62, false),
            Self::CastHeal => ("甘露回春", "Cast Heal", C::SpellCast, 20, 0.64, false),
            Self::CastWardShield => (
                "灵盾护体",
                "Cast Ward Shield",
                C::SpellCast,
                20,
                0.60,
                false,
            ),
            Self::CastBuffAura => ("清风加持", "Cast Buff Aura", C::SpellCast, 19, 0.62, false),
            Self::CastCleanse => ("净心驱邪", "Cast Cleanse", C::SpellCast, 20, 0.58, false),
            // Dodge, Backstep & Recovery
            Self::DodgeBackstep => (
                "凌波后撤",
                "Backstep Dodge",
                C::DefenseDodge,
                24,
                0.34,
                false,
            ),
            Self::DodgeSidestep => (
                "幻影侧闪",
                "Sidestep Dodge",
                C::DefenseDodge,
                24,
                0.32,
                false,
            ),
            Self::DodgeRoll => ("翻身避险", "Evasive Roll", C::DefenseDodge, 24, 0.42, false),
            Self::AirEvade => ("踏虚闪避", "Air Evade", C::DefenseDodge, 24, 0.38, false),
            Self::GuardImpact => ("御守格挡", "Guard Block", C::DefenseDodge, 28, 0.32, false),
            Self::PerfectParry => (
                "完美卸力",
                "Perfect Parry",
                C::DefenseDodge,
                30,
                0.38,
                false,
            ),
            Self::RecoverStance => (
                "稳步回势",
                "Stance Recovery",
                C::DefenseDodge,
                12,
                0.30,
                false,
            ),
            Self::FleeDash => ("纵身脱战", "Flee Dash", C::DefenseDodge, 26, 0.48, false),
            // Hitstun, Stagger & Knockback
            Self::HitstunFront => ("正面受创", "Front Hitstun", C::HitReaction, 32, 0.34, false),
            Self::HitstunBack => ("背袭受创", "Back Hitstun", C::HitReaction, 32, 0.34, false),
            Self::HitstunUp => (
                "挑飞受创",
                "Upward Hitstun",
                C::HitReaction,
                33,
                0.40,
                false,
            ),
            Self::HitstunHeavy => ("重创仰身", "Heavy Hitstun", C::HitReaction, 35, 0.44, false),
            Self::HitstopFreeze => ("顿帧震颤", "Hitstop Tremor", C::HitReaction, 38, 0.14, true),
            Self::StaggerWobble => ("破势踉跄", "Stagger Wobble", C::HitReaction, 34, 0.85, true),
            Self::StaggerKneel => ("破甲单膝", "Stagger Kneel", C::HitReaction, 35, 0.95, true),
            Self::KnockbackSlide => (
                "击退滑步",
                "Knockback Slide",
                C::HitReaction,
                34,
                0.42,
                false,
            ),
            Self::KnockdownFlat => ("击倒伏地", "Knockdown", C::HitReaction, 36, 0.60, false),
            Self::GetupRise => ("撑剑起身", "Getup Rise", C::HitReaction, 30, 0.46, false),
            // Status Ailments (12)
            Self::AilmentBurning => (
                "灼烧扑火",
                "Burning Flail",
                C::StatusAilment,
                15,
                0.56,
                true,
            ),
            Self::AilmentChilled => (
                "寒气战栗",
                "Chilled Shiver",
                C::StatusAilment,
                15,
                0.48,
                true,
            ),
            Self::AilmentFrozen => ("冰封僵直", "Frozen Rigid", C::StatusAilment, 36, 1.20, true),
            Self::AilmentRooted => (
                "荆棘挣扎",
                "Rooted Strain",
                C::StatusAilment,
                16,
                0.68,
                true,
            ),
            Self::AilmentBleeding => (
                "失血捂伤",
                "Bleeding Clutch",
                C::StatusAilment,
                15,
                0.82,
                true,
            ),
            Self::AilmentVulnerable => (
                "破绽失衡",
                "Vulnerable Exposed",
                C::StatusAilment,
                17,
                0.74,
                true,
            ),
            Self::AilmentPoisoned => (
                "中毒佝偻",
                "Poisoned Hunch",
                C::StatusAilment,
                15,
                0.94,
                true,
            ),
            Self::AilmentCursed => (
                "诅咒痉挛",
                "Cursed Tremor",
                C::StatusAilment,
                16,
                0.62,
                true,
            ),
            Self::AilmentFeared => ("震慑畏缩", "Feared Cower", C::StatusAilment, 33, 0.64, true),
            Self::AilmentSilenced => (
                "封脉缄默",
                "Silenced Stifle",
                C::StatusAilment,
                15,
                0.80,
                true,
            ),
            Self::AilmentExhausted => (
                "枯竭喘息",
                "Exhausted Gasp",
                C::StatusAilment,
                14,
                1.10,
                true,
            ),
            Self::AilmentConfused => (
                "迷惘晕眩",
                "Confused Sway",
                C::StatusAilment,
                16,
                0.90,
                true,
            ),
            // Consumables, Interactions, Level-Up, Victory & Death
            Self::UsePotion => (
                "饮药回血",
                "Drink Potion",
                C::ConsumableInteract,
                19,
                0.56,
                false,
            ),
            Self::UseTalisman => (
                "祭符施印",
                "Activate Talisman",
                C::ConsumableInteract,
                19,
                0.52,
                false,
            ),
            Self::OpenChest => (
                "开启宝箱",
                "Open Chest",
                C::ConsumableInteract,
                14,
                0.54,
                false,
            ),
            Self::InspectShrine => (
                "参拜灵坛",
                "Inspect Shrine",
                C::ConsumableInteract,
                14,
                0.68,
                false,
            ),
            Self::CampRest => (
                "篝火歇息",
                "Camp Rest",
                C::ConsumableInteract,
                10,
                1.80,
                true,
            ),
            Self::TalkGesture => (
                "拱手交谈",
                "Talk Gesture",
                C::ConsumableInteract,
                11,
                0.72,
                false,
            ),
            Self::LevelUpAscend => (
                "破境飞升",
                "Level Up Ascend",
                C::VictoryDeath,
                40,
                0.96,
                false,
            ),
            Self::VictoryFlourish => (
                "挽剑胜势",
                "Victory Flourish",
                C::VictoryDeath,
                42,
                1.10,
                true,
            ),
            Self::DefeatCollapse => (
                "力竭倒地",
                "Defeat Collapse",
                C::VictoryDeath,
                50,
                0.82,
                false,
            ),
            Self::ReviveAwaken => (
                "灵光复生",
                "Revive Awaken",
                C::VictoryDeath,
                45,
                0.78,
                false,
            ),
            // Enemy & Boss Motions
            Self::BossEntranceRoar => (
                "首领怒啸",
                "Boss Entrance Roar",
                C::EnemyBoss,
                28,
                0.88,
                false,
            ),
            Self::BossPhaseShift => (
                "真身蜕变",
                "Boss Phase Shift",
                C::EnemyBoss,
                39,
                0.92,
                false,
            ),
            Self::EnemyHeavyWindup => (
                "妖兽蓄势",
                "Enemy Heavy Windup",
                C::EnemyBoss,
                17,
                0.86,
                true,
            ),
            Self::EnemyLungeBite => (
                "扑咬突袭",
                "Enemy Lunge Bite",
                C::EnemyBoss,
                22,
                0.62,
                false,
            ),
            Self::EnemyDrainHex => ("噬灵诡术", "Enemy Drain Hex", C::EnemyBoss, 21, 0.68, false),
            Self::EnemyIronBrace => ("铁甲盘踞", "Enemy Iron Brace", C::EnemyBoss, 17, 0.84, true),
            Self::EnemySpecialUnleash => {
                ("绝招倾泻", "Special Unleash", C::EnemyBoss, 26, 0.78, false)
            }
            Self::EnemyStaggerCollapse => {
                ("妖势崩塌", "Stagger Collapse", C::EnemyBoss, 36, 0.90, true)
            }
            Self::EnemyEnragePulse => ("狂暴鼓动", "Enrage Pulse", C::EnemyBoss, 24, 0.72, true),
            Self::EnemyDissolve => ("妖气溃散", "Enemy Dissolve", C::EnemyBoss, 50, 0.86, false),
            // Party, Co-op & NPC Emotes
            Self::EmoteRally => ("举剑鼓舞", "Party Rally", C::PartyEmote, 12, 0.72, false),
            Self::EmoteSalute => ("抱拳致意", "Martial Salute", C::PartyEmote, 11, 0.66, false),
            Self::EmoteCheer => ("欣喜欢呼", "Victory Cheer", C::PartyEmote, 13, 0.76, true),
            Self::EmotePoint => ("剑指前方", "Point Ahead", C::PartyEmote, 11, 0.58, false),
            Self::EmoteBeckon => ("招手示意", "Beckon Wave", C::PartyEmote, 11, 0.68, false),
            Self::EmoteBow => ("颔首行礼", "Courteous Bow", C::PartyEmote, 11, 0.64, false),
            Self::EmoteHealAssist => ("灵息驰援", "Heal Assist", C::PartyEmote, 18, 0.68, false),
            Self::EmoteFollowupReady => (
                "蓄势协攻",
                "Follow-up Ready",
                C::PartyEmote,
                18,
                0.64,
                false,
            ),
            Self::EmoteGuardAlly => ("护友身前", "Guard Ally", C::PartyEmote, 18, 0.62, false),
            Self::EmoteNodAgree => ("轻颔赞同", "Agree Nod", C::PartyEmote, 10, 0.44, false),
        };
        MotionMeta {
            id: self,
            zh_name: zh,
            en_name: en,
            category: cat,
            priority: prio,
            duration: dur,
            looping,
        }
    }

    pub const fn is_badge_worthy(self) -> bool {
        matches!(
            self.meta().category,
            MotionCategory::ChargeStance | MotionCategory::StatusAilment
        ) || matches!(
            self,
            Self::StaggerWobble
                | Self::StaggerKneel
                | Self::EnemyStaggerCollapse
                | Self::EnemyHeavyWindup
                | Self::EnemyIronBrace
                | Self::BossPhaseShift
                | Self::BossEntranceRoar
                | Self::EnemyEnragePulse
                | Self::HitstopFreeze
        )
    }

    pub fn badge_label(self) -> String {
        let m = self.meta();
        format!("{} · {}", m.zh_name, m.en_name)
    }
}

// ---------------------------------------------------------------------------
// 2D Pose & Non-Linear Anticipation -> Release -> Overshoot Curves
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose2d {
    pub offset: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
    pub tint_flash: Option<Color>,
}

impl Pose2d {
    pub const IDENTITY: Self = Self {
        offset: Vec2::ZERO,
        rotation: 0.0,
        scale: Vec2::ONE,
        tint_flash: None,
    };

    /// Critically-damped exponential blend toward `target`.
    pub fn blend_towards(self, target: Self, blend_rate: f32, dt: f32) -> Self {
        let alpha = if dt <= 0.0 {
            0.0
        } else {
            1.0 - (-blend_rate.max(1.0) * dt).exp()
        };
        let tint_flash = match (self.tint_flash, target.tint_flash) {
            (Some(a), Some(b)) => Some(lerp_srgba(a, b, alpha)),
            (None, Some(b)) => {
                let s = b.to_srgba();
                Some(Color::srgba(s.red, s.green, s.blue, s.alpha * alpha))
            }
            (Some(a), None) => {
                let s = a.to_srgba();
                let next_a = s.alpha * (1.0 - alpha);
                (next_a > 0.015).then_some(Color::srgba(s.red, s.green, s.blue, next_a))
            }
            (None, None) => None,
        };
        Self {
            offset: self.offset.lerp(target.offset, alpha),
            rotation: self.rotation + (target.rotation - self.rotation) * alpha,
            scale: self.scale.lerp(target.scale, alpha),
            tint_flash,
        }
    }
}

impl Default for Pose2d {
    fn default() -> Self {
        Self::IDENTITY
    }
}

fn lerp_srgba(a: Color, b: Color, t: f32) -> Color {
    let a = a.to_srgba();
    let b = b.to_srgba();
    Color::srgba(
        a.red + (b.red - a.red) * t,
        a.green + (b.green - a.green) * t,
        a.blue + (b.blue - a.blue) * t,
        a.alpha + (b.alpha - a.alpha) * t,
    )
}

/// Blends an optional `Pose2d::tint_flash` over a sprite's base color.
pub fn blend_pose_tint(base: Color, tint_flash: Option<Color>) -> Color {
    let Some(flash) = tint_flash else {
        return base;
    };
    let b = base.to_srgba();
    let f = flash.to_srgba();
    let w = f.alpha.clamp(0.0, 0.85);
    Color::srgba(
        b.red + (f.red - b.red) * w,
        b.green + (f.green - b.green) * w,
        b.blue + (f.blue - b.blue) * w,
        b.alpha,
    )
}

/// Non-linear AAA anticipation -> explosive release -> elastic overshoot curve.
/// Returns `(drive, squash_pulse)`:
/// - `drive` dips negative during anticipation (`0..0.22`), explodes past `+1.15`
///   on release (`0.22..0.46`), and eases elastically back to `0.0`.
/// - `squash_pulse` compresses on coil, stretches on strike, then rebounds.
pub fn anticipation_release_curve(p: f32) -> (f32, f32) {
    let p = p.clamp(0.0, 1.0);
    if p < 0.22 {
        let u = p / 0.22;
        let coil = (u * PI * 0.5).sin();
        (-0.34 * coil, -0.14 * coil)
    } else if p < 0.44 {
        let u = (p - 0.22) / 0.22;
        let snap = 1.0 - (1.0 - u).powi(4);
        let drive = -0.34 + 1.50 * snap;
        let stretch = -0.14 + 0.32 * snap;
        (drive, stretch)
    } else {
        let u = (p - 0.44) / 0.56;
        let decay = (1.0 - u).powi(2);
        let elastic = decay * (u * PI * 2.2).cos();
        (1.16 * decay + 0.08 * elastic, 0.18 * elastic)
    }
}

/// Evaluates the procedural 2D pose for any `MotionId` at time `elapsed`
/// with facing direction `facing` (`+1.0` right, `-1.0` left).
pub fn evaluate_pose(id: MotionId, elapsed: f32, facing: f32) -> Pose2d {
    let meta = id.meta();
    let f = if facing < 0.0 { -1.0 } else { 1.0 };
    let dur = meta.duration.max(0.05);
    let p = if meta.looping {
        (elapsed.rem_euclid(dur)) / dur
    } else {
        (elapsed / dur).clamp(0.0, 1.0)
    };
    let phase = p * TAU;

    match id {
        // --- Locomotion ---
        MotionId::IdleCalm => {
            let b = phase.sin();
            Pose2d {
                offset: Vec2::new(0.0, b * 2.2),
                rotation: b * 0.012 * f,
                scale: Vec2::new(1.0 - b * 0.015, 1.0 + b * 0.018),
                tint_flash: None,
            }
        }
        MotionId::IdleCombat => {
            let b = phase.sin();
            Pose2d {
                offset: Vec2::new(f * b * 2.5, b.abs() * 2.8),
                rotation: -f * (0.025 + b * 0.018),
                scale: Vec2::new(1.02 - b * 0.02, 0.98 + b * 0.024),
                tint_flash: None,
            }
        }
        MotionId::IdleExhausted => {
            let gasp = (phase.sin() * 0.5 + 0.5).powf(1.6);
            Pose2d {
                offset: Vec2::new(-f * 2.0, -5.0 - gasp * 3.5),
                rotation: -f * (0.08 + gasp * 0.04),
                scale: Vec2::new(1.05 + gasp * 0.03, 0.91 - gasp * 0.04),
                tint_flash: Some(Color::srgba(0.65, 0.68, 0.78, 0.14)),
            }
        }
        MotionId::IdleInjured => {
            let sway = phase.sin();
            Pose2d {
                offset: Vec2::new(-f * 4.0 + sway * 1.8, -4.0 - sway.abs() * 2.2),
                rotation: f * (0.09 + sway * 0.025),
                scale: Vec2::new(1.03, 0.94 + sway * 0.015),
                tint_flash: Some(Color::srgba(0.95, 0.35, 0.30, 0.14)),
            }
        }
        MotionId::WalkCasual => {
            let stride = phase.sin();
            let hop = stride.abs();
            Pose2d {
                offset: Vec2::new(f * stride * 2.4, hop * 4.5),
                rotation: -f * (0.04 + stride * 0.035),
                scale: Vec2::new(1.0 - hop * 0.035, 1.0 + hop * 0.042),
                tint_flash: None,
            }
        }
        MotionId::WalkCautious => {
            let stride = phase.sin();
            Pose2d {
                offset: Vec2::new(f * stride * 2.0, -3.0 + stride.abs() * 2.2),
                rotation: -f * (0.07 + stride * 0.02),
                scale: Vec2::new(1.04, 0.93 + stride.abs() * 0.03),
                tint_flash: None,
            }
        }
        MotionId::RunStriding => {
            let stride = phase.sin();
            let hop = stride.abs();
            Pose2d {
                offset: Vec2::new(f * (3.5 + stride * 4.2), hop * 7.5 - 1.5),
                rotation: -f * (0.11 + stride * 0.055),
                scale: Vec2::new(1.04 - hop * 0.05, 0.96 + hop * 0.075),
                tint_flash: None,
            }
        }
        MotionId::SprintDash => {
            let stride = phase.sin();
            Pose2d {
                offset: Vec2::new(f * (8.0 + stride * 4.0), stride.abs() * 5.0 - 2.0),
                rotation: -f * 0.18,
                scale: Vec2::new(1.12, 0.90 + stride.abs() * 0.05),
                tint_flash: Some(Color::srgba(0.75, 0.94, 1.0, 0.20)),
            }
        }
        MotionId::SkidStop => {
            // Lean sharply back against momentum, squash feet into ground, rebound upright.
            let brake = (1.0 - p).powi(2);
            let rebound = (p * PI * 2.0).sin() * (1.0 - p) * 0.35;
            Pose2d {
                offset: Vec2::new(f * 10.0 * brake, -5.5 * brake + rebound * 3.0),
                rotation: f * (0.21 * brake - 0.04 * rebound),
                scale: Vec2::new(1.0 + 0.14 * brake, 1.0 - 0.12 * brake),
                tint_flash: None,
            }
        }
        MotionId::TurnPivot => {
            let twist = (p * PI).sin();
            Pose2d {
                offset: Vec2::new(-f * 3.0 * twist, 2.5 * twist),
                rotation: f * 0.09 * twist,
                scale: Vec2::new(1.0 - 0.24 * twist, 1.0 + 0.07 * twist),
                tint_flash: None,
            }
        }
        MotionId::JumpAscend => {
            let arc = (p * PI).sin();
            Pose2d {
                offset: Vec2::new(f * 6.0 * p, 24.0 * arc),
                rotation: -f * 0.08 * arc,
                scale: Vec2::new(1.0 - 0.10 * arc, 1.0 + 0.15 * arc),
                tint_flash: None,
            }
        }
        MotionId::LandCushion => {
            let sq = (1.0 - p).powi(2);
            Pose2d {
                offset: Vec2::new(0.0, -7.5 * sq),
                rotation: -f * 0.05 * sq,
                scale: Vec2::new(1.0 + 0.18 * sq, 1.0 - 0.16 * sq),
                tint_flash: None,
            }
        }
        MotionId::StepForward => {
            let u = (p * PI).sin();
            Pose2d {
                offset: Vec2::new(f * 12.0 * u, 3.0 * u),
                rotation: -f * 0.08 * u,
                scale: Vec2::new(1.0 + 0.05 * u, 1.0 - 0.03 * u),
                tint_flash: None,
            }
        }
        MotionId::StepBack => {
            let u = (p * PI).sin();
            Pose2d {
                offset: Vec2::new(-f * 14.0 * u, 4.0 * u),
                rotation: f * 0.09 * u,
                scale: Vec2::new(0.95, 1.04),
                tint_flash: None,
            }
        }

        // --- Charge, Windup & Channel Stances ---
        MotionId::ChargeHeavyWeapon => {
            let pulse = (phase * 1.0).sin() * 0.5 + 0.5;
            let hum = (elapsed * 34.0).sin() * 1.6;
            Pose2d {
                offset: Vec2::new(-f * (11.0 + pulse * 4.0) + hum, -8.0 - pulse * 3.0),
                rotation: f * (0.17 + pulse * 0.04),
                scale: Vec2::new(1.13 + pulse * 0.04, 0.87 - pulse * 0.03),
                tint_flash: Some(Color::srgba(1.0, 0.78, 0.32, 0.22 + pulse * 0.16)),
            }
        }
        MotionId::ChargeSwordQi => {
            let pulse = phase.sin() * 0.5 + 0.5;
            let hum = (elapsed * 38.0).sin() * 1.2;
            Pose2d {
                offset: Vec2::new(-f * 8.0 + hum, -4.0 + pulse * 4.0),
                rotation: f * (0.12 + pulse * 0.03),
                scale: Vec2::new(1.08, 0.92 + pulse * 0.05),
                tint_flash: Some(Color::srgba(0.68, 0.94, 1.0, 0.24 + pulse * 0.18)),
            }
        }
        MotionId::ChannelSpell => {
            let float = phase.sin();
            let hum = (elapsed * 28.0).cos() * 1.1;
            Pose2d {
                offset: Vec2::new(hum, 9.0 + float * 5.0),
                rotation: -f * (0.04 + float * 0.03),
                scale: Vec2::new(0.94 - float * 0.02, 1.08 + float * 0.03),
                tint_flash: Some(Color::srgba(0.54, 0.82, 1.0, 0.28 + float.abs() * 0.16)),
            }
        }
        MotionId::ChannelSpirit => {
            let float = phase.sin();
            Pose2d {
                offset: Vec2::new(0.0, 7.0 + float * 4.0),
                rotation: float * 0.04,
                scale: Vec2::new(0.96, 1.06 + float * 0.025),
                tint_flash: Some(Color::srgba(0.42, 1.0, 0.72, 0.24 + float.abs() * 0.14)),
            }
        }
        MotionId::DrawBow => {
            let taut = phase.sin() * 0.5 + 0.5;
            let hum = (elapsed * 42.0).sin() * 0.8;
            Pose2d {
                offset: Vec2::new(-f * (9.5 + taut * 3.0) + hum, -2.5),
                rotation: f * (0.11 + taut * 0.025),
                scale: Vec2::new(1.09, 0.93),
                tint_flash: Some(Color::srgba(1.0, 0.90, 0.55, 0.18)),
            }
        }
        MotionId::GuardBrace => {
            let pulse = phase.sin() * 0.5 + 0.5;
            Pose2d {
                offset: Vec2::new(-f * 7.0, -7.5 - pulse * 1.5),
                rotation: f * 0.10,
                scale: Vec2::new(1.12, 0.88),
                tint_flash: Some(Color::srgba(0.68, 0.88, 1.0, 0.22 + pulse * 0.10)),
            }
        }
        MotionId::ParryReady => {
            let pulse = phase.sin() * 0.5 + 0.5;
            Pose2d {
                offset: Vec2::new(f * 4.0, -5.0 + pulse * 2.0),
                rotation: -f * (0.09 + pulse * 0.02),
                scale: Vec2::new(1.06, 0.94 + pulse * 0.03),
                tint_flash: Some(Color::srgba(1.0, 0.88, 0.42, 0.28 + pulse * 0.18)),
            }
        }
        MotionId::UltimateAwaken => {
            let surge = phase.sin() * 0.5 + 0.5;
            let hum = (elapsed * 48.0).sin() * 1.8;
            Pose2d {
                offset: Vec2::new(hum, 13.0 + surge * 5.0),
                rotation: -f * 0.05,
                scale: Vec2::new(1.05 + surge * 0.05, 1.10 + surge * 0.05),
                tint_flash: Some(Color::srgba(1.0, 0.90, 0.45, 0.38 + surge * 0.22)),
            }
        }
        MotionId::BossTelegraph => {
            let swell = phase.sin() * 0.5 + 0.5;
            let hum = (elapsed * 44.0).sin() * 2.4;
            Pose2d {
                offset: Vec2::new(-f * (12.0 + swell * 4.0) + hum, 6.0 + swell * 5.0),
                rotation: f * (0.14 + swell * 0.04),
                scale: Vec2::new(1.14 + swell * 0.06, 1.06 + swell * 0.06),
                tint_flash: Some(Color::srgba(1.0, 0.26, 0.18, 0.30 + swell * 0.24)),
            }
        }
        MotionId::FocusBreath => {
            let b = phase.sin();
            Pose2d {
                offset: Vec2::new(0.0, b * 3.5),
                rotation: 0.0,
                scale: Vec2::new(1.0 - b * 0.04, 1.0 + b * 0.05),
                tint_flash: Some(Color::srgba(0.62, 0.95, 0.88, 0.18)),
            }
        }
        MotionId::GatherQi => {
            let b = phase.sin() * 0.5 + 0.5;
            Pose2d {
                offset: Vec2::new(0.0, -4.0 + b * 8.0),
                rotation: f * 0.04 * (b - 0.5),
                scale: Vec2::new(1.06 - b * 0.06, 0.94 + b * 0.09),
                tint_flash: Some(Color::srgba(0.55, 0.84, 1.0, 0.24 + b * 0.14)),
            }
        }
        MotionId::TalismanAim => {
            let pulse = phase.sin() * 0.5 + 0.5;
            Pose2d {
                offset: Vec2::new(f * 5.0, 3.0 + pulse * 2.0),
                rotation: -f * 0.07,
                scale: Vec2::new(0.97, 1.04),
                tint_flash: Some(Color::srgba(1.0, 0.85, 0.38, 0.22)),
            }
        }

        // --- Multi-Hit Melee Combo & Skill Attacks ---
        MotionId::SlashCombo1 => {
            let (drive, sq) = anticipation_release_curve(p);
            Pose2d {
                offset: Vec2::new(f * 18.0 * drive, 5.0 * drive.max(0.0)),
                rotation: -f * 0.20 * drive,
                scale: Vec2::new(1.0 + sq * 0.85, 1.0 - sq * 0.55),
                tint_flash: (p > 0.20 && p < 0.44).then_some(Color::srgba(0.95, 0.98, 1.0, 0.28)),
            }
        }
        MotionId::SlashCombo2 => {
            let (drive, sq) = anticipation_release_curve(p);
            Pose2d {
                offset: Vec2::new(f * 22.0 * drive, -4.0 * drive + 6.0 * (p * PI).sin()),
                rotation: f * 0.18 * drive,
                scale: Vec2::new(1.0 + sq * 0.95, 1.0 - sq * 0.50),
                tint_flash: (p > 0.20 && p < 0.46).then_some(Color::srgba(0.84, 0.96, 1.0, 0.32)),
            }
        }
        MotionId::SlashCombo3 => {
            let (drive, sq) = anticipation_release_curve(p);
            Pose2d {
                offset: Vec2::new(f * 28.0 * drive, 10.0 * (p * PI).sin() - 3.0 * drive),
                rotation: -f * 0.27 * drive,
                scale: Vec2::new(1.0 + sq * 1.15, 1.0 - sq * 0.65),
                tint_flash: (p > 0.20 && p < 0.48).then_some(Color::srgba(1.0, 0.90, 0.50, 0.42)),
            }
        }
        MotionId::LungeThrust => {
            let (drive, sq) = anticipation_release_curve(p);
            Pose2d {
                offset: Vec2::new(f * 34.0 * drive, -2.0 * drive.abs()),
                rotation: -f * 0.14 * drive,
                scale: Vec2::new(1.0 + sq * 1.35, 1.0 - sq * 0.72),
                tint_flash: (p > 0.22 && p < 0.48).then_some(Color::srgba(0.88, 0.98, 1.0, 0.34)),
            }
        }
        MotionId::SpinCleave => {
            let (drive, sq) = anticipation_release_curve(p);
            let spin = (p * TAU * 1.5).sin();
            Pose2d {
                offset: Vec2::new(f * 20.0 * drive, 9.0 * (p * PI).sin()),
                rotation: -f * (0.28 * drive + spin * 0.14),
                scale: Vec2::new(1.0 + sq * 1.10 + spin.abs() * 0.08, 1.0 - sq * 0.55),
                tint_flash: (p > 0.18 && p < 0.55).then_some(Color::srgba(0.85, 0.95, 1.0, 0.30)),
            }
        }
        MotionId::UppercutLift => {
            let (drive, sq) = anticipation_release_curve(p);
            Pose2d {
                offset: Vec2::new(f * 16.0 * drive, 24.0 * drive.max(-0.2)),
                rotation: f * 0.22 * drive,
                scale: Vec2::new(1.0 - sq * 0.65, 1.0 + sq * 1.25),
                tint_flash: (p > 0.22 && p < 0.46).then_some(Color::srgba(1.0, 0.94, 0.68, 0.32)),
            }
        }
        MotionId::DownwardSlam => {
            let (drive, sq) = anticipation_release_curve(p);
            let leap = if p < 0.28 {
                26.0 * (p / 0.28)
            } else {
                26.0 * (1.0 - ((p - 0.28) / 0.18).min(1.0)) - 8.0 * (1.0 - p)
            };
            Pose2d {
                offset: Vec2::new(f * 24.0 * drive, leap),
                rotation: -f * 0.28 * drive,
                scale: Vec2::new(1.0 + sq * 1.25, 1.0 - sq * 0.78),
                tint_flash: (p > 0.26 && p < 0.52).then_some(Color::srgba(1.0, 0.72, 0.32, 0.40)),
            }
        }
        MotionId::FlurryStrikes => {
            let burst = (p * PI * 6.0).sin();
            let env = (p * PI).sin();
            Pose2d {
                offset: Vec2::new(f * (18.0 * env + 7.0 * burst), 5.0 * burst),
                rotation: -f * (0.14 * env + 0.12 * burst),
                scale: Vec2::new(
                    1.0 + 0.14 * burst.abs() * env,
                    1.0 - 0.08 * burst.abs() * env,
                ),
                tint_flash: Some(Color::srgba(0.90, 0.96, 1.0, 0.26 * env)),
            }
        }
        MotionId::CounterRiposte | MotionId::BreakerStrike | MotionId::PursuitFinisher => {
            let (drive, sq) = anticipation_release_curve(p);
            Pose2d {
                offset: Vec2::new(f * 32.0 * drive, 7.0 * (p * PI).sin()),
                rotation: -f * 0.26 * drive,
                scale: Vec2::new(1.0 + sq * 1.35, 1.0 - sq * 0.70),
                tint_flash: (p > 0.18 && p < 0.54).then_some(Color::srgba(1.0, 0.88, 0.34, 0.48)),
            }
        }
        MotionId::CrossCut => {
            let (drive, sq) = anticipation_release_curve(p);
            let cross = (p * PI * 4.0).sin();
            Pose2d {
                offset: Vec2::new(f * 22.0 * drive, 6.0 * cross * (1.0 - p)),
                rotation: -f * (0.20 * drive + 0.10 * cross),
                scale: Vec2::new(1.0 + sq * 1.05, 1.0 - sq * 0.55),
                tint_flash: (p > 0.20 && p < 0.52).then_some(Color::srgba(0.82, 0.96, 1.0, 0.38)),
            }
        }
        MotionId::SwordArrayBurst | MotionId::DualCoopSlash => {
            let (drive, sq) = anticipation_release_curve(p);
            Pose2d {
                offset: Vec2::new(f * 26.0 * drive, 14.0 * (p * PI).sin()),
                rotation: -f * 0.22 * drive,
                scale: Vec2::new(1.0 + sq * 1.15, 1.0 + 0.08 * (p * PI).sin()),
                tint_flash: Some(Color::srgba(1.0, 0.90, 0.50, 0.36 * (p * PI).sin())),
            }
        }

        // --- Spell Cast, Ranged & Support ---
        MotionId::CastBolt
        | MotionId::CastMeteor
        | MotionId::CastFrostLance
        | MotionId::CastLightning
        | MotionId::CastEarthRift
        | MotionId::CastTidalWave
        | MotionId::CastThornBind
        | MotionId::VolleyRelease => {
            let (drive, sq) = anticipation_release_curve(p);
            let tint = match id {
                MotionId::CastMeteor => Color::srgba(1.0, 0.44, 0.14, 0.40),
                MotionId::CastFrostLance => Color::srgba(0.48, 0.90, 1.0, 0.40),
                MotionId::CastLightning => Color::srgba(0.76, 0.66, 1.0, 0.44),
                MotionId::CastEarthRift => Color::srgba(0.96, 0.66, 0.24, 0.38),
                MotionId::CastTidalWave => Color::srgba(0.32, 0.84, 1.0, 0.38),
                MotionId::CastThornBind => Color::srgba(0.46, 0.96, 0.42, 0.36),
                MotionId::VolleyRelease => Color::srgba(1.0, 0.92, 0.56, 0.34),
                _ => Color::srgba(0.68, 0.92, 1.0, 0.36),
            };
            Pose2d {
                offset: Vec2::new(f * 14.0 * drive, 12.0 * (p * PI).sin()),
                rotation: -f * 0.13 * drive,
                scale: Vec2::new(1.0 - sq * 0.45, 1.0 + sq * 0.90),
                tint_flash: (p > 0.14 && p < 0.62).then_some(tint),
            }
        }
        MotionId::CastHeal
        | MotionId::CastWardShield
        | MotionId::CastBuffAura
        | MotionId::CastCleanse => {
            let lift = (p * PI).sin();
            let tint = match id {
                MotionId::CastHeal => Color::srgba(0.45, 1.0, 0.64, 0.42 * lift),
                MotionId::CastWardShield => Color::srgba(0.58, 0.88, 1.0, 0.44 * lift),
                MotionId::CastBuffAura => Color::srgba(1.0, 0.88, 0.44, 0.38 * lift),
                _ => Color::srgba(0.90, 1.0, 0.96, 0.42 * lift),
            };
            Pose2d {
                offset: Vec2::new(0.0, 15.0 * lift),
                rotation: f * 0.04 * (p * TAU).sin(),
                scale: Vec2::new(1.0 - 0.06 * lift, 1.0 + 0.11 * lift),
                tint_flash: Some(tint),
            }
        }

        // --- Dodge, Backstep & Recovery ---
        MotionId::DodgeBackstep => {
            let arc = (p * PI).sin();
            Pose2d {
                offset: Vec2::new(-f * 26.0 * arc, 10.0 * arc),
                rotation: f * 0.18 * arc,
                scale: Vec2::new(1.0 + 0.12 * arc, 1.0 - 0.09 * arc),
                tint_flash: Some(Color::srgba(0.82, 0.94, 1.0, 0.20 * arc)),
            }
        }
        MotionId::DodgeSidestep => {
            let arc = (p * PI).sin();
            Pose2d {
                offset: Vec2::new(-f * 14.0 * arc, -6.0 * arc),
                rotation: -f * 0.12 * arc,
                scale: Vec2::new(1.0 - 0.22 * arc, 1.0 + 0.10 * arc),
                tint_flash: Some(Color::srgba(0.76, 0.92, 1.0, 0.24 * arc)),
            }
        }
        MotionId::DodgeRoll => {
            let arc = (p * PI).sin();
            Pose2d {
                offset: Vec2::new(-f * 20.0 * arc, -4.0 + 8.0 * arc),
                rotation: f * p * TAU,
                scale: Vec2::new(1.08, 0.92),
                tint_flash: None,
            }
        }
        MotionId::AirEvade => {
            let arc = (p * PI).sin();
            Pose2d {
                offset: Vec2::new(-f * 22.0 * arc, 22.0 * arc),
                rotation: f * 0.24 * arc,
                scale: Vec2::new(0.92, 1.12),
                tint_flash: Some(Color::srgba(0.80, 0.96, 1.0, 0.25 * arc)),
            }
        }
        MotionId::GuardImpact => {
            let recoil = (1.0 - p).powi(2);
            let vib = (elapsed * 62.0 * TAU).sin() * 2.2 * recoil;
            Pose2d {
                offset: Vec2::new(-f * 14.0 * recoil + vib, -5.0 * recoil),
                rotation: f * 0.12 * recoil,
                scale: Vec2::new(1.0 + 0.15 * recoil, 1.0 - 0.12 * recoil),
                tint_flash: Some(Color::srgba(0.78, 0.94, 1.0, 0.45 * recoil)),
            }
        }
        MotionId::PerfectParry => {
            let pop = (1.0 - p).powi(2);
            Pose2d {
                offset: Vec2::new(f * 10.0 * pop, 4.0 * pop),
                rotation: -f * 0.15 * pop,
                scale: Vec2::new(1.0 + 0.16 * pop, 1.0 + 0.12 * pop),
                tint_flash: Some(Color::srgba(1.0, 0.95, 0.55, 0.62 * pop)),
            }
        }
        MotionId::RecoverStance => {
            let settle = (1.0 - p).powi(2);
            Pose2d {
                offset: Vec2::new(-f * 6.0 * settle, -3.0 * settle),
                rotation: f * 0.05 * settle,
                scale: Vec2::new(1.0 + 0.05 * settle, 1.0 - 0.05 * settle),
                tint_flash: None,
            }
        }
        MotionId::FleeDash => {
            let dash = p * p;
            Pose2d {
                offset: Vec2::new(-f * 42.0 * dash, 8.0 * (p * PI).sin()),
                rotation: f * 0.24 * (1.0 - 0.2 * p),
                scale: Vec2::new(1.16, 0.88),
                tint_flash: None,
            }
        }

        // --- Directional Hitstun, Stagger & Knockback ---
        MotionId::HitstunFront => {
            let k = (1.0 - p).powi(2);
            Pose2d {
                offset: Vec2::new(-f * 18.0 * k, 3.0 * k),
                rotation: f * 0.18 * k,
                scale: Vec2::new(1.0 - 0.14 * k, 1.0 + 0.12 * k),
                tint_flash: Some(Color::srgba(1.0, 0.86, 0.82, 0.52 * k)),
            }
        }
        MotionId::HitstunBack => {
            let k = (1.0 - p).powi(2);
            Pose2d {
                offset: Vec2::new(f * 18.0 * k, 2.0 * k),
                rotation: -f * 0.20 * k,
                scale: Vec2::new(1.0 - 0.12 * k, 1.0 + 0.10 * k),
                tint_flash: Some(Color::srgba(1.0, 0.72, 0.70, 0.50 * k)),
            }
        }
        MotionId::HitstunUp => {
            let arc = (p * PI).sin();
            Pose2d {
                offset: Vec2::new(-f * 10.0 * arc, 24.0 * arc),
                rotation: f * 0.25 * arc,
                scale: Vec2::new(1.0 - 0.14 * arc, 1.0 + 0.18 * arc),
                tint_flash: Some(Color::srgba(1.0, 0.92, 0.78, 0.55 * (1.0 - p))),
            }
        }
        MotionId::HitstunHeavy => {
            let k = (1.0 - p).powi(2);
            let wobble = (p * PI * 3.0).cos() * k;
            Pose2d {
                offset: Vec2::new(-f * 28.0 * k, 6.0 * k),
                rotation: f * 0.28 * k,
                scale: Vec2::new(1.0 - 0.18 * wobble, 1.0 + 0.15 * wobble),
                tint_flash: Some(Color::srgba(1.0, 0.45, 0.36, 0.60 * k)),
            }
        }
        MotionId::HitstopFreeze => {
            // 62.5 Hz zero-drift 2D micro-tremor during impact freeze!
            let vx = (elapsed * 62.5 * TAU).sin() * 4.2;
            let vy = (elapsed * 64.0 * TAU + 1.2).cos() * 2.0;
            Pose2d {
                offset: Vec2::new(vx, vy),
                rotation: f * 0.06,
                scale: Vec2::new(0.90, 1.11),
                tint_flash: Some(Color::srgba(1.0, 0.98, 0.92, 0.72)),
            }
        }
        MotionId::StaggerWobble => {
            let sway = phase.sin();
            Pose2d {
                offset: Vec2::new(sway * 9.0, -7.0 - sway.abs() * 3.0),
                rotation: sway * 0.16,
                scale: Vec2::new(1.06, 0.90 + sway.abs() * 0.03),
                tint_flash: Some(Color::srgba(1.0, 0.84, 0.34, 0.26)),
            }
        }
        MotionId::StaggerKneel => {
            let breath = phase.sin();
            Pose2d {
                offset: Vec2::new(-f * 6.0 + breath * 2.5, -16.0 - breath * 2.0),
                rotation: -f * (0.14 + breath * 0.03),
                scale: Vec2::new(1.13, 0.82 + breath * 0.02),
                tint_flash: Some(Color::srgba(1.0, 0.74, 0.26, 0.32)),
            }
        }
        MotionId::KnockbackSlide => {
            let k = (1.0 - p).powi(2);
            Pose2d {
                offset: Vec2::new(-f * 36.0 * k, -4.0 * k),
                rotation: f * 0.22 * k,
                scale: Vec2::new(1.14 * k + (1.0 - k), 0.88 * k + (1.0 - k)),
                tint_flash: Some(Color::srgba(1.0, 0.65, 0.55, 0.35 * k)),
            }
        }
        MotionId::KnockdownFlat => {
            let fall = 1.0 - (1.0 - p).powi(3);
            Pose2d {
                offset: Vec2::new(-f * 20.0 * fall, -24.0 * fall),
                rotation: f * 0.52 * fall,
                scale: Vec2::new(1.18, 0.78),
                tint_flash: Some(Color::srgba(0.95, 0.42, 0.35, 0.30)),
            }
        }
        MotionId::GetupRise => {
            let k = 1.0 - p;
            Pose2d {
                offset: Vec2::new(-f * 8.0 * k, -18.0 * k),
                rotation: f * 0.24 * k,
                scale: Vec2::new(1.0 + 0.10 * k, 1.0 - 0.12 * k),
                tint_flash: None,
            }
        }

        // --- All 12 Status Ailments (distinct silhouettes + tints) ---
        MotionId::AilmentBurning => {
            let flail = (phase * 2.0).sin();
            let hop = (phase * 2.0).cos().abs();
            Pose2d {
                offset: Vec2::new(flail * 6.5, hop * 6.0),
                rotation: flail * 0.15,
                scale: Vec2::new(0.96 + hop * 0.08, 1.04 - hop * 0.06),
                tint_flash: Some(Color::srgba(1.0, 0.36, 0.08, 0.32 + hop * 0.18)),
            }
        }
        MotionId::AilmentChilled => {
            let shiver = (elapsed * 32.0).sin();
            Pose2d {
                offset: Vec2::new(shiver * 3.8, -3.5),
                rotation: shiver * 0.045,
                scale: Vec2::new(0.95, 1.03),
                tint_flash: Some(Color::srgba(0.55, 0.88, 1.0, 0.30)),
            }
        }
        MotionId::AilmentFrozen => {
            // Rigid upright lock with subtle crystalline shimmer.
            let glint = (phase.sin() * 0.5 + 0.5) * 0.12;
            Pose2d {
                offset: Vec2::new(0.0, 0.5),
                rotation: 0.0,
                scale: Vec2::new(0.97, 1.04),
                tint_flash: Some(Color::srgba(0.46, 0.90, 1.0, 0.48 + glint)),
            }
        }
        MotionId::AilmentRooted => {
            let tug = (phase * 2.0).sin();
            Pose2d {
                offset: Vec2::new(tug * 4.0, -5.0 + tug.abs() * 2.0),
                rotation: tug * 0.11,
                scale: Vec2::new(1.07, 0.91 + tug.abs() * 0.04),
                tint_flash: Some(Color::srgba(0.36, 0.86, 0.34, 0.30)),
            }
        }
        MotionId::AilmentBleeding => {
            let throb = (phase.sin() * 0.5 + 0.5).powi(2);
            Pose2d {
                offset: Vec2::new(-f * 4.5, -6.0 - throb * 3.0),
                rotation: -f * (0.12 + throb * 0.05),
                scale: Vec2::new(1.05, 0.92 - throb * 0.03),
                tint_flash: Some(Color::srgba(0.92, 0.14, 0.16, 0.26 + throb * 0.22)),
            }
        }
        MotionId::AilmentVulnerable => {
            let sway = phase.sin();
            Pose2d {
                offset: Vec2::new(-f * 7.0 + sway * 5.0, -4.0),
                rotation: f * (0.14 + sway * 0.08),
                scale: Vec2::new(1.04, 0.94),
                tint_flash: Some(Color::srgba(1.0, 0.78, 0.22, 0.30 + sway.abs() * 0.14)),
            }
        }
        MotionId::AilmentPoisoned => {
            let hunch = phase.sin() * 0.5 + 0.5;
            Pose2d {
                offset: Vec2::new(phase.cos() * 2.5, -8.0 - hunch * 4.0),
                rotation: -f * (0.11 + hunch * 0.05),
                scale: Vec2::new(1.08 + hunch * 0.03, 0.89 - hunch * 0.04),
                tint_flash: Some(Color::srgba(0.42, 0.96, 0.22, 0.28 + hunch * 0.16)),
            }
        }
        MotionId::AilmentCursed => {
            let spasm = (elapsed * 22.0).sin() * (phase.sin().abs());
            Pose2d {
                offset: Vec2::new(spasm * 5.0, -2.0 + spasm.abs() * 3.0),
                rotation: spasm * 0.13,
                scale: Vec2::new(0.96 + spasm * 0.06, 1.04 - spasm * 0.06),
                tint_flash: Some(Color::srgba(0.66, 0.30, 0.96, 0.36)),
            }
        }
        MotionId::AilmentFeared => {
            let tremor = (elapsed * 30.0).sin();
            Pose2d {
                offset: Vec2::new(-f * 9.0 + tremor * 3.2, -9.0),
                rotation: f * (0.15 + tremor * 0.04),
                scale: Vec2::new(0.90, 0.88),
                tint_flash: Some(Color::srgba(0.76, 0.68, 1.0, 0.32)),
            }
        }
        MotionId::AilmentSilenced => {
            let sway = phase.sin();
            Pose2d {
                offset: Vec2::new(-f * 3.0, -3.0 + sway * 1.5),
                rotation: sway * 0.05,
                scale: Vec2::new(0.94, 0.98),
                tint_flash: Some(Color::srgba(0.58, 0.64, 0.78, 0.34)),
            }
        }
        MotionId::AilmentExhausted => {
            let heave = (phase.sin() * 0.5 + 0.5).powi(2);
            Pose2d {
                offset: Vec2::new(-f * 2.5, -10.0 - heave * 4.0),
                rotation: -f * (0.13 + heave * 0.04),
                scale: Vec2::new(1.09 + heave * 0.04, 0.86 - heave * 0.04),
                tint_flash: Some(Color::srgba(0.55, 0.58, 0.65, 0.24)),
            }
        }
        MotionId::AilmentConfused => {
            let wobble = phase.sin();
            let nod = (phase * 2.0).cos();
            Pose2d {
                offset: Vec2::new(wobble * 8.0, nod * 3.0),
                rotation: wobble * 0.21,
                scale: Vec2::new(1.0 + nod * 0.04, 1.0 - nod * 0.04),
                tint_flash: Some(Color::srgba(1.0, 0.64, 0.92, 0.26)),
            }
        }

        // --- Consumables, Interactions, Level-Up, Victory & Death ---
        MotionId::UsePotion => {
            let lift = (p * PI).sin();
            Pose2d {
                offset: Vec2::new(0.0, 9.0 * lift),
                rotation: f * 0.09 * lift,
                scale: Vec2::new(1.0 - 0.05 * lift, 1.0 + 0.10 * lift),
                tint_flash: Some(Color::srgba(0.45, 1.0, 0.62, 0.38 * lift)),
            }
        }
        MotionId::UseTalisman => {
            let snap = (p * PI).sin();
            Pose2d {
                offset: Vec2::new(f * 10.0 * snap, 6.0 * snap),
                rotation: -f * 0.11 * snap,
                scale: Vec2::new(1.04, 1.04),
                tint_flash: Some(Color::srgba(1.0, 0.88, 0.36, 0.40 * snap)),
            }
        }
        MotionId::OpenChest | MotionId::InspectShrine => {
            let lean = (p * PI).sin();
            Pose2d {
                offset: Vec2::new(f * 6.0 * lean, -4.0 * lean),
                rotation: -f * 0.12 * lean,
                scale: Vec2::new(1.04, 0.96),
                tint_flash: None,
            }
        }
        MotionId::CampRest => {
            let b = phase.sin();
            Pose2d {
                offset: Vec2::new(0.0, -5.0 + b * 2.0),
                rotation: b * 0.025,
                scale: Vec2::new(1.04, 0.95 + b * 0.02),
                tint_flash: Some(Color::srgba(1.0, 0.72, 0.36, 0.12)),
            }
        }
        MotionId::TalkGesture => {
            let bob = (p * PI * 2.0).sin().abs();
            Pose2d {
                offset: Vec2::new(f * 3.0 * bob, 4.0 * bob),
                rotation: -f * 0.05 * bob,
                scale: Vec2::new(1.0 - 0.02 * bob, 1.0 + 0.04 * bob),
                tint_flash: None,
            }
        }
        MotionId::LevelUpAscend => {
            let rise = (p * PI).sin();
            Pose2d {
                offset: Vec2::new(0.0, 26.0 * rise),
                rotation: f * 0.06 * (p * TAU).sin(),
                scale: Vec2::new(1.0 + 0.10 * rise, 1.0 + 0.16 * rise),
                tint_flash: Some(Color::srgba(1.0, 0.94, 0.48, 0.56 * rise)),
            }
        }
        MotionId::VictoryFlourish => {
            let cheer = (phase.sin() * 0.5 + 0.5).powf(1.4);
            Pose2d {
                offset: Vec2::new(0.0, 6.0 + cheer * 9.0),
                rotation: -f * (0.04 + cheer * 0.05),
                scale: Vec2::new(1.0 - 0.03 * cheer, 1.05 + 0.06 * cheer),
                tint_flash: Some(Color::srgba(1.0, 0.92, 0.56, 0.20 + cheer * 0.14)),
            }
        }
        MotionId::DefeatCollapse => {
            let down = 1.0 - (1.0 - p).powi(3);
            Pose2d {
                offset: Vec2::new(-f * 14.0 * down, -26.0 * down),
                rotation: f * 0.48 * down,
                scale: Vec2::new(1.16, 0.76),
                tint_flash: Some(Color::srgba(0.55, 0.50, 0.62, 0.45 * down)),
            }
        }
        MotionId::ReviveAwaken => {
            let rise = 1.0 - (1.0 - p).powi(2);
            Pose2d {
                offset: Vec2::new(0.0, -14.0 * (1.0 - rise) + 12.0 * (p * PI).sin()),
                rotation: 0.0,
                scale: Vec2::new(1.0 + 0.08 * (p * PI).sin(), 1.0 + 0.14 * (p * PI).sin()),
                tint_flash: Some(Color::srgba(1.0, 0.96, 0.72, 0.55 * (1.0 - p))),
            }
        }

        // --- Enemy & Boss Motions ---
        MotionId::BossEntranceRoar => {
            let swell = (p * PI).sin();
            let roar_vib = (elapsed * 52.0).sin() * 3.0 * swell;
            Pose2d {
                offset: Vec2::new(roar_vib, 12.0 * swell),
                rotation: f * 0.08 * swell,
                scale: Vec2::new(1.0 + 0.18 * swell, 1.0 + 0.18 * swell),
                tint_flash: Some(Color::srgba(1.0, 0.34, 0.20, 0.42 * swell)),
            }
        }
        MotionId::BossPhaseShift => {
            let surge = (p * PI).sin();
            let vib = (elapsed * 58.0).sin() * 3.6 * surge;
            Pose2d {
                offset: Vec2::new(vib, 18.0 * surge),
                rotation: (p * TAU).sin() * 0.09 * surge,
                scale: Vec2::new(1.0 + 0.22 * surge, 1.0 + 0.24 * surge),
                tint_flash: Some(Color::srgba(1.0, 0.24, 0.48, 0.55 * surge)),
            }
        }
        MotionId::EnemyHeavyWindup => {
            let coil = phase.sin() * 0.5 + 0.5;
            let hum = (elapsed * 38.0).sin() * 2.0;
            Pose2d {
                offset: Vec2::new(-f * (14.0 + coil * 5.0) + hum, -6.0 - coil * 3.0),
                rotation: f * (0.16 + coil * 0.05),
                scale: Vec2::new(1.14 + coil * 0.05, 0.88 - coil * 0.04),
                tint_flash: Some(Color::srgba(1.0, 0.38, 0.18, 0.28 + coil * 0.20)),
            }
        }
        MotionId::EnemyLungeBite => {
            let (drive, sq) = anticipation_release_curve(p);
            Pose2d {
                offset: Vec2::new(f * 30.0 * drive, 5.0 * drive.max(0.0)),
                rotation: -f * 0.19 * drive,
                scale: Vec2::new(1.0 + sq * 1.15, 1.0 - sq * 0.60),
                tint_flash: (p > 0.20 && p < 0.52).then_some(Color::srgba(1.0, 0.62, 0.34, 0.32)),
            }
        }
        MotionId::EnemyDrainHex => {
            let wave = (p * PI).sin();
            Pose2d {
                offset: Vec2::new(f * 10.0 * wave, 14.0 * wave),
                rotation: -f * 0.09 * wave,
                scale: Vec2::new(0.94, 1.10),
                tint_flash: Some(Color::srgba(0.72, 0.36, 1.0, 0.42 * wave)),
            }
        }
        MotionId::EnemyIronBrace => {
            let pulse = phase.sin() * 0.5 + 0.5;
            Pose2d {
                offset: Vec2::new(0.0, -7.0),
                rotation: 0.0,
                scale: Vec2::new(1.16 + pulse * 0.03, 0.86),
                tint_flash: Some(Color::srgba(0.72, 0.84, 0.96, 0.26 + pulse * 0.12)),
            }
        }
        MotionId::EnemySpecialUnleash => {
            let (drive, sq) = anticipation_release_curve(p);
            let vib = (elapsed * 48.0).sin() * 2.5 * (p * PI).sin();
            Pose2d {
                offset: Vec2::new(f * 36.0 * drive + vib, 10.0 * (p * PI).sin()),
                rotation: -f * 0.24 * drive,
                scale: Vec2::new(1.0 + sq * 1.35, 1.0 + 0.10 * (p * PI).sin()),
                tint_flash: Some(Color::srgba(1.0, 0.28, 0.16, 0.48 * (p * PI).sin())),
            }
        }
        MotionId::EnemyStaggerCollapse => {
            let sway = phase.sin();
            Pose2d {
                offset: Vec2::new(-f * 10.0 + sway * 7.0, -18.0 - sway.abs() * 3.0),
                rotation: f * (0.20 + sway * 0.09),
                scale: Vec2::new(1.15, 0.80),
                tint_flash: Some(Color::srgba(1.0, 0.80, 0.28, 0.34)),
            }
        }
        MotionId::EnemyEnragePulse => {
            let pulse = phase.sin() * 0.5 + 0.5;
            Pose2d {
                offset: Vec2::new((elapsed * 40.0).sin() * 1.8, 5.0 * pulse),
                rotation: 0.0,
                scale: Vec2::new(1.08 + pulse * 0.08, 1.08 + pulse * 0.08),
                tint_flash: Some(Color::srgba(1.0, 0.20, 0.14, 0.34 + pulse * 0.20)),
            }
        }
        MotionId::EnemyDissolve => {
            let fade = 1.0 - (1.0 - p).powi(2);
            Pose2d {
                offset: Vec2::new(0.0, -16.0 * fade),
                rotation: f * 0.22 * fade,
                scale: Vec2::new(1.0 + 0.12 * fade, 1.0 - 0.22 * fade),
                tint_flash: Some(Color::srgba(0.62, 0.72, 0.92, 0.42 * fade)),
            }
        }

        // --- Party, Co-op & NPC Emotes ---
        MotionId::EmoteRally => {
            let lift = (p * PI).sin();
            Pose2d {
                offset: Vec2::new(f * 6.0 * lift, 14.0 * lift),
                rotation: -f * 0.10 * lift,
                scale: Vec2::new(1.04, 1.08),
                tint_flash: Some(Color::srgba(1.0, 0.88, 0.44, 0.28 * lift)),
            }
        }
        MotionId::EmoteSalute => {
            let bow = (p * PI).sin();
            Pose2d {
                offset: Vec2::new(f * 4.0 * bow, -2.0 * bow),
                rotation: -f * 0.08 * bow,
                scale: Vec2::new(1.02, 0.98),
                tint_flash: None,
            }
        }
        MotionId::EmoteCheer => {
            let hop = (phase * 2.0).sin().abs();
            Pose2d {
                offset: Vec2::new(0.0, hop * 13.0),
                rotation: (phase).sin() * 0.07,
                scale: Vec2::new(1.0 - hop * 0.06, 1.0 + hop * 0.10),
                tint_flash: Some(Color::srgba(1.0, 0.95, 0.65, 0.16 * hop)),
            }
        }
        MotionId::EmotePoint => {
            let thrust = (p * PI).sin();
            Pose2d {
                offset: Vec2::new(f * 12.0 * thrust, 3.0 * thrust),
                rotation: -f * 0.11 * thrust,
                scale: Vec2::new(1.06, 0.97),
                tint_flash: None,
            }
        }
        MotionId::EmoteBeckon => {
            let wave = (p * PI * 3.0).sin();
            Pose2d {
                offset: Vec2::new(wave * 4.0, 5.0 * (p * PI).sin()),
                rotation: wave * 0.09,
                scale: Vec2::ONE,
                tint_flash: None,
            }
        }
        MotionId::EmoteBow => {
            let dip = (p * PI).sin();
            Pose2d {
                offset: Vec2::new(f * 5.0 * dip, -7.0 * dip),
                rotation: -f * 0.18 * dip,
                scale: Vec2::new(1.05, 0.92),
                tint_flash: None,
            }
        }
        MotionId::EmoteHealAssist => {
            let lift = (p * PI).sin();
            Pose2d {
                offset: Vec2::new(f * 8.0 * lift, 15.0 * lift),
                rotation: -f * 0.06 * lift,
                scale: Vec2::new(0.97, 1.08),
                tint_flash: Some(Color::srgba(0.46, 0.96, 1.0, 0.36 * lift)),
            }
        }
        MotionId::EmoteFollowupReady => {
            let (drive, sq) = anticipation_release_curve(p);
            Pose2d {
                offset: Vec2::new(f * 20.0 * drive, 8.0 * (p * PI).sin()),
                rotation: -f * 0.16 * drive,
                scale: Vec2::new(1.0 + sq * 0.90, 1.0 - sq * 0.45),
                tint_flash: Some(Color::srgba(1.0, 0.62, 0.38, 0.30 * (p * PI).sin())),
            }
        }
        MotionId::EmoteGuardAlly => {
            let brace = (p * PI).sin();
            Pose2d {
                offset: Vec2::new(-f * 6.0 * brace, -5.0 * brace),
                rotation: f * 0.08 * brace,
                scale: Vec2::new(1.08, 0.93),
                tint_flash: Some(Color::srgba(0.70, 0.90, 1.0, 0.24 * brace)),
            }
        }
        MotionId::EmoteNodAgree => {
            let nod = (p * PI * 2.0).sin().max(0.0);
            Pose2d {
                offset: Vec2::new(0.0, -4.0 * nod),
                rotation: -f * 0.08 * nod,
                scale: Vec2::new(1.02, 0.96),
                tint_flash: None,
            }
        }
    }
}

// ---------------------------------------------------------------------------
// MotionState2d Component (Smooth Blending, Priority & Skid-Stop Inertia)
// ---------------------------------------------------------------------------

#[derive(Component, Clone, Debug)]
pub struct MotionState2d {
    pub current: MotionId,
    pub elapsed: f32,
    pub facing: f32,
    pub pose: Pose2d,
    pub blend_rate: f32,
    pub base_motion: MotionId,
    /// Continuous movement time (seconds) used to ramp walk -> run -> skid-stop.
    pub move_streak: f32,
    pub last_move_dir: Vec2,
    /// Set to true on the frame a skid-stop triggers so callers can emit dust puffs.
    pub skid_triggered: bool,
}

impl MotionState2d {
    pub fn new(initial: MotionId, facing: f32) -> Self {
        Self {
            current: initial,
            elapsed: 0.0,
            facing: if facing < 0.0 { -1.0 } else { 1.0 },
            pose: Pose2d::IDENTITY,
            blend_rate: 18.0,
            base_motion: initial,
            move_streak: 0.0,
            last_move_dir: Vec2::X,
            skid_triggered: false,
        }
    }

    pub fn set_base(&mut self, base: MotionId) {
        self.base_motion = base;
    }

    /// Requests a motion transition respecting priority; does not restart an
    /// already-playing looping motion.
    pub fn request(&mut self, next: MotionId) {
        if self.current == next {
            if !next.meta().looping && self.elapsed >= next.meta().duration {
                self.elapsed = 0.0;
            }
            return;
        }
        let cur_meta = self.current.meta();
        let next_meta = next.meta();
        let finished = !cur_meta.looping && self.elapsed >= cur_meta.duration;
        if finished || cur_meta.looping || next_meta.priority >= cur_meta.priority {
            self.current = next;
            self.elapsed = 0.0;
        }
    }

    /// Unconditionally switches to `next` (resets elapsed if different).
    pub fn force(&mut self, next: MotionId) {
        if self.current != next {
            self.current = next;
            self.elapsed = 0.0;
        }
    }

    /// Advances the motion clock and smoothly blends `self.pose` toward the
    /// procedural target pose.
    pub fn advance(&mut self, dt: f32) -> Pose2d {
        let dt = dt.max(0.0);
        self.elapsed += dt;
        let meta = self.current.meta();
        if !meta.looping && self.elapsed >= meta.duration {
            self.current = self.base_motion;
            self.elapsed = 0.0;
        }
        let target = evaluate_pose(self.current, self.elapsed, self.facing);
        // Hitstop micro-tremor snaps immediately for razor-crisp 62 Hz vibration.
        let rate = if self.current == MotionId::HitstopFreeze {
            60.0
        } else {
            self.blend_rate
        };
        self.pose = self.pose.blend_towards(target, rate, dt);
        self.pose
    }

    /// Drives overworld locomotion: WalkCasual -> RunStriding -> SkidStop inertia
    /// when halting abruptly after sustained running.
    pub fn step_locomotion(&mut self, moving: bool, dir: Vec2, dt: f32) -> Pose2d {
        self.skid_triggered = false;
        if dir.x.abs() > 0.01 {
            self.facing = if dir.x < 0.0 { -1.0 } else { 1.0 };
        }
        if moving {
            if dir.length_squared() > 0.01 {
                self.last_move_dir = dir.normalize();
            }
            self.move_streak += dt.max(0.0);
            let target = if self.move_streak >= 0.28 {
                MotionId::RunStriding
            } else {
                MotionId::WalkCasual
            };
            self.set_base(target);
            self.request(target);
        } else {
            self.set_base(MotionId::IdleCalm);
            if self.move_streak >= 0.28 {
                self.skid_triggered = true;
                self.force(MotionId::SkidStop);
            } else if self.current != MotionId::SkidStop {
                self.force(MotionId::IdleCalm);
            }
            self.move_streak = 0.0;
        }
        self.advance(dt)
    }

    pub fn badge_text(&self) -> Option<String> {
        self.current
            .is_badge_worthy()
            .then(|| self.current.badge_label())
    }
}

// ---------------------------------------------------------------------------
// Lightweight 2D Impact Juice Particle & Smear Engine
// ---------------------------------------------------------------------------

#[derive(Component, Clone, Copy)]
pub struct ImpactParticle2d {
    pub age: f32,
    pub duration: f32,
    pub velocity: Vec2,
    pub drag: f32,
    pub gravity: f32,
    pub spin: f32,
    pub start_scale: Vec2,
    pub end_scale: Vec2,
    pub start_color: Color,
    pub end_color: Color,
    pub battle_synced: bool,
    pub align_to_velocity: bool,
}

impl ImpactParticle2d {
    fn spawn(
        self,
        commands: &mut Commands,
        pos: Vec3,
        size: Vec2,
        rotation: f32,
        exit_state: AppState,
    ) {
        commands.spawn((
            Sprite::from_color(self.start_color, size),
            Transform::from_translation(pos)
                .with_rotation(Quat::from_rotation_z(rotation))
                .with_scale(self.start_scale.extend(1.0)),
            self,
            DespawnOnExit(exit_state),
        ));
    }
}

/// Spawns directional 2D razor hit-sparks bursting along `direction`.
#[allow(clippy::too_many_arguments)]
pub fn spawn_hit_sparks(
    commands: &mut Commands,
    origin: Vec3,
    direction: Vec2,
    count: usize,
    accent: Color,
    is_crit: bool,
    exit_state: AppState,
) {
    let dir = direction.normalize_or_zero();
    let base_angle = if dir == Vec2::ZERO {
        0.0
    } else {
        dir.y.atan2(dir.x)
    };
    let spread = if is_crit { 1.35 } else { 0.98 };
    let core = Color::srgba(1.0, 0.99, 0.90, 0.98);
    let n = count.max(1);
    for i in 0..n {
        let frac = if n == 1 {
            0.0
        } else {
            (i as f32 / (n - 1) as f32) * 2.0 - 1.0
        };
        let jitter = ((i as f32 * 2.399).sin()) * 0.14;
        let angle = base_angle + frac * spread + jitter;
        let speed = (if is_crit { 420.0 } else { 310.0 }) * (0.68 + ((i * 7) % 5) as f32 * 0.12);
        let vel = Vec2::new(angle.cos(), angle.sin()) * speed;
        let len = if is_crit { 34.0 } else { 24.0 } + (i % 3) as f32 * 7.0;
        let thick = if is_crit { 4.2 } else { 3.0 };
        ImpactParticle2d {
            age: 0.0,
            duration: if is_crit { 0.32 } else { 0.24 } + (i % 3) as f32 * 0.03,
            velocity: vel,
            drag: 6.2,
            gravity: -280.0,
            spin: 0.0,
            start_scale: Vec2::new(1.15, 1.0),
            end_scale: Vec2::new(0.22, 0.25),
            start_color: if i % 2 == 0 { core } else { accent },
            end_color: accent.with_alpha(0.0),
            battle_synced: exit_state == AppState::Battle,
            align_to_velocity: true,
        }
        .spawn(
            commands,
            origin + Vec3::new(0.0, 0.0, 3.1),
            Vec2::new(len, thick),
            angle,
            exit_state,
        );
    }
}

/// Spawns a crisp 8-point impact flash star right on contact.
pub fn spawn_impact_flash_star(
    commands: &mut Commands,
    origin: Vec3,
    size: f32,
    color: Color,
    exit_state: AppState,
) {
    let white = Color::srgba(1.0, 0.99, 0.94, 0.96);
    for (idx, angle) in [0.0_f32, PI * 0.5, PI * 0.25, -PI * 0.25]
        .into_iter()
        .enumerate()
    {
        let len = if idx < 2 { size } else { size * 0.72 };
        let thick = if idx < 2 { size * 0.14 } else { size * 0.10 };
        ImpactParticle2d {
            age: 0.0,
            duration: 0.14,
            velocity: Vec2::ZERO,
            drag: 0.0,
            gravity: 0.0,
            spin: if idx < 2 { 1.4 } else { -1.4 },
            start_scale: Vec2::new(0.35, 1.15),
            end_scale: Vec2::new(1.35, 0.08),
            start_color: if idx < 2 { white } else { color },
            end_color: color.with_alpha(0.0),
            battle_synced: false, // flashes crisply even during hit-stop!
            align_to_velocity: false,
        }
        .spawn(
            commands,
            origin + Vec3::new(0.0, 0.0, 3.3),
            Vec2::new(len, thick),
            angle,
            exit_state,
        );
    }
}

/// Spawns an iconic golden-white critical hit cross-glint.
pub fn spawn_crit_cross_glint(
    commands: &mut Commands,
    origin: Vec3,
    color: Color,
    exit_state: AppState,
) {
    let gold_white = Color::srgba(1.0, 0.96, 0.72, 1.0);
    for (angle, len, thick, dur) in [
        (0.0_f32, 195.0, 9.0, 0.26),
        (PI * 0.5, 155.0, 8.0, 0.26),
        (PI * 0.25, 130.0, 5.5, 0.22),
        (-PI * 0.25, 130.0, 5.5, 0.22),
    ] {
        ImpactParticle2d {
            age: 0.0,
            duration: dur,
            velocity: Vec2::ZERO,
            drag: 0.0,
            gravity: 0.0,
            spin: 0.45,
            start_scale: Vec2::new(0.25, 1.4),
            end_scale: Vec2::new(1.45, 0.05),
            start_color: gold_white,
            end_color: color.with_alpha(0.0),
            battle_synced: false,
            align_to_velocity: false,
        }
        .spawn(
            commands,
            origin + Vec3::new(0.0, 0.0, 3.45),
            Vec2::new(len, thick),
            angle,
            exit_state,
        );
    }
}

/// Spawns a dramatic kill-finisher segmented shockwave ring + radial rays.
pub fn spawn_kill_finisher_ring(
    commands: &mut Commands,
    origin: Vec3,
    color: Color,
    exit_state: AppState,
) {
    let segments = 16;
    let ring_core = Color::srgba(1.0, 0.94, 0.64, 0.95);
    for i in 0..segments {
        let angle = i as f32 * TAU / segments as f32;
        let tangent = angle + PI * 0.5;
        let radial = Vec2::new(angle.cos(), angle.sin());
        ImpactParticle2d {
            age: 0.0,
            duration: 0.42,
            velocity: radial * 360.0,
            drag: 3.4,
            gravity: 0.0,
            spin: 0.0,
            start_scale: Vec2::new(0.75, 1.25),
            end_scale: Vec2::new(2.1, 0.10),
            start_color: ring_core,
            end_color: color.with_alpha(0.0),
            battle_synced: exit_state == AppState::Battle,
            align_to_velocity: false,
        }
        .spawn(
            commands,
            origin + (radial * 22.0).extend(3.25),
            Vec2::new(28.0, 6.5),
            tangent,
            exit_state,
        );
    }
    // 8 finisher burst rays
    for i in 0..8 {
        let angle = i as f32 * TAU / 8.0 + 0.2;
        let radial = Vec2::new(angle.cos(), angle.sin());
        ImpactParticle2d {
            age: 0.0,
            duration: 0.34,
            velocity: radial * 240.0,
            drag: 4.0,
            gravity: 0.0,
            spin: 0.0,
            start_scale: Vec2::new(0.4, 1.1),
            end_scale: Vec2::new(1.8, 0.08),
            start_color: Color::srgba(1.0, 0.98, 0.88, 0.94),
            end_color: color.with_alpha(0.0),
            battle_synced: false,
            align_to_velocity: true,
        }
        .spawn(
            commands,
            origin + Vec3::new(0.0, 0.0, 3.35),
            Vec2::new(72.0, 5.5),
            angle,
            exit_state,
        );
    }
}

/// Spawns a curved fan of tapered slash arc smears across a strike zone.
pub fn spawn_slash_arc_smear(
    commands: &mut Commands,
    origin: Vec3,
    facing: f32,
    radius: f32,
    color: Color,
    heavy: bool,
    exit_state: AppState,
) {
    let f = if facing < 0.0 { -1.0 } else { 1.0 };
    let segments = if heavy { 7 } else { 5 };
    let core = Color::srgba(1.0, 0.98, 0.90, 0.94);
    for i in 0..segments {
        let t = i as f32 / (segments - 1).max(1) as f32;
        // Taper thickness from fine tail -> broad leading blade edge -> fine tip.
        let taper = (t * PI).sin().max(0.28);
        let arc_angle = (-0.95 + t * 1.90) * f;
        let local = Vec2::new(
            arc_angle.cos() * f * radius * 0.52,
            arc_angle.sin() * radius,
        );
        let tangent = (arc_angle + PI * 0.5) * f;
        let seg_len = (radius * 0.44) * (0.75 + taper * 0.5);
        let seg_thick = (if heavy { 14.0 } else { 9.5 }) * taper;
        ImpactParticle2d {
            age: -t * 0.028, // slight sweep progression along the arc
            duration: if heavy { 0.22 } else { 0.17 },
            velocity: Vec2::new(f * 55.0, (t - 0.5) * 40.0),
            drag: 5.0,
            gravity: 0.0,
            spin: -f * 0.8,
            start_scale: Vec2::new(1.15, 1.1),
            end_scale: Vec2::new(1.35, 0.06),
            start_color: if i + 1 == segments { core } else { color },
            end_color: color.with_alpha(0.0),
            battle_synced: exit_state == AppState::Battle,
            align_to_velocity: false,
        }
        .spawn(
            commands,
            origin + local.extend(3.05),
            Vec2::new(seg_len, seg_thick),
            tangent,
            exit_state,
        );
    }
}

/// Spawns lightweight footstep or skid-stop dust puffs at character feet.
pub fn spawn_dust_puffs(
    commands: &mut Commands,
    feet: Vec3,
    direction: Vec2,
    count: usize,
    color: Color,
    exit_state: AppState,
) {
    let dir = direction.normalize_or_zero();
    let base = if dir == Vec2::ZERO { -Vec2::X } else { -dir };
    for i in 0..count.max(1) {
        let spread = (i as f32 - (count as f32 - 1.0) * 0.5) * 0.42;
        let rot = Vec2::from_angle(spread).rotate(base);
        let speed = 38.0 + (i % 3) as f32 * 22.0;
        let vel = Vec2::new(
            rot.x * speed,
            rot.y * speed * 0.35 + 24.0 + (i % 2) as f32 * 14.0,
        );
        let size = 10.0 + (i % 3) as f32 * 4.5;
        ImpactParticle2d {
            age: 0.0,
            duration: 0.34 + (i % 3) as f32 * 0.05,
            velocity: vel,
            drag: 4.5,
            gravity: -24.0,
            spin: if i % 2 == 0 { 1.2 } else { -1.2 },
            start_scale: Vec2::splat(0.55),
            end_scale: Vec2::splat(1.45),
            start_color: color,
            end_color: color.with_alpha(0.0),
            battle_synced: exit_state == AppState::Battle,
            align_to_velocity: false,
        }
        .spawn(
            commands,
            feet + Vec3::new((i as f32 - 1.0) * 5.0, (i % 2) as f32 * 2.0, 0.2),
            Vec2::new(size * 1.25, size * 0.85),
            spread,
            exit_state,
        );
    }
}

// ---------------------------------------------------------------------------
// MotionPlugin
// ---------------------------------------------------------------------------

pub struct MotionPlugin;

impl Plugin for MotionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_impact_particles);
    }
}

fn update_impact_particles(
    time: Res<Time>,
    feel: Option<Res<BattleFeel>>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut ImpactParticle2d, &mut Transform, &mut Sprite)>,
) {
    let real_dt = time.delta_secs().max(0.0);
    let battle_dt = feel.map_or(real_dt, |f| f.dt());

    for (entity, mut p, mut transform, mut sprite) in &mut query {
        let dt = if p.battle_synced { battle_dt } else { real_dt };
        p.age += dt;
        if p.age < 0.0 {
            sprite.color.set_alpha(0.0);
            continue;
        }
        if p.age >= p.duration {
            commands.entity(entity).despawn();
            continue;
        }
        let t = (p.age / p.duration).clamp(0.0, 1.0);
        // Integrate damped velocity + gravity.
        let damping = (-p.drag * dt).exp();
        p.velocity *= damping;
        p.velocity.y += p.gravity * dt;
        transform.translation.x += p.velocity.x * dt;
        transform.translation.y += p.velocity.y * dt;

        if p.align_to_velocity && p.velocity.length_squared() > 4.0 {
            let angle = p.velocity.y.atan2(p.velocity.x);
            transform.rotation = Quat::from_rotation_z(angle);
        } else if p.spin != 0.0 {
            transform.rotate_z(p.spin * dt);
        }

        let eased = 1.0 - (1.0 - t).powi(3);
        let scale = p.start_scale.lerp(p.end_scale, eased);
        transform.scale = Vec3::new(scale.x.max(0.01), scale.y.max(0.01), 1.0);

        // Stay crisp then snap-fade in the final 35% of life.
        let fade_t = if t < 0.65 {
            t * 0.4
        } else {
            0.26 + ((t - 0.65) / 0.35) * 0.74
        };
        sprite.color = lerp_srgba(p.start_color, p.end_color, fade_t.clamp(0.0, 1.0));
    }
}

// ---------------------------------------------------------------------------
// Unit Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn catalog_has_at_least_100_unique_valid_motions() {
        assert!(
            MotionId::ALL.len() >= 100,
            "expected >= 100 motions, got {}",
            MotionId::ALL.len()
        );
        let mut ids = HashSet::new();
        let mut zh_names = HashSet::new();
        let mut en_names = HashSet::new();
        let mut categories = HashSet::new();
        for &id in MotionId::ALL {
            assert!(ids.insert(id), "duplicate MotionId: {id:?}");
            let meta = id.meta();
            assert_eq!(meta.id, id);
            assert!(!meta.zh_name.trim().is_empty(), "{id:?} empty zh_name");
            assert!(!meta.en_name.trim().is_empty(), "{id:?} empty en_name");
            assert!(
                zh_names.insert(meta.zh_name),
                "duplicate zh_name: {}",
                meta.zh_name
            );
            assert!(
                en_names.insert(meta.en_name),
                "duplicate en_name: {}",
                meta.en_name
            );
            assert!(meta.duration > 0.0 && meta.duration <= 5.0);
            assert!(meta.priority > 0);
            categories.insert(meta.category);
            // Verify every motion evaluates finite, sane Pose2d values across its curve.
            for frac in [0.0_f32, 0.25, 0.5, 0.75, 1.0] {
                let pose = evaluate_pose(id, meta.duration * frac, 1.0);
                assert!(pose.offset.is_finite(), "{id:?} non-finite offset");
                assert!(pose.rotation.is_finite(), "{id:?} non-finite rotation");
                assert!(
                    pose.scale.is_finite() && pose.scale.x > 0.2 && pose.scale.y > 0.2,
                    "{id:?} invalid scale {:?}",
                    pose.scale
                );
            }
        }
        assert_eq!(categories.len(), 11, "all 11 motion categories populated");
    }

    #[test]
    fn charge_stagger_and_ailment_motions_produce_distinct_poses() {
        let charge = evaluate_pose(MotionId::ChargeHeavyWeapon, 0.22, 1.0);
        let channel = evaluate_pose(MotionId::ChannelSpell, 0.20, 1.0);
        let stagger = evaluate_pose(MotionId::StaggerKneel, 0.20, 1.0);
        let burning = evaluate_pose(MotionId::AilmentBurning, 0.07, 1.0);
        let frozen = evaluate_pose(MotionId::AilmentFrozen, 0.20, 1.0);
        let poisoned = evaluate_pose(MotionId::AilmentPoisoned, 0.30, 1.0);
        let feared = evaluate_pose(MotionId::AilmentFeared, 0.15, 1.0);
        let confused = evaluate_pose(MotionId::AilmentConfused, 0.22, 1.0);

        // Charge coils backward and squashes down; spell channel floats upward and stretches.
        assert!(charge.offset.x < -8.0 && charge.scale.x > 1.08 && charge.scale.y < 0.92);
        assert!(channel.offset.y > 8.0 && channel.scale.y > 1.05);
        // Stagger kneel drops low with wide squash.
        assert!(stagger.offset.y < -12.0 && stagger.scale.y < 0.86);
        // Frozen is strictly rigid upright (rotation == 0), while burning & confused tilt dynamically.
        assert_eq!(frozen.rotation, 0.0);
        assert!(burning.rotation.abs() > 0.05 || burning.offset.x.abs() > 2.0);
        assert!(poisoned.offset.y < -7.0 && poisoned.tint_flash.is_some());
        assert!(feared.scale.x < 0.95 && feared.scale.y < 0.95);
        assert!(confused.rotation.abs() > 0.10);
        // Distinct poses across states
        assert_ne!(charge, channel);
        assert_ne!(stagger, frozen);
        assert_ne!(burning, poisoned);
        assert_ne!(feared, confused);
    }

    #[test]
    fn directional_hitstun_separates_front_back_and_upward_recoil() {
        let front = evaluate_pose(MotionId::HitstunFront, 0.05, 1.0);
        let back = evaluate_pose(MotionId::HitstunBack, 0.05, 1.0);
        let upward = evaluate_pose(MotionId::HitstunUp, 0.18, 1.0);
        let heavy = evaluate_pose(MotionId::HitstunHeavy, 0.04, 1.0);

        // Front hit pushes backward (-X), back hit pushes forward (+X), upward launches (+Y).
        assert!(front.offset.x < -10.0);
        assert!(back.offset.x > 10.0);
        assert!(front.rotation > 0.0 && back.rotation < 0.0);
        assert!(upward.offset.y > 15.0 && upward.scale.y > 1.10);
        assert!(heavy.offset.x < front.offset.x);
    }

    #[test]
    fn hitstop_micro_tremor_vibrates_at_high_frequency_without_drift() {
        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut sum_x = 0.0;
        let samples = 64;
        // Sample one full 62.5 Hz cycle (0.016s) evenly -> mean ~ 0 (zero drift), symmetric excursion.
        for i in 0..samples {
            let t = i as f32 / samples as f32 * (1.0 / 62.5);
            let p = evaluate_pose(MotionId::HitstopFreeze, t, 1.0);
            min_x = min_x.min(p.offset.x);
            max_x = max_x.max(p.offset.x);
            sum_x += p.offset.x;
        }
        assert!(max_x > 3.5 && min_x < -3.5, "min={min_x} max={max_x}");
        assert!((sum_x / samples as f32).abs() < 0.05, "drifted: {sum_x}");
    }

    #[test]
    fn skid_stop_inertia_triggers_after_sustained_run_and_recovers_to_idle() {
        let mut state = MotionState2d::new(MotionId::IdleCalm, 1.0);
        // Walk for 0.10s -> still WalkCasual
        for _ in 0..6 {
            state.step_locomotion(true, Vec2::X, 1.0 / 60.0);
        }
        assert_eq!(state.current, MotionId::WalkCasual);
        // Continue moving past 0.28s -> RunStriding
        for _ in 0..20 {
            state.step_locomotion(true, Vec2::X, 1.0 / 60.0);
        }
        assert_eq!(state.current, MotionId::RunStriding);
        // Release movement -> triggers SkidStop inertia & dust flag!
        let skid_pose = state.step_locomotion(false, Vec2::ZERO, 1.0 / 60.0);
        assert_eq!(state.current, MotionId::SkidStop);
        assert!(state.skid_triggered);
        assert!(skid_pose.offset.x > 0.0, "inertia carries forward slightly");
        // Advance past skid duration -> smoothly returns to IdleCalm
        for _ in 0..30 {
            state.step_locomotion(false, Vec2::ZERO, 1.0 / 60.0);
        }
        assert_eq!(state.current, MotionId::IdleCalm);
        assert!(!state.skid_triggered);
    }

    #[test]
    fn motion_state_blends_smoothly_and_respects_priority() {
        let mut state = MotionState2d::new(MotionId::IdleCombat, 1.0);
        state.request(MotionId::SlashCombo1);
        assert_eq!(state.current, MotionId::SlashCombo1);
        // Lower-priority walk cannot interrupt an active attack.
        state.request(MotionId::WalkCasual);
        assert_eq!(state.current, MotionId::SlashCombo1);
        // Higher-priority hitstun interrupts the attack.
        state.request(MotionId::HitstunHeavy);
        assert_eq!(state.current, MotionId::HitstunHeavy);
        // Critically-damped blend moves gradually rather than teleporting on 1ms step.
        let p0 = state.pose;
        let p1 = state.advance(0.008);
        let target = evaluate_pose(MotionId::HitstunHeavy, state.elapsed, 1.0);
        assert!(p1.offset.x < p0.offset.x && p1.offset.x > target.offset.x);
        // Badge text appears for charge/ailment/stagger, not calm idle.
        state.force(MotionId::BossTelegraph);
        assert!(state.badge_text().unwrap().contains("Boss Telegraph"));
        state.force(MotionId::IdleCalm);
        assert!(state.badge_text().is_none());
    }
}

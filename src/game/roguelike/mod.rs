//! Roguelike run mode (御剑行·轮回).
//!
//! One "run" is: Title → chapter node map (randomly generated DAG) → per-node
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
pub mod scene;
pub mod screens;

use super::battle::{EncounterKind, EncounterZone};
use super::core::Rng;
use super::explore::MapKind;
use super::quest::BossKind;
use super::state::AppState;
use graph::NodeKind;

// ---------------------------------------------------------------------------
// Relics (法宝) — persistent passive items collected during a run
// ---------------------------------------------------------------------------

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

/// Chapter index: 0..=2 are the three story chapters, 3 is the finale.
pub const CHAPTER_COUNT: usize = 4;

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

pub const CHAPTERS: [ChapterDef; CHAPTER_COUNT] = [
    ChapterDef {
        title: "第一卷 · 桃溪村誓",
        maps: &[
            MapKind::Village,
            MapKind::Bamboo,
            MapKind::Cave,
            MapKind::MoonEchoCorridor,
        ],
        zones: &[EncounterZone::Village, EncounterZone::Bamboo],
        bosses: &[BossKind::MoonWraith],
        enemy_hp_mul: 0.75,
        enemy_atk_mul: 0.70,
        depth: 4,
    },
    ChapterDef {
        title: "第二卷 · 江雾疫火",
        maps: &[
            MapKind::RiverTown,
            MapKind::RiverReedBed,
            MapKind::PlagueVillage,
            MapKind::PlagueShrinePath,
        ],
        zones: &[EncounterZone::RiverTown, EncounterZone::PlagueVillage],
        bosses: &[BossKind::RiverDemon, BossKind::MiasmaRoot],
        enemy_hp_mul: 1.10,
        enemy_atk_mul: 1.00,
        depth: 4,
    },
    ChapterDef {
        title: "第三卷 · 京华南疆",
        maps: &[
            MapKind::Capital,
            MapKind::CapitalMansion,
            MapKind::MansionMirrorGallery,
            MapKind::SouthernRoad,
            MapKind::ThunderDrumPath,
        ],
        zones: &[EncounterZone::Capital, EncounterZone::SouthernRoad],
        bosses: &[BossKind::MirrorMinister, BossKind::ThunderQilin],
        enemy_hp_mul: 1.50,
        enemy_atk_mul: 1.30,
        depth: 4,
    },
    ChapterDef {
        title: "终卷 · 心渊照影",
        maps: &[MapKind::FinalSanctum, MapKind::DreamWaterway],
        zones: &[EncounterZone::FinalSanctum],
        bosses: &[BossKind::DreamEclipse],
        enemy_hp_mul: 1.85,
        enemy_atk_mul: 1.50,
        depth: 3,
    },
];

/// 境界突破 applied after clearing each chapter boss (story-driven power in
/// place of grinding).
pub const BREAKTHROUGH_HP: i32 = 22;
pub const BREAKTHROUGH_MP: i32 = 8;
pub const BREAKTHROUGH_ATK: i32 = 6;
pub const BREAKTHROUGH_DEF: i32 = 3;

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
    /// Map used for the previous stage (avoid immediate repeats).
    pub last_map: Option<MapKind>,
    /// The chapter's 「缘」 story beat still needs to be placed on a map.
    pub story_pending: bool,
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
    /// Set when the run ends; read by the Ending screen.
    pub outcome: Option<RunOutcome>,
    /// Totals for the ending screen.
    pub fights_won: u32,
    /// One-shot revive from 檀木符 has been consumed.
    pub revive_used: bool,
    /// 白狐三遇链:0 未遇,1/2/3 已推进到第几遇。
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
            last_map: None,
            story_pending: true,
            boss,
            relics: Vec::new(),
            daoxin: 0,
            qingyuan: 0,
            current_fight: None,
            card_shown: false,
            seen_events: Vec::new(),
            outcome: None,
            fights_won: 0,
            revive_used: false,
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
        self.story_pending = true;
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

    pub fn has_relic(&self, relic: Relic) -> bool {
        self.relics.contains(&relic)
    }

    /// Pick a random walkable map for this chapter, avoiding an immediate
    /// repeat of the previous stage's map.
    pub fn roll_map(&mut self, rng: &mut Rng) -> MapKind {
        let maps = self.chapter_def().maps;
        let mut pick = maps[rng.range(0, maps.len() as i32 - 1) as usize];
        if maps.len() > 1 {
            while Some(pick) == self.last_map {
                pick = maps[rng.range(0, maps.len() as i32 - 1) as usize];
            }
        }
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
        if self.fox_stage == 1 && self.chapter >= 1 && rng.chance(0.6) {
            self.fox_stage = 2;
            return if self.fox_kind > 0 {
                c::EV_FOX2_WARM
            } else {
                c::EV_FOX2_COLD
            };
        }
        if self.fox_stage == 2 && self.chapter >= 2 && rng.chance(0.7) {
            self.fox_stage = 3;
            return if self.fox_kind > 0 {
                c::EV_FOX3_WARM
            } else {
                c::EV_FOX3_COLD
            };
        }
        if self.qin_stage == 0 && self.chapter == 1 && rng.chance(0.5) {
            self.qin_stage = 1;
            return c::EV_QIN1;
        }
        if self.qin_stage == 1 && self.chapter >= 2 && rng.chance(0.6) {
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

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct RoguelikePlugin;

impl Plugin for RoguelikePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<event::RunDialogue>()
            .init_resource::<screens::RewardChoices>()
            .init_resource::<scene::InventoryOpen>()
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
                    scene::inventory_toggle,
                    scene::run_scene_movement,
                    scene::animate_hero,
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

        // 救了白狐 → 善缘;第二卷应触发「白衣回礼」。
        run.fox_kind = 1;
        run.chapter = 1;
        let mut second = None;
        for _ in 0..64 {
            let idx = run.draw_chain_or_event(&mut rng);
            if idx >= content::CHAIN_START {
                second = Some(idx);
                break;
            }
        }
        assert_eq!(second, Some(content::EV_FOX2_WARM));

        // 第三卷善缘收束于「狐仙赠丹」。
        run.chapter = 2;
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

    /// 无视白狐的一世,走向「白影避走」与「妖狐拦路」。
    #[test]
    fn fox_chain_cold_branch() {
        let mut rng = Rng::default();
        let mut run = RunState::new(&mut rng);
        run.fox_stage = 1;
        run.fox_kind = -1;
        run.chapter = 1;
        let mut second = None;
        for _ in 0..64 {
            let idx = run.draw_chain_or_event(&mut rng);
            if idx >= content::CHAIN_START && idx != content::EV_QIN1 && idx != content::EV_QIN2 {
                second = Some(idx);
                break;
            }
        }
        assert_eq!(second, Some(content::EV_FOX2_COLD));

        run.chapter = 2;
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

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
pub mod map_ui;
pub mod screens;

use super::battle::{EncounterKind, EncounterZone};
use super::core::Rng;
use super::quest::BossKind;
use super::state::AppState;
use graph::{NodeGraph, NodeKind};

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
}

pub const ALL_RELICS: [Relic; 14] = [
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
        zones: &[EncounterZone::Village, EncounterZone::Bamboo],
        bosses: &[BossKind::MoonWraith],
        enemy_hp_mul: 0.75,
        enemy_atk_mul: 0.70,
        depth: 4,
    },
    ChapterDef {
        title: "第二卷 · 江雾疫火",
        zones: &[EncounterZone::RiverTown, EncounterZone::PlagueVillage],
        bosses: &[BossKind::RiverDemon, BossKind::MiasmaRoot],
        enemy_hp_mul: 1.10,
        enemy_atk_mul: 1.00,
        depth: 4,
    },
    ChapterDef {
        title: "第三卷 · 京华南疆",
        zones: &[EncounterZone::Capital, EncounterZone::SouthernRoad],
        bosses: &[BossKind::MirrorMinister, BossKind::ThunderQilin],
        enemy_hp_mul: 1.50,
        enemy_atk_mul: 1.30,
        depth: 4,
    },
    ChapterDef {
        title: "终卷 · 心渊照影",
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
}

/// The whole roguelike run. Present ⇔ a run is active (battle systems use
/// this to tell run-mode battles from legacy Explore battles).
#[derive(Resource)]
pub struct RunState {
    pub chapter: usize,
    pub graph: NodeGraph,
    /// Node the player currently stands on (index into `graph.nodes`).
    pub position: usize,
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
    /// Set when the run ends; read by the Ending screen.
    pub outcome: Option<RunOutcome>,
    /// Totals for the ending screen.
    pub fights_won: u32,
    /// One-shot revive from 檀木符 has been consumed.
    pub revive_used: bool,
}

impl RunState {
    pub fn new(rng: &mut Rng) -> Self {
        let chapter = 0;
        let def = &CHAPTERS[chapter];
        let graph = NodeGraph::generate(def, rng);
        let boss = def.bosses[rng.range(0, def.bosses.len() as i32 - 1) as usize];
        Self {
            chapter,
            graph,
            position: usize::MAX, // not on any node yet: pick from entry layer
            boss,
            relics: Vec::new(),
            daoxin: 0,
            qingyuan: 0,
            current_fight: None,
            card_shown: false,
            outcome: None,
            fights_won: 0,
            revive_used: false,
        }
    }

    /// Advance to the next chapter, regenerating the node graph.
    pub fn next_chapter(&mut self, rng: &mut Rng) {
        self.chapter += 1;
        let def = &CHAPTERS[self.chapter];
        self.graph = NodeGraph::generate(def, rng);
        self.boss = def.bosses[rng.range(0, def.bosses.len() as i32 - 1) as usize];
        self.position = usize::MAX;
        self.card_shown = false;
    }

    pub fn chapter_def(&self) -> &'static ChapterDef {
        &CHAPTERS[self.chapter]
    }

    pub fn has_relic(&self, relic: Relic) -> bool {
        self.relics.contains(&relic)
    }

    /// Pick a random encounter zone for this chapter.
    pub fn roll_zone(&self, rng: &mut Rng) -> EncounterZone {
        let zones = self.chapter_def().zones;
        zones[rng.range(0, zones.len() as i32 - 1) as usize]
    }

    /// Nodes reachable from the current position (entry layer if none).
    pub fn reachable(&self) -> Vec<usize> {
        if self.position == usize::MAX {
            self.graph.entry_nodes()
        } else {
            self.graph.nodes[self.position].next.clone()
        }
    }

    // --- relic-driven battle modifiers, summed over owned relics ---

    pub fn attack_bonus(&self) -> i32 {
        if self.has_relic(Relic::SwordTassel) {
            5
        } else {
            0
        }
    }

    pub fn spell_bonus(&self) -> i32 {
        if self.has_relic(Relic::SwordSutra) {
            9
        } else {
            0
        }
    }

    pub fn spell_cost_delta(&self) -> i32 {
        if self.has_relic(Relic::SpiritPendant) {
            -2
        } else {
            0
        }
    }

    pub fn potion_bonus(&self) -> i32 {
        if self.has_relic(Relic::JadeVial) {
            25
        } else {
            0
        }
    }

    pub fn incoming_reduction(&self) -> i32 {
        if self.has_relic(Relic::TortoiseArmor) {
            3
        } else {
            0
        }
    }

    pub fn flee_always(&self) -> bool {
        self.has_relic(Relic::CloudSleeves)
    }

    pub fn on_kill_heal(&self) -> i32 {
        if self.has_relic(Relic::BloodBead) {
            12
        } else {
            0
        }
    }

    pub fn opening_strike(&self) -> i32 {
        if self.has_relic(Relic::ThunderDrum) {
            15
        } else {
            0
        }
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
            .init_resource::<map_ui::MapCursor>()
            .add_systems(OnEnter(AppState::Title), screens::spawn_title)
            .add_systems(
                Update,
                screens::title_input.run_if(in_state(AppState::Title)),
            )
            .add_systems(OnEnter(AppState::NodeMap), map_ui::spawn_node_map)
            .add_systems(
                Update,
                (
                    event::run_dialogue_input,
                    map_ui::node_map_input,
                    map_ui::update_node_cursor,
                    map_ui::update_run_hud,
                    event::update_run_dialogue_ui,
                )
                    .chain()
                    .run_if(in_state(AppState::NodeMap)),
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

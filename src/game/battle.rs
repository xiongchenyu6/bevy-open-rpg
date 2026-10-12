use bevy::prelude::*;
mod enemy_animation;
#[cfg(test)]
mod sim_tests;
mod tactics;
mod ui;
use tactics::{EnemyStyle, StrikeKind, Tactics};

use super::animation::{self, AnimationAssets, AnimationClip, SpriteAnimation};
use super::core::{GameFont, Intent, PlayerStats, Rng};
use super::cutout::{COMPANION_BATTLE_SIZE, companion_cutout_path};
use super::lighting::{self, LightingAssets};
use super::quest::{
    BondBonus, BondScene, BossKind, CampBonus, CampScene, Chapter, Companion, CompanionScene,
    QuestLog, ShrineBlessing, SideQuest,
};
use super::roguelike::{FightRank, RunBattleMods, RunCampTactic, RunOutcome, RunState};
use super::skill_vfx::{self, SkillVfxAssets, StatusBurstKind, StatusVfxState};
use super::state::AppState;

const MENU: [&str; 6] = ["攻击", "御守", "仙术", "合击", "物品", "逃跑"];
const COMBO_COST: i32 = 8;

// ---------------------------------------------------------------------------
// Data
// ---------------------------------------------------------------------------

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncounterZone {
    Village,
    Bamboo,
    Cave,
    RiverTown,
    PlagueVillage,
    Capital,
    SouthernRoad,
    FinalSanctum,
}

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PendingEncounter {
    pub zone: EncounterZone,
    pub kind: EncounterKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncounterKind {
    Random,
    Boss(BossKind),
}

struct EnemyDef {
    name: &'static str,
    max_hp: i32,
    atk: i32,
    def: i32,
    image: &'static str,
    size: f32,
    light: [f32; 4],
    exp: u32,
}

#[derive(Clone, Copy)]
struct BattleBackdrop {
    light: Color,
}

const ENEMIES: [EnemyDef; 9] = [
    EnemyDef {
        name: "碧水妖蛇",
        max_hp: 38,
        atk: 12,
        def: 3,
        image: "creatures/ai_water_serpent.png",
        size: 210.0,
        light: [0.34, 0.74, 1.0, 0.28],
        exp: 12,
    },
    EnemyDef {
        name: "苔甲石卫",
        max_hp: 52,
        atk: 15,
        def: 5,
        image: "creatures/ai_stone_guardian.png",
        size: 214.0,
        light: [0.48, 1.0, 0.56, 0.24],
        exp: 18,
    },
    EnemyDef {
        name: "洞翼夜魇",
        max_hp: 46,
        atk: 18,
        def: 4,
        image: "creatures/ai_cave_bat.png",
        size: 210.0,
        light: [0.72, 0.46, 1.0, 0.24],
        exp: 22,
    },
    EnemyDef {
        name: "火魄灯灵",
        max_hp: 36,
        atk: 20,
        def: 2,
        image: "creatures/ai_fire_wisp.png",
        size: 196.0,
        light: [1.0, 0.45, 0.12, 0.36],
        exp: 20,
    },
    EnemyDef {
        name: "雷羽妖鹏",
        max_hp: 58,
        atk: 21,
        def: 4,
        image: "creatures/ai_thunder_roc.png",
        size: 230.0,
        light: [0.45, 0.82, 1.0, 0.34],
        exp: 28,
    },
    EnemyDef {
        name: "血刃螳妖",
        max_hp: 62,
        atk: 23,
        def: 5,
        image: "creatures/ai_blood_mantis.png",
        size: 218.0,
        light: [1.0, 0.18, 0.14, 0.28],
        exp: 30,
    },
    EnemyDef {
        name: "影面剑魇",
        max_hp: 68,
        atk: 25,
        def: 6,
        image: "creatures/ai_shadow_swordsman.png",
        size: 226.0,
        light: [0.72, 0.56, 1.0, 0.30],
        exp: 34,
    },
    EnemyDef {
        name: "苍苔灵龟",
        max_hp: 74,
        atk: 18,
        def: 8,
        image: "creatures/ai_moss_turtle.png",
        size: 222.0,
        light: [0.46, 1.0, 0.62, 0.26],
        exp: 32,
    },
    EnemyDef {
        name: "莲华水魅",
        max_hp: 44,
        atk: 17,
        def: 3,
        image: "creatures/ai_lotus_spirit.png",
        size: 206.0,
        light: [1.0, 0.56, 0.82, 0.30],
        exp: 24,
    },
];

const VILLAGE_ENCOUNTERS: [usize; 3] = [0, 1, 3];
const BAMBOO_ENCOUNTERS: [usize; 4] = [4, 5, 6, 7];
const CAVE_ENCOUNTERS: [usize; 4] = [2, 6, 7, 8];
const RIVER_TOWN_ENCOUNTERS: [usize; 4] = [0, 3, 5, 8];
const PLAGUE_VILLAGE_ENCOUNTERS: [usize; 4] = [1, 3, 5, 7];
const CAPITAL_ENCOUNTERS: [usize; 4] = [4, 5, 6, 8];
const SOUTHERN_ROAD_ENCOUNTERS: [usize; 4] = [2, 4, 6, 7];
const FINAL_SANCTUM_ENCOUNTERS: [usize; 4] = [0, 2, 6, 8];

const MOUNTAIN_FIEND_BOSS: EnemyDef = EnemyDef {
    name: "赤鬼山妖",
    max_hp: 112,
    atk: 23,
    def: 6,
    image: "creatures/boss_mountain_fiend.png",
    size: 280.0,
    light: [1.0, 0.34, 0.16, 0.40],
    exp: 52,
};

const MOON_WRAITH_BOSS: EnemyDef = EnemyDef {
    name: "月魄妖",
    max_hp: 128,
    atk: 26,
    def: 7,
    image: "creatures/boss_moon_wraith.png",
    size: 286.0,
    light: [0.95, 0.55, 1.0, 0.42],
    exp: 60,
};

const RIVER_DEMON_BOSS: EnemyDef = EnemyDef {
    name: "河魇蛟",
    max_hp: 156,
    atk: 29,
    def: 8,
    image: "creatures/boss_river_demon.png",
    size: 292.0,
    light: [0.30, 0.82, 1.0, 0.46],
    exp: 82,
};

const MIASMA_ROOT_BOSS: EnemyDef = EnemyDef {
    name: "瘴母根",
    max_hp: 184,
    atk: 32,
    def: 10,
    image: "creatures/boss_miasma_root.png",
    size: 300.0,
    light: [0.50, 0.90, 0.42, 0.44],
    exp: 104,
};

const MIRROR_MINISTER_BOSS: EnemyDef = EnemyDef {
    name: "照影国师",
    max_hp: 216,
    atk: 35,
    def: 12,
    image: "creatures/boss_mirror_minister.png",
    size: 290.0,
    light: [0.72, 0.62, 1.0, 0.48],
    exp: 132,
};

const THUNDER_QILIN_BOSS: EnemyDef = EnemyDef {
    name: "雷麟",
    max_hp: 252,
    atk: 39,
    def: 13,
    image: "creatures/boss_thunder_qilin.png",
    size: 296.0,
    light: [0.52, 0.88, 1.0, 0.52],
    exp: 160,
};

const DREAM_ECLIPSE_BOSS: EnemyDef = EnemyDef {
    name: "宿命水影",
    max_hp: 288,
    atk: 43,
    def: 15,
    image: "creatures/boss_dream_eclipse.png",
    size: 300.0,
    light: [0.46, 0.78, 1.0, 0.56],
    exp: 190,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Menu,
    PlayerActing,
    EnemyActing,
    Won,
    Lost,
    Fled,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PlayerAction {
    Attack,
    Guard,
    Spell,
    Combo,
    Item,
    Flee,
}

/// 敌人下一手的预告(杀戮尖塔式意图):回合开始就亮出来,
/// 玩家据此决定是抢输出还是御守卸力。
/// One-time contextual battle reminders, preserved throughout the run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BattleLesson {
    Charge,
    Heavy,
    Counter,
    Break,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EnemyIntent {
    Charge,
    Brace,
    Special,
    /// 普通攻击。
    Strike,
    /// 蓄力重击(约 1.8×)——御守的最佳时机。
    Heavy,
    /// 凝气:不攻击,回血并提升防御——抢输出的最佳时机。
    Gather,
    /// 摄灵:较轻的一击,但吸走灵力。
    Drain,
}

impl EnemyIntent {
    fn describe(self) -> &'static str {
        match self {
            EnemyIntent::Charge => "蓄力 · 下手强袭；现在施压可打断",
            EnemyIntent::Brace => "架甲反击 · 普攻减半；仙术可穿透",
            EnemyIntent::Special => "秘法将至 · 御守反击 / 破势打断",
            EnemyIntent::Strike => "意图:张爪欲击",
            EnemyIntent::Heavy => "重击将至 · 御守反击或击破架势",
            EnemyIntent::Gather => "凝气回息 · 恢复气血，这回合不攻击",
            EnemyIntent::Drain => "意图:虚影缠绕,欲摄灵力",
        }
    }
}

/// 气势上限:攻击/仙术各叠 1 层,满层可施展绝技·剑气爆发。
const MOMENTUM_MAX: u32 = 3;
/// 御守回合回复的灵力。
const GUARD_MP_RESTORE: i32 = 4;

struct EnemyInstance {
    name: String,
    hp: i32,
    max_hp: i32,
    atk: i32,
    def: i32,
    exp: u32,
}

#[derive(Resource)]
struct BattleState {
    enemy: EnemyInstance,
    encounter_kind: EncounterKind,
    phase: Phase,
    timer: f32,
    enemy_turns: u32,
    tactics: Tactics,
    executing_intent: EnemyIntent,
    enemy_attacked: bool,
    phase_duration: f32,
    menu_index: usize,
    player_action: Option<PlayerAction>,
    intent: EnemyIntent,
    guarding: bool,
    momentum: u32,
    /// Boss 半血后的真身阶段:变身宣言 + 专属强化机制。
    boss_phase2: bool,
    /// Finite shared pool for Gather, phase regeneration, and signature healing.
    boss_heal_remaining: i32,
    /// 终章水影的分歧:true = 以情乱心(情缘压道心的一世)。
    eclipse_heart: bool,
    /// 雷引纹:本战的玩家首次攻击已经打出。
    hex_first_hit_used: bool,
    /// 二十式仙术秘卷子菜单:打开状态与光标(页由光标推出)。
    skill_menu: bool,
    skill_cursor: usize,
    /// 「护」形态结成的护罩,先于气血抵伤。
    player_shield: i32,
    /// 中毒/灼烧/冰冻/震慑的统一状态结算。
    statuses: EnemyStatuses,
    message: String,
    blessing: Option<ShrineBlessing>,
    camp_bonus: Option<CampBonus>,
    run_camp_tactic: Option<RunCampTactic>,
    bond_bonus: Option<BondBonus>,
}

/// Read-only battle state for deterministic traversal and accessibility
/// drivers. Inputs still pass through the normal command menu.
#[derive(Resource, Default)]
pub struct BattleAutomationView {
    pub menu_open: bool,
    pub menu_index: usize,
    pub skill_menu_open: bool,
    pub skill_cursor: usize,
    pub enemy_hp: i32,
    pub enemy_max_hp: i32,
    pub enemy_heavy_intent: bool,
    pub enemy_charging: bool,
    pub enemy_bracing: bool,
    pub pressure: u8,
    pub pressure_limit: u8,
    pub riposte_ready: bool,
    pub exposed: bool,
    pub breaks: u32,
    pub counters: u32,
    pub spell_cost: i32,
    pub momentum: u32,
    pub poison_stacks: u8,
    pub burn_turns: u8,
    pub frozen: bool,
    pub stunned: bool,
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
struct BattleHero;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BattleCompanionKind {
    Linger,
    SwordSister,
    SpiritWitch,
}

#[derive(Component)]
struct BattleCompanion {
    kind: BattleCompanionKind,
    origin: Vec3,
    age: f32,
}

#[derive(Component)]
struct BattleEnemy;

#[derive(Component)]
struct BattleEnemyMotion {
    origin: Vec3,
    age: f32,
}

#[derive(Component)]
struct BattleEffect {
    age: f32,
    duration: f32,
    start_scale: f32,
    end_scale: f32,
    spin: f32,
}

impl BattleEffect {
    fn new(duration: f32, start_scale: f32, end_scale: f32, spin: f32) -> Self {
        Self {
            age: 0.0,
            duration,
            start_scale,
            end_scale,
            spin,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct SpellImpactProfile {
    core_size: f32,
    core_duration: f32,
    core_start_scale: f32,
    core_end_scale: f32,
    ring_size: f32,
    ring_duration: f32,
    ring_end_scale: f32,
    flash_radius: f32,
    flash_alpha: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SpellImpactElement {
    Village,
    Moon,
    River,
    Plague,
    Mirror,
    Thunder,
    Dream,
}

#[derive(Clone, Copy, Debug)]
struct SpellImpactShard {
    offset: Vec2,
    size: Vec2,
    rotation: f32,
    duration: f32,
    start_scale: f32,
    end_scale: f32,
    spin: f32,
    color: Color,
}

#[derive(Clone, Copy, Debug)]
struct SpellDamageText {
    amount: i32,
    offset: Vec2,
    velocity: Vec2,
    size: f32,
}

#[derive(Component)]
struct FloatingCombatText {
    age: f32,
    duration: f32,
    velocity: Vec2,
    color: Color,
}

impl FloatingCombatText {
    fn new(duration: f32, velocity: Vec2, color: Color) -> Self {
        Self {
            age: 0.0,
            duration,
            velocity,
            color,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct EnemyAttackResult {
    damage: i32,
    strong: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct TimedDamageStatus {
    damage: i32,
    turns: u8,
    stacks: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct EnemyStatuses {
    poison: TimedDamageStatus,
    burn: TimedDamageStatus,
    frozen: bool,
    stunned: bool,
    freeze_immunity: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct EnemyTurnResult {
    attack: Option<EnemyAttackResult>,
    poison_damage: i32,
    burn_damage: i32,
    control: Option<StatusBurstKind>,
}

pub const ENEMY_POS: Vec3 = Vec3::new(300.0, 72.0, 1.0);
pub const HERO_POS: Vec3 = Vec3::new(-400.0, 52.0, 1.0);
const LINGER_POS: Vec3 = Vec3::new(HERO_POS.x + 130.0, HERO_POS.y + 8.0, 0.9);
const SWORD_SISTER_POS: Vec3 = Vec3::new(HERO_POS.x + 245.0, HERO_POS.y - 6.0, 0.92);
const SPIRIT_WITCH_POS: Vec3 = Vec3::new(HERO_POS.x + 340.0, HERO_POS.y + 16.0, 0.88);

fn spawn_battle_companion_sprite(
    commands: &mut Commands,
    asset_server: &AssetServer,
    companion: Companion,
    position: Vec3,
) -> Entity {
    let actor = commands
        .spawn((
            Sprite {
                image: asset_server.load(companion_cutout_path(companion)),
                custom_size: Some(Vec2::splat(COMPANION_BATTLE_SIZE)),
                ..default()
            },
            Transform::from_translation(position),
            DespawnOnExit(AppState::Battle),
        ))
        .id();
    let shadow = super::ui::contact_shadow(
        commands,
        asset_server,
        Vec3::new(position.x, position.y - 66.0, 0.65),
        82.0,
        AppState::Battle,
    );
    commands.entity(shadow).insert(super::ui::ShadowAnchor {
        actor,
        offset: Vec2::new(0.0, -66.0),
    });
    actor
}

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BattleAutomationView>()
            .add_systems(OnEnter(AppState::Battle), spawn_battle)
            .add_systems(
                Update,
                (
                    ui::menu_pointer_input,
                    battle_input,
                    battle_tick,
                    sync_status_vfx_state,
                    update_tactical_feedback,
                    update_battle_animations,
                    update_battle_effects,
                    update_floating_combat_text,
                    battle_lessons,
                    ui::update,
                    sync_battle_automation_view,
                )
                    .chain()
                    .run_if(in_state(AppState::Battle)),
            );
    }
}

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

fn spawn_battle(
    mut commands: Commands,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    font: Res<GameFont>,
    asset_server: Res<AssetServer>,
    lights: Res<LightingAssets>,
    anims: Res<AnimationAssets>,
    mut quest: ResMut<QuestLog>,
    mut rng: ResMut<Rng>,
    mut stats: ResMut<PlayerStats>,
    mut vfx_status: ResMut<StatusVfxState>,
    encounter: Option<Res<PendingEncounter>>,
    mut run: Option<ResMut<RunState>>,
    mods: Option<Res<RunBattleMods>>,
) {
    *vfx_status = StatusVfxState::default();
    let zone = encounter
        .as_ref()
        .map(|encounter| encounter.zone)
        .unwrap_or(EncounterZone::Village);
    let kind = encounter
        .as_ref()
        .map(|encounter| encounter.kind)
        .unwrap_or(EncounterKind::Random);
    let teaching_opening = matches!(kind, EncounterKind::Random)
        && run.as_ref().is_some_and(|r| {
            r.fights_won == 0 && !r.battle_lessons_seen.contains(&BattleLesson::Charge)
        });
    let def = if teaching_opening {
        &ENEMIES[0]
    } else {
        choose_enemy_without_repeat(
            zone,
            kind,
            run.as_ref().and_then(|r| r.last_enemy_image),
            &mut rng,
        )
    };
    if matches!(kind, EncounterKind::Random) {
        if let Some(run) = run.as_mut() {
            run.last_enemy_image = Some(def.image);
        }
    }
    let mut enemy = scaled_enemy(def, mods.as_deref(), &mut rng);

    let blessing = quest.take_shrine_blessing();
    let camp_bonus = quest.take_camp_bonus();
    let run_camp_tactic = run.as_mut().and_then(|run| run.take_camp_tactic());
    let bond_bonus = quest.take_bond_bonus();
    let mut message = format!("一只 {} 拦住了去路！", enemy.name);
    if let Some(run) = run.as_ref() {
        apply_run_chapter_vow_to_boss(run, kind, &mut enemy, &mut message);
        apply_run_route_guidance_to_boss(run, kind, &mut stats, &mut enemy, &mut message);
    } else if let EncounterKind::Boss(boss) = kind {
        apply_legacy_boss_preparation_to_boss(boss, &quest, &mut stats, &mut enemy, &mut message);
    }

    // Relic effects that trigger at battle start (run mode only).
    if let Some(run) = run.as_ref() {
        let strike = run.opening_strike();
        if strike > 0 {
            enemy.hp = (enemy.hp - strike).max(1);
            message.push_str(&format!(
                "\n【雷泽鼓】开战惊雷落下,敌人受了 {strike} 点伤！"
            ));
        }
        let debuff = run.enemy_atk_debuff();
        if debuff > 0 {
            enemy.atk = (enemy.atk - debuff).max(1);
            message.push_str(&format!("\n【阴阳镜】镜光一晃,敌人攻势弱了 {debuff} 分。"));
        }
        let start_heal = run.battle_start_heal();
        if start_heal > 0 && stats.hp < stats.max_hp {
            let healed = start_heal.min(stats.max_hp - stats.hp);
            stats.hp += healed;
            message.push_str(&format!("\n【息壤袋】土息养身,回复 {healed} 点气血。"));
        }
        let shield = run.bond_shield();
        if shield > 0 && stats.hp < stats.max_hp {
            let healed = shield.min(stats.max_hp - stats.hp);
            stats.hp += healed;
            message.push_str(&format!(
                "\n【情缘·灵息罩】灵儿抢先布下灵息,回复 {healed} 点气血。"
            ));
        }
        let tithe_mp = run.hex_battle_start_mp_loss();
        if tithe_mp > 0 {
            stats.mp = (stats.mp - tithe_mp).max(0);
            message.push_str(&format!("\n【血偿纹】妖血索价,先失 {tithe_mp} 点灵力。"));
        }
    }
    if let Some(bond_bonus) = bond_bonus {
        message.push_str(&format!(
            "\n【羁绊】{}生效：{}",
            bond_bonus.name(),
            bond_bonus.battle_line()
        ));
    }
    if let Some(blessing) = blessing {
        message.push_str(&format!(
            "\n【祝福】{}生效：{}",
            blessing.name(),
            blessing.battle_line()
        ));
    }
    if let Some(camp_bonus) = camp_bonus {
        message.push_str(&format!(
            "\n【营地】{}生效：{}",
            camp_bonus.name(),
            camp_bonus.battle_line()
        ));
    }
    if let Some(tactic) = run_camp_tactic {
        message.push_str(&format!(
            "\n【营策】{}生效：{}",
            tactic.name(),
            tactic.battle_line()
        ));
        if let Some(healed) = apply_run_camp_start_heal(tactic, &mut stats) {
            message.push_str(&format!("\n【营策】火边余息护身,回复 {healed} 点气血。"));
        }
        if let Some(restored) = apply_run_camp_start_mana(tactic, &mut stats) {
            message.push_str(&format!("\n【营策】灵纹回稳,回复 {restored} 点灵力。"));
        }
    }
    let run_ref = run.as_deref();

    let boss_heal_budget = if matches!(kind, EncounterKind::Boss(_)) {
        enemy.max_hp / 4
    } else {
        0
    };
    commands.insert_resource(BattleState {
        message,
        enemy,
        encounter_kind: kind,
        phase: Phase::Menu,
        timer: 0.0,
        enemy_turns: 0,
        tactics: Tactics::new(enemy_style(def), matches!(kind, EncounterKind::Boss(_))),
        executing_intent: EnemyIntent::Strike,
        enemy_attacked: false,
        phase_duration: 0.8,
        menu_index: 0,
        player_action: None,
        intent: if teaching_opening {
            EnemyIntent::Charge
        } else {
            initial_intent(def, kind)
        },
        guarding: false,
        boss_phase2: false,
        boss_heal_remaining: boss_heal_budget,
        eclipse_heart: false,
        hex_first_hit_used: false,
        skill_menu: false,
        skill_cursor: 0,
        player_shield: 0,
        statuses: EnemyStatuses::default(),
        momentum: 0,
        blessing,
        camp_bonus,
        run_camp_tactic,
        bond_bonus,
    });

    spawn_battle_backdrop(&mut commands, &asset_server, &lights, zone);

    // A single painted body keeps edges intact during breathing and recoil.
    lighting::spawn_light(
        &mut commands,
        &lights,
        Vec3::new(ENEMY_POS.x, ENEMY_POS.y - 2.0, 0.5),
        360.0,
        Color::srgba(def.light[0], def.light[1], def.light[2], def.light[3]),
        AppState::Battle,
    );
    let authored_sprite = enemy_animation::sprite_for(def, &asset_server, &mut atlas_layouts);
    let animated = authored_sprite.is_some();
    let mut sprite = authored_sprite.unwrap_or_else(|| Sprite {
        image: asset_server.load(enemy_primary_image(def)),
        custom_size: Some(Vec2::splat(enemy_primary_size(def))),
        ..default()
    });
    sprite.color = enemy_primary_color(Phase::Menu, 0.0);
    let enemy_entity = commands
        .spawn((
            BattleEnemy,
            sprite,
            BattleEnemyMotion {
                origin: ENEMY_POS,
                age: rng.range(0, 100) as f32 * 0.07,
            },
            Transform::from_translation(ENEMY_POS),
            DespawnOnExit(AppState::Battle),
        ))
        .id();
    if animated {
        commands
            .entity(enemy_entity)
            .insert(enemy_animation::EnemyAtlas);
    }

    // Player sprite
    lighting::spawn_light(
        &mut commands,
        &lights,
        Vec3::new(HERO_POS.x, HERO_POS.y, 0.5),
        300.0,
        Color::srgba(1.0, 0.84, 0.44, 0.26),
        AppState::Battle,
    );
    let hero = animation::spawn_animated_sprite(
        &mut commands,
        &anims,
        AnimationClip::HeroIdle,
        HERO_POS,
        Vec2::splat(220.0),
        AppState::Battle,
    );
    commands.entity(hero).insert(BattleHero);
    let shadow = super::ui::contact_shadow(
        &mut commands,
        &asset_server,
        Vec3::new(HERO_POS.x, HERO_POS.y - 92.0, 0.65),
        104.0,
        AppState::Battle,
    );
    commands.entity(shadow).insert(super::ui::ShadowAnchor {
        actor: hero,
        offset: Vec2::new(0.0, -92.0),
    });
    super::ui::contact_shadow(
        &mut commands,
        &asset_server,
        Vec3::new(ENEMY_POS.x, ENEMY_POS.y - def.size * 0.40, 0.65),
        def.size * 0.58,
        AppState::Battle,
    );

    if battle_companion_visible(&quest, run_ref, Companion::Linger) {
        lighting::spawn_light(
            &mut commands,
            &lights,
            Vec3::new(LINGER_POS.x, LINGER_POS.y, 0.5),
            250.0,
            Color::srgba(0.72, 0.92, 1.0, 0.24),
            AppState::Battle,
        );
        let companion = spawn_battle_companion_sprite(
            &mut commands,
            &asset_server,
            Companion::Linger,
            LINGER_POS,
        );
        commands.entity(companion).insert(BattleCompanion {
            kind: BattleCompanionKind::Linger,
            origin: LINGER_POS,
            age: 0.0,
        });
    }

    if battle_companion_visible(&quest, run_ref, Companion::SwordSister) {
        lighting::spawn_light(
            &mut commands,
            &lights,
            Vec3::new(SWORD_SISTER_POS.x, SWORD_SISTER_POS.y, 0.5),
            250.0,
            Color::srgba(1.0, 0.58, 0.36, 0.24),
            AppState::Battle,
        );
        let companion = spawn_battle_companion_sprite(
            &mut commands,
            &asset_server,
            Companion::SwordSister,
            SWORD_SISTER_POS,
        );
        commands.entity(companion).insert(BattleCompanion {
            kind: BattleCompanionKind::SwordSister,
            origin: SWORD_SISTER_POS,
            age: 0.0,
        });
    }

    if battle_companion_visible(&quest, run_ref, Companion::SpiritWitch) {
        lighting::spawn_light(
            &mut commands,
            &lights,
            Vec3::new(SPIRIT_WITCH_POS.x, SPIRIT_WITCH_POS.y, 0.5),
            255.0,
            Color::srgba(0.48, 1.0, 0.62, 0.23),
            AppState::Battle,
        );
        let companion = spawn_battle_companion_sprite(
            &mut commands,
            &asset_server,
            Companion::SpiritWitch,
            SPIRIT_WITCH_POS,
        );
        commands.entity(companion).insert(BattleCompanion {
            kind: BattleCompanionKind::SpiritWitch,
            origin: SPIRIT_WITCH_POS,
            age: 0.0,
        });
    }

    ui::spawn(&mut commands, &font, &asset_server);
}

fn apply_run_chapter_vow_to_boss(
    run: &RunState,
    kind: EncounterKind,
    enemy: &mut EnemyInstance,
    message: &mut String,
) {
    if !matches!(kind, EncounterKind::Boss(_)) {
        return;
    }
    let Some(vow) = run.current_chapter_vow() else {
        return;
    };
    let hp_mul = vow.boss_hp_multiplier();
    let atk_mul = vow.boss_atk_multiplier();
    if hp_mul < 1.0 {
        enemy.max_hp = ((enemy.max_hp as f32 * hp_mul).round() as i32).max(1);
        enemy.hp = enemy.hp.min(enemy.max_hp);
    }
    if atk_mul < 1.0 {
        enemy.atk = ((enemy.atk as f32 * atk_mul).round() as i32).max(1);
    }
    message.push_str(&format!("\n【本卷誓记·{}】{}", vow.name(), vow.boss_line()));
}

fn apply_run_route_guidance_to_boss(
    run: &RunState,
    kind: EncounterKind,
    stats: &mut PlayerStats,
    enemy: &mut EnemyInstance,
    message: &mut String,
) {
    if !matches!(kind, EncounterKind::Boss(_)) {
        return;
    }
    if run.route_boss_preparation_rank() == 0 {
        return;
    }

    let hp_mul = run.route_boss_hp_multiplier();
    let atk_mul = run.route_boss_atk_multiplier();
    if hp_mul < 1.0 {
        enemy.max_hp = ((enemy.max_hp as f32 * hp_mul).round() as i32).max(1);
        enemy.hp = enemy.hp.min(enemy.max_hp);
    }
    if atk_mul < 1.0 {
        enemy.atk = ((enemy.atk as f32 * atk_mul).round() as i32).max(1);
    }

    let (hp_pct, mp) = run.route_boss_restore();
    let heal = if hp_pct > 0 && stats.hp < stats.max_hp {
        let amount = (stats.max_hp * hp_pct / 100).max(1);
        let healed = amount.min(stats.max_hp - stats.hp);
        stats.hp += healed;
        healed
    } else {
        0
    };
    let restored = if mp > 0 && stats.mp < stats.max_mp {
        let restored = mp.min(stats.max_mp - stats.mp);
        stats.mp += restored;
        restored
    } else {
        0
    };

    let mut line = format!(
        "\n【本卷路人照应·{}】{}",
        run.route_boss_preparation_label(),
        run.route_boss_preparation_summary()
    );
    if heal > 0 || restored > 0 {
        line.push_str(&format!("；开战前补给 气血 +{heal} 灵力 +{restored}。"));
    } else {
        line.push('。');
    }
    message.push_str(&line);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LegacyBossPreparationState {
    Prepared,
    Partial,
    Strained,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LegacyBossPreparation {
    state: LegacyBossPreparationState,
    completed: usize,
    total: usize,
    hp_loss: i32,
    atk_delta: i32,
    hp_restore: i32,
    mp_restore: i32,
    pressure_damage: i32,
}

fn legacy_boss_chapter(boss: BossKind) -> Chapter {
    match boss {
        BossKind::MountainFiend => Chapter::VillageOath,
        BossKind::MoonWraith => Chapter::MoonCave,
        BossKind::RiverDemon => Chapter::RiverMedicine,
        BossKind::MiasmaRoot => Chapter::PlagueRain,
        BossKind::MirrorMinister => Chapter::CapitalMirror,
        BossKind::ThunderQilin => Chapter::SouthernThunder,
        BossKind::DreamEclipse => Chapter::FinalDream,
    }
}

fn legacy_boss_chapter_label(chapter: Chapter) -> &'static str {
    match chapter {
        Chapter::VillageOath => "余杭村",
        Chapter::MoonCave => "水月洞天",
        Chapter::RiverMedicine => "江岸药庐",
        Chapter::PlagueRain => "瘴雨村",
        Chapter::CapitalMirror => "云都府邸",
        Chapter::SouthernThunder => "南疆灵道",
        Chapter::FinalDream => "灵渊终门",
    }
}

fn legacy_boss_chapter_rank(chapter: Chapter) -> i32 {
    match chapter {
        Chapter::VillageOath => 0,
        Chapter::MoonCave => 1,
        Chapter::RiverMedicine => 2,
        Chapter::PlagueRain => 3,
        Chapter::CapitalMirror => 4,
        Chapter::SouthernThunder => 5,
        Chapter::FinalDream => 6,
    }
}

fn legacy_boss_care_scenes(chapter: Chapter) -> (BondScene, CampScene) {
    match chapter {
        Chapter::VillageOath => (BondScene::VillageFirstNight, CampScene::VillageHearth),
        Chapter::MoonCave => (BondScene::MoonCavePromise, CampScene::MoonCavePool),
        Chapter::RiverMedicine => (BondScene::RiverLampWish, CampScene::RiverTownInn),
        Chapter::PlagueRain => (BondScene::PlagueRainShelter, CampScene::PlagueSickroom),
        Chapter::CapitalMirror => (BondScene::CapitalRooftop, CampScene::CapitalSafehouse),
        Chapter::SouthernThunder => (BondScene::SouthernRoadOath, CampScene::SouthernCampfire),
        Chapter::FinalDream => (BondScene::FinalGateQuiet, CampScene::FinalStillWater),
    }
}

fn legacy_boss_commissions(chapter: Chapter) -> [SideQuest; 2] {
    match chapter {
        Chapter::VillageOath => [SideQuest::VillageTrail, SideQuest::VillageHerbs],
        Chapter::MoonCave => [SideQuest::MoonCaveCrystals, SideQuest::MoonCaveEchoes],
        Chapter::RiverMedicine => [SideQuest::RiverLanterns, SideQuest::RiverCargo],
        Chapter::PlagueRain => [SideQuest::PlagueRelief, SideQuest::PlagueMedicine],
        Chapter::CapitalMirror => [SideQuest::CapitalPatrol, SideQuest::CapitalRumors],
        Chapter::SouthernThunder => [SideQuest::SouthernThunder, SideQuest::SouthernDrums],
        Chapter::FinalDream => [SideQuest::FinalDreamEchoes, SideQuest::FinalHomewardVows],
    }
}

fn legacy_boss_companion_scenes(chapter: Chapter) -> &'static [CompanionScene] {
    const NONE: [CompanionScene; 0] = [];
    const TRAIL: [CompanionScene; 1] = [CompanionScene::SwordSisterTrailGuard];
    const CAPITAL: [CompanionScene; 1] = [CompanionScene::SwordSisterCapitalMirror];
    const SOUTHERN: [CompanionScene; 1] = [CompanionScene::SpiritWitchSouthernTotem];
    const FINAL: [CompanionScene; 2] = [
        CompanionScene::SwordSisterFinalReturn,
        CompanionScene::SpiritWitchFinalVow,
    ];

    match chapter {
        Chapter::VillageOath | Chapter::MoonCave => &TRAIL,
        Chapter::RiverMedicine | Chapter::PlagueRain => &NONE,
        Chapter::CapitalMirror => &CAPITAL,
        Chapter::SouthernThunder => &SOUTHERN,
        Chapter::FinalDream => &FINAL,
    }
}

fn legacy_boss_preparation(
    boss: BossKind,
    quest: &QuestLog,
    enemy: &EnemyInstance,
) -> LegacyBossPreparation {
    let chapter = legacy_boss_chapter(boss);
    let (bond_scene, camp_scene) = legacy_boss_care_scenes(chapter);
    let care_ready = quest.has_seen_bond_scene(bond_scene) && quest.has_seen_camp_scene(camp_scene);
    let commissions = legacy_boss_commissions(chapter);
    let commissions_ready = commissions
        .iter()
        .all(|side| quest.is_side_quest_completed(*side));
    let companion_scenes = legacy_boss_companion_scenes(chapter);
    let companion_done = companion_scenes
        .iter()
        .filter(|scene| quest.has_seen_companion_scene(**scene))
        .count();
    let total = 2 + companion_scenes.len();
    let completed = usize::from(care_ready) + usize::from(commissions_ready) + companion_done;
    let rank = legacy_boss_chapter_rank(chapter);

    match completed {
        count if count == total => LegacyBossPreparation {
            state: LegacyBossPreparationState::Prepared,
            completed,
            total,
            hp_loss: ((enemy.max_hp as f32 * 0.09).round() as i32 + completed as i32 * 2).max(1),
            atk_delta: -(2 + companion_done as i32),
            hp_restore: 8 + completed as i32 * 2,
            mp_restore: 3 + companion_done as i32,
            pressure_damage: 0,
        },
        0 => LegacyBossPreparation {
            state: LegacyBossPreparationState::Strained,
            completed,
            total,
            hp_loss: 0,
            atk_delta: 1 + rank / 2,
            hp_restore: 0,
            mp_restore: 0,
            pressure_damage: 4 + rank,
        },
        _ => LegacyBossPreparation {
            state: LegacyBossPreparationState::Partial,
            completed,
            total,
            hp_loss: ((enemy.max_hp as f32 * 0.04).round() as i32).max(1),
            atk_delta: -1,
            hp_restore: 5 + completed as i32,
            mp_restore: 1,
            pressure_damage: 0,
        },
    }
}

fn apply_legacy_boss_preparation_to_boss(
    boss: BossKind,
    quest: &QuestLog,
    stats: &mut PlayerStats,
    enemy: &mut EnemyInstance,
    message: &mut String,
) -> LegacyBossPreparation {
    let prep = legacy_boss_preparation(boss, quest, enemy);
    let place = legacy_boss_chapter_label(legacy_boss_chapter(boss));

    if prep.hp_loss > 0 {
        enemy.hp = (enemy.hp - prep.hp_loss).max(1);
    }
    enemy.atk = if prep.atk_delta < 0 {
        (enemy.atk + prep.atk_delta).max(1)
    } else {
        enemy.atk + prep.atk_delta
    };
    if prep.hp_restore > 0 && stats.hp < stats.max_hp {
        stats.hp = (stats.hp + prep.hp_restore).min(stats.max_hp);
    }
    if prep.mp_restore > 0 && stats.mp < stats.max_mp {
        stats.mp = (stats.mp + prep.mp_restore).min(stats.max_mp);
    }
    if prep.pressure_damage > 0 {
        stats.hp = (stats.hp - prep.pressure_damage).max(1);
    }

    let line = match prep.state {
        LegacyBossPreparationState::Prepared => format!(
            "【首领照应·周全】{place}照应 {}/{} 已接上，首领开局露出破口：气血 -{}，攻势 {}；队伍回复 {} 气血、{} 灵力。",
            prep.completed,
            prep.total,
            prep.hp_loss,
            prep.atk_delta,
            prep.hp_restore,
            prep.mp_restore
        ),
        LegacyBossPreparationState::Partial => format!(
            "【首领照应·半备】{place}照应 {}/{} 已接上，仍能压住首领一瞬：气血 -{}，攻势 {}；队伍回复 {} 气血、{} 灵力。",
            prep.completed,
            prep.total,
            prep.hp_loss,
            prep.atk_delta,
            prep.hp_restore,
            prep.mp_restore
        ),
        LegacyBossPreparationState::Strained => format!(
            "【首领照应·欠备】{place}照应 {}/{} 未接上，首领抢占先机：攻势 +{}，队伍受 {} 点开局压制。",
            prep.completed, prep.total, prep.atk_delta, prep.pressure_damage
        ),
    };
    message.push('\n');
    message.push_str(&line);
    prep
}

fn choose_enemy(zone: EncounterZone, kind: EncounterKind, rng: &mut Rng) -> &'static EnemyDef {
    match kind {
        EncounterKind::Boss(BossKind::MountainFiend) => return &MOUNTAIN_FIEND_BOSS,
        EncounterKind::Boss(BossKind::MoonWraith) => return &MOON_WRAITH_BOSS,
        EncounterKind::Boss(BossKind::RiverDemon) => return &RIVER_DEMON_BOSS,
        EncounterKind::Boss(BossKind::MiasmaRoot) => return &MIASMA_ROOT_BOSS,
        EncounterKind::Boss(BossKind::MirrorMinister) => return &MIRROR_MINISTER_BOSS,
        EncounterKind::Boss(BossKind::ThunderQilin) => return &THUNDER_QILIN_BOSS,
        EncounterKind::Boss(BossKind::DreamEclipse) => return &DREAM_ECLIPSE_BOSS,
        EncounterKind::Random => {}
    }

    let pool = encounter_pool(zone);
    let index = pool[rng.range(0, pool.len() as i32 - 1) as usize];
    &ENEMIES[index]
}

fn encounter_pool(zone: EncounterZone) -> &'static [usize] {
    match zone {
        EncounterZone::Village => &VILLAGE_ENCOUNTERS[..],
        EncounterZone::Bamboo => &BAMBOO_ENCOUNTERS[..],
        EncounterZone::Cave => &CAVE_ENCOUNTERS[..],
        EncounterZone::RiverTown => &RIVER_TOWN_ENCOUNTERS[..],
        EncounterZone::PlagueVillage => &PLAGUE_VILLAGE_ENCOUNTERS[..],
        EncounterZone::Capital => &CAPITAL_ENCOUNTERS[..],
        EncounterZone::SouthernRoad => &SOUTHERN_ROAD_ENCOUNTERS[..],
        EncounterZone::FinalSanctum => &FINAL_SANCTUM_ENCOUNTERS[..],
    }
}

fn choose_enemy_without_repeat(
    zone: EncounterZone,
    kind: EncounterKind,
    previous: Option<&str>,
    rng: &mut Rng,
) -> &'static EnemyDef {
    if !matches!(kind, EncounterKind::Random) {
        return choose_enemy(zone, kind, rng);
    }
    let pool = encounter_pool(zone);
    let candidates: Vec<usize> = pool
        .iter()
        .copied()
        .filter(|index| Some(ENEMIES[*index].image) != previous)
        .collect();
    let eligible = if candidates.is_empty() {
        pool
    } else {
        &candidates
    };
    &ENEMIES[eligible[rng.range(0, eligible.len() as i32 - 1) as usize]]
}

fn battle_companion_visible(
    quest: &QuestLog,
    run: Option<&RunState>,
    companion: Companion,
) -> bool {
    quest.has_companion(companion)
        || run.is_some_and(|run| run.party_companions().contains(&companion))
}

fn battle_party_summary(quest: &QuestLog, run: Option<&RunState>) -> String {
    run.map(|run| run.party_summary())
        .unwrap_or_else(|| quest.party_summary().to_string())
}

fn battle_has_late_spell(quest: &QuestLog, run: Option<&RunState>) -> bool {
    quest.has_late_spell()
        || run.is_some_and(|run| run.party_companions().contains(&Companion::SpiritWitch))
}

fn battle_spell_name(quest: &QuestLog, run: Option<&RunState>) -> &'static str {
    if battle_has_late_spell(quest, run) {
        "万剑诀"
    } else {
        quest.spell_name()
    }
}

fn battle_spell_cost(quest: &QuestLog, run: Option<&RunState>) -> i32 {
    if battle_has_late_spell(quest, run) {
        9
    } else {
        quest.spell_cost()
    }
}

/// Fallback spell pricing shared by gameplay and read-only automation.
fn fallback_spell_cost(quest: &QuestLog, run: Option<&RunState>) -> i32 {
    (battle_spell_cost(quest, run)
        + run.map_or(0, RunState::spell_cost_delta)
        + run.map_or(0, RunState::hex_spell_cost_delta))
    .max(1)
}

fn battle_spell_power_multiplier(quest: &QuestLog, run: Option<&RunState>) -> i32 {
    if battle_has_late_spell(quest, run) {
        3
    } else {
        quest.spell_power_multiplier()
    }
}

fn battle_spell_chapter(quest: &QuestLog, run: Option<&RunState>) -> Chapter {
    let Some(run) = run else {
        return quest.current_chapter();
    };
    match run.chapter {
        0 if run.stage <= 1 => Chapter::VillageOath,
        0 => Chapter::MoonCave,
        1 if run.stage <= 2 => Chapter::RiverMedicine,
        1 => Chapter::PlagueRain,
        2 if run.stage <= 2 => Chapter::CapitalMirror,
        2 => Chapter::SouthernThunder,
        _ => Chapter::FinalDream,
    }
}

fn spawn_battle_backdrop(
    commands: &mut Commands,
    asset_server: &AssetServer,
    lights: &LightingAssets,
    zone: EncounterZone,
) {
    let backdrop = battle_backdrop(zone);
    commands.spawn((
        Sprite {
            image: asset_server.load(battle_background_path(zone)),
            custom_size: Some(Vec2::new(1280.0, 720.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -3.0),
        DespawnOnExit(AppState::Battle),
    ));
    lighting::spawn_light(
        commands,
        lights,
        Vec3::new(-100.0, 60.0, -1.8),
        720.0,
        backdrop.light,
        AppState::Battle,
    );
}

fn battle_background_path(zone: EncounterZone) -> &'static str {
    match zone {
        EncounterZone::Village => "backgrounds/battle-village.png",
        EncounterZone::Bamboo => "backgrounds/battle-bamboo.png",
        EncounterZone::Cave => "backgrounds/battle-cave.png",
        EncounterZone::RiverTown => "backgrounds/battle-river.png",
        EncounterZone::PlagueVillage => "backgrounds/battle-plague.png",
        EncounterZone::Capital => "backgrounds/battle-capital.png",
        EncounterZone::SouthernRoad => "backgrounds/battle-southern.png",
        EncounterZone::FinalSanctum => "backgrounds/battle-dream.png",
    }
}

/// Shared encounter construction: renderer and simulations use identical scaling/affixes.
fn scaled_enemy(def: &EnemyDef, mods: Option<&RunBattleMods>, rng: &mut Rng) -> EnemyInstance {
    let mut enemy = EnemyInstance {
        name: def.name.into(),
        hp: def.max_hp,
        max_hp: def.max_hp,
        atk: def.atk,
        def: def.def,
        exp: def.exp,
    };

    // Roguelike run: scale enemies by chapter/elite/boss multipliers, and
    // give elites a random affix so repeat fights feel different.
    if let Some(mods) = mods {
        enemy.max_hp = (enemy.max_hp as f32 * mods.hp_mul).round() as i32;
        enemy.hp = enemy.max_hp;
        enemy.atk = (enemy.atk as f32 * mods.atk_mul).round() as i32;
        if mods.rank == FightRank::Elite {
            let affix = match rng.range(0, 2) {
                0 => {
                    enemy.atk = (enemy.atk as f32 * 1.25).round() as i32;
                    "狂暴"
                }
                1 => {
                    enemy.def += 3;
                    "坚鳞"
                }
                _ => {
                    enemy.max_hp = (enemy.max_hp as f32 * 1.25).round() as i32;
                    enemy.hp = enemy.max_hp;
                    "嗜血"
                }
            };
            enemy.name = format!("精英 · {affix} · {}", enemy.name);
        }
    }

    enemy
}

fn enemy_style(def: &EnemyDef) -> EnemyStyle {
    match def.image {
        "creatures/ai_stone_guardian.png" | "creatures/ai_moss_turtle.png" => EnemyStyle::Armored,
        "creatures/ai_cave_bat.png" | "creatures/ai_shadow_swordsman.png" => EnemyStyle::Trickster,
        "creatures/ai_fire_wisp.png"
        | "creatures/ai_lotus_spirit.png"
        | "creatures/ai_water_serpent.png" => EnemyStyle::Spirit,
        _ => EnemyStyle::Assault,
    }
}

fn initial_intent(def: &EnemyDef, kind: EncounterKind) -> EnemyIntent {
    if matches!(kind, EncounterKind::Boss(_)) {
        EnemyIntent::Charge
    } else {
        enemy_style(def).intent(0)
    }
}

fn tactical_damage(state: &mut BattleState, damage: i32, kind: StrikeKind) -> i32 {
    state.tactics.hit(damage, kind, state.intent).damage
}

fn battle_backdrop(zone: EncounterZone) -> BattleBackdrop {
    match zone {
        EncounterZone::Village => BattleBackdrop {
            light: Color::srgba(0.82, 0.96, 0.54, 0.20),
        },
        EncounterZone::Bamboo => BattleBackdrop {
            light: Color::srgba(0.54, 1.0, 0.52, 0.22),
        },
        EncounterZone::Cave => BattleBackdrop {
            light: Color::srgba(0.52, 0.72, 1.0, 0.28),
        },
        EncounterZone::RiverTown => BattleBackdrop {
            light: Color::srgba(0.42, 0.82, 1.0, 0.26),
        },
        EncounterZone::PlagueVillage => BattleBackdrop {
            light: Color::srgba(0.62, 0.96, 0.42, 0.22),
        },
        EncounterZone::Capital => BattleBackdrop {
            light: Color::srgba(0.64, 0.72, 1.0, 0.24),
        },
        EncounterZone::SouthernRoad => BattleBackdrop {
            light: Color::srgba(0.72, 1.0, 0.48, 0.24),
        },
        EncounterZone::FinalSanctum => BattleBackdrop {
            light: Color::srgba(0.52, 0.68, 1.0, 0.32),
        },
    }
}

// ---------------------------------------------------------------------------
// Input (player's turn only)
// ---------------------------------------------------------------------------

fn battle_input(
    mut commands: Commands,
    intent: Res<Intent>,
    font: Res<GameFont>,
    anims: Res<AnimationAssets>,
    lights: Res<LightingAssets>,
    vfx_assets: Res<SkillVfxAssets>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    mut state: ResMut<BattleState>,
    mut stats: ResMut<PlayerStats>,
    quest: Res<QuestLog>,
    mut rng: ResMut<Rng>,
    run: Option<Res<RunState>>,
) {
    if state.phase != Phase::Menu {
        return;
    }

    // 二十式仙术秘卷子菜单:选式、翻页、施放、返回。
    if state.skill_menu {
        let skills: Vec<super::roguelike::skill::SkillId> =
            run.as_ref().map(|r| r.skills.clone()).unwrap_or_default();
        let total = skills.len() + 1; // 末位是「返回」
        if intent.up {
            state.skill_cursor = (state.skill_cursor + total - 1) % total;
        }
        if intent.down {
            state.skill_cursor = (state.skill_cursor + 1) % total;
        }
        if intent.cancel {
            state.skill_menu = false;
            return;
        }
        if !intent.confirm {
            return;
        }
        if state.skill_cursor >= skills.len() {
            state.skill_menu = false;
            return;
        }
        let id = skills[state.skill_cursor];
        if let Some(run) = run.as_ref() {
            if cast_skill(
                &mut commands,
                &font,
                &lights,
                &vfx_assets,
                &mut color_materials,
                &mut state,
                &mut stats,
                run,
                &mut rng,
                id,
            ) {
                state.skill_menu = false;
                start_player_acting(&mut state, PlayerAction::Spell);
            } else {
                // 灵力不足:退回主菜单,不困在技谱里。
                state.skill_menu = false;
            }
        }
        return;
    }

    // Roguelike relic modifiers (all zero/1.0 outside run mode).
    let relic_attack = run.as_ref().map_or(0, |r| r.attack_bonus());
    let relic_spell = run.as_ref().map_or(0, |r| r.spell_bonus());
    let relic_potion = run.as_ref().map_or(0, |r| r.potion_bonus());
    let relic_flee = run.as_ref().is_some_and(|r| r.flee_always());
    let hunter_mul = run.as_ref().map_or(1.0, |r| {
        r.hunter_multiplier(r.current_fight.unwrap_or(FightRank::Normal))
    });
    let run_ref = run.as_deref();
    let run_camp_tactic = state.run_camp_tactic;

    if intent.up {
        state.menu_index = (state.menu_index + MENU.len() - 1) % MENU.len();
    }
    if intent.down {
        state.menu_index = (state.menu_index + 1) % MENU.len();
    }

    if !intent.confirm {
        return;
    }

    match state.menu_index {
        0 => {
            // 攻击
            let blessing = state.blessing;
            let camp_bonus = state.camp_bonus;
            let bond_bonus = state.bond_bonus;
            let hex_atk = run
                .as_ref()
                .map_or(0, |r| r.hex_atk_delta(stats.hp, stats.max_hp));
            let base = (stats.atk + hex_atk - state.enemy.def + rng.range(-2, 3)).max(1);
            let blessing_bonus = blessing_damage_bonus(blessing, PlayerAction::Attack);
            let camp_damage = camp_damage_bonus(camp_bonus, PlayerAction::Attack);
            let run_camp_damage = run_camp_damage_bonus(run_camp_tactic, PlayerAction::Attack);
            let bond_damage = bond_damage_bonus(bond_bonus, PlayerAction::Attack);
            let first_mul = if !state.hex_first_hit_used {
                run.as_ref().map_or(1.0, |r| r.hex_first_strike_mul())
            } else {
                1.0
            };
            state.hex_first_hit_used = true;
            let dmg = (((base
                + blessing_bonus
                + camp_damage
                + run_camp_damage
                + bond_damage
                + relic_attack) as f32)
                * hunter_mul
                * first_mul)
                .round() as i32;
            let dmg = tactical_damage(&mut state, dmg, StrikeKind::Sword);
            state.enemy.hp -= dmg;
            state.message = format!(
                "李逍遥 挥剑而上，对 {} 造成 {} 点伤害！",
                state.enemy.name, dmg
            );
            if first_mul > 1.0 {
                state.message.push_str("\n【雷引纹】首击引雷,伤害加半!");
            }
            if relic_attack > 0 {
                state
                    .message
                    .push_str(&format!("\n【青锋剑穗】剑势更利,伤害 +{relic_attack}。"));
            }
            append_blessing_damage_line(&mut state.message, blessing, blessing_bonus);
            append_camp_damage_line(&mut state.message, camp_bonus, camp_damage);
            append_run_camp_damage_line(&mut state.message, run_camp_tactic, run_camp_damage);
            append_bond_damage_line(&mut state.message, bond_bonus, bond_damage);
            if let Some(line) = mirror_backlash(&state, &mut stats, dmg) {
                state.message.push_str(&line);
            }
            spawn_weapon_hit(&mut commands, &anims, &lights, ENEMY_POS);
            spawn_damage_text(
                &mut commands,
                &font,
                ENEMY_POS + Vec3::new(96.0, 72.0, 0.0),
                dmg,
            );
            if let Some(extra) =
                sword_sister_followup(&quest, run_ref, &stats, &mut state.enemy, &mut rng)
            {
                state
                    .message
                    .push_str(&format!("\n林月衡 补上一剑，追加 {extra} 点伤害！"));
                spawn_sword_sister_followup(&mut commands, &anims, &lights);
                spawn_damage_text(
                    &mut commands,
                    &font,
                    ENEMY_POS + Vec3::new(144.0, 46.0, 0.0),
                    extra,
                );
            }
            state.momentum = (state.momentum + 1).min(MOMENTUM_MAX);
            start_player_acting(&mut state, PlayerAction::Attack);
        }
        1 => {
            // 御守:本回合卸去大半来势,顺势回灵。
            state.guarding = true;
            let restore = run
                .as_ref()
                .map_or(GUARD_MP_RESTORE, |r| r.guard_mp_restore());
            let run_camp_restore = run_camp_guard_mp_bonus(run_camp_tactic);
            let hex_mp = run.as_ref().map_or(0, |r| r.hex_guard_mp_delta());
            let total_restore = (restore + run_camp_restore + hex_mp).max(0);
            let before_mp = stats.mp;
            stats.mp = (stats.mp + total_restore).min(stats.max_mp);
            let restored = stats.mp - before_mp;
            if run.as_ref().is_some_and(|r| r.hex_guard_momentum()) {
                state.momentum = (state.momentum + 1).min(MOMENTUM_MAX);
            }
            state.message = format!("李逍遥 剑交左手,凝神御守——气随息回,恢复 {restored} 点灵力。");
            if restore > GUARD_MP_RESTORE {
                state
                    .message
                    .push_str("\n【道心·御守精进】身形如渊渟岳峙。");
            }
            if run_camp_restore > 0 {
                state.message.push_str(&format!(
                    "\n【营策】{}加持御守；本次实际恢复 {restored} 点灵力。",
                    run_camp_tactic.map(RunCampTactic::name).unwrap_or("凝灵")
                ));
            }
            start_player_acting(&mut state, PlayerAction::Guard);
            spawn_guard_halo(&mut commands, &lights);
        }
        2 => {
            // 仙术：御剑术 / 万剑诀
            let late_spell = battle_has_late_spell(&quest, run_ref);
            // run 模式已拓技能谱:仙术槽打开二十式秘卷子菜单。
            if run.as_ref().is_some_and(|r| !r.skills.is_empty()) {
                state.skill_menu = true;
                state.skill_cursor = 0;
                return;
            }
            let spell_cost = fallback_spell_cost(&quest, run_ref);
            let spell_name = battle_spell_name(&quest, run_ref);
            if stats.mp < spell_cost {
                state.message = format!("灵力不足，无法施展{spell_name}！");
            } else {
                let blessing = state.blessing;
                let camp_bonus = state.camp_bonus;
                let bond_bonus = state.bond_bonus;
                stats.mp -= spell_cost;
                let roll_max = if late_spell { 10 } else { 6 };
                let base = (stats.atk * battle_spell_power_multiplier(&quest, run_ref)
                    - state.enemy.def
                    + rng.range(0, roll_max))
                .max(1);
                let blessing_bonus = blessing_damage_bonus(blessing, PlayerAction::Spell);
                let camp_damage = camp_damage_bonus(camp_bonus, PlayerAction::Spell);
                let run_camp_damage = run_camp_damage_bonus(run_camp_tactic, PlayerAction::Spell);
                let bond_damage = bond_damage_bonus(bond_bonus, PlayerAction::Spell);
                let dmg = (((base
                    + blessing_bonus
                    + camp_damage
                    + run_camp_damage
                    + bond_damage
                    + relic_spell
                    + run.as_ref().map_or(0, |r| r.hex_spell_bonus()))
                    as f32)
                    * hunter_mul)
                    .round() as i32;
                let dmg = tactical_damage(&mut state, dmg, StrikeKind::Spell);
                state.enemy.hp -= dmg;
                let spell_element = spell_impact_element(battle_spell_chapter(&quest, run_ref));
                let flavor = spell_impact_flavor(spell_element, late_spell);
                state.message = format!("李逍遥 施展{spell_name}，{flavor}，造成 {dmg} 点伤害！");
                if relic_spell > 0 {
                    state
                        .message
                        .push_str(&format!("\n【御剑心诀】术随心动,伤害 +{relic_spell}。"));
                }
                append_blessing_damage_line(&mut state.message, blessing, blessing_bonus);
                append_camp_damage_line(&mut state.message, camp_bonus, camp_damage);
                append_run_camp_damage_line(&mut state.message, run_camp_tactic, run_camp_damage);
                append_bond_damage_line(&mut state.message, bond_bonus, bond_damage);
                if let Some(extra) =
                    sword_sister_followup(&quest, run_ref, &stats, &mut state.enemy, &mut rng)
                {
                    state
                        .message
                        .push_str(&format!("\n林月衡 趁势追击，追加 {extra} 点伤害！"));
                    spawn_sword_sister_followup(&mut commands, &anims, &lights);
                    spawn_damage_text(
                        &mut commands,
                        &font,
                        ENEMY_POS + Vec3::new(150.0, 54.0, 0.0),
                        extra,
                    );
                }
                spawn_spell_impact(&mut commands, &anims, &lights, late_spell, spell_element);
                spawn_spell_damage_texts(&mut commands, &font, dmg, late_spell, spell_element);
                state.momentum = (state.momentum + 1).min(MOMENTUM_MAX);
                start_player_acting(&mut state, PlayerAction::Spell);
            }
        }
        3 if run.is_some() => {
            // 绝技·剑气爆发:气势够层时的一锤定音(妖契纹只需 2 层)。
            let burst_need = run.as_ref().map_or(MOMENTUM_MAX, |r| r.hex_burst_cost());
            if state.momentum < burst_need {
                state.message = format!(
                    "气势未足({}/{burst_need})——连续攻击或施术蓄满气势,方可施展绝技。",
                    state.momentum
                );
            } else {
                let base = (stats.atk * 2 - state.enemy.def + rng.range(2, 9)).max(3);
                let burst_mul = run.as_ref().map_or(1.0, |r| r.resolve_burst_mul());
                let run_camp_damage = run_camp_damage_bonus(run_camp_tactic, PlayerAction::Combo);
                let dmg = (((base + relic_attack + relic_spell + run_camp_damage) as f32)
                    * hunter_mul
                    * burst_mul)
                    .round() as i32;
                let dmg = tactical_damage(&mut state, dmg, StrikeKind::Burst);
                state.enemy.hp -= dmg;
                state.momentum = 0;
                state.message =
                    format!("李逍遥 气势鼎盛,施展绝技·剑气爆发!剑光如潮水倾泻,造成 {dmg} 点伤害!");
                if burst_mul > 1.0 {
                    state.message.push_str("\n【道心·剑意如磐】绝技威力更盛!");
                }
                append_run_camp_damage_line(&mut state.message, run_camp_tactic, run_camp_damage);
                let self_hurt = run.as_ref().map_or(0, |r| r.hex_burst_self_hurt());
                if self_hurt > 0 {
                    stats.hp = (stats.hp - self_hurt).max(1);
                    state
                        .message
                        .push_str(&format!("\n【妖契纹】剑气反啮,自伤 {self_hurt} 点。"));
                }
                if let Some(line) = mirror_backlash(&state, &mut stats, dmg) {
                    state.message.push_str(&line);
                }
                spawn_combo_impact(&mut commands, &anims, &lights);
                spawn_damage_text(
                    &mut commands,
                    &font,
                    ENEMY_POS + Vec3::new(118.0, 84.0, 0.0),
                    dmg,
                );
                start_player_acting(&mut state, PlayerAction::Combo);
            }
        }
        3 => {
            // 合击：逍遥、灵儿、林月衡，南瑶入队后扩展为四人阵。
            if !combo_unlocked(&quest) {
                state.message = "羁绊未成，暂时无法施展合击。".into();
            } else if stats.mp < COMBO_COST {
                state.message = format!("灵力不足，无法施展合击！需要 {COMBO_COST} 点灵力。");
            } else {
                let blessing = state.blessing;
                let camp_bonus = state.camp_bonus;
                let bond_bonus = state.bond_bonus;
                stats.mp -= COMBO_COST;
                let base = combo_damage(&quest, &stats, &state.enemy, &mut rng);
                let blessing_bonus = blessing_damage_bonus(blessing, PlayerAction::Combo);
                let camp_damage = camp_damage_bonus(camp_bonus, PlayerAction::Combo);
                let run_camp_damage = run_camp_damage_bonus(run_camp_tactic, PlayerAction::Combo);
                let bond_damage = bond_damage_bonus(bond_bonus, PlayerAction::Combo);
                let dmg = (((base + blessing_bonus + camp_damage + run_camp_damage + bond_damage)
                    as f32)
                    * hunter_mul)
                    .round() as i32;
                let dmg = tactical_damage(&mut state, dmg, StrikeKind::Burst);
                state.enemy.hp -= dmg;
                state.message = combo_party_message(&quest, dmg);
                append_blessing_damage_line(&mut state.message, blessing, blessing_bonus);
                append_camp_damage_line(&mut state.message, camp_bonus, camp_damage);
                append_run_camp_damage_line(&mut state.message, run_camp_tactic, run_camp_damage);
                append_bond_damage_line(&mut state.message, bond_bonus, bond_damage);
                spawn_combo_party_casts(&mut commands, &anims, &lights, &quest);
                spawn_combo_impact(&mut commands, &anims, &lights);
                spawn_damage_text(
                    &mut commands,
                    &font,
                    ENEMY_POS + Vec3::new(118.0, 84.0, 0.0),
                    dmg,
                );
                start_player_acting(&mut state, PlayerAction::Combo);
            }
        }
        4 => {
            // 物品：药水
            if stats.potions == 0 {
                state.message = "药水已经用完了！".into();
            } else {
                stats.potions -= 1;
                let before = stats.hp;
                let hex_potion = run.as_ref().map_or(0, |r| r.hex_potion_delta());
                stats.hp = (stats.hp
                    + (PlayerStats::POTION_HEAL + relic_potion + hex_potion).max(5))
                .min(stats.max_hp);
                let healed = stats.hp - before;
                state.message = format!("李逍遥 饮下药水，恢复了 {} 点气血。", healed);
                spawn_heal_text(
                    &mut commands,
                    &font,
                    HERO_POS + Vec3::new(10.0, 100.0, 0.0),
                    healed,
                );
                start_player_acting(&mut state, PlayerAction::Item);
            }
        }
        _ => {
            // 逃跑(首领战避无可避)
            if run
                .as_ref()
                .is_some_and(|r| matches!(r.current_fight, Some(FightRank::Boss)))
            {
                state.message = "此战避无可避——首领拦住了所有退路！".into();
                return;
            }
            if relic_flee || rng.chance(0.5) {
                state.message = if relic_flee {
                    "【云袖】袖里乾坤一转，李逍遥 从容抽身而去。".into()
                } else {
                    "李逍遥 觑得空隙，抽身逃走了……".into()
                };
                state.phase = Phase::Fled;
                state.timer = 1.4;
                state.player_action = Some(PlayerAction::Flee);
            } else {
                state.message = "逃跑失败，妖兽紧追不舍！".into();
                start_player_acting(&mut state, PlayerAction::Flee);
            }
        }
    }
}

fn battle_lessons(mut state: ResMut<BattleState>, mut run: Option<ResMut<RunState>>) {
    if state.phase != Phase::Menu {
        return;
    }
    let Some(run) = run.as_mut() else {
        return;
    };
    let lesson = if state.tactics.riposte {
        Some((
            BattleLesson::Counter,
            "反击已就绪：现在攻击，伤害 +50%；调息或吃药会放弃机会。",
        ))
    } else if state.tactics.exposed {
        Some((
            BattleLesson::Break,
            "架势已破：下一次进攻伤害 +50%，抓住这次追击。",
        ))
    } else if matches!(state.intent, EnemyIntent::Heavy | EnemyIntent::Special) {
        Some((BattleLesson::Heavy, "重击将至：选择御守可卸力并准备反击。"))
    } else if state.intent == EnemyIntent::Charge {
        Some((
            BattleLesson::Charge,
            "敌人正在蓄力：这一回合不会攻击，可以挥剑施压；下一回合留意御守。",
        ))
    } else {
        None
    };
    if let Some((lesson, line)) = lesson {
        if !run.battle_lessons_seen.contains(&lesson) {
            run.battle_lessons_seen.push(lesson);
            state.message.push_str(&format!("\n【战斗提示】{line}"));
        }
    }
}

fn sync_battle_automation_view(
    state: Res<BattleState>,
    quest: Res<QuestLog>,
    run: Option<Res<RunState>>,
    mut view: ResMut<BattleAutomationView>,
) {
    let run_ref = run.as_deref();
    *view = BattleAutomationView {
        menu_open: state.phase == Phase::Menu,
        menu_index: state.menu_index,
        skill_menu_open: state.skill_menu,
        skill_cursor: state.skill_cursor,
        enemy_hp: state.enemy.hp.max(0),
        enemy_max_hp: state.enemy.max_hp,
        enemy_heavy_intent: matches!(state.intent, EnemyIntent::Heavy | EnemyIntent::Special),
        enemy_charging: state.intent == EnemyIntent::Charge,
        enemy_bracing: state.intent == EnemyIntent::Brace,
        pressure: state.tactics.pressure,
        pressure_limit: state.tactics.limit,
        riposte_ready: state.tactics.riposte,
        exposed: state.tactics.exposed,
        breaks: state.tactics.breaks,
        counters: state.tactics.counters,
        spell_cost: run_ref
            .filter(|r| !r.skills.is_empty())
            .map(|r| {
                r.skills
                    .iter()
                    .map(|id| skill_cost(*id, r))
                    .min()
                    .unwrap_or(4)
            })
            .unwrap_or_else(|| fallback_spell_cost(&quest, run_ref)),
        momentum: state.momentum,
        poison_stacks: state.statuses.poison.stacks,
        burn_turns: state.statuses.burn.turns,
        frozen: state.statuses.frozen,
        stunned: state.statuses.stunned,
    };
}

fn start_player_acting(state: &mut BattleState, action: PlayerAction) {
    state.phase = Phase::PlayerActing;
    state.phase_duration = match action {
        PlayerAction::Attack => 0.72,
        PlayerAction::Guard => 0.32,
        PlayerAction::Spell => 0.88,
        PlayerAction::Combo => 0.92,
        PlayerAction::Item => 0.40,
        PlayerAction::Flee => 0.45,
    };
    state.timer = state.phase_duration;
    state.player_action = Some(action);
    if matches!(
        action,
        PlayerAction::Guard | PlayerAction::Item | PlayerAction::Flee
    ) {
        state.tactics.wait();
    }
    if !state.tactics.notice.is_empty() {
        state.message.push('\n');
        state
            .message
            .push_str(&std::mem::take(&mut state.tactics.notice));
    }
}

fn spell_impact_profile(late_spell: bool) -> SpellImpactProfile {
    if late_spell {
        SpellImpactProfile {
            core_size: 350.0,
            core_duration: AnimationClip::SkillImpact.duration() + 0.32,
            core_start_scale: 0.62,
            core_end_scale: 1.34,
            ring_size: 430.0,
            ring_duration: 0.58,
            ring_end_scale: 1.52,
            flash_radius: 430.0,
            flash_alpha: 0.68,
        }
    } else {
        SpellImpactProfile {
            core_size: 290.0,
            core_duration: AnimationClip::SkillImpact.duration() + 0.22,
            core_start_scale: 0.70,
            core_end_scale: 1.18,
            ring_size: 340.0,
            ring_duration: 0.46,
            ring_end_scale: 1.30,
            flash_radius: 340.0,
            flash_alpha: 0.54,
        }
    }
}

fn spell_impact_element(chapter: Chapter) -> SpellImpactElement {
    match chapter {
        Chapter::VillageOath => SpellImpactElement::Village,
        Chapter::MoonCave => SpellImpactElement::Moon,
        Chapter::RiverMedicine => SpellImpactElement::River,
        Chapter::PlagueRain => SpellImpactElement::Plague,
        Chapter::CapitalMirror => SpellImpactElement::Mirror,
        Chapter::SouthernThunder => SpellImpactElement::Thunder,
        Chapter::FinalDream => SpellImpactElement::Dream,
    }
}

fn spell_impact_flavor(element: SpellImpactElement, late_spell: bool) -> &'static str {
    match (element, late_spell) {
        (SpellImpactElement::Village, false) => "剑气纵横",
        (SpellImpactElement::Moon, false) => "月水映剑",
        (SpellImpactElement::River, false) => "水纹破妖",
        (SpellImpactElement::Plague, false) => "净瘴剑光",
        (SpellImpactElement::Mirror, false) => "照影成锋",
        (SpellImpactElement::Thunder, false) => "雷纹引剑",
        (SpellImpactElement::Dream, false) => "梦水留痕",
        (SpellImpactElement::Village, true) => "万剑归心",
        (SpellImpactElement::Moon, true) => "月影万剑",
        (SpellImpactElement::River, true) => "江潮剑雨",
        (SpellImpactElement::Plague, true) => "百剑净瘴",
        (SpellImpactElement::Mirror, true) => "镜光万刃",
        (SpellImpactElement::Thunder, true) => "万剑引雷",
        (SpellImpactElement::Dream, true) => "梦水剑阵",
    }
}

fn spell_element_colors(element: SpellImpactElement) -> ([f32; 3], [f32; 3]) {
    match element {
        SpellImpactElement::Village => ([1.0, 0.78, 0.32], [1.0, 0.96, 0.66]),
        SpellImpactElement::Moon => ([0.66, 0.82, 1.0], [0.98, 0.74, 1.0]),
        SpellImpactElement::River => ([0.34, 0.86, 1.0], [0.78, 1.0, 0.88]),
        SpellImpactElement::Plague => ([0.58, 0.94, 0.46], [0.94, 1.0, 0.58]),
        SpellImpactElement::Mirror => ([0.70, 0.68, 1.0], [1.0, 0.88, 1.0]),
        SpellImpactElement::Thunder => ([0.42, 0.78, 1.0], [1.0, 0.92, 0.40]),
        SpellImpactElement::Dream => ([0.62, 0.58, 1.0], [0.70, 1.0, 0.98]),
    }
}

fn boosted_channel(value: f32, late_spell: bool) -> f32 {
    if late_spell {
        (value * 1.12 + 0.05).clamp(0.0, 1.0)
    } else {
        value
    }
}

fn spell_impact_color(
    element: SpellImpactElement,
    late_spell: bool,
    use_accent: bool,
    alpha: f32,
) -> Color {
    let (base, accent) = spell_element_colors(element);
    let [r, g, b] = if use_accent { accent } else { base };
    Color::srgba(
        boosted_channel(r, late_spell),
        boosted_channel(g, late_spell),
        boosted_channel(b, late_spell),
        alpha,
    )
}

fn spell_shard_color(element: SpellImpactElement, late_spell: bool, index: usize) -> Color {
    spell_impact_color(
        element,
        late_spell,
        index % 2 == 0,
        if late_spell { 0.84 } else { 0.78 },
    )
}

fn spell_impact_shards(late_spell: bool) -> Vec<SpellImpactShard> {
    let mut shards = vec![
        SpellImpactShard {
            offset: Vec2::new(-76.0, 52.0),
            size: Vec2::new(96.0, 220.0),
            rotation: -0.62,
            duration: 0.34,
            start_scale: 0.36,
            end_scale: 0.92,
            spin: 0.40,
            color: Color::srgba(1.0, 0.92, 0.54, 0.88),
        },
        SpellImpactShard {
            offset: Vec2::new(36.0, 18.0),
            size: Vec2::new(88.0, 190.0),
            rotation: 0.78,
            duration: 0.38,
            start_scale: 0.34,
            end_scale: 0.88,
            spin: -0.52,
            color: Color::srgba(1.0, 0.72, 0.34, 0.84),
        },
        SpellImpactShard {
            offset: Vec2::new(86.0, 82.0),
            size: Vec2::new(78.0, 170.0),
            rotation: -0.18,
            duration: 0.32,
            start_scale: 0.30,
            end_scale: 0.78,
            spin: 0.34,
            color: Color::srgba(1.0, 0.96, 0.72, 0.80),
        },
    ];

    if late_spell {
        shards.extend([
            SpellImpactShard {
                offset: Vec2::new(-122.0, 4.0),
                size: Vec2::new(74.0, 178.0),
                rotation: 1.06,
                duration: 0.40,
                start_scale: 0.28,
                end_scale: 0.86,
                spin: -0.62,
                color: Color::srgba(0.78, 0.92, 1.0, 0.78),
            },
            SpellImpactShard {
                offset: Vec2::new(-18.0, 116.0),
                size: Vec2::new(90.0, 220.0),
                rotation: -0.04,
                duration: 0.44,
                start_scale: 0.32,
                end_scale: 1.02,
                spin: 0.52,
                color: Color::srgba(0.88, 0.96, 1.0, 0.82),
            },
            SpellImpactShard {
                offset: Vec2::new(124.0, 28.0),
                size: Vec2::new(78.0, 188.0),
                rotation: -1.08,
                duration: 0.40,
                start_scale: 0.28,
                end_scale: 0.86,
                spin: 0.70,
                color: Color::srgba(1.0, 0.86, 0.54, 0.78),
            },
            SpellImpactShard {
                offset: Vec2::new(12.0, -44.0),
                size: Vec2::new(96.0, 190.0),
                rotation: 0.36,
                duration: 0.36,
                start_scale: 0.30,
                end_scale: 0.92,
                spin: -0.48,
                color: Color::srgba(1.0, 0.78, 0.42, 0.82),
            },
        ]);
    }

    shards
}

fn themed_spell_impact_shards(
    late_spell: bool,
    element: SpellImpactElement,
) -> Vec<SpellImpactShard> {
    let mut shards = spell_impact_shards(late_spell);
    for (index, shard) in shards.iter_mut().enumerate() {
        shard.color = spell_shard_color(element, late_spell, index);
    }
    shards
}

fn spawn_spell_impact(
    commands: &mut Commands,
    anims: &AnimationAssets,
    lights: &LightingAssets,
    late_spell: bool,
    element: SpellImpactElement,
) {
    let profile = spell_impact_profile(late_spell);
    let mut sprite = anims.sprite(AnimationClip::SkillImpact, Vec2::splat(profile.core_size));
    sprite.color = spell_impact_color(
        element,
        late_spell,
        true,
        if late_spell { 0.98 } else { 0.94 },
    );

    commands.spawn((
        sprite,
        SpriteAnimation::once(AnimationClip::SkillImpact),
        BattleEffect::new(
            profile.core_duration,
            profile.core_start_scale,
            profile.core_end_scale,
            -0.55,
        ),
        Transform::from_xyz(ENEMY_POS.x, ENEMY_POS.y + 4.0, 2.8),
        DespawnOnExit(AppState::Battle),
    ));

    let mut ring = anims.sprite(AnimationClip::SkillImpact, Vec2::splat(profile.ring_size));
    ring.color = spell_impact_color(
        element,
        late_spell,
        false,
        if late_spell { 0.70 } else { 0.58 },
    );
    commands.spawn((
        ring,
        SpriteAnimation::once(AnimationClip::SkillImpact),
        BattleEffect::new(profile.ring_duration, 0.20, profile.ring_end_scale, 0.42),
        Transform::from_xyz(ENEMY_POS.x, ENEMY_POS.y + 2.0, 2.55),
        DespawnOnExit(AppState::Battle),
    ));

    for shard in themed_spell_impact_shards(late_spell, element) {
        spawn_spell_impact_shard(commands, anims, shard);
    }

    let flash = lighting::spawn_light(
        commands,
        lights,
        Vec3::new(ENEMY_POS.x, ENEMY_POS.y + 4.0, 2.1),
        profile.flash_radius,
        spell_impact_color(element, late_spell, false, profile.flash_alpha),
        AppState::Battle,
    );
    commands
        .entity(flash)
        .insert(BattleEffect::new(0.40, 0.28, 1.12, 0.0));
}

fn spawn_spell_impact_shard(
    commands: &mut Commands,
    anims: &AnimationAssets,
    shard: SpellImpactShard,
) {
    let mut sprite = anims.sprite(AnimationClip::SkillImpact, shard.size);
    sprite.color = shard.color;
    let mut transform = Transform::from_xyz(
        ENEMY_POS.x + shard.offset.x,
        ENEMY_POS.y + shard.offset.y,
        3.0,
    );
    transform.rotation = Quat::from_rotation_z(shard.rotation);

    commands.spawn((
        sprite,
        SpriteAnimation::once(AnimationClip::SkillImpact),
        BattleEffect::new(
            shard.duration,
            shard.start_scale,
            shard.end_scale,
            shard.spin,
        ),
        transform,
        DespawnOnExit(AppState::Battle),
    ));
}

fn spell_damage_texts(total: i32, late_spell: bool) -> Vec<SpellDamageText> {
    let slices = spell_damage_slices(total, late_spell);
    if slices.is_empty() {
        return Vec::new();
    }

    let offsets: &[Vec2] = if late_spell {
        &[
            Vec2::new(42.0, 112.0),
            Vec2::new(118.0, 84.0),
            Vec2::new(76.0, 44.0),
            Vec2::new(154.0, 32.0),
            Vec2::new(18.0, 56.0),
            Vec2::new(138.0, 122.0),
            Vec2::new(92.0, 18.0),
        ]
    } else {
        &[
            Vec2::new(64.0, 96.0),
            Vec2::new(118.0, 56.0),
            Vec2::new(42.0, 38.0),
        ]
    };
    let velocities: &[Vec2] = if late_spell {
        &[
            Vec2::new(-18.0, 86.0),
            Vec2::new(16.0, 92.0),
            Vec2::new(-8.0, 78.0),
            Vec2::new(22.0, 82.0),
            Vec2::new(-24.0, 74.0),
            Vec2::new(12.0, 96.0),
            Vec2::new(-4.0, 72.0),
        ]
    } else {
        &[
            Vec2::new(-10.0, 78.0),
            Vec2::new(14.0, 82.0),
            Vec2::new(-4.0, 72.0),
        ]
    };

    slices
        .into_iter()
        .enumerate()
        .map(|(index, amount)| SpellDamageText {
            amount,
            offset: offsets[index],
            velocity: velocities[index],
            size: if late_spell { 27.0 } else { 31.0 },
        })
        .collect()
}

fn spell_damage_slices(total: i32, late_spell: bool) -> Vec<i32> {
    if total <= 0 {
        return Vec::new();
    }
    let weights: &[i32] = if late_spell {
        &[20, 17, 15, 14, 13, 11, 10]
    } else {
        &[38, 34, 28]
    };
    let hit_count = weights.len().min(total as usize);
    let weights = &weights[..hit_count];
    let weight_sum: i32 = weights.iter().sum();
    let mut slices: Vec<i32> = weights
        .iter()
        .map(|weight| ((total * *weight + weight_sum / 2) / weight_sum).max(1))
        .collect();

    let mut diff = total - slices.iter().sum::<i32>();
    let mut index = 0;
    while diff != 0 {
        let slot = index % slices.len();
        if diff > 0 {
            slices[slot] += 1;
            diff -= 1;
        } else if slices[slot] > 1 {
            slices[slot] -= 1;
            diff += 1;
        }
        index += 1;
    }

    slices
}

fn spell_damage_text_color(element: SpellImpactElement, late_spell: bool) -> Color {
    spell_impact_color(
        element,
        late_spell,
        true,
        if late_spell { 0.96 } else { 0.90 },
    )
}

fn spawn_spell_damage_texts(
    commands: &mut Commands,
    font: &GameFont,
    total: i32,
    late_spell: bool,
    element: SpellImpactElement,
) {
    let color = spell_damage_text_color(element, late_spell);
    for text in spell_damage_texts(total, late_spell) {
        spawn_floating_combat_text(
            commands,
            font,
            format!("-{}", text.amount),
            ENEMY_POS + Vec3::new(text.offset.x, text.offset.y, 0.0),
            color,
            text.velocity,
            text.size,
        );
    }
}

fn spawn_combo_impact(commands: &mut Commands, anims: &AnimationAssets, lights: &LightingAssets) {
    let duration = AnimationClip::SkillImpact.duration() + 0.34;
    let mut sprite = anims.sprite(AnimationClip::SkillImpact, Vec2::splat(360.0));
    sprite.color = Color::srgba(0.96, 0.84, 1.0, 0.98);

    commands.spawn((
        sprite,
        SpriteAnimation::once(AnimationClip::SkillImpact),
        BattleEffect::new(duration, 0.82, 1.38, 0.72),
        Transform::from_xyz(ENEMY_POS.x, ENEMY_POS.y + 4.0, 2.9),
        DespawnOnExit(AppState::Battle),
    ));

    let flash = lighting::spawn_light(
        commands,
        lights,
        Vec3::new(ENEMY_POS.x, ENEMY_POS.y + 4.0, 2.1),
        410.0,
        Color::srgba(0.78, 0.54, 1.0, 0.62),
        AppState::Battle,
    );
    commands
        .entity(flash)
        .insert(BattleEffect::new(0.42, 0.34, 1.18, 0.0));
}

fn spawn_combo_party_casts(
    commands: &mut Commands,
    anims: &AnimationAssets,
    lights: &LightingAssets,
    quest: &QuestLog,
) {
    spawn_companion_cast_rune(
        commands,
        anims,
        lights,
        HERO_POS + Vec3::new(26.0, 92.0, 0.0),
        Color::srgba(1.0, 0.78, 0.28, 0.78),
        126.0,
    );
    spawn_companion_cast_rune(
        commands,
        anims,
        lights,
        LINGER_POS + Vec3::new(2.0, 86.0, 0.0),
        Color::srgba(0.42, 0.92, 1.0, 0.78),
        118.0,
    );
    spawn_companion_cast_rune(
        commands,
        anims,
        lights,
        SWORD_SISTER_POS + Vec3::new(8.0, 82.0, 0.0),
        Color::srgba(1.0, 0.46, 0.32, 0.78),
        118.0,
    );
    if quest.has_companion(Companion::SpiritWitch) {
        spawn_companion_cast_rune(
            commands,
            anims,
            lights,
            SPIRIT_WITCH_POS + Vec3::new(2.0, 90.0, 0.0),
            Color::srgba(0.46, 1.0, 0.58, 0.76),
            114.0,
        );
    }
}

fn spawn_companion_cast_rune(
    commands: &mut Commands,
    anims: &AnimationAssets,
    lights: &LightingAssets,
    pos: Vec3,
    color: Color,
    size: f32,
) {
    let mut sprite = anims.sprite(AnimationClip::SkillImpact, Vec2::splat(size));
    sprite.color = color;
    commands.spawn((
        sprite,
        SpriteAnimation::once(AnimationClip::SkillImpact),
        BattleEffect::new(0.42, 0.30, 0.78, 0.65),
        Transform::from_xyz(pos.x, pos.y, 2.55),
        DespawnOnExit(AppState::Battle),
    ));

    let srgba = color.to_srgba();
    let flash = lighting::spawn_light(
        commands,
        lights,
        Vec3::new(pos.x, pos.y, 1.9),
        170.0,
        Color::srgba(srgba.red, srgba.green, srgba.blue, 0.34),
        AppState::Battle,
    );
    commands
        .entity(flash)
        .insert(BattleEffect::new(0.34, 0.26, 0.70, 0.0));
}

fn spawn_sword_sister_followup(
    commands: &mut Commands,
    anims: &AnimationAssets,
    lights: &LightingAssets,
) {
    let mut sprite = anims.sprite(AnimationClip::SkillImpact, Vec2::splat(205.0));
    sprite.color = Color::srgba(1.0, 0.36, 0.25, 0.90);
    commands.spawn((
        sprite,
        SpriteAnimation::once(AnimationClip::SkillImpact),
        BattleEffect::new(0.34, 0.46, 1.08, 1.15),
        Transform::from_xyz(ENEMY_POS.x - 22.0, ENEMY_POS.y + 24.0, 2.85),
        DespawnOnExit(AppState::Battle),
    ));

    let flash = lighting::spawn_light(
        commands,
        lights,
        Vec3::new(ENEMY_POS.x - 22.0, ENEMY_POS.y + 24.0, 2.0),
        250.0,
        Color::srgba(1.0, 0.28, 0.18, 0.44),
        AppState::Battle,
    );
    commands
        .entity(flash)
        .insert(BattleEffect::new(0.28, 0.28, 0.90, 0.0));
}

fn spawn_linger_support_aura(
    commands: &mut Commands,
    anims: &AnimationAssets,
    lights: &LightingAssets,
) {
    let mut sprite = anims.sprite(AnimationClip::SkillImpact, Vec2::splat(205.0));
    sprite.color = Color::srgba(0.46, 0.92, 1.0, 0.74);
    commands.spawn((
        sprite,
        SpriteAnimation::once(AnimationClip::SkillImpact),
        BattleEffect::new(0.52, 0.42, 1.05, -0.55),
        Transform::from_xyz(HERO_POS.x + 58.0, HERO_POS.y + 82.0, 2.7),
        DespawnOnExit(AppState::Battle),
    ));

    let flash = lighting::spawn_light(
        commands,
        lights,
        Vec3::new(HERO_POS.x + 58.0, HERO_POS.y + 82.0, 2.0),
        260.0,
        Color::srgba(0.38, 0.92, 1.0, 0.42),
        AppState::Battle,
    );
    commands
        .entity(flash)
        .insert(BattleEffect::new(0.42, 0.28, 0.96, 0.0));
}

fn spawn_spirit_witch_support_aura(
    commands: &mut Commands,
    anims: &AnimationAssets,
    lights: &LightingAssets,
) {
    let mut sprite = anims.sprite(AnimationClip::SkillImpact, Vec2::splat(185.0));
    sprite.color = Color::srgba(0.42, 1.0, 0.58, 0.68);
    commands.spawn((
        sprite,
        SpriteAnimation::once(AnimationClip::SkillImpact),
        BattleEffect::new(0.36, 0.34, 0.92, -0.72),
        Transform::from_xyz(HERO_POS.x + 68.0, HERO_POS.y + 82.0, 2.7),
        DespawnOnExit(AppState::Battle),
    ));

    let flash = lighting::spawn_light(
        commands,
        lights,
        Vec3::new(HERO_POS.x + 68.0, HERO_POS.y + 82.0, 2.0),
        210.0,
        Color::srgba(0.34, 1.0, 0.56, 0.36),
        AppState::Battle,
    );
    commands
        .entity(flash)
        .insert(BattleEffect::new(0.30, 0.24, 0.76, 0.0));
}

fn spawn_guard_halo(commands: &mut Commands, lights: &LightingAssets) {
    for (size, y, alpha) in [
        (Vec2::new(150.0, 190.0), 8.0, 0.18),
        (Vec2::new(170.0, 24.0), -85.0, 0.35),
    ] {
        commands.spawn((
            Sprite {
                image: lights.orb.clone(),
                color: Color::srgba(0.72, 0.90, 1.0, alpha),
                custom_size: Some(size),
                ..default()
            },
            BattleEffect::new(0.95, 0.70, 1.10, 0.0),
            Transform::from_xyz(HERO_POS.x, HERO_POS.y + y, 2.2),
            DespawnOnExit(AppState::Battle),
        ));
    }
}

fn spawn_weapon_hit(
    commands: &mut Commands,
    anims: &AnimationAssets,
    lights: &LightingAssets,
    pos: Vec3,
) {
    let mut sprite = anims.sprite(AnimationClip::SkillImpact, Vec2::splat(145.0));
    sprite.color = Color::srgba(1.0, 0.94, 0.64, 0.78);
    commands.spawn((
        sprite,
        SpriteAnimation::once(AnimationClip::SkillImpact),
        BattleEffect::new(0.34, 0.45, 0.86, -0.9),
        Transform::from_xyz(pos.x, pos.y + 8.0, 2.7),
        DespawnOnExit(AppState::Battle),
    ));

    let flash = lighting::spawn_light(
        commands,
        lights,
        Vec3::new(pos.x, pos.y + 8.0, 2.0),
        180.0,
        Color::srgba(1.0, 0.78, 0.28, 0.34),
        AppState::Battle,
    );
    commands
        .entity(flash)
        .insert(BattleEffect::new(0.24, 0.30, 0.72, 0.0));
}

fn spawn_enemy_strike_impact(
    commands: &mut Commands,
    anims: &AnimationAssets,
    lights: &LightingAssets,
    strong: bool,
) {
    let size = if strong { 230.0 } else { 175.0 };
    let mut sprite = anims.sprite(AnimationClip::SkillImpact, Vec2::splat(size));
    sprite.color = if strong {
        Color::srgba(1.0, 0.30, 0.18, 0.88)
    } else {
        Color::srgba(1.0, 0.55, 0.32, 0.76)
    };
    commands.spawn((
        sprite,
        SpriteAnimation::once(AnimationClip::SkillImpact),
        BattleEffect::new(0.38, 0.50, 1.0, 0.82),
        Transform::from_xyz(HERO_POS.x + 22.0, HERO_POS.y + 58.0, 2.75),
        DespawnOnExit(AppState::Battle),
    ));

    let flash = lighting::spawn_light(
        commands,
        lights,
        Vec3::new(HERO_POS.x + 22.0, HERO_POS.y + 58.0, 2.0),
        if strong { 300.0 } else { 220.0 },
        if strong {
            Color::srgba(1.0, 0.22, 0.12, 0.46)
        } else {
            Color::srgba(1.0, 0.50, 0.22, 0.34)
        },
        AppState::Battle,
    );
    commands
        .entity(flash)
        .insert(BattleEffect::new(0.30, 0.32, 0.94, 0.0));
}

fn spawn_damage_text(commands: &mut Commands, font: &GameFont, pos: Vec3, amount: i32) {
    spawn_floating_combat_text(
        commands,
        font,
        format!("-{amount}"),
        pos,
        Color::srgb(1.0, 0.42, 0.30),
        Vec2::new(8.0, 78.0),
        36.0,
    );
}

fn spawn_heal_text(commands: &mut Commands, font: &GameFont, pos: Vec3, amount: i32) {
    spawn_floating_combat_text(
        commands,
        font,
        format!("+{amount}"),
        pos,
        Color::srgb(0.48, 1.0, 0.62),
        Vec2::new(-4.0, 70.0),
        30.0,
    );
}

fn spawn_floating_combat_text(
    commands: &mut Commands,
    font: &GameFont,
    label: String,
    pos: Vec3,
    color: Color,
    velocity: Vec2,
    size: f32,
) {
    commands.spawn((
        FloatingCombatText::new(0.92, velocity, color),
        Text2d::new(label),
        font.text_font(size),
        TextColor(color),
        Transform::from_xyz(pos.x, pos.y, 4.2),
        DespawnOnExit(AppState::Battle),
    ));
}

// ---------------------------------------------------------------------------
// Phase machine for timed (non-menu) phases
// ---------------------------------------------------------------------------

fn battle_tick(
    mut commands: Commands,
    time: Res<Time>,
    font: Res<GameFont>,
    anims: Res<AnimationAssets>,
    lights: Res<LightingAssets>,
    vfx_assets: Res<SkillVfxAssets>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    mut state: ResMut<BattleState>,
    mut stats: ResMut<PlayerStats>,
    mut quest: ResMut<QuestLog>,
    encounter: Option<Res<PendingEncounter>>,
    mut rng: ResMut<Rng>,
    mut next: ResMut<NextState<AppState>>,
    mut run: Option<ResMut<RunState>>,
) {
    if state.phase == Phase::Menu {
        return;
    }

    state.timer -= time.delta_secs();
    if state.timer > 0.0 {
        return;
    }

    match state.phase {
        Phase::PlayerActing => {
            if state.enemy.hp <= 0 {
                let exp = state.enemy.exp;
                let kind = encounter.as_ref().map(|encounter| encounter.kind);
                let boss = matches!(kind, Some(EncounterKind::Boss(_)));

                // Roguelike run: no experience — gold, relic heals, then the
                // Reward screen (via Phase::Won) does the rest.
                if let Some(run) = run.as_mut() {
                    run.fights_won += 1;
                    let gold = ((battle_gold_reward(exp, boss) as f32
                        * run.gold_multiplier()
                        * run.hex_gold_mul())
                    .round() as u32)
                        + run.hex_win_gold();
                    stats.gold += gold;
                    state.message = format!("{} 被击败了！拾得 {} 文钱。", state.enemy.name, gold);
                    let tithe = run.hex_kill_heal();
                    if tithe > 0 && stats.hp < stats.max_hp {
                        let healed = tithe.min(stats.max_hp - stats.hp);
                        stats.hp += healed;
                        state
                            .message
                            .push_str(&format!("\n【血偿纹】饮下妖血,回复 {healed} 点气血。"));
                    }
                    let burn = run.hex_battle_end_hp_loss();
                    if burn > 0 {
                        stats.hp = (stats.hp - burn).max(1);
                        state
                            .message
                            .push_str(&format!("\n【燃魂纹】魂火灼身,失去 {burn} 点气血。"));
                    }
                    let heal = run.on_kill_heal();
                    if heal > 0 && stats.hp < stats.max_hp {
                        let healed = heal.min(stats.max_hp - stats.hp);
                        stats.hp += healed;
                        state
                            .message
                            .push_str(&format!("\n【嗜血珠】吸纳妖气，回复 {healed} 点气血。"));
                    }
                    let mana = run.on_kill_mana();
                    if mana > 0 && stats.mp < stats.max_mp {
                        let restored = mana.min(stats.max_mp - stats.mp);
                        stats.mp += restored;
                        state
                            .message
                            .push_str(&format!("\n【引魂灯】灯芯一亮，回复 {restored} 点灵力。"));
                    }
                    state.phase = Phase::Won;
                    state.timer = 1.2;
                    return;
                }

                let gold = battle_gold_reward(exp, boss);
                let levels = stats.gain_exp(exp);
                stats.gold += gold;
                state.message = if levels > 0 {
                    format!(
                        "{} 被击败了！获得 {} 点经验、{} 文钱。\n境界提升至 Lv.{}！",
                        state.enemy.name, exp, gold, stats.level
                    )
                } else {
                    format!(
                        "{} 被击败了！获得 {} 点经验、{} 文钱。",
                        state.enemy.name, exp, gold
                    )
                };
                let progress = match kind {
                    Some(EncounterKind::Boss(boss)) => quest.record_boss_victory(boss),
                    _ => quest.record_victory(),
                };
                let main_progressed = progress.is_some();
                if let Some(progress) = progress {
                    state.message.push('\n');
                    state.message.push_str(&progress);
                }
                if main_progressed
                    && let Some(EncounterKind::Boss(boss)) = kind
                    && let Some(breakthrough) = apply_legacy_boss_breakthrough(boss, &mut stats)
                {
                    state.message.push('\n');
                    state.message.push_str(&breakthrough);
                }
                if !matches!(kind, Some(EncounterKind::Boss(_))) {
                    if let Some(side_progress) = quest.record_side_victory() {
                        state.message.push('\n');
                        state.message.push_str(&side_progress);
                    }
                }
                if main_progressed {
                    if let Some(chapter_card) = quest.take_chapter_card() {
                        for line in chapter_card {
                            state.message.push('\n');
                            state.message.push_str(&line);
                        }
                    }
                }
                state.phase = Phase::Won;
                state.timer = 1.6;
            } else {
                // Boss 半血:先声夺人的变身回合(不出手),再入真身机制。
                if let EncounterKind::Boss(boss) = state.encounter_kind
                    && !state.boss_phase2
                    && state.enemy.hp * 2 <= state.enemy.max_hp
                {
                    state.boss_phase2 = true;
                    // 终章水影读你这一世的道心与情缘,决定用哪种方式压垮你。
                    state.eclipse_heart = run.as_ref().is_some_and(|r| r.qingyuan > r.daoxin);
                    let heart = state.eclipse_heart;
                    state.message = boss_phase2_transform(boss, &mut state.enemy, heart);
                    state.enemy_turns += 1;
                    state.intent = EnemyIntent::Strike;
                    state.intent = next_intent(&state, &mut rng);
                    state.guarding = false;
                    state.tactics.wait();
                    state.executing_intent = EnemyIntent::Gather;
                    state.enemy_attacked = false;
                    state.phase = Phase::EnemyActing;
                    state.phase_duration = 0.72;
                    state.timer = state.phase_duration;
                    state.player_action = None;
                    return;
                }
                let relic_guard = run.as_ref().map_or(0, |r| r.incoming_reduction());
                let strong_guard = run.as_ref().map_or(0, |r| r.strong_hit_guard());
                let guard_keep = run.as_ref().map_or(35, |r| r.guard_keep_pct());
                let enemy_first = run.as_ref().map_or(0, |r| r.hex_enemy_first_hit_bonus());
                let turn = begin_enemy_turn(
                    &mut state,
                    &mut stats,
                    &mut rng,
                    relic_guard,
                    strong_guard,
                    guard_keep,
                    enemy_first,
                );
                spawn_enemy_turn_result_vfx(
                    &mut commands,
                    &font,
                    &anims,
                    &lights,
                    &vfx_assets,
                    &mut color_materials,
                    turn,
                );
            }
        }
        Phase::EnemyActing => {
            if state.enemy.hp <= 0 && stats.hp > 0 {
                state.phase = Phase::PlayerActing;
                state.timer = 0.0;
                state.player_action = None;
                return;
            }
            if stats.hp <= 0 {
                // 檀木符:一次原地复活。
                if let Some(run) = run.as_mut() {
                    if run.try_revive() {
                        stats.hp = stats.max_hp / 2;
                        state.message =
                            "【檀木符】符纸燃尽,一缕暖意把你从鬼门关拽了回来!\n你的回合，请选择行动。"
                                .into();
                        state.phase = Phase::Menu;
                        state.player_action = None;
                        return;
                    }
                }
                stats.hp = 0;
                state.message = if run.is_some() {
                    "李逍遥 力竭倒地……此世轮回，到此为止。".into()
                } else {
                    "李逍遥 力竭倒地……\n（灵气护体，气血已被恢复）".into()
                };
                state.phase = Phase::Lost;
                state.timer = 1.6;
            } else {
                if let Some(run) = run.as_ref() {
                    let regen = run.bond_regen() + run.hex_enemy_turn_regen();
                    if regen > 0 && stats.hp > 0 && stats.hp < stats.max_hp {
                        let healed = regen.min(stats.max_hp - stats.hp);
                        stats.hp += healed;
                        spawn_heal_text(
                            &mut commands,
                            &font,
                            HERO_POS + Vec3::new(-24.0, 96.0, 0.0),
                            healed,
                        );
                    }
                }
                let run_ref = run.as_deref();
                let heal = companion_support(&quest, run_ref, state.bond_bonus, &mut stats);
                let restored = spirit_witch_support(&quest, run_ref, &mut stats);
                state.message.push('\n');
                state
                    .message
                    .push_str(&party_support_message(heal, restored));
                if let Some(heal) = heal {
                    spawn_linger_support_aura(&mut commands, &anims, &lights);
                    spawn_heal_text(
                        &mut commands,
                        &font,
                        HERO_POS + Vec3::new(12.0, 106.0, 0.0),
                        heal,
                    );
                }
                if let Some(restored) = restored {
                    spawn_spirit_witch_support_aura(&mut commands, &anims, &lights);
                    spawn_heal_text(
                        &mut commands,
                        &font,
                        HERO_POS + Vec3::new(74.0, 88.0, 0.0),
                        restored,
                    );
                }
                state.phase = Phase::Menu;
                state.player_action = None;
            }
        }
        Phase::Won => {
            commands.remove_resource::<PendingEncounter>();
            if run.is_some() {
                next.set(AppState::Reward);
            } else {
                next.set(AppState::Explore);
            }
        }
        Phase::Fled => {
            commands.remove_resource::<PendingEncounter>();
            if let Some(run) = run.as_mut() {
                run.current_fight = None;
                commands.remove_resource::<RunBattleMods>();
                next.set(AppState::RunScene);
            } else {
                next.set(AppState::Explore);
            }
        }
        Phase::Lost => {
            commands.remove_resource::<PendingEncounter>();
            if let Some(run) = run.as_mut() {
                run.outcome = Some(RunOutcome::Defeat);
                run.current_fight = None;
                commands.remove_resource::<RunBattleMods>();
                next.set(AppState::Ending);
            } else {
                stats.full_restore();
                next.set(AppState::Explore);
            }
        }
        Phase::Menu => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_enemy_turn_result_vfx(
    commands: &mut Commands,
    font: &GameFont,
    anims: &AnimationAssets,
    lights: &LightingAssets,
    vfx_assets: &SkillVfxAssets,
    color_materials: &mut Assets<ColorMaterial>,
    result: EnemyTurnResult,
) {
    if result.poison_damage > 0 {
        skill_vfx::spawn_status_burst(
            commands,
            color_materials,
            vfx_assets,
            lights,
            StatusBurstKind::Poison,
            ENEMY_POS + Vec3::new(0.0, 12.0, 2.7),
        );
        spawn_floating_combat_text(
            commands,
            font,
            format!("毒 -{}", result.poison_damage),
            ENEMY_POS + Vec3::new(82.0, 98.0, 0.0),
            Color::srgb(0.56, 1.0, 0.22),
            Vec2::new(8.0, 84.0),
            31.0,
        );
    }
    if result.burn_damage > 0 {
        skill_vfx::spawn_status_burst(
            commands,
            color_materials,
            vfx_assets,
            lights,
            StatusBurstKind::Burn,
            ENEMY_POS + Vec3::new(0.0, 12.0, 2.7),
        );
        spawn_floating_combat_text(
            commands,
            font,
            format!("火 -{}", result.burn_damage),
            ENEMY_POS + Vec3::new(118.0, 72.0, 0.0),
            Color::srgb(1.0, 0.48, 0.10),
            Vec2::new(-5.0, 88.0),
            31.0,
        );
    }
    if let Some(control) = result.control {
        skill_vfx::spawn_status_burst(
            commands,
            color_materials,
            vfx_assets,
            lights,
            control,
            ENEMY_POS + Vec3::new(0.0, 12.0, 2.7),
        );
    }
    if let Some(attack) = result.attack
        && attack.damage > 0
    {
        spawn_enemy_strike_impact(commands, anims, lights, attack.strong);
        spawn_damage_text(
            commands,
            font,
            HERO_POS + Vec3::new(34.0, 116.0, 0.0),
            attack.damage,
        );
    }
}

fn companion_support(
    quest: &QuestLog,
    run: Option<&RunState>,
    bond_bonus: Option<BondBonus>,
    stats: &mut PlayerStats,
) -> Option<i32> {
    if !battle_companion_visible(quest, run, Companion::Linger) || stats.hp >= stats.max_hp {
        return None;
    }

    let bond_heal = if matches!(bond_bonus, Some(BondBonus::Tender)) {
        2
    } else {
        0
    };
    let run_bond = run.map_or(0, |run| run.qingyuan.max(0) / 3);
    let heal = (4
        + stats.level as i32
        + quest.bond_level() as i32 * 2
        + quest.bond_tender_level() as i32
        + run_bond
        + bond_heal)
        .min(stats.max_hp - stats.hp);
    stats.hp += heal;
    Some(heal)
}

fn spirit_witch_support(
    quest: &QuestLog,
    run: Option<&RunState>,
    stats: &mut PlayerStats,
) -> Option<i32> {
    if !battle_companion_visible(quest, run, Companion::SpiritWitch) || stats.mp >= stats.max_mp {
        return None;
    }

    let restore = (2
        + stats.level as i32 / 3
        + if battle_has_late_spell(quest, run) {
            2
        } else {
            0
        })
    .min(stats.max_mp - stats.mp);
    stats.mp += restore;
    Some(restore)
}

fn party_support_message(heal: Option<i32>, restored: Option<i32>) -> String {
    match (heal, restored) {
        (Some(heal), Some(restored)) => format!(
            "赵灵儿 以灵息护住你，恢复 {heal} 点气血。\n南瑶 叩响袖中铜铃，回稳 {restored} 点灵力。\n你的回合，请选择行动。"
        ),
        (Some(heal), None) => {
            format!("赵灵儿 以灵息护住你，恢复 {heal} 点气血。\n你的回合，请选择行动。")
        }
        (None, Some(restored)) => {
            format!("南瑶 叩响袖中铜铃，回稳 {restored} 点灵力。\n你的回合，请选择行动。")
        }
        (None, None) => "你的回合，请选择行动。".into(),
    }
}

fn battle_gold_reward(exp: u32, boss: bool) -> u32 {
    exp / 2 + if boss { 40 } else { 6 }
}

fn apply_legacy_boss_breakthrough(boss: BossKind, stats: &mut PlayerStats) -> Option<String> {
    let (name, hp, mp, atk, def) = match boss {
        BossKind::MountainFiend => ("余杭赤火", 16, 6, 4, 2),
        BossKind::MoonWraith => ("水月灵誓", 18, 8, 4, 2),
        BossKind::RiverDemon => ("苏州河灯", 20, 8, 5, 2),
        BossKind::MiasmaRoot => ("白河清瘴", 22, 9, 5, 3),
        BossKind::MirrorMinister => ("京华破镜", 24, 9, 6, 3),
        BossKind::ThunderQilin => ("南疆雷誓", 26, 10, 6, 4),
        BossKind::DreamEclipse => {
            return Some(
                "【章末突破】心渊照影：终局不再增长数值，所有章印转为结局回响。".to_string(),
            );
        }
    };

    stats.max_hp += hp;
    stats.hp = (stats.hp + hp).min(stats.max_hp);
    stats.max_mp += mp;
    stats.mp = (stats.mp + mp).min(stats.max_mp);
    stats.atk += atk;
    stats.def += def;

    Some(format!(
        "【章末突破】{name}：气血+{hp} 灵力+{mp} 攻+{atk} 防+{def}。"
    ))
}

fn combo_unlocked(quest: &QuestLog) -> bool {
    quest.has_companion(Companion::Linger)
        && quest.has_companion(Companion::SwordSister)
        && quest.bond_level() > 0
}

fn combo_damage(
    quest: &QuestLog,
    stats: &PlayerStats,
    enemy: &EnemyInstance,
    rng: &mut Rng,
) -> i32 {
    (stats.atk * 3
        + quest.bond_level() as i32 * 5
        + quest.bond_courage_level() as i32 * 3
        + spirit_witch_combo_bonus(quest)
        + rng.range(4, 12)
        - enemy.def)
        .max(6)
}

fn spirit_witch_combo_bonus(quest: &QuestLog) -> i32 {
    if quest.has_companion(Companion::SpiritWitch) {
        6 + if quest.has_late_spell() { 3 } else { 0 }
    } else {
        0
    }
}

fn combo_party_message(quest: &QuestLog, damage: i32) -> String {
    if quest.has_companion(Companion::SpiritWitch) {
        format!(
            "李逍遥、赵灵儿、林月衡、南瑶 四人定阵，剑光、灵息与雷鼓旧律齐落，造成 {damage} 点伤害！"
        )
    } else {
        format!("李逍遥、赵灵儿、林月衡 心念相合，剑光与灵息齐落，造成 {damage} 点伤害！")
    }
}

fn blessing_damage_bonus(blessing: Option<ShrineBlessing>, action: PlayerAction) -> i32 {
    match (blessing, action) {
        (Some(ShrineBlessing::Sword), PlayerAction::Attack) => 4,
        (Some(ShrineBlessing::Sword), PlayerAction::Spell | PlayerAction::Combo) => 3,
        (Some(ShrineBlessing::Spirit), PlayerAction::Spell | PlayerAction::Combo) => 6,
        _ => 0,
    }
}

fn camp_damage_bonus(bonus: Option<CampBonus>, action: PlayerAction) -> i32 {
    match (bonus, action) {
        (Some(CampBonus::Warmth), PlayerAction::Attack | PlayerAction::Spell) => 1,
        (Some(CampBonus::Warmth), PlayerAction::Combo) => 2,
        (Some(CampBonus::Focus), PlayerAction::Attack | PlayerAction::Spell) => 2,
        (Some(CampBonus::Focus), PlayerAction::Combo) => 3,
        _ => 0,
    }
}

fn run_camp_damage_bonus(tactic: Option<RunCampTactic>, action: PlayerAction) -> i32 {
    match (tactic, action) {
        (Some(RunCampTactic::Breath), PlayerAction::Attack | PlayerAction::Spell) => 1,
        (Some(RunCampTactic::Breath), PlayerAction::Combo) => 2,
        (Some(RunCampTactic::SwordGuard), PlayerAction::Attack) => 3,
        (Some(RunCampTactic::SwordGuard), PlayerAction::Combo) => 4,
        (Some(RunCampTactic::SpiritFocus), PlayerAction::Spell) => 3,
        (Some(RunCampTactic::SpiritFocus), PlayerAction::Combo) => 4,
        _ => 0,
    }
}

fn run_camp_start_heal_amount(tactic: RunCampTactic) -> i32 {
    match tactic {
        RunCampTactic::Breath => 8,
        RunCampTactic::LingerWard => 10,
        RunCampTactic::SwordGuard | RunCampTactic::SpiritFocus => 0,
    }
}

fn run_camp_start_mana_amount(tactic: RunCampTactic) -> i32 {
    match tactic {
        RunCampTactic::Breath => 4,
        RunCampTactic::SpiritFocus => 8,
        RunCampTactic::SwordGuard | RunCampTactic::LingerWard => 0,
    }
}

fn apply_run_camp_start_heal(tactic: RunCampTactic, stats: &mut PlayerStats) -> Option<i32> {
    let amount = run_camp_start_heal_amount(tactic);
    if amount <= 0 || stats.hp >= stats.max_hp {
        return None;
    }
    let healed = amount.min(stats.max_hp - stats.hp);
    stats.hp += healed;
    Some(healed)
}

fn apply_run_camp_start_mana(tactic: RunCampTactic, stats: &mut PlayerStats) -> Option<i32> {
    let amount = run_camp_start_mana_amount(tactic);
    if amount <= 0 || stats.mp >= stats.max_mp {
        return None;
    }
    let restored = amount.min(stats.max_mp - stats.mp);
    stats.mp += restored;
    Some(restored)
}

fn run_camp_guard_mp_bonus(tactic: Option<RunCampTactic>) -> i32 {
    match tactic {
        Some(RunCampTactic::SpiritFocus) => 2,
        _ => 0,
    }
}

fn bond_damage_bonus(bonus: Option<BondBonus>, action: PlayerAction) -> i32 {
    match (bonus, action) {
        (Some(BondBonus::Courage), PlayerAction::Attack) => 2,
        (Some(BondBonus::Courage), PlayerAction::Spell) => 1,
        (Some(BondBonus::Courage), PlayerAction::Combo) => 4,
        (Some(BondBonus::Tender), PlayerAction::Spell | PlayerAction::Combo) => 2,
        _ => 0,
    }
}

fn append_blessing_damage_line(message: &mut String, blessing: Option<ShrineBlessing>, bonus: i32) {
    if bonus <= 0 {
        return;
    }

    if let Some(blessing) = blessing {
        message.push_str(&format!(
            "\n【祝福】{}追加 {bonus} 点威力。",
            blessing.name()
        ));
    }
}

fn append_camp_damage_line(message: &mut String, bonus: Option<CampBonus>, damage: i32) {
    if damage <= 0 {
        return;
    }

    if let Some(bonus) = bonus {
        message.push_str(&format!("\n【营地】{}追加 {damage} 点威力。", bonus.name()));
    }
}

fn append_run_camp_damage_line(message: &mut String, tactic: Option<RunCampTactic>, damage: i32) {
    if damage <= 0 {
        return;
    }

    if let Some(tactic) = tactic {
        message.push_str(&format!(
            "\n【营策】{}追加 {damage} 点威力。",
            tactic.name()
        ));
    }
}

fn append_bond_damage_line(message: &mut String, bonus: Option<BondBonus>, damage: i32) {
    if damage <= 0 {
        return;
    }

    if let Some(bonus) = bonus {
        message.push_str(&format!("\n【羁绊】{}追加 {damage} 点威力。", bonus.name()));
    }
}

fn sword_sister_followup(
    quest: &QuestLog,
    run: Option<&RunState>,
    stats: &PlayerStats,
    enemy: &mut EnemyInstance,
    rng: &mut Rng,
) -> Option<i32> {
    if !battle_companion_visible(quest, run, Companion::SwordSister) || enemy.hp <= 0 {
        return None;
    }

    let run_edge = run.map_or(0, |run| run.daoxin.max(0) / 4);
    let dmg = (stats.atk / 2
        + stats.level as i32
        + quest.bond_level() as i32
        + quest.bond_courage_level() as i32
        + run_edge
        + rng.range(0, 4)
        - enemy.def / 2)
        .max(2);
    enemy.hp -= dmg;
    Some(dmg)
}

fn apply_skill_status(state: &mut BattleState, status: SkillStatus, rng: &mut Rng) -> String {
    match status {
        SkillStatus::Poison { damage, turns } => {
            let poison = &mut state.statuses.poison;
            poison.damage = poison.damage.max(damage);
            poison.stacks = (poison.stacks + 1).min(3);
            poison.turns = turns.max(poison.turns);
            format!(
                "\n【中毒】毒息缠身:每层 {} 点,当前 {} 层,持续 {} 回合。",
                poison.damage, poison.stacks, poison.turns
            )
        }
        SkillStatus::Burn { damage, turns } => {
            let burn = &mut state.statuses.burn;
            if burn.turns > 0 {
                burn.damage = (burn.damage + damage).min(24);
                burn.turns = burn.turns.saturating_add(turns).min(5);
            } else {
                burn.damage = damage;
                burn.turns = turns.min(5);
                burn.stacks = 1;
            }
            format!(
                "\n【灼烧】妖火附体:行动后 {} 点,持续 {} 回合。",
                burn.damage, burn.turns
            )
        }
        SkillStatus::Freeze { chance } => {
            if state.statuses.freeze_immunity > 0
                && matches!(state.encounter_kind, EncounterKind::Boss(_))
            {
                "\n【抗冻】首领震碎寒意,本回合不会再次被冻结。".to_string()
            } else if rng.chance(chance) {
                state.statuses.frozen = true;
                "\n【冰冻】寒晶封住妖物,下一次行动将被跳过。".to_string()
            } else {
                "\n寒晶掠过,妖物没有被完全冻住。".to_string()
            }
        }
        SkillStatus::Stun { chance } => {
            if rng.chance(chance) {
                state.statuses.stunned = true;
                "\n【震慑】雷意锁住妖物,下一次行动将被跳过。".to_string()
            } else {
                "\n雷光散去,妖物勉强稳住了身形。".to_string()
            }
        }
    }
}

fn begin_enemy_turn(
    state: &mut BattleState,
    stats: &mut PlayerStats,
    rng: &mut Rng,
    relic_guard: i32,
    strong_guard: i32,
    guard_keep: i32,
    enemy_first_bonus: i32,
) -> EnemyTurnResult {
    let turn_index = state.enemy_turns;
    state.executing_intent = state.intent;
    state.enemy_attacked = false;
    state.enemy_turns += 1;
    let guarding = state.guarding;
    state.guarding = false;

    // 中毒先于敌方行动结算,因此致死时可以直接阻止这一手。
    let poison_damage = tick_poison(state);
    let poison_note = (poison_damage > 0)
        .then(|| format!("【中毒】毒息在行动前发作,造成 {poison_damage} 点伤害。"));
    if state.enemy.hp <= 0 {
        state.message = format!(
            "{}\n{} 毒发倒下,未能出手。",
            poison_note.as_deref().unwrap_or("【中毒】"),
            state.enemy.name
        );
        state.intent = EnemyIntent::Strike;
        state.intent = next_intent(state, rng);
        state.phase = Phase::EnemyActing;
        state.phase_duration = 0.34;
        state.timer = state.phase_duration;
        state.player_action = None;
        return EnemyTurnResult {
            poison_damage,
            ..default()
        };
    }

    // 冰冻与震慑都跳过行动;灼烧仍在这个回合结束时结算。
    let control = if state.statuses.frozen {
        state.statuses.frozen = false;
        if matches!(state.encounter_kind, EncounterKind::Boss(_)) {
            state.statuses.freeze_immunity = 1;
        }
        Some(StatusBurstKind::Freeze)
    } else if state.statuses.stunned {
        state.statuses.stunned = false;
        Some(StatusBurstKind::Stun)
    } else {
        None
    };
    if let Some(control) = control {
        let label = if control == StatusBurstKind::Freeze {
            "冰封碎裂"
        } else {
            "雷意震慑"
        };
        state.message = poison_note.unwrap_or_default();
        if !state.message.is_empty() {
            state.message.push('\n');
        }
        state.message.push_str(&format!(
            "{} 被{label}困住,这一回合动弹不得!",
            state.enemy.name
        ));
        let burn_damage = tick_burn(state);
        if control == StatusBurstKind::Stun {
            age_freeze_immunity(state);
        }
        state.intent = EnemyIntent::Strike;
        state.intent = next_intent(state, rng);
        state.phase = Phase::EnemyActing;
        state.phase_duration = 0.34;
        state.timer = state.phase_duration;
        state.player_action = None;
        return EnemyTurnResult {
            poison_damage,
            burn_damage,
            control: Some(control),
            ..default()
        };
    }

    if state.tactics.cancel_action(state.intent) {
        state.message = format!(
            "【破势】{} 架势崩解，预告的行动被打断！下一击可追击。",
            state.enemy.name
        );
        prepend_status_note(&mut state.message, poison_note.as_deref());
        let burn_damage = tick_burn(state);
        age_freeze_immunity(state);
        state.intent = EnemyIntent::Strike;
        state.intent = next_intent(state, rng);
        state.phase = Phase::EnemyActing;
        state.phase_duration = 0.42;
        state.timer = state.phase_duration;
        state.player_action = None;
        return EnemyTurnResult {
            poison_damage,
            burn_damage,
            ..default()
        };
    }

    if let EncounterKind::Boss(boss) = state.encounter_kind {
        if let Some(result) = begin_boss_special_turn(
            boss,
            turn_index,
            state,
            stats,
            rng,
            guarding,
            guard_keep,
            relic_guard,
            strong_guard,
        ) {
            state.enemy_attacked = true;
            if guarding {
                state.tactics.successful_guard(EnemyIntent::Special);
                state.momentum = (state.momentum + 1).min(MOMENTUM_MAX);
                state
                    .message
                    .push_str("\n【借势】御守秘法，反击已就绪，气势 +1。");
            }
            prepend_status_note(&mut state.message, poison_note.as_deref());
            let burn_damage = tick_burn(state);
            age_freeze_immunity(state);
            state.intent = EnemyIntent::Strike;
            state.intent = next_intent(state, rng);
            return EnemyTurnResult {
                attack: Some(result),
                poison_damage,
                burn_damage,
                control: None,
            };
        }
    }

    let intent = state.intent;
    let (dmg, strong) = if intent == EnemyIntent::Charge {
        state.message = format!(
            "{} 蓄势未发；下一手将释放{}，现在可以施压打断！",
            state.enemy.name,
            if matches!(state.encounter_kind, EncounterKind::Boss(_)) {
                "秘法"
            } else {
                "重击"
            }
        );
        (0, false)
    } else if intent == EnemyIntent::Gather {
        // 凝气回合不攻击；首领回复消耗本战有限的再生余力。
        let requested = (state.enemy.max_hp / 12).max(3);
        let heal = heal_enemy(state, requested);
        state.message = format!(
            "{} 凝气回息,恢复 {heal} 点气血,重新稳住了呼吸。",
            state.enemy.name
        );
        if guarding {
            state.message.push_str("\n御守落空——妖物这一手并未出击。");
        }
        (0, false)
    } else {
        let (mult, strong) = match intent {
            EnemyIntent::Heavy => (1.8, true),
            EnemyIntent::Drain => (0.7, false),
            EnemyIntent::Brace => (0.65, false),
            _ => (1.0, false),
        };
        state.enemy_attacked = true;
        if intent == EnemyIntent::Brace {
            state.tactics.pressure = state.tactics.pressure.saturating_sub(1);
        }
        let first_bonus = if turn_index == 0 {
            enemy_first_bonus
        } else {
            0
        };
        let raw = ((state.enemy.atk as f32 * mult) as i32 + first_bonus - stats.def
            + rng.range(-2, 4))
        .max(1);
        let blocked = blessing_guard_block(state.blessing, raw);
        let camp_blocked = camp_guard_block(state.camp_bonus, raw - blocked);
        let run_camp_blocked =
            run_camp_guard_block(state.run_camp_tactic, raw - blocked - camp_blocked);
        let bond_blocked = bond_guard_block(
            state.bond_bonus,
            raw - blocked - camp_blocked - run_camp_blocked,
        );
        let strong_cut = if strong { strong_guard } else { 0 };
        let mut dmg = (raw
            - blocked
            - camp_blocked
            - run_camp_blocked
            - bond_blocked
            - relic_guard
            - strong_cut)
            .max(0);
        let mut guard_note = String::new();
        if guarding {
            let absorbed = dmg - dmg * guard_keep / 100;
            dmg -= absorbed;
            guard_note = format!("\n李逍遥 御守卸力,挡下 {absorbed} 点伤害!");
        }
        if guarding && strong {
            state.tactics.successful_guard(intent);
            state.momentum = (state.momentum + 1).min(MOMENTUM_MAX);
            guard_note.push_str("\n【借势】重击被卸开，反击已就绪，气势 +1。");
        }
        state.message = match intent {
            EnemyIntent::Brace => format!(
                "{} 架甲反击，造成 {dmg} 点伤害，并回稳 1 点架势。",
                state.enemy.name
            ),
            EnemyIntent::Heavy => format!(
                "{} 蓄力已足,一记重击轰然落下!造成 {dmg} 点伤害!",
                state.enemy.name
            ),
            EnemyIntent::Drain => {
                let drained = 3.min(stats.mp);
                stats.mp -= drained;
                format!(
                    "{} 虚影缠身,造成 {dmg} 点伤害,并摄走 {drained} 点灵力!",
                    state.enemy.name
                )
            }
            _ => format!(
                "{} 张牙舞爪，对 李逍遥 造成 {dmg} 点伤害！",
                state.enemy.name
            ),
        };
        append_blessing_guard_line(&mut state.message, blocked);
        append_camp_guard_line(&mut state.message, state.camp_bonus, camp_blocked);
        append_run_camp_guard_line(&mut state.message, state.run_camp_tactic, run_camp_blocked);
        append_bond_guard_line(&mut state.message, state.bond_bonus, bond_blocked);
        append_relic_guard_line(&mut state.message, relic_guard);
        state.message.push_str(&guard_note);
        (dmg, strong)
    };
    let dmg = absorb_with_shield(state, dmg);
    stats.hp -= dmg;
    if state.boss_phase2 {
        match state.encounter_kind {
            EncounterKind::Boss(BossKind::MiasmaRoot) => {
                let heal = heal_enemy(state, 6);
                if heal > 0 {
                    state
                        .message
                        .push_str(&format!("\n根须自大地汲取生机,回复 {heal} 点气血。"));
                }
            }
            EncounterKind::Boss(BossKind::DreamEclipse) if state.eclipse_heart => {
                let heal = heal_enemy(state, 4);
                if heal > 0 {
                    state
                        .message
                        .push_str(&format!("\n旧梦潮水抚过伤口,水影回复 {heal} 点气血。"));
                }
            }
            EncounterKind::Boss(BossKind::MoonWraith) if dmg > 0 => {
                let drained = 2.min(stats.mp);
                if drained > 0 {
                    stats.mp -= drained;
                    state
                        .message
                        .push_str(&format!("\n月魄真形拂过,又摄走 {drained} 点灵力。"));
                }
            }
            _ => {}
        }
    }
    prepend_status_note(&mut state.message, poison_note.as_deref());
    let burn_damage = tick_burn(state);
    age_freeze_immunity(state);
    state.intent = next_intent(state, rng);
    state.phase = Phase::EnemyActing;
    state.phase_duration = if state.enemy_attacked { 0.60 } else { 0.34 };
    state.timer = state.phase_duration;
    state.player_action = None;
    EnemyTurnResult {
        attack: state.enemy_attacked.then_some(EnemyAttackResult {
            damage: dmg,
            strong,
        }),
        poison_damage,
        burn_damage,
        control: None,
    }
}

fn tick_poison(state: &mut BattleState) -> i32 {
    let poison = &mut state.statuses.poison;
    if poison.turns == 0 || poison.damage <= 0 || poison.stacks == 0 {
        return 0;
    }
    let raw = poison.damage * poison.stacks as i32;
    let damage = if matches!(state.encounter_kind, EncounterKind::Boss(_)) {
        raw.min((state.enemy.max_hp * 8 / 100).max(1))
    } else {
        raw
    };
    state.enemy.hp -= damage;
    poison.turns -= 1;
    if poison.turns == 0 {
        *poison = TimedDamageStatus::default();
    }
    damage
}

fn tick_burn(state: &mut BattleState) -> i32 {
    let burn = &mut state.statuses.burn;
    if burn.turns == 0 || burn.damage <= 0 {
        return 0;
    }
    let damage = burn.damage;
    state.enemy.hp -= damage;
    burn.turns -= 1;
    state.message.push_str(&format!(
        "\n【灼烧】妖火在行动后爆燃,造成 {damage} 点伤害。"
    ));
    if burn.turns == 0 {
        *burn = TimedDamageStatus::default();
    }
    damage
}

fn prepend_status_note(message: &mut String, note: Option<&str>) {
    let Some(note) = note else {
        return;
    };
    *message = format!("{note}\n{message}");
}

fn age_freeze_immunity(state: &mut BattleState) {
    if state.statuses.freeze_immunity > 0 && !state.statuses.frozen {
        state.statuses.freeze_immunity -= 1;
    }
}

/// Boss healing cannot create an endless damage/regeneration equilibrium.
fn heal_enemy(state: &mut BattleState, requested: i32) -> i32 {
    if state.enemy.hp <= 0 {
        return 0;
    }
    let mut heal = requested
        .max(0)
        .min((state.enemy.max_hp - state.enemy.hp).max(0));
    if matches!(state.encounter_kind, EncounterKind::Boss(_)) {
        heal = heal.min(state.boss_heal_remaining.max(0));
        state.boss_heal_remaining -= heal;
    }
    state.enemy.hp += heal;
    heal
}

/// 「护」形态的护罩:先于气血抵挡伤害,返回剩余伤害。
fn absorb_with_shield(state: &mut BattleState, dmg: i32) -> i32 {
    if state.player_shield <= 0 || dmg <= 0 {
        return dmg;
    }
    let absorbed = state.player_shield.min(dmg);
    state.player_shield -= absorbed;
    state
        .message
        .push_str(&format!("\n灵光护罩挡下 {absorbed} 点伤害。"));
    dmg - absorbed
}

/// 掷下一回合的意图;首领每逢秘法回合提前亮出重击预警,
/// 二阶段真身还会扭曲意图池(雷麟连环蓄力、月魄嗜灵)。
fn next_intent(state: &BattleState, _rng: &mut Rng) -> EnemyIntent {
    if state.intent == EnemyIntent::Charge {
        return if matches!(state.encounter_kind, EncounterKind::Boss(_)) {
            EnemyIntent::Special
        } else {
            EnemyIntent::Heavy
        };
    }
    if let EncounterKind::Boss(boss) = state.encounter_kind {
        if state.enemy_turns % boss_special_cadence(boss, state.boss_phase2) == 0 {
            return EnemyIntent::Charge;
        }
        return match (boss, state.boss_phase2, state.enemy_turns % 3) {
            (BossKind::MoonWraith, _, _) => EnemyIntent::Drain,
            (BossKind::MiasmaRoot, true, _) if state.boss_heal_remaining > 0 => EnemyIntent::Gather,
            (BossKind::MirrorMinister, _, 2) => EnemyIntent::Brace,
            (BossKind::DreamEclipse, true, _) if state.eclipse_heart => EnemyIntent::Drain,
            (_, _, _) => EnemyIntent::Strike,
        };
    }
    state.tactics.style.intent(state.enemy_turns)
}

/// 首领秘法节奏:默认每三回合;宿命水影二阶段加速到每两回合。
fn boss_special_cadence(boss: BossKind, phase2: bool) -> u32 {
    if phase2 && boss == BossKind::DreamEclipse {
        2
    } else {
        3
    }
}

fn begin_boss_special_turn(
    boss: BossKind,
    _turn_index: u32,
    state: &mut BattleState,
    stats: &mut PlayerStats,
    rng: &mut Rng,
    guarding: bool,
    guard_keep: i32,
    relic_guard: i32,
    strong_guard: i32,
) -> Option<EnemyAttackResult> {
    if state.intent != EnemyIntent::Special {
        return None;
    }

    let (raw, message, mp_drain, enemy_heal) = boss_special_attack(boss, state, stats, rng);
    let blocked = blessing_guard_block(state.blessing, raw);
    let camp_blocked = camp_guard_block(state.camp_bonus, raw - blocked);
    let run_camp_blocked =
        run_camp_guard_block(state.run_camp_tactic, raw - blocked - camp_blocked);
    let bond_blocked = bond_guard_block(
        state.bond_bonus,
        raw - blocked - camp_blocked - run_camp_blocked,
    );
    let mut dmg = (raw
        - blocked
        - camp_blocked
        - run_camp_blocked
        - bond_blocked
        - relic_guard
        - strong_guard)
        .max(0);
    let mut guard_note = String::new();
    if guarding {
        let absorbed = dmg - dmg * guard_keep / 100;
        dmg -= absorbed;
        guard_note = format!("\n李逍遥 御守卸力,挡下 {absorbed} 点伤害!");
    }
    state.message = message.to_string();
    let dmg = absorb_with_shield(state, dmg);
    let shield_note = state
        .message
        .strip_prefix(message)
        .unwrap_or("")
        .to_string();
    stats.hp -= dmg;
    if mp_drain > 0 {
        stats.mp = (stats.mp - mp_drain).max(0);
    }
    let enemy_heal = heal_enemy(state, enemy_heal);

    state.message = format!("{message}造成 {dmg} 点伤害！{shield_note}");
    append_relic_guard_line(&mut state.message, relic_guard + strong_guard);
    append_blessing_guard_line(&mut state.message, blocked);
    append_camp_guard_line(&mut state.message, state.camp_bonus, camp_blocked);
    append_run_camp_guard_line(&mut state.message, state.run_camp_tactic, run_camp_blocked);
    append_bond_guard_line(&mut state.message, state.bond_bonus, bond_blocked);
    state.message.push_str(&guard_note);
    if mp_drain > 0 {
        state
            .message
            .push_str(&format!("\n【首领】灵力被扰乱，失去 {mp_drain} 点灵力。"));
    }
    if enemy_heal > 0 {
        state.message.push_str(&format!(
            "\n【首领】{} 回稳 {enemy_heal} 点气血。",
            state.enemy.name
        ));
    }

    state.phase = Phase::EnemyActing;
    state.phase_duration = 0.72;
    state.timer = state.phase_duration;
    state.player_action = None;
    Some(EnemyAttackResult {
        damage: dmg,
        strong: true,
    })
}

/// 照影国师二阶段:玩家武力攻击被镜界照回两成。
fn mirror_backlash(state: &BattleState, stats: &mut PlayerStats, dmg: i32) -> Option<String> {
    if dmg <= 0 {
        return None;
    }
    if !state.boss_phase2
        || state.encounter_kind != EncounterKind::Boss(BossKind::MirrorMinister)
        || dmg <= 0
    {
        return None;
    }
    let backlash = (dmg / 5).max(1);
    stats.hp = (stats.hp - backlash).max(1);
    Some(format!(
        "\n【镜界】剑影被铜镜照回,你受到 {backlash} 点反噬。"
    ))
}

/// Boss 二阶段变身:属性调整 + 宣言文案。每个 boss 的真身机制不同,
/// 与 `next_intent` / `mirror_backlash` / 秘术节奏配合。
fn boss_phase2_transform(boss: BossKind, enemy: &mut EnemyInstance, eclipse_heart: bool) -> String {
    match boss {
        BossKind::MountainFiend => {
            enemy.atk += 3;
            format!(
                "{} 扯断身上赤绳,山火般的妖气沿石阶炸开!\n(攻击提升,秘法回合会重踏地脉)",
                enemy.name
            )
        }
        BossKind::MoonWraith => {
            enemy.atk += 2;
            format!(
                "{} 仰首长啸,月轮倒悬——妖身化作半透明的月魄真形!\n(攻击提升,此后招招摄取灵力)",
                enemy.name
            )
        }
        BossKind::RiverDemon => {
            enemy.atk += 6;
            enemy.def = (enemy.def - 2).max(0);
            format!(
                "{} 怒啸破浪,鳞甲尽张——狂化之下攻势滔天,破绽亦现!\n(攻击大幅提升,防御下降)",
                enemy.name
            )
        }
        BossKind::MiasmaRoot => format!(
            "{} 的根须疯长,扎入大地深处汲取生机!\n(地脉回复总量有限；耗尽后根须失去再生之力)",
            enemy.name
        ),
        BossKind::MirrorMinister => format!(
            "{} 袖中铜镜升空,镜界铺展——你的剑影会被照回来!\n(攻击将遭镜光反噬)",
            enemy.name
        ),
        BossKind::ThunderQilin => {
            enemy.atk += 3;
            format!(
                "{} 踏出雷劫连环的第一步,周身电弧不熄!\n(蓄力重击将接连不断——看准御守!)",
                enemy.name
            )
        }
        BossKind::DreamEclipse => {
            if eclipse_heart {
                format!(
                    "{} 水面一晃,竟化作灵儿的眉眼:「逍遥哥哥,别打了……」\n(以情乱心:招招摄灵,水影以有限余力自愈——稳住心神,耗尽它的旧梦)",
                    enemy.name
                )
            } else {
                enemy.atk += 2;
                format!(
                    "{} 举起一柄与你一模一样的剑,剑势如渊:「你的道,不过如此。」\n(以剑势压人:蓄力重击连绵不绝,秘法更密)",
                    enemy.name
                )
            }
        }
    }
}

fn boss_special_attack(
    boss: BossKind,
    state: &BattleState,
    stats: &PlayerStats,
    rng: &mut Rng,
) -> (i32, &'static str, i32, i32) {
    match boss {
        BossKind::MountainFiend => (
            (state.enemy.atk + 5 - stats.def / 2 + rng.range(0, 4)).max(2),
            "赤鬼山妖 重踏破庙石阶，碎石与妖火一并砸下，",
            0,
            0,
        ),
        BossKind::MoonWraith => (
            (state.enemy.atk + 5 - stats.def / 2 + rng.range(0, 4)).max(2),
            "月魄妖 施展月影噬灵，冷光穿过护体灵息，",
            3,
            0,
        ),
        BossKind::RiverDemon => (
            (state.enemy.atk + 7 - stats.def / 2 + rng.range(0, 5)).max(3),
            "河魇蛟 掀起逆浪压船，水势卷向众人，",
            0,
            0,
        ),
        BossKind::MiasmaRoot => (
            (state.enemy.atk + 4 - stats.def / 3 + rng.range(0, 4)).max(2),
            "瘴母根 喷出瘴雨毒雾，藤影从脚下缠上来，",
            2,
            0,
        ),
        BossKind::MirrorMinister => (
            (state.enemy.atk + 6 - stats.def / 2 + rng.range(0, 5)).max(3),
            "照影国师 开镜反照，剑影与心影同时刺回，",
            4,
            0,
        ),
        BossKind::ThunderQilin => (
            (state.enemy.atk + 9 - stats.def / 2 + rng.range(1, 6)).max(4),
            "雷麟 踏碎雷图腾，电光沿地脉炸开，",
            0,
            0,
        ),
        BossKind::DreamEclipse => (
            (state.enemy.atk + 7 - stats.def / 2 + rng.range(0, 6)).max(4),
            "宿命水影 拨动旧梦潮声，水光倒卷成刃，",
            3,
            10,
        ),
    }
}

fn blessing_guard_block(blessing: Option<ShrineBlessing>, raw_damage: i32) -> i32 {
    if blessing != Some(ShrineBlessing::Guard) {
        return 0;
    }

    5.min(raw_damage.saturating_sub(1))
}

fn camp_guard_block(bonus: Option<CampBonus>, remaining_damage: i32) -> i32 {
    let block = match bonus {
        Some(CampBonus::Warmth) => 1,
        Some(CampBonus::Vigil) => 3,
        Some(CampBonus::Focus) | None => 0,
    };
    block.min(remaining_damage.saturating_sub(1))
}

fn run_camp_guard_block(tactic: Option<RunCampTactic>, remaining_damage: i32) -> i32 {
    let block = match tactic {
        Some(RunCampTactic::Breath) => 1,
        Some(RunCampTactic::SwordGuard) => 1,
        Some(RunCampTactic::LingerWard) => 3,
        Some(RunCampTactic::SpiritFocus) | None => 0,
    };
    block.min(remaining_damage.saturating_sub(1))
}

fn bond_guard_block(bonus: Option<BondBonus>, remaining_damage: i32) -> i32 {
    let block = match bonus {
        Some(BondBonus::Tender) => 2,
        Some(BondBonus::Courage) => 1,
        None => 0,
    };
    block.min(remaining_damage.saturating_sub(1))
}

fn append_blessing_guard_line(message: &mut String, blocked: i32) {
    if blocked > 0 {
        message.push_str(&format!("\n【祝福】护身香火挡下 {blocked} 点伤害。"));
    }
}

fn append_camp_guard_line(message: &mut String, bonus: Option<CampBonus>, blocked: i32) {
    if blocked <= 0 {
        return;
    }

    if let Some(bonus) = bonus {
        message.push_str(&format!(
            "\n【营地】{}挡下 {blocked} 点伤害。",
            bonus.name()
        ));
    }
}

fn append_run_camp_guard_line(message: &mut String, tactic: Option<RunCampTactic>, blocked: i32) {
    if blocked <= 0 {
        return;
    }

    if let Some(tactic) = tactic {
        message.push_str(&format!(
            "\n【营策】{}挡下 {blocked} 点伤害。",
            tactic.name()
        ));
    }
}

fn append_relic_guard_line(message: &mut String, blocked: i32) {
    if blocked <= 0 {
        return;
    }
    message.push_str(&format!("\n【龟灵甲】护住要害，减免 {blocked} 点伤害。"));
}

fn append_bond_guard_line(message: &mut String, bonus: Option<BondBonus>, blocked: i32) {
    if blocked <= 0 {
        return;
    }

    if let Some(bonus) = bonus {
        message.push_str(&format!(
            "\n【羁绊】{}挡下 {blocked} 点伤害。",
            bonus.name()
        ));
    }
}

fn update_tactical_feedback(
    mut commands: Commands,
    font: Res<GameFont>,
    state: Res<BattleState>,
    mut seen: Local<(u32, u32)>,
) {
    if state.tactics.breaks < seen.0 || state.tactics.counters < seen.1 {
        *seen = (0, 0);
    }
    if state.tactics.breaks > seen.0 {
        spawn_floating_combat_text(
            &mut commands,
            &font,
            "破势！".into(),
            ENEMY_POS + Vec3::new(-32.0, 100.0, 3.0),
            super::ui::GOLD,
            Vec2::new(0.0, 55.0),
            38.0,
        );
    }
    if state.tactics.counters > seen.1 {
        spawn_floating_combat_text(
            &mut commands,
            &font,
            "反击就绪".into(),
            HERO_POS + Vec3::new(0.0, 80.0, 3.0),
            super::ui::GOLD,
            Vec2::new(0.0, 45.0),
            26.0,
        );
    }
    *seen = (state.tactics.breaks, state.tactics.counters);
}

fn update_battle_animations(
    time: Res<Time>,
    state: Res<BattleState>,
    anims: Res<AnimationAssets>,
    mut hero: Query<
        (&mut Sprite, &mut SpriteAnimation, &mut Transform),
        (
            With<BattleHero>,
            Without<BattleCompanion>,
            Without<BattleEnemy>,
        ),
    >,
    mut companions: Query<
        (&mut BattleCompanion, &mut Transform, &mut Sprite),
        (
            With<BattleCompanion>,
            Without<BattleHero>,
            Without<BattleEnemy>,
        ),
    >,
    mut enemy: Query<
        (
            &mut Transform,
            &mut BattleEnemyMotion,
            &mut Sprite,
            Option<&enemy_animation::EnemyAtlas>,
        ),
        (
            With<BattleEnemy>,
            Without<BattleHero>,
            Without<BattleCompanion>,
        ),
    >,
) {
    if let Ok((mut sprite, mut animation, mut transform)) = hero.single_mut() {
        let clip = if state.phase == Phase::PlayerActing
            && matches!(
                state.player_action,
                Some(PlayerAction::Attack | PlayerAction::Combo)
            ) {
            AnimationClip::HeroAttack
        } else {
            AnimationClip::HeroIdle
        };
        animation::set_clip(&anims, &mut sprite, &mut animation, clip);
        // Anticipation, a short grounded step, then recovery; recoil belongs
        // to the body while the contact shadow remains on the ground.
        let acting = state.phase == Phase::PlayerActing;
        let attacking = acting
            && matches!(
                state.player_action,
                Some(PlayerAction::Attack | PlayerAction::Combo)
            );
        let p = phase_progress(state.timer, state.phase_duration);
        let contact = ENEMY_POS.x - HERO_POS.x - 120.0;
        let step = if attacking {
            if p < 0.12 {
                -10.0 * (p / 0.12)
            } else if p < 0.30 {
                -10.0 + (contact + 10.0) * ((p - 0.12) / 0.18)
            } else if p < 0.68 {
                contact
            } else {
                contact * ((1.0 - p) / 0.32)
            }
        } else {
            0.0
        };
        let recoil = if state.phase == Phase::EnemyActing && state.enemy_attacked {
            (phase_progress(state.timer, state.phase_duration) * std::f32::consts::PI).sin()
        } else {
            0.0
        };
        transform.translation.x = HERO_POS.x + step - recoil * 20.0;
        transform.translation.y = HERO_POS.y;
        sprite.color = if recoil > 0.0 && state.tactics.riposte {
            Color::srgb(1.0, 0.94, 0.72)
        } else {
            Color::srgb(1.0, 1.0 - recoil * 0.35, 1.0 - recoil * 0.35)
        };
    }

    for (mut companion, mut transform, mut sprite) in &mut companions {
        companion.age += time.delta_secs();
        let visual_phase = if (state.phase == Phase::EnemyActing && !state.enemy_attacked)
            || (state.phase == Phase::PlayerActing && state.tactics.last_hit.damage == 0)
        {
            Phase::Menu
        } else {
            state.phase
        };
        let motion_timer = state.timer / state.phase_duration.max(0.001)
            * if visual_phase == Phase::EnemyActing {
                0.9
            } else {
                0.8
            };
        let (offset, scale, tint) = companion_motion(
            companion.kind,
            visual_phase,
            state.player_action,
            motion_timer,
            companion.age,
        );

        transform.translation.x = companion.origin.x + offset.x;
        transform.translation.y = companion.origin.y + offset.y;
        transform.scale = Vec3::splat(scale);
        sprite.color = tint;
    }

    if let Ok((mut transform, mut motion, mut sprite, atlas)) = enemy.single_mut() {
        motion.age += time.delta_secs();
        let breath = (motion.age * 2.0).sin();
        let visual_phase = if (state.phase == Phase::EnemyActing && !state.enemy_attacked)
            || (state.phase == Phase::PlayerActing && state.tactics.last_hit.damage == 0)
        {
            Phase::Menu
        } else {
            state.phase
        };
        let normalized_timer = (state.timer / state.phase_duration.max(0.001))
            * if visual_phase == Phase::EnemyActing {
                0.9
            } else {
                0.8
            };
        let (x, y, scale, hit) =
            enemy_phase_motion(visual_phase, state.player_action, normalized_timer);
        transform.translation.x = motion.origin.x + x;
        transform.translation.y = motion.origin.y + y;
        transform.scale = Vec3::new(scale, scale + breath * 0.008, 1.0);
        sprite.color = enemy_primary_color(state.phase, hit);
        if atlas.is_some() {
            enemy_animation::update(&mut sprite, &state, motion.age);
        }
    }
}

fn enemy_primary_image(def: &EnemyDef) -> &'static str {
    def.image
}

fn enemy_primary_size(def: &EnemyDef) -> f32 {
    def.size
}

fn enemy_phase_motion(
    phase: Phase,
    action: Option<PlayerAction>,
    timer: f32,
) -> (f32, f32, f32, f32) {
    let mut x = 0.0;
    let mut y = 0.0;
    let mut scale = 1.0;
    let mut hit = 0.0;

    match phase {
        Phase::EnemyActing => {
            let p = phase_progress(timer, 0.9);
            let lunge = (std::f32::consts::PI * p).sin();
            x -= 42.0 * lunge;
            y -= 7.0 * lunge;
            scale += 0.08 * lunge;
        }
        Phase::PlayerActing => {
            let p = phase_progress(timer, 0.8);
            hit = (std::f32::consts::PI * p).sin();
            let impact = match action {
                Some(PlayerAction::Spell) => 1.42,
                Some(PlayerAction::Combo) => 1.56,
                _ => 1.0,
            };
            let shake = if matches!(action, Some(PlayerAction::Spell | PlayerAction::Combo)) {
                (p * std::f32::consts::TAU * 3.0).sin() * 5.0
            } else {
                0.0
            };
            x += 18.0 * hit * impact + shake;
            y += 3.0 * hit * impact;
            scale -= 0.045 * hit * impact;
        }
        Phase::Won => {
            y -= 10.0;
            scale = 0.93;
        }
        Phase::Menu | Phase::Lost | Phase::Fled => {}
    }

    (x, y, scale.max(0.82), hit)
}

fn companion_motion(
    kind: BattleCompanionKind,
    phase: Phase,
    action: Option<PlayerAction>,
    timer: f32,
    age: f32,
) -> (Vec2, f32, Color) {
    let idle = (age * 2.5 + companion_phase_offset(kind)).sin();
    let mut offset = Vec2::new(0.0, idle * 4.0);
    let mut scale = 1.0 + idle * 0.012;
    let mut color = companion_tint(kind, 0.0);

    match phase {
        Phase::PlayerActing => {
            let p = phase_progress(timer, 0.8);
            let pulse = (std::f32::consts::PI * p).sin();
            match (kind, action) {
                (
                    BattleCompanionKind::SwordSister,
                    Some(PlayerAction::Attack | PlayerAction::Spell),
                ) => {
                    offset.x += 34.0 * pulse;
                    offset.y += 14.0 * pulse;
                    scale += 0.05 * pulse;
                    color = companion_tint(kind, pulse);
                }
                (BattleCompanionKind::SwordSister, Some(PlayerAction::Combo)) => {
                    offset.x += 30.0 * pulse;
                    offset.y += 12.0 * pulse;
                    scale += 0.06 * pulse;
                    color = companion_tint(kind, pulse);
                }
                (BattleCompanionKind::Linger, Some(PlayerAction::Combo)) => {
                    offset.x += 18.0 * pulse;
                    offset.y += 22.0 * pulse;
                    scale += 0.05 * pulse;
                    color = companion_tint(kind, pulse);
                }
                (
                    BattleCompanionKind::SpiritWitch,
                    Some(PlayerAction::Spell | PlayerAction::Combo),
                ) => {
                    offset.x += 12.0 * pulse;
                    offset.y += 28.0 * pulse;
                    scale += 0.045 * pulse;
                    color = companion_tint(kind, pulse);
                }
                _ => {}
            }
        }
        Phase::EnemyActing => {
            let p = phase_progress(timer, 0.9);
            let brace = (std::f32::consts::PI * p).sin();
            offset.x -= 5.0 * brace;
            offset.y -= 2.0 * brace;
            scale -= 0.012 * brace;
        }
        Phase::Menu | Phase::Won | Phase::Lost | Phase::Fled => {}
    }

    (offset, scale.max(0.86), color)
}

fn companion_tint(kind: BattleCompanionKind, pulse: f32) -> Color {
    match kind {
        BattleCompanionKind::Linger => Color::srgba(1.0 - pulse * 0.10, 1.0, 1.0, 1.0),
        BattleCompanionKind::SwordSister => {
            Color::srgba(1.0, 1.0 - pulse * 0.08, 1.0 - pulse * 0.14, 1.0)
        }
        BattleCompanionKind::SpiritWitch => {
            Color::srgba(1.0 - pulse * 0.18, 1.0, 1.0 - pulse * 0.10, 1.0)
        }
    }
}

fn companion_phase_offset(kind: BattleCompanionKind) -> f32 {
    match kind {
        BattleCompanionKind::Linger => 0.7,
        BattleCompanionKind::SwordSister => 1.4,
        BattleCompanionKind::SpiritWitch => 2.1,
    }
}

fn phase_progress(timer: f32, duration: f32) -> f32 {
    (1.0 - timer / duration).clamp(0.0, 1.0)
}

fn enemy_primary_color(phase: Phase, hit: f32) -> Color {
    match phase {
        Phase::PlayerActing if hit > 0.48 => Color::srgba(1.0, 0.72, 0.72, 0.98),
        Phase::EnemyActing => Color::srgba(1.0, 0.96, 0.86, 1.0),
        Phase::Won => Color::srgba(0.70, 0.78, 0.92, 0.34),
        _ => Color::srgba(1.0, 1.0, 1.0, 0.98),
    }
}

fn update_battle_effects(
    time: Res<Time>,
    mut commands: Commands,
    mut effects: Query<(Entity, &mut BattleEffect, &mut Sprite, &mut Transform)>,
) {
    for (entity, mut effect, mut sprite, mut transform) in &mut effects {
        effect.age += time.delta_secs();
        let p = (effect.age / effect.duration).clamp(0.0, 1.0);
        let eased = 1.0 - (1.0 - p).powi(3);

        transform.scale = Vec3::splat(effect.start_scale.lerp(effect.end_scale, eased));
        if effect.spin != 0.0 {
            transform.rotate_z(effect.spin * time.delta_secs());
        }

        let mut srgba = sprite.color.to_srgba();
        let fade = if p < 0.68 {
            1.0
        } else {
            1.0 - ((p - 0.68) / 0.32).clamp(0.0, 1.0)
        };
        srgba.alpha *= fade;
        sprite.color = Color::Srgba(srgba);

        if effect.age >= effect.duration {
            commands.entity(entity).despawn();
        }
    }
}

fn update_floating_combat_text(
    time: Res<Time>,
    mut commands: Commands,
    mut texts: Query<(
        Entity,
        &mut FloatingCombatText,
        &mut Transform,
        &mut TextColor,
    )>,
) {
    for (entity, mut floating, mut transform, mut color) in &mut texts {
        let dt = time.delta_secs();
        floating.age += dt;
        transform.translation.x += floating.velocity.x * dt;
        transform.translation.y += floating.velocity.y * dt;

        let p = (floating.age / floating.duration).clamp(0.0, 1.0);
        let mut srgba = floating.color.to_srgba();
        srgba.alpha *= 1.0 - p.powi(2);
        *color = TextColor(Color::Srgba(srgba));

        if floating.age >= floating.duration {
            commands.entity(entity).despawn();
        }
    }
}

// ---------------------------------------------------------------------------
// UI refresh
// ---------------------------------------------------------------------------

fn sync_status_vfx_state(state: Res<BattleState>, mut vfx: ResMut<StatusVfxState>) {
    *vfx = StatusVfxState {
        poison_stacks: state.statuses.poison.stacks,
        burn_turns: state.statuses.burn.turns,
        frozen: state.statuses.frozen,
        stunned: state.statuses.stunned,
    };
}

fn battle_blessing_summary(blessing: Option<ShrineBlessing>) -> &'static str {
    match blessing {
        Some(ShrineBlessing::Guard) => "祝福 护身",
        Some(ShrineBlessing::Sword) => "祝福 剑心",
        Some(ShrineBlessing::Spirit) => "祝福 灵息",
        None => "祝福 无",
    }
}

fn battle_camp_summary(bonus: Option<CampBonus>, tactic: Option<RunCampTactic>) -> String {
    match (bonus, tactic) {
        (Some(bonus), Some(tactic)) => format!("营地 {} / 营策 {}", bonus.name(), tactic.name()),
        (_, Some(tactic)) => format!("营策 {}", tactic.name()),
        (Some(CampBonus::Warmth), None) => "营地 余温".to_string(),
        (Some(CampBonus::Focus), None) => "营地 静心".to_string(),
        (Some(CampBonus::Vigil), None) => "营地 守夜".to_string(),
        (None, None) => "营地 无".to_string(),
    }
}

fn battle_bond_summary(bonus: Option<BondBonus>) -> &'static str {
    match bonus {
        Some(BondBonus::Courage) => "护念 勇心",
        Some(BondBonus::Tender) => "护念 柔心",
        None => "护念 无",
    }
}

// ---------------------------------------------------------------------------
// 二十式仙术秘卷:逐招结算与独立特效
// ---------------------------------------------------------------------------

use super::roguelike::skill::{self, SkillKind, SkillStatus};

/// Preserve the first three chapters; practiced skills grow from chapter four.
pub fn skill_mastery_bonus(run: &RunState) -> i32 {
    let chapter = run.chapter.min(6);
    chapter.saturating_sub(2) as i32 * 2 + chapter.saturating_sub(4) as i32 * 2
}

pub fn skill_cost(id: skill::SkillId, run: &RunState) -> i32 {
    (skill::skill(id).cost + run.spell_cost_delta() + run.hex_spell_cost_delta()).max(2)
}

/// Apply offensive bonuses once per cast, including multi-hit skills.
fn skill_hit_damage(state: &mut BattleState, run: &RunState, raw: i32, kind: StrikeKind) -> i32 {
    let camp = run_camp_damage_bonus(state.run_camp_tactic, PlayerAction::Spell);
    let multiplier = run.hunter_multiplier(run.current_fight.unwrap_or(FightRank::Normal));
    let damage = ((raw + run.spell_bonus() + camp) as f32 * multiplier).round() as i32;
    tactical_damage(state, damage, kind)
}

/// 施放一式技能:扣灵、结算、叠气势。返回 false 表示灵力不足未施放。
#[allow(clippy::too_many_arguments)]
fn cast_skill(
    commands: &mut Commands,
    font: &GameFont,
    lights: &LightingAssets,
    vfx_assets: &SkillVfxAssets,
    color_materials: &mut Assets<ColorMaterial>,
    state: &mut BattleState,
    stats: &mut PlayerStats,
    run: &super::roguelike::RunState,
    rng: &mut Rng,
    id: skill::SkillId,
) -> bool {
    let def = skill::skill(id);
    let cost = skill_cost(id, run);
    if stats.mp < cost {
        state.message = format!("灵力不足,施展不出「{}」(需 {cost} 灵)。", def.name());
        return false;
    }
    stats.mp -= cost;
    let hex_spell = run.hex_spell_bonus();
    let base = (def.power + skill_mastery_bonus(run) + hex_spell + rng.range(-2, 3)).max(1);
    let name = def.name();
    let strike_kind = if matches!(
        def.kind,
        SkillKind::Bind | SkillKind::Flurry | SkillKind::Stun
    ) {
        StrikeKind::Breaker
    } else {
        StrikeKind::Spell
    };
    let before_hp = state.enemy.hp;
    match def.kind {
        SkillKind::Slash | SkillKind::Burst => {
            let dmg = (base - state.enemy.def / 2).max(2);
            let dmg = skill_hit_damage(state, run, dmg, strike_kind);
            state.enemy.hp -= dmg;
            state.message = format!("「{name}」剑气迸发,造成 {dmg} 点元素伤害!");
            spawn_damage_text(commands, font, ENEMY_POS + Vec3::new(104.0, 76.0, 0.0), dmg);
        }
        SkillKind::Corrode => {
            let dmg = (base - state.enemy.def / 3).max(2);
            let dmg = skill_hit_damage(state, run, dmg, strike_kind);
            state.enemy.hp -= dmg;
            state.message = format!("「{name}」蚀入妖躯,造成 {dmg} 点伤害!");
            spawn_damage_text(commands, font, ENEMY_POS + Vec3::new(104.0, 76.0, 0.0), dmg);
        }
        SkillKind::Bind => {
            let dmg = (base - state.enemy.def / 3).max(2);
            let dmg = skill_hit_damage(state, run, dmg, strike_kind);
            state.enemy.hp -= dmg;
            state.enemy.def = (state.enemy.def - 3).max(0);
            state.message = format!("「{name}」缠住妖物,造成 {dmg} 点伤害并削去 3 点防御!");
            spawn_damage_text(commands, font, ENEMY_POS + Vec3::new(104.0, 76.0, 0.0), dmg);
        }
        SkillKind::Drain => {
            let dmg = (base - state.enemy.def / 2).max(2);
            let dmg = skill_hit_damage(state, run, dmg, strike_kind);
            state.enemy.hp -= dmg;
            let heal = (dmg / 2).min(stats.max_hp - stats.hp).max(0);
            stats.hp += heal;
            state.message = format!("「{name}」摄取精魄,造成 {dmg} 点伤害,吸回 {heal} 点气血!");
            spawn_damage_text(commands, font, ENEMY_POS + Vec3::new(104.0, 76.0, 0.0), dmg);
            if heal > 0 {
                spawn_heal_text(commands, font, HERO_POS + Vec3::new(10.0, 100.0, 0.0), heal);
            }
        }
        SkillKind::Flurry => {
            let hits = rng.range(2, 3);
            let mut total = 0;
            for _ in 0..hits {
                total += (base - state.enemy.def / 2 + rng.range(-1, 2)).max(1);
            }
            let total = skill_hit_damage(state, run, total, strike_kind);
            state.enemy.hp -= total;
            state.message = format!("「{name}」连绵 {hits} 段,共造成 {total} 点伤害!");
            spawn_damage_text(
                commands,
                font,
                ENEMY_POS + Vec3::new(104.0, 76.0, 0.0),
                total,
            );
        }
        SkillKind::Ward => {
            state.tactics.wait();
            state.player_shield = state.player_shield.max(base);
            state.message = format!("「{name}」结成 {base} 点灵光护罩,先于气血抵伤。");
        }
        SkillKind::Mend => {
            state.tactics.wait();
            let heal = base.min(stats.max_hp - stats.hp).max(0);
            stats.hp += heal;
            state.message = format!("「{name}」灵息回环,恢复 {heal} 点气血。");
            if heal > 0 {
                spawn_heal_text(commands, font, HERO_POS + Vec3::new(10.0, 100.0, 0.0), heal);
            }
        }
        SkillKind::Stun => {
            let dmg = (base - state.enemy.def / 3).max(2);
            let dmg = skill_hit_damage(state, run, dmg, strike_kind);
            state.enemy.hp -= dmg;
            state.message = format!("「{name}」轰然震荡,造成 {dmg} 点伤害!");
            spawn_damage_text(commands, font, ENEMY_POS + Vec3::new(104.0, 76.0, 0.0), dmg);
        }
        SkillKind::Execute => {
            let low = state.enemy.hp * 10 < state.enemy.max_hp * 3;
            let mut dmg = (base - state.enemy.def / 2).max(2);
            if low {
                dmg *= 2;
            }
            let dmg = skill_hit_damage(state, run, dmg, strike_kind);
            state.enemy.hp -= dmg;
            state.message = if low {
                format!("「{name}」诛机已现——{dmg} 点致命一击!")
            } else {
                format!("「{name}」落下,造成 {dmg} 点伤害。")
            };
            spawn_damage_text(commands, font, ENEMY_POS + Vec3::new(104.0, 76.0, 0.0), dmg);
        }
    }
    if !matches!(def.kind, SkillKind::Ward | SkillKind::Mend) {
        let camp_damage = run_camp_damage_bonus(state.run_camp_tactic, PlayerAction::Spell);
        append_run_camp_damage_line(&mut state.message, state.run_camp_tactic, camp_damage);
        let bonus = run.spell_bonus();
        if bonus > 0 {
            state
                .message
                .push_str(&format!("\n【御剑心诀】法宝助术，伤害 +{bonus}。"));
        }
    }
    if state.enemy.hp > 0
        && let Some(status) = def.status
    {
        let status_line = apply_skill_status(state, status, rng);
        state.message.push_str(&status_line);
    }
    skill_vfx::spawn_skill_vfx(
        commands,
        color_materials,
        vfx_assets,
        lights,
        def.vfx,
        HERO_POS + Vec3::new(18.0, 38.0, 2.4),
        ENEMY_POS + Vec3::new(0.0, 12.0, 2.5),
    );
    state.momentum = (state.momentum + 1).min(MOMENTUM_MAX);
    if let Some(line) = mirror_backlash(state, stats, (before_hp - state.enemy.hp).max(0)) {
        state.message.push_str(&line);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::quest::{
        BondResponse, MansionMirrorNode, MoonCrystal, PlagueWard, QuestRole, RiverLantern,
    };
    use crate::game::roguelike::RunChapterVow;
    use crate::game::roguelike::graph::NodeKind;

    pub(super) fn status_test_state(encounter_kind: EncounterKind) -> BattleState {
        BattleState {
            enemy: EnemyInstance {
                name: "试炼妖".into(),
                hp: 100,
                max_hp: 100,
                atk: 8,
                def: 2,
                exp: 0,
            },
            encounter_kind,
            phase: Phase::PlayerActing,
            timer: 0.0,
            enemy_turns: 0,
            tactics: Tactics::new(
                EnemyStyle::Assault,
                matches!(encounter_kind, EncounterKind::Boss(_)),
            ),
            executing_intent: EnemyIntent::Strike,
            enemy_attacked: false,
            phase_duration: 0.8,
            menu_index: 0,
            player_action: Some(PlayerAction::Spell),
            intent: EnemyIntent::Strike,
            guarding: false,
            momentum: 0,
            boss_phase2: false,
            boss_heal_remaining: 25,
            eclipse_heart: false,
            hex_first_hit_used: false,
            skill_menu: false,
            skill_cursor: 0,
            player_shield: 0,
            statuses: EnemyStatuses::default(),
            message: String::new(),
            blessing: None,
            camp_bonus: None,
            run_camp_tactic: None,
            bond_bonus: None,
        }
    }

    #[test]
    fn poison_stacks_to_three_and_refreshes_duration() {
        let mut state = status_test_state(EncounterKind::Random);
        let mut rng = Rng::default();
        for _ in 0..4 {
            apply_skill_status(
                &mut state,
                SkillStatus::Poison {
                    damage: 3,
                    turns: 3,
                },
                &mut rng,
            );
        }
        assert_eq!(state.statuses.poison.damage, 3);
        assert_eq!(state.statuses.poison.stacks, 3);
        assert_eq!(state.statuses.poison.turns, 3);
    }

    #[test]
    fn lethal_poison_prevents_the_enemy_action_and_burn_tick() {
        let mut state = status_test_state(EncounterKind::Random);
        state.enemy.hp = 5;
        state.statuses.poison = TimedDamageStatus {
            damage: 5,
            turns: 2,
            stacks: 1,
        };
        state.statuses.burn = TimedDamageStatus {
            damage: 4,
            turns: 2,
            stacks: 1,
        };
        let mut stats = PlayerStats::default();
        let hp_before = stats.hp;
        let mut rng = Rng::default();

        let turn = begin_enemy_turn(&mut state, &mut stats, &mut rng, 0, 0, 35, 0);

        assert_eq!(turn.poison_damage, 5);
        assert_eq!(turn.burn_damage, 0);
        assert!(turn.attack.is_none());
        assert_eq!(stats.hp, hp_before);
        assert!(state.message.contains("未能出手"));
    }

    #[test]
    fn burn_resolves_after_the_enemy_action() {
        let mut state = status_test_state(EncounterKind::Random);
        state.statuses.burn = TimedDamageStatus {
            damage: 4,
            turns: 2,
            stacks: 1,
        };
        let enemy_before = state.enemy.hp;
        let mut stats = PlayerStats::default();
        let player_before = stats.hp;
        let mut rng = Rng::default();

        let turn = begin_enemy_turn(&mut state, &mut stats, &mut rng, 0, 0, 35, 0);

        assert!(turn.attack.is_some());
        assert!(stats.hp < player_before);
        assert_eq!(turn.burn_damage, 4);
        assert_eq!(state.enemy.hp, enemy_before - 4);
        assert!(state.message.contains("行动后爆燃"));
    }

    #[test]
    fn boss_gains_one_turn_of_freeze_immunity_after_thawing() {
        let mut state = status_test_state(EncounterKind::Boss(BossKind::MountainFiend));
        let mut stats = PlayerStats::default();
        let mut rng = Rng::default();
        apply_skill_status(&mut state, SkillStatus::Freeze { chance: 1.0 }, &mut rng);
        assert!(state.statuses.frozen);

        let frozen_turn = begin_enemy_turn(&mut state, &mut stats, &mut rng, 0, 0, 35, 0);
        assert_eq!(frozen_turn.control, Some(StatusBurstKind::Freeze));
        assert_eq!(state.statuses.freeze_immunity, 1);

        let resisted =
            apply_skill_status(&mut state, SkillStatus::Freeze { chance: 1.0 }, &mut rng);
        assert!(resisted.contains("抗冻"));
        assert!(!state.statuses.frozen);

        state.phase = Phase::PlayerActing;
        let _ = begin_enemy_turn(&mut state, &mut stats, &mut rng, 0, 0, 35, 0);
        assert_eq!(state.statuses.freeze_immunity, 0);
        apply_skill_status(&mut state, SkillStatus::Freeze { chance: 1.0 }, &mut rng);
        assert!(state.statuses.frozen);
    }

    #[test]
    fn run_party_companions_are_visible_in_run_battles() {
        let mut rng = Rng::default();
        let mut run = RunState::new(&mut rng);
        run.stage = 3;
        let quest = QuestLog::default();
        assert!(battle_companion_visible(
            &quest,
            Some(&run),
            Companion::Linger
        ));
        assert!(!battle_companion_visible(
            &quest,
            Some(&run),
            Companion::SwordSister
        ));

        run.chapter = 5;
        run.stage = 0;
        assert!(battle_companion_visible(
            &quest,
            Some(&run),
            Companion::SpiritWitch
        ));
        assert_eq!(
            battle_party_summary(&quest, Some(&run)),
            run.party_summary()
        );
        assert_eq!(battle_spell_name(&quest, Some(&run)), "万剑诀");
        assert_eq!(battle_spell_cost(&quest, Some(&run)), 9);
        assert_eq!(battle_spell_power_multiplier(&quest, Some(&run)), 3);
    }

    #[test]
    fn run_party_companions_drive_support_and_followups() {
        let mut rng = Rng::default();
        let quest = QuestLog::default();
        let mut run = RunState::new(&mut rng);
        run.stage = 3;
        run.qingyuan = 3;

        let mut stats = PlayerStats::default();
        stats.hp = stats.max_hp - 20;
        assert_eq!(
            companion_support(&quest, Some(&run), None, &mut stats),
            Some(6)
        );

        run.chapter = 3;
        run.stage = 1;
        run.daoxin = 4;
        let mut enemy = EnemyInstance {
            name: "试炼妖".into(),
            hp: 40,
            max_hp: 40,
            atk: 1,
            def: 4,
            exp: 0,
        };
        let before = enemy.hp;
        let dmg = sword_sister_followup(&quest, Some(&run), &stats, &mut enemy, &mut rng)
            .expect("run sword sister should follow up");
        assert!(dmg >= 2);
        assert_eq!(enemy.hp, before - dmg);

        run.chapter = 5;
        run.stage = 0;
        stats.mp = stats.max_mp - 8;
        assert_eq!(
            spirit_witch_support(&quest, Some(&run), &mut stats),
            Some(4)
        );
    }

    #[test]
    fn chapter_vow_weakens_matching_run_boss_opening() {
        let mut rng = Rng::default();
        let mut run = RunState::new(&mut rng);
        run.record_chapter_vow(0, RunChapterVow::Heart);
        let mut enemy = EnemyInstance {
            name: "赤鬼山妖".into(),
            hp: 100,
            max_hp: 100,
            atk: 30,
            def: 6,
            exp: 0,
        };
        let mut message = "一只赤鬼山妖拦住了去路！".to_string();

        apply_run_chapter_vow_to_boss(
            &run,
            EncounterKind::Boss(BossKind::MountainFiend),
            &mut enemy,
            &mut message,
        );

        assert_eq!(enemy.max_hp, 100);
        assert_eq!(enemy.atk, 27);
        assert!(message.contains("本卷誓记·护心誓"));

        run.next_chapter(&mut rng);
        run.record_chapter_vow(1, RunChapterVow::Resolve);
        let mut boss = EnemyInstance {
            name: "月魄妖".into(),
            hp: 100,
            max_hp: 100,
            atk: 30,
            def: 7,
            exp: 0,
        };
        let mut boss_message = String::new();
        apply_run_chapter_vow_to_boss(
            &run,
            EncounterKind::Boss(BossKind::MoonWraith),
            &mut boss,
            &mut boss_message,
        );
        assert_eq!(boss.max_hp, 92);
        assert_eq!(boss.hp, 92);
        assert_eq!(boss.atk, 30);

        let mut random = EnemyInstance {
            name: "路边妖".into(),
            hp: 100,
            max_hp: 100,
            atk: 30,
            def: 4,
            exp: 0,
        };
        apply_run_chapter_vow_to_boss(&run, EncounterKind::Random, &mut random, &mut boss_message);
        assert_eq!(random.max_hp, 100);
        assert_eq!(random.atk, 30);
    }

    #[test]
    fn run_route_guidance_prepares_chapter_boss_opening() {
        let mut rng = Rng::default();
        let mut run = RunState::new(&mut rng);
        assert!(run.accept_route_commission(NodeKind::Story));
        assert!(run.complete_route_commission(NodeKind::Story));
        run.stage = 1;
        assert!(run.accept_route_commission(NodeKind::Event));
        assert!(run.complete_route_commission(NodeKind::Event));
        run.stage = run.stage_count() - 1;

        let mut stats = PlayerStats {
            hp: 40,
            max_hp: 100,
            mp: 3,
            max_mp: 20,
            ..Default::default()
        };
        let mut enemy = EnemyInstance {
            name: "赤鬼山妖".into(),
            hp: 100,
            max_hp: 100,
            atk: 50,
            def: 6,
            exp: 0,
        };
        let mut message = "一只 赤鬼山妖 拦住了去路！".to_string();

        apply_run_route_guidance_to_boss(
            &run,
            EncounterKind::Boss(BossKind::MountainFiend),
            &mut stats,
            &mut enemy,
            &mut message,
        );

        assert_eq!(enemy.max_hp, 100);
        assert_eq!(enemy.hp, 100);
        assert_eq!(enemy.atk, 48);
        assert_eq!(stats.hp, 48);
        assert_eq!(stats.mp, 7);
        assert!(message.contains("本卷路人照应·乡路照应"));
        assert!(message.contains("气血 +8 灵力 +4"));

        run.stage = 2;
        assert!(run.accept_route_commission(NodeKind::Rest));
        assert!(run.complete_route_commission(NodeKind::Rest));
        run.stage = run.stage_count() - 1;
        let mut stats = PlayerStats {
            hp: 40,
            max_hp: 100,
            mp: 3,
            max_mp: 20,
            ..Default::default()
        };
        let mut enemy = EnemyInstance {
            name: "赤鬼山妖".into(),
            hp: 100,
            max_hp: 100,
            atk: 50,
            def: 6,
            exp: 0,
        };
        let mut message = String::new();

        apply_run_route_guidance_to_boss(
            &run,
            EncounterKind::Boss(BossKind::MountainFiend),
            &mut stats,
            &mut enemy,
            &mut message,
        );

        assert_eq!(enemy.max_hp, 96);
        assert_eq!(enemy.hp, 96);
        assert_eq!(enemy.atk, 46);
        assert_eq!(stats.hp, 52);
        assert_eq!(stats.mp, 11);
        assert!(message.contains("本卷路人照应·熟路照应"));
        assert!(message.contains("首领气血-4% 攻势-8%"));

        let mut random_enemy = EnemyInstance {
            name: "路边妖".into(),
            hp: 100,
            max_hp: 100,
            atk: 50,
            def: 4,
            exp: 0,
        };
        let mut random_message = String::new();
        apply_run_route_guidance_to_boss(
            &run,
            EncounterKind::Random,
            &mut stats,
            &mut random_enemy,
            &mut random_message,
        );
        assert_eq!(random_enemy.max_hp, 100);
        assert_eq!(random_enemy.atk, 50);
        assert!(random_message.is_empty());
    }

    fn complete_battle_side_quest(quest: &mut QuestLog, side: SideQuest) {
        quest.interact_side_quest(side);
        for _ in 0..quest.side_quest_goal(side) {
            quest.record_side_victory();
        }
        quest.interact_side_quest(side);
    }

    fn quest_at_river_boss() -> QuestLog {
        let mut quest = QuestLog::default();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
        quest.talk(QuestRole::StarMage);
        quest.record_victory();
        quest.record_victory();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Merchant);
        quest.talk(QuestRole::BambooScout);
        quest.talk(QuestRole::CavePriestess);
        quest.record_victory();
        quest.record_victory();
        quest.record_victory();
        quest.activate_moon_crystal(MoonCrystal::North);
        quest.activate_moon_crystal(MoonCrystal::South);
        quest.record_boss_victory(BossKind::MoonWraith);
        quest.talk(QuestRole::Linger);
        quest.talk(QuestRole::HerbHealer);
        quest.record_victory();
        quest.record_victory();
        quest.talk(QuestRole::HerbHealer);
        quest.talk(QuestRole::RiverBoatman);
        quest.activate_river_lantern(RiverLantern::Upstream);
        quest.activate_river_lantern(RiverLantern::Midstream);
        quest.activate_river_lantern(RiverLantern::Dock);
        quest
    }

    #[test]
    fn legacy_boss_opening_reflects_chapter_preparation() {
        let mut prepared = quest_at_river_boss();
        prepared.interact_bond_scene();
        prepared.interact_camp_scene();
        complete_battle_side_quest(&mut prepared, SideQuest::RiverLanterns);
        complete_battle_side_quest(&mut prepared, SideQuest::RiverCargo);
        let mut stats = PlayerStats::default();
        stats.hp = stats.max_hp - 20;
        stats.mp = stats.max_mp - 8;
        let mut enemy = EnemyInstance {
            name: "河魇蛟".into(),
            hp: 184,
            max_hp: 184,
            atk: 26,
            def: 8,
            exp: 76,
        };
        let mut message = "一只 河魇蛟 拦住了去路！".to_string();

        let prep = apply_legacy_boss_preparation_to_boss(
            BossKind::RiverDemon,
            &prepared,
            &mut stats,
            &mut enemy,
            &mut message,
        );

        assert_eq!(prep.state, LegacyBossPreparationState::Prepared);
        assert_eq!((prep.completed, prep.total), (2, 2));
        assert!(enemy.hp < enemy.max_hp);
        assert_eq!(enemy.atk, 24);
        assert!(stats.hp > stats.max_hp - 20);
        assert!(stats.mp > stats.max_mp - 8);
        assert!(message.contains("首领照应·周全"));
        assert!(message.contains("江岸药庐照应 2/2"));

        let unprepared = quest_at_river_boss();
        let mut stats = PlayerStats::default();
        stats.hp = stats.max_hp - 20;
        let mut enemy = EnemyInstance {
            name: "河魇蛟".into(),
            hp: 184,
            max_hp: 184,
            atk: 26,
            def: 8,
            exp: 76,
        };
        let mut message = String::new();

        let prep = apply_legacy_boss_preparation_to_boss(
            BossKind::RiverDemon,
            &unprepared,
            &mut stats,
            &mut enemy,
            &mut message,
        );

        assert_eq!(prep.state, LegacyBossPreparationState::Strained);
        assert_eq!((prep.completed, prep.total), (0, 2));
        assert_eq!(enemy.hp, enemy.max_hp);
        assert_eq!(enemy.atk, 28);
        assert!(stats.hp < stats.max_hp - 20);
        assert!(message.contains("首领照应·欠备"));
        assert!(message.contains("队伍受 6 点开局压制"));
    }

    fn quest_with_spirit_witch() -> QuestLog {
        let mut quest = QuestLog::default();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
        quest.interact_bond_scene();
        quest.talk(QuestRole::StarMage);
        quest.record_victory();
        quest.record_victory();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Merchant);
        quest.talk(QuestRole::BambooScout);
        quest.talk(QuestRole::CavePriestess);
        quest.record_victory();
        quest.record_victory();
        quest.record_victory();
        quest.activate_moon_crystal(MoonCrystal::North);
        quest.activate_moon_crystal(MoonCrystal::South);
        quest.record_boss_victory(BossKind::MoonWraith);
        quest.talk(QuestRole::Linger);
        quest.talk(QuestRole::HerbHealer);
        quest.record_victory();
        quest.record_victory();
        quest.talk(QuestRole::HerbHealer);
        quest.talk(QuestRole::RiverBoatman);
        quest.activate_river_lantern(RiverLantern::Upstream);
        quest.activate_river_lantern(RiverLantern::Midstream);
        quest.activate_river_lantern(RiverLantern::Dock);
        quest.record_boss_victory(BossKind::RiverDemon);
        quest.talk(QuestRole::PlagueElder);
        quest.talk(QuestRole::ShrineKeeper);
        quest.record_victory();
        quest.record_victory();
        quest.record_victory();
        quest.seal_plague_ward(PlagueWard::OldShrine);
        quest.seal_plague_ward(PlagueWard::BitterWell);
        quest.seal_plague_ward(PlagueWard::Sickroom);
        quest.talk(QuestRole::ShrineKeeper);
        quest.record_boss_victory(BossKind::MiasmaRoot);
        quest.talk(QuestRole::CapitalEnvoy);
        quest.talk(QuestRole::MansionSpy);
        quest.record_victory();
        quest.record_victory();
        quest.align_mansion_mirror(MansionMirrorNode::Ledger);
        quest.align_mansion_mirror(MansionMirrorNode::Witness);
        quest.talk(QuestRole::MansionSpy);
        quest.record_boss_victory(BossKind::MirrorMinister);
        quest.talk(QuestRole::SpiritGuide);
        quest.talk(QuestRole::TribalChief);
        quest
    }

    #[test]
    fn boss_encounter_uses_fixed_boss_def() {
        let mut rng = Rng::default();
        let enemy = choose_enemy(
            EncounterZone::Bamboo,
            EncounterKind::Boss(BossKind::MountainFiend),
            &mut rng,
        );

        assert_eq!(enemy.name, "赤鬼山妖");
        assert_eq!(enemy.max_hp, 112);

        let enemy = choose_enemy(
            EncounterZone::Village,
            EncounterKind::Boss(BossKind::MoonWraith),
            &mut rng,
        );

        assert_eq!(enemy.name, "月魄妖");
        assert_eq!(enemy.max_hp, 128);

        let enemy = choose_enemy(
            EncounterZone::RiverTown,
            EncounterKind::Boss(BossKind::RiverDemon),
            &mut rng,
        );

        assert_eq!(enemy.name, "河魇蛟");
        assert_eq!(enemy.max_hp, 156);

        let enemy = choose_enemy(
            EncounterZone::PlagueVillage,
            EncounterKind::Boss(BossKind::MiasmaRoot),
            &mut rng,
        );

        assert_eq!(enemy.name, "瘴母根");
        assert_eq!(enemy.max_hp, 184);

        let enemy = choose_enemy(
            EncounterZone::Capital,
            EncounterKind::Boss(BossKind::MirrorMinister),
            &mut rng,
        );

        assert_eq!(enemy.name, "照影国师");
        assert_eq!(enemy.max_hp, 216);

        let enemy = choose_enemy(
            EncounterZone::SouthernRoad,
            EncounterKind::Boss(BossKind::ThunderQilin),
            &mut rng,
        );

        assert_eq!(enemy.name, "雷麟");
        assert_eq!(enemy.max_hp, 252);

        let enemy = choose_enemy(
            EncounterZone::FinalSanctum,
            EncounterKind::Boss(BossKind::DreamEclipse),
            &mut rng,
        );

        assert_eq!(enemy.name, "宿命水影");
        assert_eq!(enemy.max_hp, 288);
    }

    #[test]
    fn chapter_bosses_use_distinct_generated_cutouts() {
        let bosses = [
            &MOUNTAIN_FIEND_BOSS,
            &MOON_WRAITH_BOSS,
            &RIVER_DEMON_BOSS,
            &MIASMA_ROOT_BOSS,
            &MIRROR_MINISTER_BOSS,
            &THUNDER_QILIN_BOSS,
            &DREAM_ECLIPSE_BOSS,
        ];

        for (index, boss) in bosses.iter().enumerate() {
            assert!(boss.image.starts_with("creatures/boss_"));
            assert_eq!(
                super::super::cutout::cutout_source_px_for_path(boss.image),
                512.0
            );
            for previous in &bosses[..index] {
                assert_ne!(boss.image, previous.image);
            }
        }
    }

    #[test]
    fn generated_enemy_cutouts_are_primary_battle_visuals() {
        let enemy = &ENEMIES[0];
        assert_eq!(enemy_primary_image(enemy), "creatures/ai_water_serpent.png");
        assert_eq!(enemy_primary_size(enemy), enemy.size);
        let primary_alpha = enemy_primary_color(Phase::Menu, 0.0).to_srgba().alpha;
        assert!(primary_alpha > 0.9);
    }

    #[test]
    fn spell_impacts_have_multihit_visual_layers() {
        let basic = spell_impact_profile(false);
        let late = spell_impact_profile(true);
        let basic_shards = spell_impact_shards(false);
        let late_shards = spell_impact_shards(true);

        assert!(basic.core_size >= 280.0);
        assert_eq!(basic_shards.len(), 3);
        assert!(late.core_size > basic.core_size);
        assert!(late.flash_radius > basic.flash_radius);
        assert!(late_shards.len() > basic_shards.len());
        assert!(late_shards.iter().any(|shard| shard.offset.y > 100.0));
        assert!(late_shards.iter().any(|shard| shard.offset.y < -30.0));
        assert!(late_shards.iter().any(|shard| shard.offset.x.abs() > 110.0));

        let basic_numbers = spell_damage_texts(31, false);
        let late_numbers = spell_damage_texts(83, true);
        assert_eq!(basic_numbers.len(), 3);
        assert_eq!(late_numbers.len(), 7);
        assert_eq!(
            basic_numbers.iter().map(|text| text.amount).sum::<i32>(),
            31
        );
        assert_eq!(late_numbers.iter().map(|text| text.amount).sum::<i32>(), 83);
        assert!(basic_numbers.iter().all(|text| text.amount > 0));
        assert!(late_numbers.iter().all(|text| text.amount > 0));
        assert!(
            late_numbers
                .iter()
                .map(|text| text.offset.x)
                .fold(0.0_f32, |spread, x| spread.max(x))
                > basic_numbers
                    .iter()
                    .map(|text| text.offset.x)
                    .fold(0.0_f32, |spread, x| spread.max(x))
        );
        assert_eq!(spell_damage_slices(2, false), vec![1, 1]);
    }

    #[test]
    fn spell_impact_theme_tracks_story_chapter() {
        assert_eq!(
            spell_impact_element(Chapter::MoonCave),
            SpellImpactElement::Moon
        );
        assert_eq!(
            spell_impact_element(Chapter::SouthernThunder),
            SpellImpactElement::Thunder
        );
        assert_eq!(
            spell_impact_flavor(SpellImpactElement::Moon, false),
            "月水映剑"
        );
        assert_eq!(
            spell_impact_flavor(SpellImpactElement::Thunder, true),
            "万剑引雷"
        );

        let river = spell_impact_color(SpellImpactElement::River, false, false, 0.5).to_srgba();
        let plague = spell_impact_color(SpellImpactElement::Plague, false, false, 0.5).to_srgba();
        assert!(river.blue > plague.blue);
        assert!(plague.green >= river.green);

        let moon_shards = themed_spell_impact_shards(false, SpellImpactElement::Moon);
        let thunder_shards = themed_spell_impact_shards(true, SpellImpactElement::Thunder);
        assert_eq!(moon_shards.len(), spell_impact_shards(false).len());
        assert_eq!(thunder_shards.len(), spell_impact_shards(true).len());
        assert_ne!(
            moon_shards[0].color.to_srgba().blue,
            thunder_shards[0].color.to_srgba().blue
        );

        let dream_number = spell_damage_text_color(SpellImpactElement::Dream, true).to_srgba();
        assert!(dream_number.alpha > 0.9);
        assert!(dream_number.blue >= dream_number.red);
    }

    #[test]
    fn spell_action_hits_enemy_harder_than_basic_attack_motion() {
        let attack = enemy_phase_motion(Phase::PlayerActing, Some(PlayerAction::Attack), 0.4);
        let spell = enemy_phase_motion(Phase::PlayerActing, Some(PlayerAction::Spell), 0.4);

        assert!(spell.0 > attack.0);
        assert!(spell.1 > attack.1);
        assert!(spell.2 < attack.2);
        assert!(spell.3 >= attack.3);
    }

    #[test]
    fn battle_environments_are_distinct_and_shipped_for_every_encounter_zone() {
        let zones = [
            EncounterZone::Village,
            EncounterZone::Bamboo,
            EncounterZone::Cave,
            EncounterZone::RiverTown,
            EncounterZone::PlagueVillage,
            EncounterZone::Capital,
            EncounterZone::SouthernRoad,
            EncounterZone::FinalSanctum,
        ];
        let paths: std::collections::HashSet<_> =
            zones.iter().map(|z| battle_background_path(*z)).collect();
        assert_eq!(paths.len(), zones.len());
        for path in paths {
            assert!(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("assets")
                    .join(path)
                    .exists(),
                "missing battlefield {path}"
            );
        }
    }

    #[test]
    fn shrine_blessings_adjust_next_battle_numbers() {
        assert_eq!(
            blessing_damage_bonus(Some(ShrineBlessing::Sword), PlayerAction::Attack),
            4
        );
        assert_eq!(
            blessing_damage_bonus(Some(ShrineBlessing::Spirit), PlayerAction::Spell),
            6
        );
        assert_eq!(
            blessing_damage_bonus(Some(ShrineBlessing::Guard), PlayerAction::Attack),
            0
        );
        assert_eq!(blessing_guard_block(Some(ShrineBlessing::Guard), 9), 5);
        assert_eq!(blessing_guard_block(Some(ShrineBlessing::Guard), 3), 2);
        assert_eq!(
            battle_blessing_summary(Some(ShrineBlessing::Spirit)),
            "祝福 灵息"
        );
    }

    #[test]
    fn camp_bonuses_adjust_next_battle_numbers() {
        assert_eq!(
            camp_damage_bonus(Some(CampBonus::Warmth), PlayerAction::Attack),
            1
        );
        assert_eq!(
            camp_damage_bonus(Some(CampBonus::Focus), PlayerAction::Combo),
            3
        );
        assert_eq!(
            camp_damage_bonus(Some(CampBonus::Vigil), PlayerAction::Spell),
            0
        );
        assert_eq!(camp_guard_block(Some(CampBonus::Vigil), 9), 3);
        assert_eq!(camp_guard_block(Some(CampBonus::Warmth), 1), 0);
        assert_eq!(
            battle_camp_summary(Some(CampBonus::Focus), None),
            "营地 静心"
        );
    }

    #[test]
    fn run_camp_tactics_adjust_next_battle_numbers() {
        assert_eq!(
            run_camp_damage_bonus(Some(RunCampTactic::SwordGuard), PlayerAction::Attack),
            3
        );
        assert_eq!(
            run_camp_damage_bonus(Some(RunCampTactic::SpiritFocus), PlayerAction::Spell),
            3
        );
        assert_eq!(
            run_camp_damage_bonus(Some(RunCampTactic::LingerWard), PlayerAction::Spell),
            0
        );
        assert_eq!(run_camp_guard_block(Some(RunCampTactic::LingerWard), 9), 3);
        assert_eq!(run_camp_guard_block(Some(RunCampTactic::Breath), 1), 0);
        assert_eq!(run_camp_start_heal_amount(RunCampTactic::LingerWard), 10);
        assert_eq!(run_camp_start_mana_amount(RunCampTactic::SpiritFocus), 8);
        assert_eq!(run_camp_guard_mp_bonus(Some(RunCampTactic::SpiritFocus)), 2);
        assert_eq!(
            battle_camp_summary(None, Some(RunCampTactic::SpiritFocus)),
            "营策 凝灵"
        );
    }

    #[test]
    fn linger_support_heals_after_joining_party() {
        let mut quest = QuestLog::default();
        let mut stats = PlayerStats::default();
        stats.hp = stats.max_hp - 10;

        assert_eq!(companion_support(&quest, None, None, &mut stats), None);
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);

        assert_eq!(companion_support(&quest, None, None, &mut stats), Some(5));
        assert_eq!(stats.hp, stats.max_hp - 5);
    }

    #[test]
    fn linger_support_scales_with_bond_level() {
        let mut quest = QuestLog::default();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
        quest.interact_bond_scene();

        let mut stats = PlayerStats::default();
        stats.hp = stats.max_hp - 20;

        assert_eq!(quest.bond_level(), 1);
        assert_eq!(companion_support(&quest, None, None, &mut stats), Some(7));
        assert_eq!(stats.hp, stats.max_hp - 13);
    }

    #[test]
    fn tender_bond_response_improves_linger_support() {
        let mut quest = QuestLog::default();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
        quest.interact_bond_scene();
        quest.record_bond_response(BondResponse::Tender);

        let mut stats = PlayerStats::default();
        stats.hp = stats.max_hp - 20;

        assert_eq!(quest.bond_tender_level(), 1);
        assert_eq!(
            companion_support(&quest, None, Some(BondBonus::Tender), &mut stats),
            Some(10)
        );
        assert_eq!(stats.hp, stats.max_hp - 10);
    }

    #[test]
    fn spirit_witch_support_restores_mp_after_southern_join() {
        let quest = quest_with_spirit_witch();
        let mut stats = PlayerStats::default();
        stats.mp = stats.max_mp - 6;

        assert_eq!(
            spirit_witch_support(&QuestLog::default(), None, &mut stats),
            None
        );
        assert_eq!(spirit_witch_support(&quest, None, &mut stats), Some(2));
        assert_eq!(stats.mp, stats.max_mp - 4);
        assert!(party_support_message(None, Some(2)).contains("南瑶 叩响袖中铜铃，回稳 2 点灵力"));
    }

    #[test]
    fn bond_bonuses_adjust_next_battle_numbers() {
        assert_eq!(
            bond_damage_bonus(Some(BondBonus::Courage), PlayerAction::Attack),
            2
        );
        assert_eq!(
            bond_damage_bonus(Some(BondBonus::Courage), PlayerAction::Combo),
            4
        );
        assert_eq!(
            bond_damage_bonus(Some(BondBonus::Tender), PlayerAction::Spell),
            2
        );
        assert_eq!(bond_guard_block(Some(BondBonus::Tender), 9), 2);
        assert_eq!(bond_guard_block(Some(BondBonus::Courage), 9), 1);
        assert_eq!(battle_bond_summary(Some(BondBonus::Tender)), "护念 柔心");
    }

    #[test]
    fn battle_gold_reward_scales_for_bosses() {
        assert_eq!(battle_gold_reward(12, false), 12);
        assert_eq!(battle_gold_reward(60, true), 70);
    }

    #[test]
    fn legacy_boss_breakthrough_grants_chapter_growth() {
        let mut stats = PlayerStats::default();
        stats.hp = 40;
        stats.mp = 6;
        let hp = stats.max_hp;
        let mp = stats.max_mp;
        let atk = stats.atk;
        let def = stats.def;

        let line = apply_legacy_boss_breakthrough(BossKind::RiverDemon, &mut stats)
            .expect("river boss should grant a breakthrough");

        assert!(line.contains("章末突破"));
        assert!(line.contains("苏州河灯"));
        assert_eq!(stats.max_hp, hp + 20);
        assert_eq!(stats.hp, 60);
        assert_eq!(stats.max_mp, mp + 8);
        assert_eq!(stats.mp, 14);
        assert_eq!(stats.atk, atk + 5);
        assert_eq!(stats.def, def + 2);

        let hp = stats.max_hp;
        let mp = stats.max_mp;
        let atk = stats.atk;
        let def = stats.def;
        let final_line = apply_legacy_boss_breakthrough(BossKind::DreamEclipse, &mut stats)
            .expect("final boss should still report chapter closure");

        assert!(final_line.contains("心渊照影"));
        assert_eq!(stats.max_hp, hp);
        assert_eq!(stats.max_mp, mp);
        assert_eq!(stats.atk, atk);
        assert_eq!(stats.def, def);
    }

    #[test]
    fn combo_unlock_requires_two_companions_and_bond() {
        let mut quest = QuestLog::default();
        assert!(!combo_unlocked(&quest));

        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
        quest.interact_bond_scene();
        assert!(!combo_unlocked(&quest));

        quest.talk(QuestRole::StarMage);
        quest.record_victory();
        quest.record_victory();
        quest.talk(QuestRole::SwordSister);
        assert!(combo_unlocked(&quest));
    }

    #[test]
    fn combo_damage_uses_bond_and_party_stats() {
        let mut quest = QuestLog::default();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
        quest.interact_bond_scene();
        quest.talk(QuestRole::StarMage);
        quest.record_victory();
        quest.record_victory();
        quest.talk(QuestRole::SwordSister);

        let stats = PlayerStats::default();
        let enemy = EnemyInstance {
            name: "试炼妖".into(),
            hp: 80,
            max_hp: 80,
            atk: 1,
            def: 4,
            exp: 0,
        };
        let mut rng = Rng::default();
        let dmg = combo_damage(&quest, &stats, &enemy, &mut rng);

        assert!(dmg >= stats.atk * 3 + quest.bond_level() as i32 * 5 - enemy.def + 4);
    }

    #[test]
    fn spirit_witch_extends_late_party_combo() {
        let mut core_party = QuestLog::default();
        core_party.talk(QuestRole::SwordSister);
        core_party.talk(QuestRole::Linger);
        core_party.interact_bond_scene();
        core_party.talk(QuestRole::StarMage);
        core_party.record_victory();
        core_party.record_victory();
        core_party.talk(QuestRole::SwordSister);
        let full_party = quest_with_spirit_witch();

        let stats = PlayerStats::default();
        let enemy = EnemyInstance {
            name: "试炼妖".into(),
            hp: 80,
            max_hp: 80,
            atk: 1,
            def: 4,
            exp: 0,
        };
        let mut rng_a = Rng::default();
        let mut rng_b = Rng::default();

        let core = combo_damage(&core_party, &stats, &enemy, &mut rng_a);
        let full = combo_damage(&full_party, &stats, &enemy, &mut rng_b);

        assert_eq!(spirit_witch_combo_bonus(&full_party), 6);
        assert_eq!(full, core + 6);
        assert!(combo_party_message(&full_party, full).contains("南瑶"));
        assert!(!combo_party_message(&core_party, core).contains("南瑶"));
    }

    #[test]
    fn courage_bond_response_improves_combo_damage() {
        let mut base_quest = QuestLog::default();
        base_quest.talk(QuestRole::SwordSister);
        base_quest.talk(QuestRole::Linger);
        base_quest.interact_bond_scene();
        base_quest.talk(QuestRole::StarMage);
        base_quest.record_victory();
        base_quest.record_victory();
        base_quest.talk(QuestRole::SwordSister);

        let mut courage_quest = QuestLog::default();
        courage_quest.talk(QuestRole::SwordSister);
        courage_quest.talk(QuestRole::Linger);
        courage_quest.interact_bond_scene();
        courage_quest.record_bond_response(BondResponse::Courage);
        courage_quest.talk(QuestRole::StarMage);
        courage_quest.record_victory();
        courage_quest.record_victory();
        courage_quest.talk(QuestRole::SwordSister);

        let stats = PlayerStats::default();
        let enemy = EnemyInstance {
            name: "试炼妖".into(),
            hp: 80,
            max_hp: 80,
            atk: 1,
            def: 4,
            exp: 0,
        };
        let mut rng_a = Rng::default();
        let mut rng_b = Rng::default();

        let base = combo_damage(&base_quest, &stats, &enemy, &mut rng_a);
        let courage = combo_damage(&courage_quest, &stats, &enemy, &mut rng_b);

        assert_eq!(courage_quest.bond_courage_level(), 1);
        assert!(courage >= base + 3);
    }

    #[test]
    fn enemy_turn_reports_damage_for_visual_feedback() {
        let mut state = BattleState {
            enemy: EnemyInstance {
                name: "碧水妖蛇".to_string(),
                hp: 30,
                max_hp: 60,
                atk: 12,
                def: 3,
                exp: 12,
            },
            encounter_kind: EncounterKind::Random,
            phase: Phase::PlayerActing,
            timer: 0.0,
            enemy_turns: 0,
            tactics: Tactics::new(EnemyStyle::Assault, false),
            executing_intent: EnemyIntent::Strike,
            enemy_attacked: false,
            phase_duration: 0.8,
            menu_index: 0,
            player_action: Some(PlayerAction::Attack),
            intent: EnemyIntent::Strike,
            guarding: false,
            boss_phase2: false,
            boss_heal_remaining: 25,
            eclipse_heart: false,
            hex_first_hit_used: false,
            skill_menu: false,
            skill_cursor: 0,
            player_shield: 0,
            statuses: EnemyStatuses::default(),
            momentum: 0,
            message: String::new(),
            blessing: None,
            camp_bonus: None,
            run_camp_tactic: None,
            bond_bonus: None,
        };
        let mut stats = PlayerStats::default();
        let before = stats.hp;
        let mut rng = Rng::default();

        let turn = begin_enemy_turn(&mut state, &mut stats, &mut rng, 0, 0, 35, 0);
        let attack = turn.attack.expect("normal strike should attack");

        assert!(attack.damage > 0);
        assert!(!attack.strong);
        assert_eq!(stats.hp, before - attack.damage);
        assert!(matches!(state.phase, Phase::EnemyActing));
        assert!(state.message.contains("造成"));
    }

    #[test]
    fn guard_blessing_reduces_enemy_turn_damage() {
        let mut state = BattleState {
            enemy: EnemyInstance {
                name: "碧水妖蛇".to_string(),
                hp: 30,
                max_hp: 60,
                atk: 12,
                def: 3,
                exp: 12,
            },
            encounter_kind: EncounterKind::Random,
            phase: Phase::PlayerActing,
            timer: 0.0,
            enemy_turns: 0,
            tactics: Tactics::new(EnemyStyle::Assault, false),
            executing_intent: EnemyIntent::Strike,
            enemy_attacked: false,
            phase_duration: 0.8,
            menu_index: 0,
            player_action: Some(PlayerAction::Attack),
            intent: EnemyIntent::Strike,
            guarding: false,
            boss_phase2: false,
            boss_heal_remaining: 25,
            eclipse_heart: false,
            hex_first_hit_used: false,
            skill_menu: false,
            skill_cursor: 0,
            player_shield: 0,
            statuses: EnemyStatuses::default(),
            momentum: 0,
            message: String::new(),
            blessing: Some(ShrineBlessing::Guard),
            camp_bonus: None,
            run_camp_tactic: None,
            bond_bonus: None,
        };
        let mut stats = PlayerStats::default();
        let before = stats.hp;
        let mut rng = Rng::default();

        let turn = begin_enemy_turn(&mut state, &mut stats, &mut rng, 0, 0, 35, 0);
        let attack = turn.attack.expect("normal strike should attack");

        assert!(attack.damage > 0);
        assert!(attack.damage < 7);
        assert_eq!(stats.hp, before - attack.damage);
        assert!(state.message.contains("护身香火"));
    }

    #[test]
    fn advertised_boss_signature_uses_its_signature_move() {
        let mut state = BattleState {
            enemy: EnemyInstance {
                name: "月魄妖".to_string(),
                hp: 128,
                max_hp: 128,
                atk: 26,
                def: 7,
                exp: 60,
            },
            encounter_kind: EncounterKind::Boss(BossKind::MoonWraith),
            phase: Phase::PlayerActing,
            timer: 0.0,
            enemy_turns: 0,
            tactics: Tactics::new(EnemyStyle::Assault, false),
            executing_intent: EnemyIntent::Special,
            enemy_attacked: false,
            phase_duration: 0.8,
            menu_index: 0,
            player_action: Some(PlayerAction::Attack),
            intent: EnemyIntent::Special,
            guarding: false,
            boss_phase2: false,
            boss_heal_remaining: 25,
            eclipse_heart: false,
            hex_first_hit_used: false,
            skill_menu: false,
            skill_cursor: 0,
            player_shield: 0,
            statuses: EnemyStatuses::default(),
            momentum: 0,
            message: String::new(),
            blessing: None,
            camp_bonus: None,
            run_camp_tactic: None,
            bond_bonus: None,
        };
        let mut stats = PlayerStats::default();
        stats.mp = 10;
        let before_hp = stats.hp;
        let mut rng = Rng::default();

        let turn = begin_enemy_turn(&mut state, &mut stats, &mut rng, 0, 0, 35, 0);
        let attack = turn.attack.expect("boss special should attack");

        assert!(attack.strong);
        assert!(state.message.contains("月影噬灵"));
        assert!(state.message.contains("失去 3 点灵力"));
        assert_eq!(stats.mp, 7);
        assert_eq!(stats.hp, before_hp - attack.damage);
        assert_eq!(state.enemy_turns, 1);
    }

    #[test]
    fn dream_eclipse_special_heals_boss() {
        let mut state = BattleState {
            enemy: EnemyInstance {
                name: "宿命水影".to_string(),
                hp: 120,
                max_hp: 288,
                atk: 43,
                def: 15,
                exp: 190,
            },
            encounter_kind: EncounterKind::Boss(BossKind::DreamEclipse),
            phase: Phase::PlayerActing,
            timer: 0.0,
            enemy_turns: 0,
            tactics: Tactics::new(EnemyStyle::Assault, false),
            executing_intent: EnemyIntent::Special,
            enemy_attacked: false,
            phase_duration: 0.8,
            menu_index: 0,
            player_action: Some(PlayerAction::Attack),
            intent: EnemyIntent::Special,
            guarding: false,
            boss_phase2: false,
            boss_heal_remaining: 25,
            eclipse_heart: false,
            hex_first_hit_used: false,
            skill_menu: false,
            skill_cursor: 0,
            player_shield: 0,
            statuses: EnemyStatuses::default(),
            momentum: 0,
            message: String::new(),
            blessing: None,
            camp_bonus: None,
            run_camp_tactic: None,
            bond_bonus: None,
        };
        let mut stats = PlayerStats::default();
        let mut rng = Rng::default();

        let turn = begin_enemy_turn(&mut state, &mut stats, &mut rng, 0, 0, 35, 0);
        let attack = turn.attack.expect("boss special should attack");

        assert!(attack.strong);
        assert!(state.message.contains("旧梦潮声"));
        assert!(state.message.contains("回稳 10 点气血"));
        assert_eq!(state.enemy.hp, 130);
    }

    #[test]
    fn sword_sister_motion_lunges_for_followup_window() {
        let (offset, scale, color) = companion_motion(
            BattleCompanionKind::SwordSister,
            Phase::PlayerActing,
            Some(PlayerAction::Attack),
            0.4,
            0.0,
        );

        assert!(offset.x > 30.0);
        assert!(offset.y > 13.0);
        assert!(scale > 1.04);
        assert!(color.to_srgba().blue < 0.90);
    }

    #[test]
    fn linger_motion_joins_combo_cast() {
        let (offset, scale, color) = companion_motion(
            BattleCompanionKind::Linger,
            Phase::PlayerActing,
            Some(PlayerAction::Combo),
            0.4,
            0.0,
        );

        assert!(offset.x > 17.0);
        assert!(offset.y > 20.0);
        assert!(scale > 1.04);
        assert!(color.to_srgba().red < 0.92);
    }

    #[test]
    fn spirit_witch_motion_joins_spell_and_combo_casts() {
        let (offset, scale, color) = companion_motion(
            BattleCompanionKind::SpiritWitch,
            Phase::PlayerActing,
            Some(PlayerAction::Combo),
            0.4,
            0.0,
        );

        assert!(offset.x > 11.0);
        assert!(offset.y > 27.0);
        assert!(scale > 1.04);
        assert!(color.to_srgba().red < 0.86);
    }

    #[test]
    fn sword_sister_followup_requires_joining_party() {
        let mut quest = QuestLog::default();
        let stats = PlayerStats::default();
        let mut rng = Rng::default();
        let mut enemy = EnemyInstance {
            name: "试炼妖".into(),
            hp: 30,
            max_hp: 30,
            atk: 1,
            def: 4,
            exp: 0,
        };

        assert_eq!(
            sword_sister_followup(&quest, None, &stats, &mut enemy, &mut rng),
            None
        );
        assert_eq!(enemy.hp, 30);

        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
        quest.talk(QuestRole::StarMage);
        quest.record_victory();
        quest.record_victory();
        quest.talk(QuestRole::SwordSister);

        let before = enemy.hp;
        let dmg = sword_sister_followup(&quest, None, &stats, &mut enemy, &mut rng)
            .expect("joined sword sister should follow up");
        assert!(dmg >= 2);
        assert_eq!(enemy.hp, before - dmg);
    }

    #[test]
    fn sword_sister_followup_scales_with_bond_level() {
        let stats = PlayerStats::default();
        let mut base_quest = QuestLog::default();
        base_quest.talk(QuestRole::SwordSister);
        base_quest.talk(QuestRole::Linger);
        base_quest.talk(QuestRole::StarMage);
        base_quest.record_victory();
        base_quest.record_victory();
        base_quest.talk(QuestRole::SwordSister);

        let mut bonded_quest = QuestLog::default();
        bonded_quest.talk(QuestRole::SwordSister);
        bonded_quest.talk(QuestRole::Linger);
        bonded_quest.interact_bond_scene();
        bonded_quest.talk(QuestRole::StarMage);
        bonded_quest.record_victory();
        bonded_quest.record_victory();
        bonded_quest.talk(QuestRole::SwordSister);

        let mut rng_a = Rng::default();
        let mut rng_b = Rng::default();
        let mut enemy_a = EnemyInstance {
            name: "试炼妖".into(),
            hp: 40,
            max_hp: 40,
            atk: 1,
            def: 4,
            exp: 0,
        };
        let mut enemy_b = EnemyInstance {
            name: "试炼妖".into(),
            hp: 40,
            max_hp: 40,
            atk: 1,
            def: 4,
            exp: 0,
        };

        let base =
            sword_sister_followup(&base_quest, None, &stats, &mut enemy_a, &mut rng_a).unwrap();
        let bonded =
            sword_sister_followup(&bonded_quest, None, &stats, &mut enemy_b, &mut rng_b).unwrap();
        assert_eq!(bonded, base + 1);
    }
}

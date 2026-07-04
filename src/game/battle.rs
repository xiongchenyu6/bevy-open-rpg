use bevy::prelude::*;

use super::animation::{self, AnimationAssets, AnimationClip, SpriteAnimation};
use super::core::{GameFont, Intent, PlayerStats, Rng};
use super::lighting::{self, LightingAssets};
use super::paperdoll::{self, PaperdollAssets, PaperdollStyle};
use super::quest::{BondBonus, BossKind, CampBonus, Companion, QuestLog, ShrineBlessing};
use super::roguelike::{FightRank, RunBattleMods, RunOutcome, RunState};
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
    base: Color,
    horizon: Color,
    far_texture: &'static str,
    far_tint: Color,
    floor_texture: &'static str,
    floor_tint: Color,
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

const MOON_WRAITH_BOSS: EnemyDef = EnemyDef {
    name: "月魄妖",
    max_hp: 128,
    atk: 26,
    def: 7,
    image: "creatures/ai_lotus_spirit.png",
    size: 258.0,
    light: [0.95, 0.55, 1.0, 0.42],
    exp: 60,
};

const RIVER_DEMON_BOSS: EnemyDef = EnemyDef {
    name: "河魇蛟",
    max_hp: 156,
    atk: 29,
    def: 8,
    image: "creatures/ai_water_serpent.png",
    size: 272.0,
    light: [0.30, 0.82, 1.0, 0.46],
    exp: 82,
};

const MIASMA_ROOT_BOSS: EnemyDef = EnemyDef {
    name: "瘴母根",
    max_hp: 184,
    atk: 32,
    def: 10,
    image: "creatures/ai_moss_turtle.png",
    size: 286.0,
    light: [0.50, 0.90, 0.42, 0.44],
    exp: 104,
};

const MIRROR_MINISTER_BOSS: EnemyDef = EnemyDef {
    name: "照影国师",
    max_hp: 216,
    atk: 35,
    def: 12,
    image: "creatures/ai_shadow_swordsman.png",
    size: 292.0,
    light: [0.72, 0.62, 1.0, 0.48],
    exp: 132,
};

const THUNDER_QILIN_BOSS: EnemyDef = EnemyDef {
    name: "雷麟",
    max_hp: 252,
    atk: 39,
    def: 13,
    image: "creatures/qilin.png",
    size: 260.0,
    light: [0.52, 0.88, 1.0, 0.52],
    exp: 160,
};

const DREAM_ECLIPSE_BOSS: EnemyDef = EnemyDef {
    name: "宿命水影",
    max_hp: 288,
    atk: 43,
    def: 15,
    image: "creatures/frost_dragon.png",
    size: 286.0,
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EnemyIntent {
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
            EnemyIntent::Strike => "意图:张爪欲击",
            EnemyIntent::Heavy => "意图:妖气翻涌,蓄力重击!",
            EnemyIntent::Gather => "意图:凝气回息(防备上升)",
            EnemyIntent::Drain => "意图:虚影缠绕,欲摄灵力",
        }
    }
}

fn roll_intent(rng: &mut Rng) -> EnemyIntent {
    match rng.range(0, 100) {
        n if n < 45 => EnemyIntent::Strike,
        n if n < 70 => EnemyIntent::Heavy,
        n if n < 85 => EnemyIntent::Gather,
        _ => EnemyIntent::Drain,
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
    menu_index: usize,
    player_action: Option<PlayerAction>,
    intent: EnemyIntent,
    guarding: bool,
    momentum: u32,
    message: String,
    blessing: Option<ShrineBlessing>,
    camp_bonus: Option<CampBonus>,
    bond_bonus: Option<BondBonus>,
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
struct EnemyHpBar;

#[derive(Component)]
struct MessageText;

#[derive(Component)]
struct EnemyInfoText;

#[derive(Component)]
struct PlayerInfoText;

#[derive(Component)]
struct MenuItem(usize);

#[derive(Component)]
struct BattleHero;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BattleCompanionKind {
    Linger,
    SwordSister,
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
struct BattleEnemyAura {
    origin: Vec3,
    light: [f32; 4],
    age: f32,
}

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

const HP_BAR_W: f32 = 220.0;
const HP_BAR_LEFT: f32 = -110.0;
const ENEMY_POS: Vec3 = Vec3::new(0.0, 120.0, 1.0);
const HERO_POS: Vec3 = Vec3::new(-360.0, -70.0, 1.0);
const LINGER_POS: Vec3 = Vec3::new(HERO_POS.x + 130.0, HERO_POS.y + 8.0, 0.9);
const SWORD_SISTER_POS: Vec3 = Vec3::new(HERO_POS.x + 245.0, HERO_POS.y - 6.0, 0.92);

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Battle), spawn_battle)
            .add_systems(
                Update,
                (
                    battle_input,
                    battle_tick,
                    update_battle_animations,
                    update_battle_effects,
                    update_floating_combat_text,
                    update_battle_ui,
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
    font: Res<GameFont>,
    asset_server: Res<AssetServer>,
    lights: Res<LightingAssets>,
    anims: Res<AnimationAssets>,
    dolls: Res<PaperdollAssets>,
    mut quest: ResMut<QuestLog>,
    mut rng: ResMut<Rng>,
    mut stats: ResMut<PlayerStats>,
    encounter: Option<Res<PendingEncounter>>,
    run: Option<Res<RunState>>,
    mods: Option<Res<RunBattleMods>>,
) {
    let zone = encounter
        .as_ref()
        .map(|encounter| encounter.zone)
        .unwrap_or(EncounterZone::Village);
    let kind = encounter
        .as_ref()
        .map(|encounter| encounter.kind)
        .unwrap_or(EncounterKind::Random);
    let def = choose_enemy(zone, kind, &mut rng);
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
    if let Some(mods) = mods.as_ref() {
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

    let blessing = quest.take_shrine_blessing();
    let camp_bonus = quest.take_camp_bonus();
    let bond_bonus = quest.take_bond_bonus();
    let mut message = format!("一只 {} 拦住了去路！", enemy.name);

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

    commands.insert_resource(BattleState {
        message,
        enemy,
        encounter_kind: kind,
        phase: Phase::Menu,
        timer: 0.0,
        enemy_turns: 0,
        menu_index: 0,
        player_action: None,
        intent: roll_intent(&mut rng),
        guarding: false,
        momentum: 0,
        blessing,
        camp_bonus,
        bond_bonus,
    });

    spawn_battle_backdrop(&mut commands, &asset_server, &lights, zone);

    // Enemy visual stack: generated creature art is the readable body, while the
    // shared monster sheet only supplies a translucent action silhouette.
    lighting::spawn_light(
        &mut commands,
        &lights,
        Vec3::new(ENEMY_POS.x, ENEMY_POS.y - 2.0, 0.5),
        360.0,
        Color::srgba(def.light[0], def.light[1], def.light[2], def.light[3]),
        AppState::Battle,
    );
    let mut aura_sprite = anims.sprite(
        AnimationClip::MonsterIdle,
        Vec2::splat(enemy_aura_size(def)),
    );
    aura_sprite.color = enemy_aura_color(def.light, Phase::Menu, 0.0);
    commands.spawn((
        BattleEnemyAura {
            origin: ENEMY_POS,
            light: def.light,
            age: rng.range(0, 100) as f32 * 0.05,
        },
        aura_sprite,
        SpriteAnimation::new(AnimationClip::MonsterIdle),
        Transform::from_xyz(ENEMY_POS.x, ENEMY_POS.y, ENEMY_POS.z + 0.08),
        DespawnOnExit(AppState::Battle),
    ));
    commands.spawn((
        BattleEnemy,
        BattleEnemyMotion {
            origin: ENEMY_POS,
            age: rng.range(0, 100) as f32 * 0.07,
        },
        Sprite {
            image: asset_server.load(enemy_primary_image(def)),
            color: enemy_primary_color(Phase::Menu, 0.0),
            custom_size: Some(Vec2::splat(enemy_primary_size(def))),
            ..default()
        },
        Transform::from_translation(ENEMY_POS),
        DespawnOnExit(AppState::Battle),
    ));

    // Enemy HP bar (back + front)
    commands.spawn((
        Sprite::from_color(Color::srgb(0.25, 0.25, 0.25), Vec2::new(HP_BAR_W, 14.0)),
        Transform::from_xyz(0.0, 275.0, 1.0),
        DespawnOnExit(AppState::Battle),
    ));
    commands.spawn((
        EnemyHpBar,
        Sprite::from_color(Color::srgb(0.85, 0.25, 0.25), Vec2::new(HP_BAR_W, 14.0)),
        Transform::from_xyz(0.0, 275.0, 1.1),
        DespawnOnExit(AppState::Battle),
    ));

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

    if quest.has_companion(Companion::Linger) {
        lighting::spawn_light(
            &mut commands,
            &lights,
            Vec3::new(LINGER_POS.x, LINGER_POS.y, 0.5),
            250.0,
            Color::srgba(0.72, 0.92, 1.0, 0.24),
            AppState::Battle,
        );
        let companion = paperdoll::spawn_paperdoll(
            &mut commands,
            &dolls,
            PaperdollStyle::Linger,
            LINGER_POS,
            paperdoll::BATTLE_SIZE * 0.72,
            AppState::Battle,
        );
        commands.entity(companion).insert(BattleCompanion {
            kind: BattleCompanionKind::Linger,
            origin: LINGER_POS,
            age: 0.0,
        });
    }

    if quest.has_companion(Companion::SwordSister) {
        lighting::spawn_light(
            &mut commands,
            &lights,
            Vec3::new(SWORD_SISTER_POS.x, SWORD_SISTER_POS.y, 0.5),
            250.0,
            Color::srgba(1.0, 0.58, 0.36, 0.24),
            AppState::Battle,
        );
        let companion = paperdoll::spawn_paperdoll(
            &mut commands,
            &dolls,
            PaperdollStyle::Ranger,
            SWORD_SISTER_POS,
            paperdoll::BATTLE_SIZE * 0.70,
            AppState::Battle,
        );
        commands.entity(companion).insert(BattleCompanion {
            kind: BattleCompanionKind::SwordSister,
            origin: SWORD_SISTER_POS,
            age: 0.0,
        });
    }

    // Enemy info text (top center)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(24.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            DespawnOnExit(AppState::Battle),
        ))
        .with_children(|p| {
            p.spawn((
                EnemyInfoText,
                Text::new(""),
                font.text_font(22.0),
                TextColor(Color::srgb(1.0, 0.85, 0.6)),
            ));
        });

    // Bottom command panel
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                height: Val::Px(220.0),
                padding: UiRect::all(Val::Px(20.0)),
                column_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.04, 0.10, 0.95)),
            DespawnOnExit(AppState::Battle),
        ))
        .with_children(|panel| {
            // Left: message + player info
            panel
                .spawn(Node {
                    width: Val::Percent(64.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(14.0),
                    ..default()
                })
                .with_children(|left| {
                    left.spawn((
                        MessageText,
                        Text::new(""),
                        font.text_font(22.0),
                        TextColor(Color::WHITE),
                    ));
                    left.spawn((
                        PlayerInfoText,
                        Text::new(""),
                        font.text_font(20.0),
                        TextColor(Color::srgb(0.7, 0.9, 1.0)),
                    ));
                });

            // Right: command menu
            panel
                .spawn((
                    Node {
                        width: Val::Percent(34.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(10.0),
                        padding: UiRect::all(Val::Px(12.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.10, 0.10, 0.20, 0.9)),
                ))
                .with_children(|menu| {
                    for (i, _) in MENU.iter().enumerate() {
                        menu.spawn((
                            MenuItem(i),
                            Text::new(""),
                            font.text_font(24.0),
                            TextColor(Color::WHITE),
                        ));
                    }
                });
        });
}

fn choose_enemy(zone: EncounterZone, kind: EncounterKind, rng: &mut Rng) -> &'static EnemyDef {
    match kind {
        EncounterKind::Boss(BossKind::MoonWraith) => return &MOON_WRAITH_BOSS,
        EncounterKind::Boss(BossKind::RiverDemon) => return &RIVER_DEMON_BOSS,
        EncounterKind::Boss(BossKind::MiasmaRoot) => return &MIASMA_ROOT_BOSS,
        EncounterKind::Boss(BossKind::MirrorMinister) => return &MIRROR_MINISTER_BOSS,
        EncounterKind::Boss(BossKind::ThunderQilin) => return &THUNDER_QILIN_BOSS,
        EncounterKind::Boss(BossKind::DreamEclipse) => return &DREAM_ECLIPSE_BOSS,
        EncounterKind::Random => {}
    }

    let pool = match zone {
        EncounterZone::Village => &VILLAGE_ENCOUNTERS[..],
        EncounterZone::Bamboo => &BAMBOO_ENCOUNTERS[..],
        EncounterZone::Cave => &CAVE_ENCOUNTERS[..],
        EncounterZone::RiverTown => &RIVER_TOWN_ENCOUNTERS[..],
        EncounterZone::PlagueVillage => &PLAGUE_VILLAGE_ENCOUNTERS[..],
        EncounterZone::Capital => &CAPITAL_ENCOUNTERS[..],
        EncounterZone::SouthernRoad => &SOUTHERN_ROAD_ENCOUNTERS[..],
        EncounterZone::FinalSanctum => &FINAL_SANCTUM_ENCOUNTERS[..],
    };
    let index = pool[rng.range(0, pool.len() as i32 - 1) as usize];
    &ENEMIES[index]
}

fn spawn_battle_backdrop(
    commands: &mut Commands,
    asset_server: &AssetServer,
    lights: &LightingAssets,
    zone: EncounterZone,
) {
    let backdrop = battle_backdrop(zone);
    commands.spawn((
        Sprite::from_color(backdrop.base, Vec2::new(1280.0, 720.0)),
        Transform::from_xyz(0.0, 0.0, -3.0),
        DespawnOnExit(AppState::Battle),
    ));
    commands.spawn((
        Sprite {
            image: asset_server.load(backdrop.far_texture),
            color: backdrop.far_tint,
            custom_size: Some(Vec2::new(1280.0, 380.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 88.0, -2.6),
        DespawnOnExit(AppState::Battle),
    ));
    commands.spawn((
        Sprite::from_color(backdrop.horizon, Vec2::new(1280.0, 190.0)),
        Transform::from_xyz(0.0, 20.0, -2.35),
        DespawnOnExit(AppState::Battle),
    ));
    commands.spawn((
        Sprite {
            image: asset_server.load(backdrop.floor_texture),
            color: backdrop.floor_tint,
            custom_size: Some(Vec2::new(1280.0, 300.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -118.0, -2.1),
        DespawnOnExit(AppState::Battle),
    ));
    commands.spawn((
        Sprite::from_color(Color::srgba(0.0, 0.0, 0.0, 0.34), Vec2::new(1280.0, 130.0)),
        Transform::from_xyz(0.0, -218.0, -1.9),
        DespawnOnExit(AppState::Battle),
    ));
    lighting::spawn_light(
        commands,
        lights,
        Vec3::new(0.0, 50.0, -1.8),
        620.0,
        backdrop.light,
        AppState::Battle,
    );
}

fn battle_backdrop(zone: EncounterZone) -> BattleBackdrop {
    match zone {
        EncounterZone::Village => BattleBackdrop {
            base: Color::srgb(0.07, 0.10, 0.08),
            horizon: Color::srgba(0.12, 0.22, 0.14, 0.58),
            far_texture: "tiles/forest_floor.png",
            far_tint: Color::srgba(0.34, 0.54, 0.32, 0.42),
            floor_texture: "tiles/ai_village_moss_path.png",
            floor_tint: Color::srgb(0.62, 0.58, 0.42),
            light: Color::srgba(0.82, 0.96, 0.54, 0.20),
        },
        EncounterZone::Bamboo => BattleBackdrop {
            base: Color::srgb(0.06, 0.12, 0.09),
            horizon: Color::srgba(0.12, 0.30, 0.16, 0.58),
            far_texture: "tiles/ai_bamboo_thicket.png",
            far_tint: Color::srgba(0.32, 0.72, 0.34, 0.46),
            floor_texture: "tiles/ai_bamboo_path.png",
            floor_tint: Color::srgb(0.72, 0.64, 0.42),
            light: Color::srgba(0.54, 1.0, 0.52, 0.22),
        },
        EncounterZone::Cave => BattleBackdrop {
            base: Color::srgb(0.07, 0.06, 0.12),
            horizon: Color::srgba(0.20, 0.18, 0.32, 0.64),
            far_texture: "tiles/ai_moon_cave_wall.png",
            far_tint: Color::srgba(0.54, 0.48, 0.82, 0.48),
            floor_texture: "tiles/ai_cave_floor.png",
            floor_tint: Color::srgb(0.48, 0.52, 0.66),
            light: Color::srgba(0.52, 0.72, 1.0, 0.28),
        },
        EncounterZone::RiverTown => BattleBackdrop {
            base: Color::srgb(0.06, 0.10, 0.12),
            horizon: Color::srgba(0.08, 0.24, 0.30, 0.56),
            far_texture: "tiles/water_edge.png",
            far_tint: Color::srgba(0.42, 0.76, 1.0, 0.46),
            floor_texture: "tiles/ai_shrine_floor.png",
            floor_tint: Color::srgb(0.56, 0.58, 0.50),
            light: Color::srgba(0.42, 0.82, 1.0, 0.26),
        },
        EncounterZone::PlagueVillage => BattleBackdrop {
            base: Color::srgb(0.08, 0.10, 0.07),
            horizon: Color::srgba(0.18, 0.30, 0.12, 0.60),
            far_texture: "tiles/ai_mystic_grass.png",
            far_tint: Color::srgba(0.44, 0.62, 0.32, 0.42),
            floor_texture: "tiles/ai_village_moss_path.png",
            floor_tint: Color::srgb(0.42, 0.46, 0.32),
            light: Color::srgba(0.62, 0.96, 0.42, 0.22),
        },
        EncounterZone::Capital => BattleBackdrop {
            base: Color::srgb(0.08, 0.08, 0.13),
            horizon: Color::srgba(0.20, 0.20, 0.34, 0.60),
            far_texture: "tiles/stone_road.png",
            far_tint: Color::srgba(0.48, 0.50, 0.66, 0.44),
            floor_texture: "tiles/ai_shrine_floor.png",
            floor_tint: Color::srgb(0.54, 0.54, 0.68),
            light: Color::srgba(0.64, 0.72, 1.0, 0.24),
        },
        EncounterZone::SouthernRoad => BattleBackdrop {
            base: Color::srgb(0.07, 0.11, 0.08),
            horizon: Color::srgba(0.14, 0.34, 0.18, 0.58),
            far_texture: "tiles/ai_bamboo_thicket.png",
            far_tint: Color::srgba(0.28, 0.68, 0.42, 0.46),
            floor_texture: "tiles/ai_bamboo_path.png",
            floor_tint: Color::srgb(0.62, 0.56, 0.34),
            light: Color::srgba(0.72, 1.0, 0.48, 0.24),
        },
        EncounterZone::FinalSanctum => BattleBackdrop {
            base: Color::srgb(0.06, 0.06, 0.14),
            horizon: Color::srgba(0.12, 0.18, 0.40, 0.64),
            far_texture: "tiles/ai_moon_cave_wall.png",
            far_tint: Color::srgba(0.46, 0.54, 1.0, 0.50),
            floor_texture: "tiles/ai_cave_floor.png",
            floor_tint: Color::srgb(0.46, 0.50, 0.76),
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
    mut state: ResMut<BattleState>,
    mut stats: ResMut<PlayerStats>,
    quest: Res<QuestLog>,
    mut rng: ResMut<Rng>,
    run: Option<Res<RunState>>,
) {
    if state.phase != Phase::Menu {
        return;
    }

    // Roguelike relic modifiers (all zero/1.0 outside run mode).
    let relic_attack = run.as_ref().map_or(0, |r| r.attack_bonus());
    let relic_spell = run.as_ref().map_or(0, |r| r.spell_bonus());
    let relic_spell_cost = run.as_ref().map_or(0, |r| r.spell_cost_delta());
    let relic_potion = run.as_ref().map_or(0, |r| r.potion_bonus());
    let relic_flee = run.as_ref().is_some_and(|r| r.flee_always());
    let hunter_mul = run.as_ref().map_or(1.0, |r| {
        r.hunter_multiplier(r.current_fight.unwrap_or(FightRank::Normal))
    });

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
            let base = (stats.atk - state.enemy.def + rng.range(-2, 3)).max(1);
            let blessing_bonus = blessing_damage_bonus(blessing, PlayerAction::Attack);
            let camp_damage = camp_damage_bonus(camp_bonus, PlayerAction::Attack);
            let bond_damage = bond_damage_bonus(bond_bonus, PlayerAction::Attack);
            let dmg = (((base + blessing_bonus + camp_damage + bond_damage + relic_attack) as f32)
                * hunter_mul)
                .round() as i32;
            state.enemy.hp -= dmg;
            state.message = format!(
                "李逍遥 挥剑而上，对 {} 造成 {} 点伤害！",
                state.enemy.name, dmg
            );
            if relic_attack > 0 {
                state
                    .message
                    .push_str(&format!("\n【青锋剑穗】剑势更利,伤害 +{relic_attack}。"));
            }
            append_blessing_damage_line(&mut state.message, blessing, blessing_bonus);
            append_camp_damage_line(&mut state.message, camp_bonus, camp_damage);
            append_bond_damage_line(&mut state.message, bond_bonus, bond_damage);
            spawn_weapon_hit(&mut commands, &anims, &lights, ENEMY_POS);
            spawn_damage_text(
                &mut commands,
                &font,
                ENEMY_POS + Vec3::new(96.0, 72.0, 0.0),
                dmg,
            );
            if let Some(extra) = sword_sister_followup(&quest, &stats, &mut state.enemy, &mut rng) {
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
            stats.mp = (stats.mp + GUARD_MP_RESTORE).min(stats.max_mp);
            state.message =
                format!("李逍遥 剑交左手,凝神御守——气随息回,恢复 {GUARD_MP_RESTORE} 点灵力。");
            start_player_acting(&mut state, PlayerAction::Guard);
        }
        2 => {
            // 仙术：御剑术 / 万剑诀
            let spell_cost = (quest.spell_cost() + relic_spell_cost).max(1);
            let spell_name = quest.spell_name();
            if stats.mp < spell_cost {
                state.message = format!("灵力不足，无法施展{spell_name}！");
            } else {
                let blessing = state.blessing;
                let camp_bonus = state.camp_bonus;
                let bond_bonus = state.bond_bonus;
                stats.mp -= spell_cost;
                let roll_max = if quest.has_late_spell() { 10 } else { 6 };
                let base = (stats.atk * quest.spell_power_multiplier() - state.enemy.def
                    + rng.range(0, roll_max))
                .max(1);
                let blessing_bonus = blessing_damage_bonus(blessing, PlayerAction::Spell);
                let camp_damage = camp_damage_bonus(camp_bonus, PlayerAction::Spell);
                let bond_damage = bond_damage_bonus(bond_bonus, PlayerAction::Spell);
                let dmg = (((base + blessing_bonus + camp_damage + bond_damage + relic_spell)
                    as f32)
                    * hunter_mul)
                    .round() as i32;
                state.enemy.hp -= dmg;
                let flavor = if quest.has_late_spell() {
                    "剑光如雨"
                } else {
                    "剑气纵横"
                };
                state.message = format!("李逍遥 施展{spell_name}，{flavor}，造成 {dmg} 点伤害！");
                if relic_spell > 0 {
                    state
                        .message
                        .push_str(&format!("\n【御剑心诀】术随心动,伤害 +{relic_spell}。"));
                }
                append_blessing_damage_line(&mut state.message, blessing, blessing_bonus);
                append_camp_damage_line(&mut state.message, camp_bonus, camp_damage);
                append_bond_damage_line(&mut state.message, bond_bonus, bond_damage);
                if let Some(extra) =
                    sword_sister_followup(&quest, &stats, &mut state.enemy, &mut rng)
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
                spawn_spell_impact(&mut commands, &anims, &lights, quest.has_late_spell());
                spawn_damage_text(
                    &mut commands,
                    &font,
                    ENEMY_POS + Vec3::new(104.0, 76.0, 0.0),
                    dmg,
                );
                state.momentum = (state.momentum + 1).min(MOMENTUM_MAX);
                start_player_acting(&mut state, PlayerAction::Spell);
            }
        }
        3 if run.is_some() => {
            // 绝技·剑气爆发:气势满层时的一锤定音。
            if state.momentum < MOMENTUM_MAX {
                state.message = format!(
                    "气势未足({}/{MOMENTUM_MAX})——连续攻击或施术蓄满气势,方可施展绝技。",
                    state.momentum
                );
            } else {
                let base = (stats.atk * 2 - state.enemy.def + rng.range(2, 9)).max(3);
                let dmg =
                    (((base + relic_attack + relic_spell) as f32) * hunter_mul).round() as i32;
                state.enemy.hp -= dmg;
                state.momentum = 0;
                state.message =
                    format!("李逍遥 气势鼎盛,施展绝技·剑气爆发!剑光如潮水倾泻,造成 {dmg} 点伤害!");
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
            // 合击：逍遥、灵儿、林月衡
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
                let bond_damage = bond_damage_bonus(bond_bonus, PlayerAction::Combo);
                let dmg = (((base + blessing_bonus + camp_damage + bond_damage) as f32)
                    * hunter_mul)
                    .round() as i32;
                state.enemy.hp -= dmg;
                state.message =
                    format!("李逍遥、赵灵儿、林月衡 心念相合，剑光与灵息齐落，造成 {dmg} 点伤害！");
                append_blessing_damage_line(&mut state.message, blessing, blessing_bonus);
                append_camp_damage_line(&mut state.message, camp_bonus, camp_damage);
                append_bond_damage_line(&mut state.message, bond_bonus, bond_damage);
                spawn_combo_party_casts(&mut commands, &anims, &lights);
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
                stats.hp = (stats.hp + PlayerStats::POTION_HEAL + relic_potion).min(stats.max_hp);
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

fn start_player_acting(state: &mut BattleState, action: PlayerAction) {
    state.phase = Phase::PlayerActing;
    state.timer = 0.8;
    state.player_action = Some(action);
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

fn spawn_spell_impact(
    commands: &mut Commands,
    anims: &AnimationAssets,
    lights: &LightingAssets,
    late_spell: bool,
) {
    let profile = spell_impact_profile(late_spell);
    let mut sprite = anims.sprite(AnimationClip::SkillImpact, Vec2::splat(profile.core_size));
    sprite.color = if late_spell {
        Color::srgba(0.96, 0.90, 1.0, 0.98)
    } else {
        Color::srgba(1.0, 0.92, 0.55, 0.96)
    };

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
    ring.color = if late_spell {
        Color::srgba(0.64, 0.82, 1.0, 0.68)
    } else {
        Color::srgba(1.0, 0.70, 0.26, 0.58)
    };
    commands.spawn((
        ring,
        SpriteAnimation::once(AnimationClip::SkillImpact),
        BattleEffect::new(profile.ring_duration, 0.20, profile.ring_end_scale, 0.42),
        Transform::from_xyz(ENEMY_POS.x, ENEMY_POS.y + 2.0, 2.55),
        DespawnOnExit(AppState::Battle),
    ));

    for shard in spell_impact_shards(late_spell) {
        spawn_spell_impact_shard(commands, anims, shard);
    }

    let flash = lighting::spawn_light(
        commands,
        lights,
        Vec3::new(ENEMY_POS.x, ENEMY_POS.y + 4.0, 2.1),
        profile.flash_radius,
        if late_spell {
            Color::srgba(0.72, 0.86, 1.0, profile.flash_alpha)
        } else {
            Color::srgba(1.0, 0.62, 0.24, profile.flash_alpha)
        },
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

    // Roguelike runs play at a brisker tempo.
    let tempo = if run.is_some() { 1.6 } else { 1.0 };
    state.timer -= time.delta_secs() * tempo;
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
                    let gold = (battle_gold_reward(exp, boss) as f32 * run.gold_multiplier())
                        .round() as u32;
                    stats.gold += gold;
                    state.message = format!("{} 被击败了！拾得 {} 文钱。", state.enemy.name, gold);
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
                let relic_guard = run.as_ref().map_or(0, |r| r.incoming_reduction());
                let strong_guard = run.as_ref().map_or(0, |r| r.strong_hit_guard());
                let attack =
                    begin_enemy_turn(&mut state, &mut stats, &mut rng, relic_guard, strong_guard);
                spawn_enemy_strike_impact(&mut commands, &anims, &lights, attack.strong);
                spawn_damage_text(
                    &mut commands,
                    &font,
                    HERO_POS + Vec3::new(34.0, 116.0, 0.0),
                    attack.damage,
                );
            }
        }
        Phase::EnemyActing => {
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
                if let Some(heal) = companion_support(&quest, state.bond_bonus, &mut stats) {
                    state.message = format!(
                        "赵灵儿 以灵息护住你，恢复 {} 点气血。\n你的回合，请选择行动。",
                        heal
                    );
                    spawn_linger_support_aura(&mut commands, &anims, &lights);
                    spawn_heal_text(
                        &mut commands,
                        &font,
                        HERO_POS + Vec3::new(12.0, 106.0, 0.0),
                        heal,
                    );
                } else {
                    state.message = "你的回合，请选择行动。".into();
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

fn companion_support(
    quest: &QuestLog,
    bond_bonus: Option<BondBonus>,
    stats: &mut PlayerStats,
) -> Option<i32> {
    if !quest.has_companion(Companion::Linger) || stats.hp >= stats.max_hp {
        return None;
    }

    let bond_heal = if matches!(bond_bonus, Some(BondBonus::Tender)) {
        2
    } else {
        0
    };
    let heal = (4
        + stats.level as i32
        + quest.bond_level() as i32 * 2
        + quest.bond_tender_level() as i32
        + bond_heal)
        .min(stats.max_hp - stats.hp);
    stats.hp += heal;
    Some(heal)
}

fn battle_gold_reward(exp: u32, boss: bool) -> u32 {
    exp / 2 + if boss { 40 } else { 6 }
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
        + rng.range(4, 12)
        - enemy.def)
        .max(6)
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
    stats: &PlayerStats,
    enemy: &mut EnemyInstance,
    rng: &mut Rng,
) -> Option<i32> {
    if !quest.has_companion(Companion::SwordSister) || enemy.hp <= 0 {
        return None;
    }

    let dmg = (stats.atk / 2
        + stats.level as i32
        + quest.bond_level() as i32
        + quest.bond_courage_level() as i32
        + rng.range(0, 4)
        - enemy.def / 2)
        .max(2);
    enemy.hp -= dmg;
    Some(dmg)
}

fn begin_enemy_turn(
    state: &mut BattleState,
    stats: &mut PlayerStats,
    rng: &mut Rng,
    relic_guard: i32,
    strong_guard: i32,
) -> EnemyAttackResult {
    let turn_index = state.enemy_turns;
    state.enemy_turns += 1;
    let guarding = state.guarding;
    state.guarding = false;

    if let EncounterKind::Boss(boss) = state.encounter_kind {
        if let Some(result) = begin_boss_special_turn(boss, turn_index, state, stats, rng, guarding)
        {
            state.intent = next_intent(state, rng);
            return result;
        }
    }

    let intent = state.intent;
    let (dmg, strong) = if intent == EnemyIntent::Gather {
        // 凝气回合:不攻击,回血并提升防御——错过输出窗口是玩家的损失。
        let heal = (state.enemy.max_hp / 12).max(3);
        state.enemy.hp = (state.enemy.hp + heal).min(state.enemy.max_hp);
        state.enemy.def += 1;
        state.message = format!(
            "{} 凝气回息,恢复 {heal} 点气血,妖气愈发凝实(防御 +1)。",
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
            _ => (1.0, false),
        };
        let raw = ((state.enemy.atk as f32 * mult) as i32 - stats.def + rng.range(-2, 4)).max(1);
        let blocked = blessing_guard_block(state.blessing, raw);
        let camp_blocked = camp_guard_block(state.camp_bonus, raw - blocked);
        let bond_blocked = bond_guard_block(state.bond_bonus, raw - blocked - camp_blocked);
        let strong_cut = if strong { strong_guard } else { 0 };
        let mut dmg =
            (raw - blocked - camp_blocked - bond_blocked - relic_guard - strong_cut).max(0);
        let mut guard_note = String::new();
        if guarding {
            let absorbed = dmg - dmg * 35 / 100;
            dmg -= absorbed;
            guard_note = format!("\n李逍遥 御守卸力,挡下 {absorbed} 点伤害!");
        }
        state.message = match intent {
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
        append_bond_guard_line(&mut state.message, state.bond_bonus, bond_blocked);
        append_relic_guard_line(&mut state.message, relic_guard);
        state.message.push_str(&guard_note);
        (dmg, strong)
    };
    stats.hp -= dmg;
    state.intent = next_intent(state, rng);
    state.phase = Phase::EnemyActing;
    state.timer = 0.8;
    state.player_action = None;
    EnemyAttackResult {
        damage: dmg,
        strong,
    }
}

/// 掷下一回合的意图;首领每逢秘法回合(每三回合)提前亮出重击预警。
fn next_intent(state: &BattleState, rng: &mut Rng) -> EnemyIntent {
    if matches!(state.encounter_kind, EncounterKind::Boss(_)) && state.enemy_turns % 3 == 0 {
        return EnemyIntent::Heavy;
    }
    roll_intent(rng)
}

fn begin_boss_special_turn(
    boss: BossKind,
    turn_index: u32,
    state: &mut BattleState,
    stats: &mut PlayerStats,
    rng: &mut Rng,
    guarding: bool,
) -> Option<EnemyAttackResult> {
    if turn_index % 3 != 0 {
        return None;
    }

    let (raw, message, mp_drain, enemy_heal) = boss_special_attack(boss, state, stats, rng);
    let blocked = blessing_guard_block(state.blessing, raw);
    let camp_blocked = camp_guard_block(state.camp_bonus, raw - blocked);
    let bond_blocked = bond_guard_block(state.bond_bonus, raw - blocked - camp_blocked);
    let mut dmg = (raw - blocked - camp_blocked - bond_blocked).max(0);
    let mut guard_note = String::new();
    if guarding {
        let absorbed = dmg - dmg * 35 / 100;
        dmg -= absorbed;
        guard_note = format!("\n李逍遥 御守卸力,挡下 {absorbed} 点伤害!");
    }
    stats.hp -= dmg;
    if mp_drain > 0 {
        stats.mp = (stats.mp - mp_drain).max(0);
    }
    if enemy_heal > 0 {
        state.enemy.hp = (state.enemy.hp + enemy_heal).min(state.enemy.max_hp);
    }

    state.message = format!("{message}造成 {dmg} 点伤害！");
    append_blessing_guard_line(&mut state.message, blocked);
    append_camp_guard_line(&mut state.message, state.camp_bonus, camp_blocked);
    append_bond_guard_line(&mut state.message, state.bond_bonus, bond_blocked);
    state.message.push_str(&guard_note);
    if mp_drain > 0 {
        state
            .message
            .push_str(&format!("\n【Boss】灵力被扰乱，失去 {mp_drain} 点灵力。"));
    }
    if enemy_heal > 0 {
        state.message.push_str(&format!(
            "\n【Boss】{} 回稳 {enemy_heal} 点气血。",
            state.enemy.name
        ));
    }

    state.phase = Phase::EnemyActing;
    state.timer = 0.9;
    state.player_action = None;
    Some(EnemyAttackResult {
        damage: dmg,
        strong: true,
    })
}

fn boss_special_attack(
    boss: BossKind,
    state: &BattleState,
    stats: &PlayerStats,
    rng: &mut Rng,
) -> (i32, &'static str, i32, i32) {
    match boss {
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

fn update_battle_animations(
    time: Res<Time>,
    state: Res<BattleState>,
    anims: Res<AnimationAssets>,
    mut hero: Query<
        (&mut Sprite, &mut SpriteAnimation),
        (
            With<BattleHero>,
            Without<BattleCompanion>,
            Without<BattleEnemy>,
            Without<BattleEnemyAura>,
        ),
    >,
    mut companions: Query<
        (&mut BattleCompanion, &mut Transform, &mut Sprite),
        (
            With<BattleCompanion>,
            Without<BattleHero>,
            Without<BattleEnemy>,
            Without<BattleEnemyAura>,
        ),
    >,
    mut enemy: Query<
        (&mut Transform, &mut Sprite, &mut BattleEnemyMotion),
        (
            With<BattleEnemy>,
            Without<BattleHero>,
            Without<BattleCompanion>,
            Without<BattleEnemyAura>,
        ),
    >,
    mut aura: Query<
        (
            &mut Transform,
            &mut Sprite,
            &mut SpriteAnimation,
            &mut BattleEnemyAura,
        ),
        (
            With<BattleEnemyAura>,
            Without<BattleEnemy>,
            Without<BattleHero>,
            Without<BattleCompanion>,
        ),
    >,
) {
    if let Ok((mut sprite, mut animation)) = hero.single_mut() {
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
    }

    for (mut companion, mut transform, mut sprite) in &mut companions {
        companion.age += time.delta_secs();
        let (offset, scale, tint) = companion_motion(
            companion.kind,
            state.phase,
            state.player_action,
            state.timer,
            companion.age,
        );

        transform.translation.x = companion.origin.x + offset.x;
        transform.translation.y = companion.origin.y + offset.y;
        transform.scale = Vec3::splat(scale);
        sprite.color = tint;
    }

    if let Ok((mut transform, mut sprite, mut motion)) = enemy.single_mut() {
        motion.age += time.delta_secs();
        let breath = (motion.age * 3.0).sin();
        let (x, y, scale, hit) = enemy_phase_motion(state.phase, state.player_action, state.timer);

        transform.translation.x = motion.origin.x + x;
        transform.translation.y = motion.origin.y + y + breath * 5.0;
        transform.scale = Vec3::splat((scale + breath * 0.025).max(0.82));
        sprite.color = enemy_primary_color(state.phase, hit);
    }

    if let Ok((mut transform, mut sprite, mut animation, mut aura)) = aura.single_mut() {
        animation::set_clip(
            &anims,
            &mut sprite,
            &mut animation,
            enemy_clip_for_phase(state.phase),
        );
        aura.age += time.delta_secs();
        let pulse = (aura.age * 4.2).sin() * 0.5 + 0.5;
        let (x, y, scale, _) = enemy_phase_motion(state.phase, state.player_action, state.timer);

        transform.translation.x = aura.origin.x + x * 0.9;
        transform.translation.y = aura.origin.y + y + pulse * 8.0 - 4.0;
        transform.scale = Vec3::splat((scale + pulse * 0.04).max(0.82));
        sprite.color = enemy_aura_color(aura.light, state.phase, pulse);
    }
}

fn enemy_clip_for_phase(phase: Phase) -> AnimationClip {
    if phase == Phase::EnemyActing {
        AnimationClip::MonsterAttack
    } else {
        AnimationClip::MonsterIdle
    }
}

fn enemy_primary_image(def: &EnemyDef) -> &'static str {
    def.image
}

fn enemy_primary_size(def: &EnemyDef) -> f32 {
    def.size
}

fn enemy_aura_size(def: &EnemyDef) -> f32 {
    def.size * 0.82
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
    }
}

fn companion_phase_offset(kind: BattleCompanionKind) -> f32 {
    match kind {
        BattleCompanionKind::Linger => 0.7,
        BattleCompanionKind::SwordSister => 1.4,
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

fn enemy_aura_color(light: [f32; 4], phase: Phase, pulse: f32) -> Color {
    let phase_boost = match phase {
        Phase::EnemyActing => 0.18,
        Phase::PlayerActing => 0.08,
        Phase::Won => -0.12,
        Phase::Menu | Phase::Lost | Phase::Fled => 0.0,
    };
    let alpha = (light[3] + phase_boost + pulse * 0.05).clamp(0.12, 0.62);

    Color::srgba(light[0], light[1], light[2], alpha)
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

fn update_battle_ui(
    state: Res<BattleState>,
    stats: Res<PlayerStats>,
    quest: Res<QuestLog>,
    run: Option<Res<RunState>>,
    mut bar: Query<(&mut Sprite, &mut Transform), With<EnemyHpBar>>,
    mut enemy_info: Query<&mut Text, (With<EnemyInfoText>, Without<MessageText>)>,
    mut message: Query<
        &mut Text,
        (
            With<MessageText>,
            Without<EnemyInfoText>,
            Without<PlayerInfoText>,
        ),
    >,
    mut player_info: Query<
        &mut Text,
        (
            With<PlayerInfoText>,
            Without<MessageText>,
            Without<EnemyInfoText>,
        ),
    >,
    mut menu: Query<
        (&MenuItem, &mut Text, &mut TextColor),
        (
            Without<MessageText>,
            Without<EnemyInfoText>,
            Without<PlayerInfoText>,
        ),
    >,
) {
    // Enemy HP bar shrinks from the left.
    if let Ok((mut sprite, mut tf)) = bar.single_mut() {
        let ratio =
            (state.enemy.hp.max(0) as f32 / state.enemy.max_hp.max(1) as f32).clamp(0.0, 1.0);
        let w = HP_BAR_W * ratio;
        sprite.custom_size = Some(Vec2::new(w, 14.0));
        tf.translation.x = HP_BAR_LEFT + w / 2.0;
    }

    if let Ok(mut t) = enemy_info.single_mut() {
        t.0 = format!(
            "{}  气血 {}/{}   {}",
            state.enemy.name,
            state.enemy.hp.max(0),
            state.enemy.max_hp,
            state.intent.describe(),
        );
    }
    if let Ok(mut t) = message.single_mut() {
        t.0 = state.message.clone();
    }
    if let Ok(mut t) = player_info.single_mut() {
        let momentum: String = (0..MOMENTUM_MAX)
            .map(|i| if i < state.momentum { '●' } else { '○' })
            .collect();
        t.0 = format!(
            "{}  气血 {}/{}   灵力 {}/{}   气势 {}   药水 x{}   钱 {}文   {}   {}   {}   {}",
            quest.party_summary(),
            stats.hp.max(0),
            stats.max_hp,
            stats.mp.max(0),
            stats.max_mp,
            momentum,
            stats.potions,
            stats.gold,
            quest.bond_summary(),
            battle_bond_summary(state.bond_bonus),
            battle_blessing_summary(state.blessing),
            battle_camp_summary(state.camp_bonus),
        );
    }

    let show_cursor = state.phase == Phase::Menu;
    for (item, mut text, mut color) in menu.iter_mut() {
        let selected = show_cursor && item.0 == state.menu_index;
        let label = match item.0 {
            2 => format!("{} (灵力{})", quest.spell_name(), quest.spell_cost()),
            3 if run.is_some() => format!("绝技·剑气爆发 (气势{MOMENTUM_MAX})"),
            3 if combo_unlocked(&quest) => format!("合击 (灵力{COMBO_COST})"),
            3 => "合击 (未解锁)".to_string(),
            _ => MENU[item.0].to_string(),
        };
        text.0 = if selected {
            format!("▶ {}", label)
        } else {
            format!("   {}", label)
        };
        *color = if selected {
            TextColor(Color::srgb(1.0, 0.9, 0.4))
        } else {
            TextColor(Color::srgb(0.8, 0.8, 0.85))
        };
    }
}

fn battle_blessing_summary(blessing: Option<ShrineBlessing>) -> &'static str {
    match blessing {
        Some(ShrineBlessing::Guard) => "祝福 护身",
        Some(ShrineBlessing::Sword) => "祝福 剑心",
        Some(ShrineBlessing::Spirit) => "祝福 灵息",
        None => "祝福 无",
    }
}

fn battle_camp_summary(bonus: Option<CampBonus>) -> &'static str {
    match bonus {
        Some(CampBonus::Warmth) => "营地 余温",
        Some(CampBonus::Focus) => "营地 静心",
        Some(CampBonus::Vigil) => "营地 守夜",
        None => "营地 无",
    }
}

fn battle_bond_summary(bonus: Option<BondBonus>) -> &'static str {
    match bonus {
        Some(BondBonus::Courage) => "护念 勇心",
        Some(BondBonus::Tender) => "护念 柔心",
        None => "护念 无",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::quest::{BondResponse, QuestRole};

    #[test]
    fn boss_encounter_uses_fixed_boss_def() {
        let mut rng = Rng::default();
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
    fn enemy_sprite_uses_attack_clip_only_during_enemy_turn() {
        assert_eq!(
            enemy_clip_for_phase(Phase::Menu),
            AnimationClip::MonsterIdle
        );
        assert_eq!(
            enemy_clip_for_phase(Phase::PlayerActing),
            AnimationClip::MonsterIdle
        );
        assert_eq!(
            enemy_clip_for_phase(Phase::EnemyActing),
            AnimationClip::MonsterAttack
        );
        assert_eq!(enemy_clip_for_phase(Phase::Won), AnimationClip::MonsterIdle);
    }

    #[test]
    fn generated_enemy_cutouts_are_primary_battle_visuals() {
        let enemy = &ENEMIES[0];

        assert_eq!(enemy_primary_image(enemy), "creatures/ai_water_serpent.png");
        assert_eq!(enemy_primary_size(enemy), enemy.size);
        assert!(enemy_primary_size(enemy) > enemy_aura_size(enemy));

        let primary_alpha = enemy_primary_color(Phase::Menu, 0.0).to_srgba().alpha;
        let aura_alpha = enemy_aura_color(enemy.light, Phase::Menu, 0.0)
            .to_srgba()
            .alpha;
        let attack_aura_alpha = enemy_aura_color(enemy.light, Phase::EnemyActing, 1.0)
            .to_srgba()
            .alpha;

        assert!(primary_alpha > aura_alpha);
        assert!(attack_aura_alpha > aura_alpha);
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
    fn battle_backdrops_use_zone_specific_map_assets() {
        assert_eq!(
            battle_backdrop(EncounterZone::Village).far_texture,
            "tiles/forest_floor.png"
        );
        assert_eq!(
            battle_backdrop(EncounterZone::Cave).floor_texture,
            "tiles/ai_cave_floor.png"
        );
        assert_eq!(
            battle_backdrop(EncounterZone::RiverTown).far_texture,
            "tiles/water_edge.png"
        );
        assert_eq!(
            battle_backdrop(EncounterZone::Capital).floor_texture,
            "tiles/ai_shrine_floor.png"
        );
        assert_ne!(
            battle_backdrop(EncounterZone::Village).far_texture,
            battle_backdrop(EncounterZone::FinalSanctum).far_texture
        );
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
        assert_eq!(battle_camp_summary(Some(CampBonus::Focus)), "营地 静心");
    }

    #[test]
    fn linger_support_heals_after_joining_party() {
        let mut quest = QuestLog::default();
        let mut stats = PlayerStats::default();
        stats.hp = stats.max_hp - 10;

        assert_eq!(companion_support(&quest, None, &mut stats), None);
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);

        assert_eq!(companion_support(&quest, None, &mut stats), Some(5));
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
        assert_eq!(companion_support(&quest, None, &mut stats), Some(7));
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
            companion_support(&quest, Some(BondBonus::Tender), &mut stats),
            Some(10)
        );
        assert_eq!(stats.hp, stats.max_hp - 10);
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
            menu_index: 0,
            player_action: Some(PlayerAction::Attack),
            intent: EnemyIntent::Strike,
            guarding: false,
            momentum: 0,
            message: String::new(),
            blessing: None,
            camp_bonus: None,
            bond_bonus: None,
        };
        let mut stats = PlayerStats::default();
        let before = stats.hp;
        let mut rng = Rng::default();

        let attack = begin_enemy_turn(&mut state, &mut stats, &mut rng, 0, 0);

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
            menu_index: 0,
            player_action: Some(PlayerAction::Attack),
            intent: EnemyIntent::Strike,
            guarding: false,
            momentum: 0,
            message: String::new(),
            blessing: Some(ShrineBlessing::Guard),
            camp_bonus: None,
            bond_bonus: None,
        };
        let mut stats = PlayerStats::default();
        let before = stats.hp;
        let mut rng = Rng::default();

        let attack = begin_enemy_turn(&mut state, &mut stats, &mut rng, 0, 0);

        assert!(attack.damage > 0);
        assert!(attack.damage < 7);
        assert_eq!(stats.hp, before - attack.damage);
        assert!(state.message.contains("护身香火"));
    }

    #[test]
    fn boss_opening_turn_uses_signature_move() {
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
            menu_index: 0,
            player_action: Some(PlayerAction::Attack),
            intent: EnemyIntent::Strike,
            guarding: false,
            momentum: 0,
            message: String::new(),
            blessing: None,
            camp_bonus: None,
            bond_bonus: None,
        };
        let mut stats = PlayerStats::default();
        stats.mp = 10;
        let before_hp = stats.hp;
        let mut rng = Rng::default();

        let attack = begin_enemy_turn(&mut state, &mut stats, &mut rng, 0, 0);

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
            menu_index: 0,
            player_action: Some(PlayerAction::Attack),
            intent: EnemyIntent::Strike,
            guarding: false,
            momentum: 0,
            message: String::new(),
            blessing: None,
            camp_bonus: None,
            bond_bonus: None,
        };
        let mut stats = PlayerStats::default();
        let mut rng = Rng::default();

        let attack = begin_enemy_turn(&mut state, &mut stats, &mut rng, 0, 0);

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
            sword_sister_followup(&quest, &stats, &mut enemy, &mut rng),
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
        let dmg = sword_sister_followup(&quest, &stats, &mut enemy, &mut rng)
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

        let base = sword_sister_followup(&base_quest, &stats, &mut enemy_a, &mut rng_a).unwrap();
        let bonded =
            sword_sister_followup(&bonded_quest, &stats, &mut enemy_b, &mut rng_b).unwrap();
        assert_eq!(bonded, base + 1);
    }
}

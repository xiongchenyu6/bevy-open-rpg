//! CPU-only integration simulations: real input and turn systems, no combat formulas.
#[path = "playtest.rs"]
mod playtest;
use super::*;
use crate::game::roguelike::{CHAPTER_CLEAR_REWARDS, battle_mods_for};
use std::time::Duration;

#[derive(Clone, Copy, Debug)]
enum Policy {
    Attack,
    Offensive,
    Tactical,
}

impl Policy {
    fn name(self) -> &'static str {
        match self {
            Self::Attack => "attack",
            Self::Offensive => "spell-burst",
            Self::Tactical => "warning-aware",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Outcome {
    won: bool,
    timeout: bool,
    rounds: u32,
    hp: i32,
    mp: i32,
    breaks: u32,
    counters: u32,
    interrupts: u32,
    guards: u32,
    spells: u32,
    potions: u32,
}

fn simulation(def: &EnemyDef, kind: EncounterKind, chapter: usize, seed: u64) -> App {
    let mut state = tests::status_test_state(kind);
    state.phase = Phase::Menu;
    state.tactics = Tactics::new(enemy_style(def), matches!(kind, EncounterKind::Boss(_)));
    state.intent = initial_intent(def, kind);
    let mut stats = PlayerStats::default();
    // Same starting bonuses as the title screen's new-run action.
    stats.atk += 2;
    stats.def += 1;
    stats.potions += 1;
    for reward in CHAPTER_CLEAR_REWARDS.iter().take(chapter) {
        stats.max_hp += reward.hp;
        stats.max_mp += reward.mp;
        stats.atk += reward.atk;
        stats.def += reward.def;
    }
    stats.hp = stats.max_hp;
    stats.mp = stats.max_mp;
    let mut rng = Rng(seed);
    let mut run = RunState::new(&mut rng);
    run.chapter = chapter;
    let rank = if matches!(kind, EncounterKind::Boss(_)) {
        FightRank::Boss
    } else {
        FightRank::Normal
    };
    run.current_fight = Some(rank);
    let mods = battle_mods_for(&run, rank);
    state.enemy = scaled_enemy(def, Some(&mods), &mut rng);
    state.boss_heal_remaining = if matches!(kind, EncounterKind::Boss(_)) {
        state.enemy.max_hp / 4
    } else {
        0
    };
    if let EncounterKind::Boss(boss) = kind {
        run.boss = boss;
    }
    run.skills = vec![
        skill::SkillId::SwordGleam,
        skill::SkillId::ThornCage,
        skill::SkillId::FrostLance,
        skill::SkillId::EarthRift,
    ];
    let mut app = App::new();
    app.insert_resource(state)
        .insert_resource(stats)
        .insert_resource(run)
        .insert_resource(mods)
        .insert_resource(rng)
        .insert_resource(QuestLog::default())
        .insert_resource(Intent::default())
        .insert_resource(BattleAutomationView::default())
        .insert_resource(Time::<()>::default())
        .insert_resource(NextState::<AppState>::default())
        .insert_resource(PendingEncounter {
            zone: EncounterZone::Village,
            kind,
        })
        .insert_resource(GameFont(Handle::default()))
        .insert_resource(AnimationAssets::simulation())
        .insert_resource(SkillVfxAssets::simulation())
        .insert_resource(LightingAssets {
            orb: Handle::default(),
        })
        .insert_resource(Assets::<ColorMaterial>::default())
        .add_systems(
            Update,
            (
                battle_input,
                battle_tick,
                battle_lessons,
                sync_battle_automation_view,
            )
                .chain(),
        );
    app
}

fn step(app: &mut App, intent: Intent) {
    app.insert_resource(intent);
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f64(1.0 / 30.0));
    app.update();
    // Finish hook-generated commands before collecting visual entities. Internal
    // system/observer entities must survive between frames.
    app.world_mut().flush();
    let entities: Vec<Entity> = app
        .world_mut()
        .query_filtered::<Entity, With<DespawnOnExit<AppState>>>()
        .iter(app.world())
        .collect();
    for entity in entities {
        if app.world().get_entity(entity).is_ok() {
            app.world_mut().despawn(entity);
        }
    }
    app.world_mut().flush();
}

fn selected_skill(app: &App, policy: Policy) -> Option<usize> {
    let state = app.world().resource::<BattleState>();
    let stats = app.world().resource::<PlayerStats>();
    let run = app.world().resource::<RunState>();
    let affordable = |id| {
        run.skills
            .iter()
            .position(|candidate| *candidate == id)
            .filter(|_| stats.mp >= skill_cost(id, run))
    };
    if matches!(policy, Policy::Tactical) {
        if stats.hp * 2 < stats.max_hp {
            if let Some(index) = run.skills.iter().position(|id| {
                skill::skill(*id).kind == SkillKind::Mend && stats.mp >= skill_cost(*id, run)
            }) {
                return Some(index);
            }
        }
        if stats.hp * 3 < stats.max_hp * 2 {
            if let Some(index) = run.skills.iter().position(|id| {
                skill::skill(*id).kind == SkillKind::Drain && stats.mp >= skill_cost(*id, run)
            }) {
                return Some(index);
            }
        }
        if state.intent == EnemyIntent::Charge && state.player_shield < 5 {
            if let Some(index) = run.skills.iter().position(|id| {
                skill::skill(*id).kind == SkillKind::Ward && stats.mp >= skill_cost(*id, run)
            }) {
                return Some(index);
            }
        }
        if state.intent == EnemyIntent::Brace {
            if let Some(index) = affordable(skill::SkillId::EarthRift) {
                return Some(index);
            }
        }
        if state.statuses.poison.turns == 0 && state.enemy.hp > 20 {
            if let Some(index) = affordable(skill::SkillId::ThornCage) {
                return Some(index);
            }
        }
        if state.intent == EnemyIntent::Charge {
            if let Some(index) = affordable(skill::SkillId::FrostLance) {
                return Some(index);
            }
        }
    }
    run.skills.iter().position(|id| {
        stats.mp >= skill_cost(*id, run)
            && !matches!(skill::skill(*id).kind, SkillKind::Ward | SkillKind::Mend)
    })
}

fn menu_choice(app: &App, policy: Policy) -> usize {
    let state = app.world().resource::<BattleState>();
    let stats = app.world().resource::<PlayerStats>();
    let run = app.world().resource::<RunState>();
    if matches!(policy, Policy::Attack) {
        return 0;
    }
    if matches!(policy, Policy::Tactical) {
        if stats.hp * 3 < stats.max_hp && stats.potions > 0 {
            return 4;
        }
        if state.tactics.riposte || state.tactics.exposed {
            return 0;
        }
        if matches!(state.intent, EnemyIntent::Heavy | EnemyIntent::Special) {
            return 1;
        }
        if state.intent == EnemyIntent::Charge && state.tactics.pressure + 2 >= state.tactics.limit
        {
            return 0;
        }
    }
    if state.momentum >= run.hex_burst_cost() {
        return 3;
    }
    if selected_skill(app, policy).is_some() {
        return 2;
    }
    // Recover mana only against heavy attacks or brace; never waste turns charging a full pool.
    if matches!(policy, Policy::Tactical) && state.intent == EnemyIntent::Brace {
        return 1;
    }
    0
}

fn run_simulation(
    def: &EnemyDef,
    kind: EncounterKind,
    chapter: usize,
    seed: u64,
    policy: Policy,
) -> Outcome {
    run_app(simulation(def, kind, chapter, seed), policy)
}

fn run_app(mut app: App, policy: Policy) -> Outcome {
    let mut rounds = 0;
    let mut guards = 0;
    let mut spells = 0;
    let mut potions = 0;
    for _ in 0..4000 {
        let state = app.world().resource::<BattleState>();
        if matches!(state.phase, Phase::Won | Phase::Lost | Phase::Fled) {
            break;
        }
        let mut intent = Intent::default();
        let mut chosen = None;
        if state.phase == Phase::Menu {
            if state.skill_menu {
                if let Some(index) = selected_skill(&app, policy) {
                    if state.skill_cursor == index {
                        intent.confirm = true;
                        chosen = Some(2);
                    } else {
                        intent.down = true;
                    }
                } else {
                    intent.cancel = true;
                }
            } else {
                let choice = menu_choice(&app, policy);
                if state.menu_index == choice {
                    intent.confirm = true;
                    chosen = Some(choice);
                } else {
                    intent.down = true;
                }
            }
        }
        step(&mut app, intent);
        if app.world().resource::<BattleState>().phase == Phase::PlayerActing {
            if let Some(choice) = chosen {
                rounds += 1;
                guards += u32::from(choice == 1);
                spells += u32::from(choice == 2);
                potions += u32::from(choice == 4);
            }
        }
    }
    let state = app.world().resource::<BattleState>();
    let stats = app.world().resource::<PlayerStats>();
    Outcome {
        won: state.phase == Phase::Won,
        timeout: !matches!(state.phase, Phase::Won | Phase::Lost | Phase::Fled),
        rounds,
        hp: stats.hp,
        mp: stats.mp,
        breaks: state.tactics.breaks,
        counters: state.tactics.counters,
        interrupts: state.tactics.interrupts,
        guards,
        spells,
        potions,
    }
}

#[test]
fn real_turn_simulation_is_deterministic_and_finishes() {
    let first = run_simulation(
        &MOUNTAIN_FIEND_BOSS,
        EncounterKind::Boss(BossKind::MountainFiend),
        0,
        12345,
        Policy::Tactical,
    );
    let second = run_simulation(
        &MOUNTAIN_FIEND_BOSS,
        EncounterKind::Boss(BossKind::MountainFiend),
        0,
        12345,
        Policy::Tactical,
    );
    assert_eq!(first, second);
    assert!(!first.timeout, "{first:?}");
    assert!(first.guards > 0 && first.spells > 0, "{first:?}");
    let normal = run_simulation(
        &ENEMIES[1],
        EncounterKind::Random,
        0,
        23456,
        Policy::Tactical,
    );
    assert!(!normal.timeout && normal.won, "{normal:?}");
}

#[test]
fn learned_spell_uses_real_menu_and_applies_break_pressure() {
    let mut app = simulation(&ENEMIES[1], EncounterKind::Random, 0, 12345);
    app.world_mut().resource_mut::<RunState>().skills =
        vec![skill::SkillId::SwordGleam, skill::SkillId::EarthRift];
    // Navigate to Spell, enter catalog, select EarthRift, confirm.
    for intent in [
        Intent {
            down: true,
            ..default()
        },
        Intent {
            down: true,
            ..default()
        },
        Intent {
            confirm: true,
            ..default()
        },
        Intent {
            down: true,
            ..default()
        },
        Intent {
            confirm: true,
            ..default()
        },
    ] {
        step(&mut app, intent);
    }
    let state = app.world().resource::<BattleState>();
    assert!(state.phase == Phase::PlayerActing);
    assert!(!state.skill_menu);
    assert!(state.enemy.hp < state.enemy.max_hp);
    assert!(state.tactics.pressure > 0);
    assert!(app.world().resource::<PlayerStats>().mp < 20);
}

#[test]
fn real_input_rejects_unaffordable_spell_without_consuming_turn() {
    let mut app = simulation(&ENEMIES[1], EncounterKind::Random, 0, 12345);
    app.world_mut().resource_mut::<PlayerStats>().mp = 0;
    app.world_mut().resource_mut::<RunState>().skills = vec![skill::SkillId::SwordGleam];
    for intent in [
        Intent {
            down: true,
            ..default()
        },
        Intent {
            down: true,
            ..default()
        },
        Intent {
            confirm: true,
            ..default()
        },
        Intent {
            confirm: true,
            ..default()
        },
    ] {
        step(&mut app, intent);
    }
    let state = app.world().resource::<BattleState>();
    assert!(state.phase == Phase::Menu);
    assert_eq!(state.enemy.hp, state.enemy.max_hp);
    assert_eq!(state.tactics.pressure, 0);
    assert_eq!(state.enemy_turns, 0);
}

#[test]
fn real_support_spell_neither_damages_nor_builds_pressure() {
    let mut app = simulation(&ENEMIES[1], EncounterKind::Random, 0, 12345);
    app.world_mut().resource_mut::<RunState>().skills = vec![skill::SkillId::MountainAegis];
    app.world_mut()
        .resource_mut::<BattleState>()
        .tactics
        .riposte = true;
    for intent in [
        Intent {
            down: true,
            ..default()
        },
        Intent {
            down: true,
            ..default()
        },
        Intent {
            confirm: true,
            ..default()
        },
        Intent {
            confirm: true,
            ..default()
        },
    ] {
        step(&mut app, intent);
    }
    let state = app.world().resource::<BattleState>();
    assert!(state.phase == Phase::PlayerActing);
    assert_eq!(state.enemy.hp, state.enemy.max_hp);
    assert_eq!(state.tactics.pressure, 0);
    assert!(!state.tactics.riposte);
    assert!(state.player_shield > 0);
}

fn finish_round(app: &mut App) {
    for _ in 0..120 {
        if matches!(
            app.world().resource::<BattleState>().phase,
            Phase::Menu | Phase::Won | Phase::Lost
        ) {
            return;
        }
        step(app, Intent::default());
    }
    panic!("real turn did not finish within four simulated seconds");
}

#[test]
fn real_boss_opening_charge_previews_special_without_attacking() {
    let mut app = simulation(
        &MOON_WRAITH_BOSS,
        EncounterKind::Boss(BossKind::MoonWraith),
        0,
        12345,
    );
    assert!(app.world().resource::<BattleState>().intent == EnemyIntent::Charge);
    let hp = app.world().resource::<PlayerStats>().hp;
    // Ordinary opening attack leaves the boss above its transformation threshold.
    step(
        &mut app,
        Intent {
            confirm: true,
            ..default()
        },
    );
    finish_round(&mut app);
    let state = app.world().resource::<BattleState>();
    assert!(state.phase == Phase::Menu);
    assert!(state.executing_intent == EnemyIntent::Charge);
    assert!(state.intent == EnemyIntent::Special);
    assert!(!state.enemy_attacked);
    assert_eq!(app.world().resource::<PlayerStats>().hp, hp);
}

#[test]
fn guarding_real_shielded_special_earns_single_next_attack_counter() {
    let mut app = simulation(
        &MOON_WRAITH_BOSS,
        EncounterKind::Boss(BossKind::MoonWraith),
        0,
        12345,
    );
    {
        let mut state = app.world_mut().resource_mut::<BattleState>();
        state.intent = EnemyIntent::Special;
        state.player_shield = 100;
    }
    let hp = app.world().resource::<PlayerStats>().hp;
    step(
        &mut app,
        Intent {
            down: true,
            ..default()
        },
    );
    step(
        &mut app,
        Intent {
            confirm: true,
            ..default()
        },
    );
    finish_round(&mut app);
    let state = app.world().resource::<BattleState>();
    assert!(state.phase == Phase::Menu);
    assert!(state.enemy_attacked);
    assert!(state.executing_intent == EnemyIntent::Special);
    assert!(state.tactics.riposte);
    assert_eq!(state.tactics.counters, 1);
    assert_eq!(app.world().resource::<PlayerStats>().hp, hp);
    // Navigate upward from Guard to Attack and spend that exact counter.
    step(
        &mut app,
        Intent {
            up: true,
            ..default()
        },
    );
    step(
        &mut app,
        Intent {
            confirm: true,
            ..default()
        },
    );
    let state = app.world().resource::<BattleState>();
    assert!(!state.tactics.riposte);
    assert!(state.tactics.last_hit.counter);
    assert!(state.tactics.pressure >= 5);
}

#[test]
fn real_attack_break_interrupts_advertised_special_and_exposes_one_followup() {
    let mut app = simulation(
        &MOON_WRAITH_BOSS,
        EncounterKind::Boss(BossKind::MoonWraith),
        0,
        12345,
    );
    {
        let mut state = app.world_mut().resource_mut::<BattleState>();
        state.intent = EnemyIntent::Special;
        state.tactics.pressure = state.tactics.limit - 2;
    }
    let hp = app.world().resource::<PlayerStats>().hp;
    let mp = app.world().resource::<PlayerStats>().mp;
    step(
        &mut app,
        Intent {
            confirm: true,
            ..default()
        },
    );
    assert!(app.world().resource::<BattleState>().tactics.broken);
    finish_round(&mut app);
    let state = app.world().resource::<BattleState>();
    assert!(!state.enemy_attacked);
    assert!(state.tactics.exposed);
    assert_eq!(state.tactics.interrupts, 1);
    assert_eq!(state.tactics.pressure, 0);
    assert_eq!(app.world().resource::<PlayerStats>().hp, hp);
    assert_eq!(app.world().resource::<PlayerStats>().mp, mp);
    step(
        &mut app,
        Intent {
            confirm: true,
            ..default()
        },
    );
    let state = app.world().resource::<BattleState>();
    assert!(state.tactics.last_hit.exposed);
    assert!(!state.tactics.exposed);
    assert_eq!(state.tactics.pressure, 0);
}

#[test]
#[ignore = "exhaustive real-system combat balance sweep; writes COMBAT_SWEEP_OUT"]
fn combat_policy_sweep() {
    let bosses = [
        (&MOUNTAIN_FIEND_BOSS, BossKind::MountainFiend),
        (&MOON_WRAITH_BOSS, BossKind::MoonWraith),
        (&RIVER_DEMON_BOSS, BossKind::RiverDemon),
        (&MIASMA_ROOT_BOSS, BossKind::MiasmaRoot),
        (&MIRROR_MINISTER_BOSS, BossKind::MirrorMinister),
        (&THUNDER_QILIN_BOSS, BossKind::ThunderQilin),
        (&DREAM_ECLIPSE_BOSS, BossKind::DreamEclipse),
    ];
    let normal_tiers = [0, 0, 1, 0, 2, 2, 3, 3, 1];
    let mut scenarios: Vec<(&EnemyDef, EncounterKind, usize)> = ENEMIES
        .iter()
        .enumerate()
        .map(|(index, def)| (def, EncounterKind::Random, normal_tiers[index]))
        .collect();
    scenarios.extend(
        bosses
            .iter()
            .enumerate()
            .map(|(chapter, (def, boss))| (*def, EncounterKind::Boss(*boss), chapter)),
    );
    let mut csv = String::from(
        "enemy,chapter,seed,policy,won,timeout,rounds,hp,mp,breaks,counters,interrupts,guards,spells,potions\n",
    );
    let mut timeouts = 0;
    for (def, kind, chapter) in scenarios {
        for seed_index in 1u64..=32 {
            let seed = 0x9e3779b97f4a7c15u64.wrapping_mul(seed_index);
            for policy in [Policy::Attack, Policy::Offensive, Policy::Tactical] {
                let out = run_simulation(def, kind, chapter, seed, policy);
                timeouts += usize::from(out.timeout);
                csv.push_str(&format!(
                    "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
                    def.name,
                    chapter + 1,
                    seed,
                    policy.name(),
                    out.won,
                    out.timeout,
                    out.rounds,
                    out.hp,
                    out.mp,
                    out.breaks,
                    out.counters,
                    out.interrupts,
                    out.guards,
                    out.spells,
                    out.potions
                ));
            }
        }
        eprintln!("combat sweep finished {}", def.name);
    }
    let path = std::env::var("COMBAT_SWEEP_OUT")
        .unwrap_or_else(|_| "/tmp/love-rpg-combat-sweep.csv".into());
    std::fs::write(&path, csv).expect("write combat sweep CSV");
    eprintln!("1536 real-system combat simulations saved to {path}; timeouts: {timeouts}");
    assert_eq!(
        timeouts, 0,
        "some policies failed to reach a terminal battle state; inspect {path}"
    );
}

#[derive(Clone, Copy, Debug)]
enum NarrativeBuild {
    Heart,
    Resolve,
    Mixed,
}

impl NarrativeBuild {
    fn name(self) -> &'static str {
        match self {
            Self::Heart => "heart",
            Self::Resolve => "resolve",
            Self::Mixed => "mixed",
        }
    }
    fn pick(self, chapter: usize) -> usize {
        match self {
            Self::Heart => 0,
            Self::Resolve => 1,
            Self::Mixed => chapter % 2,
        }
    }
}

fn narrative_simulation(
    def: &EnemyDef,
    boss: BossKind,
    chapter: usize,
    seed: u64,
    build: NarrativeBuild,
) -> App {
    use crate::game::roguelike::event::{RunDialogue, resolve_choice};
    let mut app = simulation(def, EncounterKind::Boss(boss), chapter, seed);
    let mut stats = app.world_mut().remove_resource::<PlayerStats>().unwrap();
    let mut run = app.world_mut().remove_resource::<RunState>().unwrap();
    let mut rng = app.world_mut().remove_resource::<Rng>().unwrap();
    // Fixed starter plus one fixed loot skill. No random relics or hexes.
    run.skills = vec![skill::SkillId::SwordGleam, skill::SkillId::EarthRift];
    for story_chapter in 0..=chapter {
        run.chapter = story_chapter;
        let mut dialogue = RunDialogue::default();
        dialogue.open_story(story_chapter, 0);
        resolve_choice(
            &mut dialogue,
            build.pick(story_chapter),
            &mut stats,
            &mut run,
            &mut rng,
        );
        assert!(
            dialogue.choice_error.is_none(),
            "story choice must actually resolve"
        );
    }
    run.chapter = chapter;
    // Compare rested builds at the same boss, not a simulated entire overworld route.
    stats.hp = stats.max_hp;
    stats.mp = stats.max_mp;
    let mut state = app.world_mut().resource_mut::<BattleState>();
    // The fixture skips rendering spawn; apply its real run-start effect methods.
    state.enemy.hp = (state.enemy.hp - run.opening_strike()).max(1);
    state.enemy.atk = (state.enemy.atk - run.enemy_atk_debuff()).max(1);
    drop(state);
    app.insert_resource(stats)
        .insert_resource(run)
        .insert_resource(rng);
    app
}

#[test]
fn narrative_choices_unlock_distinct_real_builds_without_repeat_rewards() {
    use crate::game::roguelike::{
        Relic,
        event::{RunDialogue, resolve_choice},
    };
    let mut heart = narrative_simulation(
        &MIRROR_MINISTER_BOSS,
        BossKind::MirrorMinister,
        4,
        12345,
        NarrativeBuild::Heart,
    );
    let resolve = narrative_simulation(
        &MIRROR_MINISTER_BOSS,
        BossKind::MirrorMinister,
        4,
        12345,
        NarrativeBuild::Resolve,
    );
    let heart_run = heart.world().resource::<RunState>();
    let resolve_run = resolve.world().resource::<RunState>();
    assert!(heart_run.skills.contains(&skill::SkillId::MoonHalo));
    assert!(resolve_run.skills.contains(&skill::SkillId::FrostLance));
    assert!(heart_run.relics.contains(&Relic::YinYangMirror));
    assert!(resolve_run.relics.contains(&Relic::SwordSutra));
    assert_ne!(heart_run.skills, resolve_run.skills);
    let skills = heart_run.skills.clone();
    let relics = heart_run.relics.clone();
    let snapshot = {
        let stats = heart.world().resource::<PlayerStats>();
        (
            stats.hp,
            stats.max_hp,
            stats.mp,
            stats.max_mp,
            stats.atk,
            stats.def,
            stats.gold,
            stats.potions,
            heart_run.daoxin,
            heart_run.qingyuan,
        )
    };
    let mut stats = heart.world_mut().remove_resource::<PlayerStats>().unwrap();
    let mut run = heart.world_mut().remove_resource::<RunState>().unwrap();
    let mut rng = heart.world_mut().remove_resource::<Rng>().unwrap();
    for chapter in 0..=4 {
        run.chapter = chapter;
        let mut dialogue = RunDialogue::default();
        dialogue.open_story(chapter, 0);
        resolve_choice(&mut dialogue, 0, &mut stats, &mut run, &mut rng);
    }
    assert_eq!(run.skills, skills);
    assert_eq!(run.relics, relics);
    assert_eq!(
        (
            stats.hp,
            stats.max_hp,
            stats.mp,
            stats.max_mp,
            stats.atk,
            stats.def,
            stats.gold,
            stats.potions,
            run.daoxin,
            run.qingyuan
        ),
        snapshot
    );
    assert!(
        run.skills
            .iter()
            .all(|id| skill_cost(*id, &run) <= stats.max_mp)
    );
}

#[test]
#[ignore = "real-system boss sweep of actual narrative rewards; writes NARRATIVE_COMBAT_OUT"]
fn narrative_reward_combat_sweep() {
    let bosses = [
        (&MOUNTAIN_FIEND_BOSS, BossKind::MountainFiend),
        (&MOON_WRAITH_BOSS, BossKind::MoonWraith),
        (&RIVER_DEMON_BOSS, BossKind::RiverDemon),
        (&MIASMA_ROOT_BOSS, BossKind::MiasmaRoot),
        (&MIRROR_MINISTER_BOSS, BossKind::MirrorMinister),
        (&THUNDER_QILIN_BOSS, BossKind::ThunderQilin),
        (&DREAM_ECLIPSE_BOSS, BossKind::DreamEclipse),
    ];
    let mut csv = String::from(
        "boss,chapter,seed,build,won,timeout,rounds,hp,mp,breaks,counters,interrupts,guards,spells,potions\n",
    );
    let mut summaries = [(0u32, 0u32, 0i64, 0u64, 0u32); 3];
    for (chapter, (def, boss)) in bosses.iter().enumerate() {
        for seed_index in 1u64..=32 {
            let seed = 0x9e3779b97f4a7c15u64.wrapping_mul(seed_index);
            for (index, build) in [
                NarrativeBuild::Heart,
                NarrativeBuild::Resolve,
                NarrativeBuild::Mixed,
            ]
            .into_iter()
            .enumerate()
            {
                let out = run_app(
                    narrative_simulation(def, *boss, chapter, seed, build),
                    Policy::Tactical,
                );
                let summary = &mut summaries[index];
                summary.0 += 1;
                summary.1 += u32::from(out.won);
                summary.2 += i64::from(out.hp);
                summary.3 += u64::from(out.rounds);
                summary.4 += u32::from(out.timeout);
                csv.push_str(&format!(
                    "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
                    def.name,
                    chapter + 1,
                    seed,
                    build.name(),
                    out.won,
                    out.timeout,
                    out.rounds,
                    out.hp,
                    out.mp,
                    out.breaks,
                    out.counters,
                    out.interrupts,
                    out.guards,
                    out.spells,
                    out.potions
                ));
            }
        }
        eprintln!("narrative combat sweep finished {}", def.name);
    }
    let path = std::env::var("NARRATIVE_COMBAT_OUT")
        .unwrap_or_else(|_| "/tmp/love-rpg-narrative-combat.csv".into());
    std::fs::write(&path, csv).expect("write narrative combat CSV");
    for (index, build) in [
        NarrativeBuild::Heart,
        NarrativeBuild::Resolve,
        NarrativeBuild::Mixed,
    ]
    .into_iter()
    .enumerate()
    {
        let (count, wins, hp, rounds, timeouts) = summaries[index];
        eprintln!(
            "{}: {wins}/{count} wins ({:.1}%), mean terminal HP {:.1}, mean rounds {:.1}, {timeouts} timeouts",
            build.name(),
            wins as f64 * 100.0 / count as f64,
            hp as f64 / count as f64,
            rounds as f64 / count as f64
        );
        assert_eq!(timeouts, 0, "inspect {path}");
    }
}

#[derive(Clone, Copy, Debug)]
enum SideRoute {
    FoxSoil,
    FoxTiger,
    QinFunded,
    QinListening,
}

impl SideRoute {
    fn name(self) -> &'static str {
        match self {
            Self::FoxSoil => "fox-soil",
            Self::FoxTiger => "fox-tiger",
            Self::QinFunded => "qin-funded",
            Self::QinListening => "qin-listening",
        }
    }
}

fn side_reward_simulation(
    def: &EnemyDef,
    boss: BossKind,
    chapter: usize,
    seed: u64,
    route: SideRoute,
) -> App {
    use crate::game::roguelike::{
        content,
        event::{RunDialogue, resolve_choice},
    };
    let mut app = simulation(def, EncounterKind::Boss(boss), chapter, seed);
    let mut stats = app.world_mut().remove_resource::<PlayerStats>().unwrap();
    let mut run = app.world_mut().remove_resource::<RunState>().unwrap();
    let mut rng = app.world_mut().remove_resource::<Rng>().unwrap();
    run.skills = vec![skill::SkillId::SwordGleam, skill::SkillId::EarthRift];
    stats.gold = 1000;
    let events: Vec<(usize, usize, usize)> = match route {
        SideRoute::FoxSoil | SideRoute::FoxTiger => vec![
            (0, content::EV_FOX1, 0),
            (2, content::EV_FOX2_WARM, 0),
            (
                5,
                content::EV_FOX3_WARM,
                usize::from(matches!(route, SideRoute::FoxTiger)),
            ),
        ],
        SideRoute::QinFunded | SideRoute::QinListening => vec![
            (
                2,
                content::EV_QIN1,
                usize::from(matches!(route, SideRoute::QinListening)),
            ),
            (6, content::EV_QIN2, 0),
        ],
    };
    for (event_chapter, event, pick) in events {
        run.chapter = event_chapter;
        let mut dialogue = RunDialogue::default();
        dialogue.open_event(event);
        resolve_choice(&mut dialogue, pick, &mut stats, &mut run, &mut rng);
        assert!(
            dialogue.choice_error.is_none(),
            "actual side-story reward choice must resolve"
        );
    }
    match route {
        SideRoute::FoxSoil => {
            assert_eq!(run.fox_stage, 3);
            assert!(
                run.relics
                    .contains(&crate::game::roguelike::Relic::BreathSoil)
            );
        }
        SideRoute::FoxTiger => {
            assert_eq!(run.fox_stage, 3);
            assert!(
                run.relics
                    .contains(&crate::game::roguelike::Relic::TigerTalisman)
            );
        }
        SideRoute::QinFunded => {
            assert_eq!(run.qin_stage, 2);
            assert!(run.skills.contains(&skill::SkillId::SoulLantern));
        }
        SideRoute::QinListening => {
            assert_eq!(run.qin_stage, 2);
            assert!(run.skills.contains(&skill::SkillId::MoonHalo));
        }
    }
    run.chapter = chapter;
    run.active_camp_tactic = None;
    // Controlled wounded encounter after earning the chain; chapter growth was
    // already applied once by simulation. Chain costs and max-MP gains remain.
    stats.hp = stats.max_hp * 60 / 100;
    stats.mp = stats.max_mp * 60 / 100;
    let healed = run.battle_start_heal() + run.bond_shield();
    stats.hp = (stats.hp + healed).min(stats.max_hp);
    stats.mp = (stats.mp - run.hex_battle_start_mp_loss()).max(0);
    {
        let mut state = app.world_mut().resource_mut::<BattleState>();
        state.enemy.hp = (state.enemy.hp - run.opening_strike()).max(1);
        state.enemy.atk = (state.enemy.atk - run.enemy_atk_debuff()).max(1);
    }
    app.insert_resource(stats)
        .insert_resource(run)
        .insert_resource(rng);
    app
}

#[test]
#[ignore = "real-system wounded boss sweep of actual fox/qin chain rewards; writes NARRATIVE_SIDE_COMBAT_OUT"]
fn side_story_reward_combat_sweep() {
    let bosses = [
        (5, &THUNDER_QILIN_BOSS, BossKind::ThunderQilin),
        (6, &DREAM_ECLIPSE_BOSS, BossKind::DreamEclipse),
    ];
    let routes = [
        SideRoute::FoxSoil,
        SideRoute::FoxTiger,
        SideRoute::QinFunded,
        SideRoute::QinListening,
    ];
    let mut csv = String::from(
        "boss,chapter,seed,route,won,timeout,rounds,hp,mp,breaks,counters,interrupts,guards,spells,potions\n",
    );
    let mut summaries = [(0u32, 0u32, 0i64, 0u64, 0u32); 4];
    for (chapter, def, boss) in bosses {
        for seed_index in 1u64..=32 {
            let seed = 0x9e3779b97f4a7c15u64.wrapping_mul(seed_index);
            for (index, route) in routes.into_iter().enumerate() {
                let out = run_app(
                    side_reward_simulation(def, boss, chapter, seed, route),
                    Policy::Tactical,
                );
                let summary = &mut summaries[index];
                summary.0 += 1;
                summary.1 += u32::from(out.won);
                summary.2 += i64::from(out.hp);
                summary.3 += u64::from(out.rounds);
                summary.4 += u32::from(out.timeout);
                csv.push_str(&format!(
                    "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
                    def.name,
                    chapter + 1,
                    seed,
                    route.name(),
                    out.won,
                    out.timeout,
                    out.rounds,
                    out.hp,
                    out.mp,
                    out.breaks,
                    out.counters,
                    out.interrupts,
                    out.guards,
                    out.spells,
                    out.potions
                ));
            }
        }
        eprintln!("side-story combat sweep finished {}", def.name);
    }
    let path = std::env::var("NARRATIVE_SIDE_COMBAT_OUT")
        .unwrap_or_else(|_| "/tmp/love-rpg-side-narrative-combat.csv".into());
    std::fs::write(&path, csv).expect("write side-story combat CSV");
    for (index, route) in routes.into_iter().enumerate() {
        let (count, wins, hp, rounds, timeouts) = summaries[index];
        eprintln!(
            "{}: {wins}/{count} wins ({:.1}%), mean terminal HP {:.1}, mean rounds {:.1}, {timeouts} timeouts",
            route.name(),
            wins as f64 * 100.0 / count as f64,
            hp as f64 / count as f64,
            rounds as f64 / count as f64
        );
        assert_eq!(timeouts, 0, "inspect {path}");
    }
}

#[test]
fn fallback_spell_view_matches_moon_bite_rejection_and_actual_mana_charge() {
    use crate::game::roguelike::hex::HexMark;
    let mut app = simulation(
        &MOON_WRAITH_BOSS,
        EncounterKind::Boss(BossKind::MoonWraith),
        1,
        12345,
    );
    {
        let mut run = app.world_mut().resource_mut::<RunState>();
        run.skills.clear();
        run.hexes.push(HexMark::MoonBite);
    }
    let cost = fallback_spell_cost(
        app.world().resource::<QuestLog>(),
        Some(app.world().resource::<RunState>()),
    );
    app.world_mut().resource_mut::<PlayerStats>().mp = cost - 1;
    // Navigate through the real main menu and confirm the fallback spell.
    step(
        &mut app,
        Intent {
            down: true,
            ..default()
        },
    );
    step(
        &mut app,
        Intent {
            down: true,
            ..default()
        },
    );
    let hp = app.world().resource::<BattleState>().enemy.hp;
    assert_eq!(
        app.world().resource::<BattleAutomationView>().spell_cost,
        cost
    );
    step(
        &mut app,
        Intent {
            confirm: true,
            ..default()
        },
    );
    assert!(app.world().resource::<BattleState>().phase == Phase::Menu);
    assert_eq!(app.world().resource::<PlayerStats>().mp, cost - 1);
    assert_eq!(app.world().resource::<BattleState>().enemy.hp, hp);
    assert_eq!(app.world().resource::<BattleState>().enemy_turns, 0);
    assert!(app.world().resource::<BattleAutomationView>().menu_open);
    app.world_mut().resource_mut::<PlayerStats>().mp = cost;
    step(
        &mut app,
        Intent {
            confirm: true,
            ..default()
        },
    );
    assert!(app.world().resource::<BattleState>().phase == Phase::PlayerActing);
    assert_eq!(app.world().resource::<PlayerStats>().mp, 0);
    assert!(app.world().resource::<BattleState>().enemy.hp < hp);
    assert_eq!(
        app.world().resource::<BattleAutomationView>().spell_cost,
        cost
    );
    assert!(!app.world().resource::<BattleAutomationView>().menu_open);
}

fn cast_modifier_fixture(
    id: skill::SkillId,
    relics: Vec<crate::game::roguelike::Relic>,
    rank: FightRank,
    camp: Option<RunCampTactic>,
) -> (i32, i32, i32, i32, String) {
    let mut app = simulation(&ENEMIES[1], EncounterKind::Random, 0, 12345);
    {
        let mut run = app.world_mut().resource_mut::<RunState>();
        run.skills = vec![id];
        run.relics = relics;
        run.current_fight = Some(rank);
    }
    {
        let mut state = app.world_mut().resource_mut::<BattleState>();
        state.intent = EnemyIntent::Strike;
        state.enemy.hp = 1000;
        state.enemy.max_hp = 1000;
        state.run_camp_tactic = camp;
    }
    app.world_mut().resource_mut::<PlayerStats>().mp = 100;
    for intent in [
        Intent {
            down: true,
            ..default()
        },
        Intent {
            down: true,
            ..default()
        },
        Intent {
            confirm: true,
            ..default()
        },
        Intent {
            confirm: true,
            ..default()
        },
    ] {
        step(&mut app, intent);
    }
    let state = app.world().resource::<BattleState>();
    assert!(state.phase == Phase::PlayerActing);
    (
        1000 - state.enemy.hp,
        state.player_shield,
        100 - app.world().resource::<PlayerStats>().mp,
        app.world().resource::<BattleAutomationView>().spell_cost,
        state.message.clone(),
    )
}

#[test]
fn real_sutra_adds_nine_once_per_single_or_flurry_cast_and_not_to_ward() {
    use crate::game::roguelike::Relic;
    for id in [skill::SkillId::SwordGleam, skill::SkillId::BladeStorm] {
        let plain = cast_modifier_fixture(id, vec![], FightRank::Normal, None);
        let sutra = cast_modifier_fixture(id, vec![Relic::SwordSutra], FightRank::Normal, None);
        assert_eq!(sutra.0 - plain.0, 9, "{id:?}");
        assert_eq!(sutra.2, plain.2);
    }
    let plain = cast_modifier_fixture(skill::SkillId::MoonHalo, vec![], FightRank::Normal, None);
    let sutra = cast_modifier_fixture(
        skill::SkillId::MoonHalo,
        vec![Relic::SwordSutra],
        FightRank::Normal,
        Some(RunCampTactic::SpiritFocus),
    );
    assert_eq!(sutra.0, 0);
    assert_eq!(sutra.1, plain.1);
    assert!(!sutra.4.contains("法宝助术"));
    assert!(!sutra.4.contains("伤害 +"));
}

#[test]
fn real_pendant_skill_charge_and_view_both_reduce_by_two() {
    use crate::game::roguelike::Relic;
    let plain = cast_modifier_fixture(skill::SkillId::EarthRift, vec![], FightRank::Normal, None);
    let pendant = cast_modifier_fixture(
        skill::SkillId::EarthRift,
        vec![Relic::SpiritPendant],
        FightRank::Normal,
        None,
    );
    assert_eq!(plain.2 - pendant.2, 2);
    assert_eq!(pendant.2, pendant.3);
    assert_eq!(plain.2, plain.3);
    assert_eq!(plain.0, pendant.0);
}

#[test]
fn real_skill_camp_bonus_and_hunter_multiplier_affect_total_damage() {
    use crate::game::roguelike::Relic;
    let plain = cast_modifier_fixture(skill::SkillId::BladeStorm, vec![], FightRank::Normal, None);
    let camp = cast_modifier_fixture(
        skill::SkillId::BladeStorm,
        vec![],
        FightRank::Normal,
        Some(RunCampTactic::SpiritFocus),
    );
    assert_eq!(
        camp.0 - plain.0,
        run_camp_damage_bonus(Some(RunCampTactic::SpiritFocus), PlayerAction::Spell)
    );
    assert!(camp.4.contains("营策"));
    for rank in [FightRank::Elite, FightRank::Boss] {
        let hunter = cast_modifier_fixture(
            skill::SkillId::BladeStorm,
            vec![Relic::KunlunMirror],
            rank,
            None,
        );
        let mut rng = Rng(12345);
        let mut run = RunState::new(&mut rng);
        run.relics = vec![Relic::KunlunMirror];
        assert_eq!(
            hunter.0,
            (plain.0 as f32 * run.hunter_multiplier(rank)).round() as i32
        );
    }
    let normal_hunter = cast_modifier_fixture(
        skill::SkillId::BladeStorm,
        vec![Relic::KunlunMirror],
        FightRank::Normal,
        None,
    );
    assert_eq!(normal_hunter.0, plain.0);
}

#[test]
fn real_opening_teaches_charge_guard_and_counter_once_without_forcing_inputs() {
    let mut app = simulation(&ENEMIES[0], EncounterKind::Random, 0, 98765);
    app.world_mut().resource_mut::<BattleState>().intent = EnemyIntent::Charge;
    step(&mut app, Intent::default());
    assert_eq!(
        app.world().resource::<RunState>().battle_lessons_seen,
        vec![BattleLesson::Charge]
    );
    step(
        &mut app,
        Intent {
            confirm: true,
            ..default()
        },
    );
    finish_round(&mut app);
    assert!(app.world().resource::<BattleState>().intent == EnemyIntent::Heavy);
    assert!(
        app.world()
            .resource::<RunState>()
            .battle_lessons_seen
            .contains(&BattleLesson::Heavy)
    );
    step(
        &mut app,
        Intent {
            down: true,
            ..default()
        },
    );
    step(
        &mut app,
        Intent {
            confirm: true,
            ..default()
        },
    );
    finish_round(&mut app);
    assert!(
        app.world()
            .resource::<RunState>()
            .battle_lessons_seen
            .contains(&BattleLesson::Counter)
    );
    let lessons = app
        .world()
        .resource::<RunState>()
        .battle_lessons_seen
        .clone();
    step(&mut app, Intent::default());
    assert_eq!(
        app.world().resource::<RunState>().battle_lessons_seen,
        lessons
    );
    step(
        &mut app,
        Intent {
            up: true,
            ..default()
        },
    );
    step(
        &mut app,
        Intent {
            confirm: true,
            ..default()
        },
    );
    assert!(
        app.world()
            .resource::<BattleState>()
            .tactics
            .last_hit
            .counter
    );
}

fn finish_transition(app: &mut App, target: AppState) {
    for _ in 0..120 {
        step(app, Intent::default());
        if matches!(app.world().resource::<NextState<AppState>>(), NextState::Pending(state) if *state == target)
        {
            return;
        }
    }
    panic!("battle failed to request its terminal transition");
}

#[test]
fn real_escape_success_returns_to_scene_and_failure_spends_enemy_turn() {
    let mut saw_success = false;
    let mut saw_failure = false;
    for seed in 1..=64 {
        let mut app = simulation(
            &ENEMIES[0],
            EncounterKind::Random,
            0,
            0x9e3779b97f4a7c15u64.wrapping_mul(seed),
        );
        app.world_mut().resource_mut::<BattleState>().intent = EnemyIntent::Strike;
        for _ in 0..5 {
            step(
                &mut app,
                Intent {
                    down: true,
                    ..default()
                },
            );
        }
        let hp = app.world().resource::<PlayerStats>().hp;
        step(
            &mut app,
            Intent {
                confirm: true,
                ..default()
            },
        );
        if app.world().resource::<BattleState>().phase == Phase::Fled {
            finish_transition(&mut app, AppState::RunScene);
            assert!(app.world().get_resource::<PendingEncounter>().is_none());
            assert!(app.world().get_resource::<RunBattleMods>().is_none());
            assert!(app.world().resource::<RunState>().current_fight.is_none());
            saw_success = true;
        } else {
            assert!(app.world().resource::<BattleState>().phase == Phase::PlayerActing);
            finish_round(&mut app);
            assert_eq!(app.world().resource::<BattleState>().enemy_turns, 1);
            assert!(app.world().resource::<PlayerStats>().hp < hp);
            saw_failure = true;
        }
        if saw_success && saw_failure {
            break;
        }
    }
    assert!(saw_success && saw_failure);
}

#[test]
fn real_death_revives_once_then_requests_ending_and_cleans_encounter() {
    use crate::game::roguelike::Relic;
    let mut app = simulation(&ENEMIES[0], EncounterKind::Random, 0, 12345);
    app.world_mut().resource_mut::<RunState>().relics = vec![Relic::SandalCharm];
    app.world_mut().resource_mut::<PlayerStats>().hp = 1;
    {
        let mut state = app.world_mut().resource_mut::<BattleState>();
        state.enemy.hp = 1000;
        state.enemy.max_hp = 1000;
        state.enemy.atk = 1000;
        state.intent = EnemyIntent::Strike;
    }
    step(
        &mut app,
        Intent {
            confirm: true,
            ..default()
        },
    );
    finish_round(&mut app);
    assert!(app.world().resource::<BattleState>().phase == Phase::Menu);
    assert!(app.world().resource::<RunState>().revive_used);
    assert_eq!(
        app.world().resource::<PlayerStats>().hp,
        app.world().resource::<PlayerStats>().max_hp / 2
    );
    app.world_mut().resource_mut::<BattleState>().intent = EnemyIntent::Strike;
    step(
        &mut app,
        Intent {
            confirm: true,
            ..default()
        },
    );
    finish_round(&mut app);
    assert!(app.world().resource::<BattleState>().phase == Phase::Lost);
    finish_transition(&mut app, AppState::Ending);
    assert!(matches!(
        app.world().resource::<RunState>().outcome,
        Some(RunOutcome::Defeat)
    ));
    assert!(app.world().get_resource::<PendingEncounter>().is_none());
    assert!(app.world().get_resource::<RunBattleMods>().is_none());
    assert_eq!(app.world().resource::<PlayerStats>().hp, 0);
}

#[test]
fn normal_enemy_selection_does_not_repeat_and_boss_selection_is_stable() {
    let mut rng = Rng(12345);
    let mut previous = None;
    for _ in 0..64 {
        let def = choose_enemy_without_repeat(
            EncounterZone::Village,
            EncounterKind::Random,
            previous,
            &mut rng,
        );
        assert_ne!(Some(def.image), previous);
        previous = Some(def.image);
    }
    assert_eq!(
        choose_enemy_without_repeat(
            EncounterZone::Village,
            EncounterKind::Boss(BossKind::MoonWraith),
            Some(MOON_WRAITH_BOSS.image),
            &mut rng
        )
        .image,
        MOON_WRAITH_BOSS.image
    );
}

#[test]
fn mastery_preserves_first_three_chapters_and_grows_damage_and_support_later() {
    let mut rng = Rng(12345);
    let mut run = RunState::new(&mut rng);
    for chapter in 0..3 {
        run.chapter = chapter;
        assert_eq!(skill_mastery_bonus(&run), 0);
    }
    run.chapter = 3;
    assert_eq!(skill_mastery_bonus(&run), 2);
    run.chapter = 6;
    assert_eq!(skill_mastery_bonus(&run), 12);
}

#[test]
fn actual_boss_healing_depletes_one_shared_pool_and_stops_regeneration() {
    let mut app = simulation(
        &MIASMA_ROOT_BOSS,
        EncounterKind::Boss(BossKind::MiasmaRoot),
        3,
        12345,
    );
    let budget = app.world().resource::<BattleState>().boss_heal_remaining;
    let mut healed = 0;
    for _ in 0..5 {
        {
            let mut state = app.world_mut().resource_mut::<BattleState>();
            state.boss_phase2 = true;
            state.intent = EnemyIntent::Gather;
            state.enemy.hp = state.enemy.max_hp / 2;
        }
        while app.world().resource::<BattleState>().menu_index != 1 {
            step(
                &mut app,
                Intent {
                    down: true,
                    ..default()
                },
            );
        }
        let before = app.world().resource::<BattleState>().enemy.hp;
        step(
            &mut app,
            Intent {
                confirm: true,
                ..default()
            },
        );
        finish_round(&mut app);
        healed += app.world().resource::<BattleState>().enemy.hp - before;
    }
    assert_eq!(healed, budget);
    assert_eq!(app.world().resource::<BattleState>().boss_heal_remaining, 0);
    assert!(app.world().resource::<BattleState>().intent != EnemyIntent::Gather);
}

#[test]
fn fourth_chapter_mastery_changes_real_damage_and_shield_by_two() {
    for id in [skill::SkillId::SwordGleam, skill::SkillId::MoonHalo] {
        let mut results = Vec::new();
        for chapter in [2, 3] {
            let mut app = simulation(&ENEMIES[1], EncounterKind::Random, chapter, 12345);
            app.world_mut().resource_mut::<RunState>().skills = vec![id];
            {
                let mut state = app.world_mut().resource_mut::<BattleState>();
                state.intent = EnemyIntent::Strike;
                state.enemy.hp = 1000;
                state.enemy.max_hp = 1000;
            }
            for intent in [
                Intent {
                    down: true,
                    ..default()
                },
                Intent {
                    down: true,
                    ..default()
                },
                Intent {
                    confirm: true,
                    ..default()
                },
                Intent {
                    confirm: true,
                    ..default()
                },
            ] {
                step(&mut app, intent);
            }
            let state = app.world().resource::<BattleState>();
            assert!(state.phase == Phase::PlayerActing);
            results.push((1000 - state.enemy.hp, state.player_shield));
        }
        if id == skill::SkillId::SwordGleam {
            assert_eq!(results[1].0 - results[0].0, 2);
        } else {
            assert_eq!(results[1].1 - results[0].1, 2);
        }
    }
}

#[test]
fn actual_enemy_damage_result_remains_visible_after_both_party_supports() {
    let mut app = simulation(&ENEMIES[0], EncounterKind::Random, 5, 12345);
    {
        let mut stats = app.world_mut().resource_mut::<PlayerStats>();
        stats.hp = stats.max_hp / 2;
        stats.mp = stats.max_mp / 2;
    }
    {
        let mut state = app.world_mut().resource_mut::<BattleState>();
        state.enemy.hp = 1000;
        state.enemy.max_hp = 1000;
        state.enemy.atk = 45;
        state.intent = EnemyIntent::Strike;
    }
    let starting_hp = app.world().resource::<PlayerStats>().hp;
    let starting_mp = app.world().resource::<PlayerStats>().mp;
    // Deliberately attack rather than defend; observe the real incoming result.
    step(
        &mut app,
        Intent {
            confirm: true,
            ..default()
        },
    );
    for _ in 0..90 {
        if app.world().resource::<BattleState>().phase == Phase::EnemyActing {
            break;
        }
        step(&mut app, Intent::default());
    }
    let state = app.world().resource::<BattleState>();
    assert!(state.phase == Phase::EnemyActing);
    assert!(state.message.contains("造成"));
    let attack_line = state.message.lines().next().unwrap().to_string();
    let damaged_hp = app.world().resource::<PlayerStats>().hp;
    assert!(damaged_hp < starting_hp);
    finish_round(&mut app);
    let state = app.world().resource::<BattleState>();
    assert!(state.phase == Phase::Menu);
    assert!(state.message.contains(&attack_line));
    assert!(state.message.contains("赵灵儿") && state.message.contains("恢复"));
    assert!(state.message.contains("南瑶") && state.message.contains("回稳"));
    assert!(app.world().resource::<PlayerStats>().hp > damaged_hp);
    assert!(app.world().resource::<PlayerStats>().mp > starting_mp);
    let displayed = ui::latest_message(&state.message);
    assert!(displayed.contains(&attack_line), "{displayed}");
    assert!(displayed.contains("赵灵儿"), "{displayed}");
}

#[test]
fn actual_full_mana_guard_reports_zero_with_or_without_camp_bonus() {
    for camp in [None, Some(RunCampTactic::SpiritFocus)] {
        let mut app = simulation(&ENEMIES[0], EncounterKind::Random, 0, 12345);
        app.world_mut()
            .resource_mut::<BattleState>()
            .run_camp_tactic = camp;
        let max_mp = app.world().resource::<PlayerStats>().max_mp;
        assert_eq!(app.world().resource::<PlayerStats>().mp, max_mp);
        step(
            &mut app,
            Intent {
                down: true,
                ..default()
            },
        );
        step(
            &mut app,
            Intent {
                confirm: true,
                ..default()
            },
        );
        let state = app.world().resource::<BattleState>();
        assert!(state.phase == Phase::PlayerActing);
        assert_eq!(app.world().resource::<PlayerStats>().mp, max_mp);
        assert!(state.message.contains("恢复 0 点灵力"), "{}", state.message);
        if camp.is_some() {
            let camp_line = state
                .message
                .lines()
                .find(|line| line.contains("营策"))
                .unwrap();
            assert!(camp_line.contains("实际恢复 0"), "{camp_line}");
            assert!(!camp_line.contains("多回稳"), "{camp_line}");
        }
        assert!(ui::latest_message(&state.message).contains("恢复 0 点灵力"));
    }
}

//! Imperfect players using public HUD information and real input/turn/reward systems.
//! This is a controlled encounter chain, not a measurement of overworld play time.
use super::*;
use crate::game::roguelike::{
    CHAPTERS,
    screens::{RewardChoices, RewardOption, reward_input},
};
use bevy::ecs::schedule::ScheduleLabel;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
struct FixedLoot;

#[derive(Clone, Copy, Debug)]
enum Player {
    HpOnly,
    Notice70,
    Fumble20,
}
impl Player {
    fn name(self) -> &'static str {
        match self {
            Self::HpOnly => "hp-only",
            Self::Notice70 => "notice-heavy-70pct",
            Self::Fumble20 => "counter-fumble-20pct",
        }
    }
}

fn ordinary_choice(app: &App) -> (usize, Option<usize>) {
    let stats = app.world().resource::<PlayerStats>();
    let run = app.world().resource::<RunState>();
    if stats.hp * 3 < stats.max_hp && stats.potions > 0 {
        return (4, None);
    }
    if let Some(index) = run.skills.iter().position(|id| {
        stats.mp >= skill_cost(*id, run)
            && !matches!(skill::skill(*id).kind, SkillKind::Ward | SkillKind::Mend)
    }) {
        return (2, Some(index));
    }
    // Basic players recover mana based on their own resource bar, not enemy pressure.
    if stats.mp < 3 {
        return (1, None);
    }
    (0, None)
}

fn player_choice(app: &App, player: Player, decisions: &mut Rng) -> (usize, Option<usize>) {
    let view = app.world().resource::<BattleAutomationView>();
    if matches!(player, Player::Fumble20) && view.riposte_ready {
        if decisions.chance(0.2) {
            return if app.world().resource::<PlayerStats>().potions > 0 {
                (4, None)
            } else {
                (1, None)
            };
        }
        return (0, None);
    }
    if view.enemy_heavy_intent
        && match player {
            Player::HpOnly => false,
            Player::Notice70 => decisions.chance(0.7),
            Player::Fumble20 => true,
        }
    {
        return (1, None);
    }
    ordinary_choice(app)
}

#[derive(Default)]
struct Metrics {
    rounds: u32,
    waste: u32,
    heavy: u32,
    guarded: u32,
    potions: u32,
    operations: u32,
    frames: u32,
    repeats: u32,
    sword_damage: i32,
    skill_damage: i32,
    shield_absorbed: i32,
}

fn imperfect_battle(app: &mut App, player: Player, decisions: &mut Rng) -> (bool, bool, Metrics) {
    let mut metrics = Metrics::default();
    let mut planned = None;
    step(app, Intent::default());
    for _ in 0..4000 {
        let state = app.world().resource::<BattleState>();
        if matches!(state.phase, Phase::Won | Phase::Lost | Phase::Fled) {
            return (state.phase == Phase::Won, false, metrics);
        }
        let menu = state.phase == Phase::Menu;
        if menu && planned.is_none() {
            planned = Some(player_choice(app, player, decisions));
        }
        let mut intent = Intent::default();
        let mut cast = false;
        if menu {
            let (main, skill) = planned.unwrap();
            if state.skill_menu {
                if let Some(index) = skill {
                    if state.skill_cursor == index {
                        intent.confirm = true;
                        cast = true;
                    } else {
                        intent.down = true;
                    }
                } else {
                    intent.cancel = true;
                }
            } else if state.menu_index == main {
                intent.confirm = true;
                cast = main != 2;
            } else {
                intent.down = true;
            }
        }
        let enemy_hp = state.enemy.hp;
        let shield = state.player_shield;
        let potions = app.world().resource::<PlayerStats>().potions;
        let full_hp = app.world().resource::<PlayerStats>().hp
            == app.world().resource::<PlayerStats>().max_hp;
        let heavy = app
            .world()
            .resource::<BattleAutomationView>()
            .enemy_heavy_intent;
        let chance = app.world().resource::<BattleAutomationView>().riposte_ready
            || app.world().resource::<BattleAutomationView>().exposed;
        metrics.operations +=
            u32::from(intent.confirm || intent.down || intent.up || intent.cancel);
        step(app, intent);
        metrics.frames += 1;
        let state = app.world().resource::<BattleState>();
        metrics.shield_absorbed += (shield - state.player_shield).max(0);
        if cast && state.phase == Phase::PlayerActing {
            let (action, _) = planned.unwrap();
            metrics.rounds += 1;
            metrics.heavy += u32::from(heavy);
            metrics.guarded += u32::from(heavy && action == 1);
            metrics.potions += potions - app.world().resource::<PlayerStats>().potions;
            let damage = (enemy_hp - state.enemy.hp).max(0);
            if action == 0 {
                metrics.sword_damage += damage;
            }
            if action == 2 {
                metrics.skill_damage += damage;
            }
            metrics.waste +=
                u32::from((chance && matches!(action, 1 | 4)) || (action == 4 && full_hp));
            planned = None;
        } else if cast && state.phase == Phase::Menu {
            metrics.waste += 1;
            planned = None;
        }
    }
    (false, true, metrics)
}

fn prepare_encounter(app: &mut App, def: &EnemyDef, kind: EncounterKind, teaching: bool) {
    let mut rng = app.world_mut().remove_resource::<Rng>().unwrap();
    let mut run = app.world_mut().remove_resource::<RunState>().unwrap();
    let rank = if matches!(kind, EncounterKind::Boss(_)) {
        FightRank::Boss
    } else {
        FightRank::Normal
    };
    run.current_fight = Some(rank);
    let mods = battle_mods_for(&run, rank);
    let mut state = tests::status_test_state(kind);
    state.enemy = scaled_enemy(def, Some(&mods), &mut rng);
    state.boss_heal_remaining = if matches!(kind, EncounterKind::Boss(_)) {
        state.enemy.max_hp / 4
    } else {
        0
    };
    state.phase = Phase::Menu;
    state.player_action = None;
    state.intent = if teaching {
        EnemyIntent::Charge
    } else {
        initial_intent(def, kind)
    };
    state.tactics = Tactics::new(enemy_style(def), rank == FightRank::Boss);
    // Apply real start effects of route-earned relics; this fixture skips renderer spawn.
    state.enemy.hp = (state.enemy.hp - run.opening_strike()).max(1);
    state.enemy.atk = (state.enemy.atk - run.enemy_atk_debuff()).max(1);
    {
        let mut stats = app.world_mut().resource_mut::<PlayerStats>();
        stats.hp = (stats.hp + run.battle_start_heal() + run.bond_shield()).min(stats.max_hp);
        stats.mp = (stats.mp - run.hex_battle_start_mp_loss()).max(0);
    }
    if rank == FightRank::Normal {
        run.last_enemy_image = Some(def.image);
    }
    app.insert_resource(state)
        .insert_resource(run)
        .insert_resource(rng)
        .insert_resource(mods)
        .insert_resource(PendingEncounter {
            zone: EncounterZone::Village,
            kind,
        })
        .insert_resource(NextState::<AppState>::default());
}

#[test]
#[ignore = "continuous imperfect-player real input/reward sweep; writes PLAYER_COMBAT_OUT"]
fn imperfect_player_journey_sweep() {
    journey_sweep(false);
}

#[test]
#[ignore = "continuous imperfect-player journeys with actual main-story route rewards; writes PLAYER_ROUTE_COMBAT_OUT"]
fn imperfect_player_story_reward_sweep() {
    journey_sweep(true);
}

fn journey_sweep(story_rewards: bool) {
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
        "seed,player,chapter,encounter,enemy,won,timeout,hp,mp,rounds,wasted_turns,heavy_opportunities,heavy_guards,potions_used,input_operations,battle_frames,repeated_enemy,sword_damage,skill_damage,shield_absorbed\n",
    );
    let mut timeouts = 0;
    for seed in 1u64..=32 {
        for player in [Player::HpOnly, Player::Notice70, Player::Fumble20] {
            let mut app = simulation(&ENEMIES[0], EncounterKind::Random, 0, seed * 12345);
            app.add_systems(FixedLoot, reward_input);
            let mut rng = app.world_mut().remove_resource::<Rng>().unwrap();
            let mut skills = vec![skill::SkillId::SwordGleam];
            if let Some(id) = skill::roll_skill(&skills, &mut rng) {
                skills.push(id);
            }
            app.world_mut().resource_mut::<RunState>().skills = skills;
            app.insert_resource(rng);
            let mut decisions = Rng(seed * 987654321);
            let mut dead = false;
            for (chapter, (boss_def, boss)) in bosses.iter().enumerate() {
                if dead {
                    break;
                }
                if story_rewards {
                    use crate::game::roguelike::event::{RunDialogue, resolve_choice};
                    let mut stats = app.world_mut().remove_resource::<PlayerStats>().unwrap();
                    let mut run = app.world_mut().remove_resource::<RunState>().unwrap();
                    let mut rng = app.world_mut().remove_resource::<Rng>().unwrap();
                    let mut dialogue = RunDialogue::default();
                    dialogue.open_story(chapter, 0);
                    resolve_choice(
                        &mut dialogue,
                        (seed as usize + chapter) % 2,
                        &mut stats,
                        &mut run,
                        &mut rng,
                    );
                    assert!(dialogue.choice_error.is_none());
                    app.insert_resource(stats)
                        .insert_resource(run)
                        .insert_resource(rng);
                }
                // Four continuous encounters per chapter. No between-fight refill.
                // Boss reward_input supplies the actual chapter breakthrough/refill.
                for encounter in 0..4 {
                    let kind = if encounter == 3 {
                        EncounterKind::Boss(*boss)
                    } else {
                        EncounterKind::Random
                    };
                    let mut rng = app.world_mut().remove_resource::<Rng>().unwrap();
                    let previous = app.world().resource::<RunState>().last_enemy_image;
                    let def = if encounter == 3 {
                        *boss_def
                    } else if chapter == 0 && encounter == 0 {
                        &ENEMIES[0]
                    } else {
                        choose_enemy_without_repeat(
                            CHAPTERS[chapter].zones[0],
                            kind,
                            previous,
                            &mut rng,
                        )
                    };
                    let repeated = u32::from(encounter != 3 && previous == Some(def.image));
                    app.insert_resource(rng);
                    prepare_encounter(&mut app, def, kind, chapter == 0 && encounter == 0);
                    let (won, timeout, mut metrics) =
                        imperfect_battle(&mut app, player, &mut decisions);
                    metrics.repeats = repeated;
                    timeouts += u32::from(timeout);
                    let stats = app.world().resource::<PlayerStats>();
                    csv.push_str(&format!("{seed},{},{},{},{},{won},{timeout},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",player.name(),chapter+1,encounter+1,def.name,
                        stats.hp,stats.mp,metrics.rounds,metrics.waste,metrics.heavy,metrics.guarded,metrics.potions,metrics.operations,
                        metrics.frames,metrics.repeats,metrics.sword_damage,metrics.skill_damage,metrics.shield_absorbed));
                    if !won {
                        dead = true;
                        break;
                    }
                    app.insert_resource(RewardChoices {
                        options: vec![RewardOption::Potions(1)],
                        selected: 0,
                    });
                    app.insert_resource(Intent {
                        confirm: true,
                        ..default()
                    });
                    app.world_mut().run_schedule(FixedLoot);
                    app.world_mut().flush();
                }
            }
        }
    }
    let (out_var, default_path) = if story_rewards {
        (
            "PLAYER_ROUTE_COMBAT_OUT",
            "/tmp/love-rpg-player-route-combat.csv",
        )
    } else {
        ("PLAYER_COMBAT_OUT", "/tmp/love-rpg-player-combat.csv")
    };
    let path = std::env::var(out_var).unwrap_or_else(|_| default_path.into());
    std::fs::write(&path, csv).expect("write imperfect player CSV");
    eprintln!(
        "96 continuous imperfect-player journeys saved to {path}; {timeouts} timeouts. battle_frames measures battle only; overworld share requires capture traversal."
    );
    assert_eq!(timeouts, 0, "inspect {path}");
}

#[test]
fn beginner_spell_loops_against_miasma_finish_in_fewer_than_sixty_rounds() {
    for seed in 1u64..=16 {
        for player in [Player::HpOnly, Player::Notice70] {
            let mut app = simulation(
                &MIASMA_ROOT_BOSS,
                EncounterKind::Boss(BossKind::MiasmaRoot),
                3,
                0x9e3779b97f4a7c15u64.wrapping_mul(seed),
            );
            app.world_mut().resource_mut::<RunState>().skills = vec![skill::SkillId::SwordGleam];
            let mut decisions = Rng(seed.wrapping_mul(987654321));
            let (_won, timeout, metrics) = imperfect_battle(&mut app, player, &mut decisions);
            assert!(!timeout, "{player:?} seed {seed} stalled");
            assert!(
                metrics.rounds < 60,
                "{player:?} seed {seed}: {} rounds",
                metrics.rounds
            );
            assert!(matches!(
                app.world().resource::<BattleState>().phase,
                Phase::Won | Phase::Lost
            ));
        }
    }
}

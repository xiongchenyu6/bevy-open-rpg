//! Identity-specific enemy atlas poses, driven by the actual combat phase clock.
use super::{BattleState, EnemyDef, EnemyIntent, Phase, PlayerAction, phase_progress};
use bevy::prelude::*;

const SERPENT_IMAGE: &str = "creatures/ai_water_serpent.png";
const SERPENT_MOTION: &str = "actors/water-serpent-motion.png";
const CELL_SIZE: u32 = 384;
const FRAMES: usize = 8;

#[derive(Component)]
pub(super) struct EnemyAtlas;

/// Returns an atlas only for an enemy with its own authored motion sheet.
pub(super) fn sprite_for(
    enemy: &EnemyDef,
    assets: &AssetServer,
    layouts: &mut Assets<TextureAtlasLayout>,
) -> Option<Sprite> {
    let path = motion_path_for(enemy.image)?;
    Some(Sprite {
        image: assets.load(path),
        texture_atlas: Some(TextureAtlas {
            layout: layouts.add(TextureAtlasLayout::from_grid(
                UVec2::splat(CELL_SIZE),
                FRAMES as u32,
                3,
                None,
                None,
            )),
            index: 0,
        }),
        custom_size: Some(Vec2::splat(enemy.size)),
        ..default()
    })
}

fn motion_path_for(image: &str) -> Option<&'static str> {
    (image == SERPENT_IMAGE).then_some(SERPENT_MOTION)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Clip {
    Idle,
    Attack,
    Hit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FramePose {
    clip: Clip,
    column: usize,
}

impl FramePose {
    fn index(self) -> usize {
        let row = match self.clip {
            Clip::Idle => 0,
            Clip::Attack => 1,
            Clip::Hit => 2,
        };
        row * FRAMES + self.column
    }
}

fn pose(state: &BattleState, idle_age: f32) -> FramePose {
    let clip = match state.phase {
        Phase::EnemyActing
            if state.enemy_attacked
                && !matches!(
                    state.executing_intent,
                    EnemyIntent::Charge | EnemyIntent::Gather | EnemyIntent::Brace
                ) =>
        {
            Clip::Attack
        }
        Phase::PlayerActing
            if state.tactics.last_hit.damage > 0
                && matches!(
                    state.player_action,
                    Some(PlayerAction::Attack | PlayerAction::Spell | PlayerAction::Combo)
                ) =>
        {
            Clip::Hit
        }
        _ => Clip::Idle,
    };
    let column = if clip == Clip::Idle {
        ((idle_age.max(0.0) / 0.15) as usize) % FRAMES
    } else {
        // The authored bite's frame 4 meets the existing phase-midpoint lunge.
        ((phase_progress(state.timer, state.phase_duration.max(0.001)) * FRAMES as f32) as usize)
            .min(FRAMES - 1)
    };
    FramePose { clip, column }
}

pub(super) fn update(sprite: &mut Sprite, state: &BattleState, idle_age: f32) {
    if let Some(atlas) = sprite.texture_atlas.as_mut() {
        atlas.index = pose(state, idle_age).index();
    }
}

#[cfg(test)]
mod tests {
    use super::super::{EncounterKind, tests::status_test_state};
    use super::*;

    #[test]
    fn motion_catalog_preserves_all_other_enemy_identities() {
        use super::super::{BossKind, ENEMIES, EncounterZone, Rng, choose_enemy};
        assert!(motion_path_for(ENEMIES[0].image).is_some());
        for enemy in &ENEMIES[1..] {
            assert!(motion_path_for(enemy.image).is_none());
        }
        for boss in [
            BossKind::MountainFiend,
            BossKind::MoonWraith,
            BossKind::RiverDemon,
            BossKind::MiasmaRoot,
            BossKind::MirrorMinister,
            BossKind::ThunderQilin,
            BossKind::DreamEclipse,
        ] {
            let enemy = choose_enemy(
                EncounterZone::Village,
                EncounterKind::Boss(boss),
                &mut Rng(12345),
            );
            assert!(motion_path_for(enemy.image).is_none());
        }
    }

    #[test]
    fn true_attack_tracks_its_phase_clock_and_recovers_without_looping() {
        let mut state = status_test_state(EncounterKind::Random);
        state.phase = Phase::EnemyActing;
        state.enemy_attacked = true;
        state.executing_intent = EnemyIntent::Heavy;
        for duration in [0.60, 0.72] {
            state.phase_duration = duration;
            state.timer = duration;
            let start = pose(&state, 100.0);
            state.timer = duration / 2.0;
            let contact = pose(&state, 100.0);
            state.timer = 0.0;
            let recovery = pose(&state, 100.0);
            assert_eq!(start.clip, Clip::Attack);
            assert!(start.column < contact.column && contact.column < recovery.column);
            assert!((contact.column as f32 / FRAMES as f32 - 0.5).abs() < 0.13);
            state.timer = -duration;
            assert_eq!(pose(&state, 100.0), recovery);
        }
        state.phase = Phase::Menu;
        assert_eq!(pose(&state, 100.0).clip, Clip::Idle);
    }

    #[test]
    fn charge_gather_brace_and_cancelled_turns_never_play_a_bite() {
        let mut state = status_test_state(EncounterKind::Random);
        state.phase = Phase::EnemyActing;
        state.enemy_attacked = true;
        for intent in [EnemyIntent::Charge, EnemyIntent::Gather, EnemyIntent::Brace] {
            state.executing_intent = intent;
            assert_eq!(pose(&state, 0.2).clip, Clip::Idle);
        }
        state.executing_intent = EnemyIntent::Heavy;
        state.enemy_attacked = false;
        assert_eq!(pose(&state, 0.2).clip, Clip::Idle);
    }

    #[test]
    fn actual_damaging_player_actions_recoil_but_support_actions_do_not() {
        let mut state = status_test_state(EncounterKind::Random);
        state.phase = Phase::PlayerActing;
        state.tactics.last_hit.damage = 10;
        for action in [
            PlayerAction::Attack,
            PlayerAction::Spell,
            PlayerAction::Combo,
        ] {
            super::super::start_player_acting(&mut state, action);
            let start = pose(&state, 0.2);
            state.timer = state.phase_duration / 2.0;
            let contact = pose(&state, 0.2);
            state.timer = 0.0;
            let recovery = pose(&state, 0.2);
            assert_eq!(start.clip, Clip::Hit);
            assert!(start.column < contact.column && contact.column < recovery.column);
            assert!((contact.column as f32 / FRAMES as f32 - 0.5).abs() < 0.13);
            state.timer = -state.phase_duration;
            assert_eq!(pose(&state, 0.2), recovery);
        }
        for action in [PlayerAction::Guard, PlayerAction::Item, PlayerAction::Flee] {
            state.player_action = Some(action);
            assert_eq!(pose(&state, 0.2).clip, Clip::Idle);
        }
        state.tactics.last_hit.damage = 0;
        super::super::start_player_acting(&mut state, PlayerAction::Spell);
        state.timer = state.phase_duration / 2.0;
        assert_eq!(pose(&state, 0.2).clip, Clip::Idle);
    }

    #[test]
    fn idle_moves_while_player_decides_and_action_rows_never_overlap() {
        let state = status_test_state(EncounterKind::Random);
        let mut idle = status_test_state(EncounterKind::Random);
        idle.phase = Phase::Menu;
        assert_ne!(pose(&idle, 0.0), pose(&idle, 0.6));
        assert_eq!(pose(&idle, 0.0), pose(&idle, 1.2));
        assert_eq!(pose(&state, 0.0).clip, Clip::Idle);
        let idle_index = pose(&idle, 0.6).index();
        let mut acting = status_test_state(EncounterKind::Random);
        acting.phase = Phase::EnemyActing;
        acting.enemy_attacked = true;
        acting.executing_intent = EnemyIntent::Strike;
        acting.timer = acting.phase_duration / 2.0;
        let attack_index = pose(&acting, 0.6).index();
        acting.phase = Phase::PlayerActing;
        acting.player_action = Some(PlayerAction::Attack);
        acting.tactics.last_hit.damage = 12;
        let hit_index = pose(&acting, 0.6).index();
        assert_ne!(idle_index, attack_index);
        assert_ne!(attack_index, hit_index);
        assert_ne!(idle_index, hit_index);
    }
}

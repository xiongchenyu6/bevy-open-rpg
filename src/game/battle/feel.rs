//! Combat feel: contact timing, hit-stop, capped camera shake and hit reactions.
//!
//! Battle rules resolve instantly when a command is confirmed; this module only
//! decides *when and how hard* that result is shown. Every cue comes from an
//! observed loss of HP, so guards, heals, failed casts and skipped turns never
//! produce hit feedback.
use bevy::prelude::*;

use super::tactics::Hit;
use super::{BattleState, EnemyIntent, Phase, PlayerAction, action_duration};
use crate::game::PlayerStats;
use crate::game::animation::AnimationHold;

/// Fraction of the hero's sword action at which the blade reaches the enemy.
pub(super) const ATTACK_CONTACT_FRACTION: f32 = 0.30;
/// Spell choreographies land their main burst about this long after the cast.
pub(super) const BASIC_SPELL_CONTACT_SECS: f32 = 0.38;
/// Fraction of an enemy action at which its lunge peaks on the hero.
pub(super) const ENEMY_CONTACT_FRACTION: f32 = 0.5;

/// A contact this close to due fires now: frame steps rarely sum to the exact
/// delay, and a rounding error must not push the hit to the next frame.
pub(super) const CONTACT_EPSILON_SECS: f32 = 1e-4;

const MAX_HIT_STOP_SECS: f32 = 0.14;
const KILL_EXTRA_STOP_SECS: f32 = 0.035;
/// After a stop ends, the scene must run this long before it may stop again.
const HIT_STOP_MIN_GAP_SECS: f32 = 0.10;
const KILL_TRAUMA: f32 = 0.9;
/// Largest camera displacement in world pixels (1280x720 view).
pub(super) const SHAKE_MAX_PX: f32 = 9.0;
/// Full trauma drains in this long; lighter hits end sooner.
const SHAKE_DECAY_SECS: f32 = 0.30;
/// A second impact inside this window cannot restart the shake.
const SHAKE_MIN_GAP_SECS: f32 = 0.09;
const FLASH_SECS: f32 = 0.085;
const ZOOM_RECOVERY_SECS: f32 = 0.28;
const ZOOM_PUNCH_MAX: f32 = 0.088;
const FOLLOWUP_GAP_SECS: f32 = 0.14;
const REACTION_SECS: f32 = 0.36;
const RECOIL_PX: f32 = 26.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Weight {
    /// Damage-over-time ticks, blocked or resisted blows: flash only.
    Light,
    Normal,
    Heavy,
    /// Counter, pursuit and stance-break hits.
    Crit,
}

impl Weight {
    fn hit_stop(self) -> f32 {
        match self {
            Self::Light => 0.0,
            Self::Normal => 0.045,
            Self::Heavy => 0.070,
            Self::Crit => 0.095,
        }
    }

    fn trauma(self) -> f32 {
        match self {
            Self::Light => 0.0,
            Self::Normal => 0.24,
            Self::Heavy => 0.50,
            Self::Crit => 0.80,
        }
    }

    fn reaction(self) -> f32 {
        match self {
            Self::Light => 0.40,
            Self::Normal => 0.80,
            Self::Heavy => 1.10,
            Self::Crit => 1.35,
        }
    }

    fn zoom_punch(self) -> f32 {
        match self {
            Self::Light => 0.0,
            Self::Normal => 0.018,
            Self::Heavy => 0.036,
            Self::Crit => 0.058,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Target {
    Enemy,
    Hero,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct HitCue {
    pub target: Target,
    pub weight: Weight,
    pub damage: i32,
    pub kill: bool,
    /// Seconds on the battle timeline until the visible contact.
    pub delay: f32,
}

#[derive(Clone, Copy, Debug)]
struct Reaction {
    strength: f32,
    /// Battle-timeline age, so recoil holds during hit-stop.
    age: f32,
    /// Real-time age, so the flash still plays during hit-stop.
    flash_age: f32,
}

impl Default for Reaction {
    fn default() -> Self {
        Self {
            strength: 0.0,
            age: REACTION_SECS,
            flash_age: FLASH_SECS,
        }
    }
}

/// How a struck actor is drawn this frame, relative to its rest pose.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Pose {
    /// Distance pushed away from the attacker, in pixels (always >= 0).
    pub recoil: f32,
    /// High-frequency (62.5 Hz) zero-drift 2D micro-tremor during hitstop/flash.
    pub tremor: Vec2,
    pub squash: Vec2,
    /// 0..1 brightness pop on the contact frame (85ms bright hit-flash).
    pub flash: f32,
    /// 0..1 lingering hurt tint that follows the recoil.
    pub hurt: f32,
}

impl Pose {
    pub const REST: Self = Self {
        recoil: 0.0,
        tremor: Vec2::ZERO,
        squash: Vec2::ONE,
        flash: 0.0,
        hurt: 0.0,
    };
}

/// Emitted on the exact contact frame so VFX juice spawns with zero drift.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FiredImpact {
    pub target: Target,
    pub weight: Weight,
    pub damage: i32,
    pub kill: bool,
}

#[derive(Resource, Default)]
pub(crate) struct BattleFeel {
    pending: Vec<HitCue>,
    fired_impacts: Vec<FiredImpact>,
    hit_stop: f32,
    stop_gap: f32,
    /// Battle-timeline seconds advanced this frame (real dt minus hit-stop).
    dt: f32,
    trauma: f32,
    shake_clock: f32,
    shake_gap: f32,
    zoom_peak: f32,
    zoom_age: f32,
    zoom_punch: f32,
    enemy: Reaction,
    hero: Reaction,
    /// Enemy and hero HP seen last frame; `None` until the first battle frame.
    seen_hp: Option<(i32, i32)>,
}

impl BattleFeel {
    /// Seconds every battle animation, effect and phase timer should advance.
    pub(crate) fn dt(&self) -> f32 {
        self.dt
    }

    pub(super) fn is_hit_stopped(&self) -> bool {
        self.hit_stop > 0.0
    }

    pub(super) fn take_fired_impacts(&mut self) -> Vec<FiredImpact> {
        std::mem::take(&mut self.fired_impacts)
    }

    pub(super) fn camera_zoom_scale(&self) -> f32 {
        (1.0 - self.zoom_punch).clamp(1.0 - ZOOM_PUNCH_MAX, 1.0)
    }

    pub(super) fn queue(&mut self, cue: HitCue) {
        if cue.damage > 0 {
            self.pending.push(cue);
        }
    }

    /// Damage already resolved by the rules but not yet shown on this target.
    pub(super) fn pending_damage(&self, target: Target) -> i32 {
        self.pending
            .iter()
            .filter(|cue| cue.target == target)
            .map(|cue| cue.damage)
            .sum()
    }

    /// Integrates one frame exactly: hit-stop consumes real time first, and a
    /// contact inside the frame splits it, so results match at any frame rate.
    pub(super) fn advance(&mut self, real_dt: f32) {
        let real_dt = real_dt.max(0.0);
        self.shake_clock += real_dt;
        self.shake_gap = (self.shake_gap - real_dt).max(0.0);
        self.stop_gap = (self.stop_gap - real_dt).max(0.0);
        self.trauma = (self.trauma - real_dt / SHAKE_DECAY_SECS).max(0.0);
        if self.zoom_peak > 0.0 {
            self.zoom_age += real_dt;
            let u = (self.zoom_age / ZOOM_RECOVERY_SECS).clamp(0.0, 1.0);
            if u >= 1.0 {
                self.zoom_peak = 0.0;
                self.zoom_punch = 0.0;
            } else {
                let decay = (1.0 - u).powi(2);
                let elastic = (u * std::f32::consts::PI * 1.5).cos();
                self.zoom_punch = (self.zoom_peak * decay * (0.74 + 0.26 * elastic)).max(0.0);
            }
        }
        self.enemy.flash_age += real_dt;
        self.hero.flash_age += real_dt;

        let mut left = real_dt;
        let mut timeline = 0.0;
        loop {
            let stopped = self.hit_stop.min(left);
            self.hit_stop -= stopped;
            left -= stopped;
            if left <= 0.0 {
                break;
            }
            let next = self
                .pending
                .iter()
                .map(|cue| cue.delay)
                .fold(f32::INFINITY, f32::min)
                .max(0.0);
            let step = next.min(left);
            for cue in &mut self.pending {
                cue.delay -= step;
            }
            self.enemy.age += step;
            self.hero.age += step;
            timeline += step;
            left -= step;
            let mut fired = false;
            let mut index = 0;
            while index < self.pending.len() {
                if self.pending[index].delay <= CONTACT_EPSILON_SECS {
                    let cue = self.pending.swap_remove(index);
                    self.contact(cue);
                    fired = true;
                } else {
                    index += 1;
                }
            }
            if !fired {
                break;
            }
        }
        self.dt = timeline;
    }

    fn contact(&mut self, cue: HitCue) {
        let mut stop = cue.weight.hit_stop();
        let mut trauma = cue.weight.trauma();
        let mut zoom = cue.weight.zoom_punch();
        if cue.kill && cue.weight > Weight::Light {
            stop += KILL_EXTRA_STOP_SECS;
            trauma = trauma.max(KILL_TRAUMA);
            zoom = zoom.max(0.082);
        }
        // Overlapping impacts take the strongest value instead of adding up,
        // and a burst of hits cannot keep re-freezing the scene.
        if stop > 0.0 && (self.hit_stop > 0.0 || self.stop_gap <= 0.0) {
            self.hit_stop = self.hit_stop.max(stop).min(MAX_HIT_STOP_SECS);
            self.stop_gap = self.hit_stop + HIT_STOP_MIN_GAP_SECS;
        }
        if trauma > 0.0 && self.shake_gap <= 0.0 {
            self.trauma = self.trauma.max(trauma).min(1.0);
            self.shake_gap = SHAKE_MIN_GAP_SECS;
        }
        if zoom > 0.0 {
            self.zoom_peak = self.zoom_peak.max(zoom).min(ZOOM_PUNCH_MAX);
            self.zoom_age = 0.0;
            self.zoom_punch = self.zoom_peak;
        }
        self.fired_impacts.push(FiredImpact {
            target: cue.target,
            weight: cue.weight,
            damage: cue.damage,
            kill: cue.kill,
        });
        let reaction = match cue.target {
            Target::Enemy => &mut self.enemy,
            Target::Hero => &mut self.hero,
        };
        *reaction = Reaction {
            strength: cue.weight.reaction(),
            age: 0.0,
            flash_age: 0.0,
        };
    }

    pub(super) fn shake_offset(&self) -> Vec2 {
        if self.trauma <= 0.0 {
            return Vec2::ZERO;
        }
        let amplitude = SHAKE_MAX_PX * self.trauma;
        // Damped multi-harmonic 2D shake (primary + secondary overtones), bounded <= amplitude.
        let t = self.shake_clock;
        let hx = (t * 83.0).sin() * 0.68 + (t * 137.0 + 0.7).sin() * 0.32;
        let hy = (t * 97.0 + 1.3).sin() * 0.65 + (t * 163.0 + 2.4).cos() * 0.35;
        Vec2::new(hx, hy * 0.62) * amplitude
    }

    pub(super) fn pose(&self, target: Target) -> Pose {
        let reaction = match target {
            Target::Enemy => self.enemy,
            Target::Hero => self.hero,
        };
        if reaction.strength <= 0.0 || reaction.age >= REACTION_SECS {
            return Pose::REST;
        }
        let k = reaction.age / REACTION_SECS;
        let settle = (1.0 - k).powi(2);
        let wobble = settle * (k * 9.0).cos();
        // 62.5 Hz zero-drift 2D micro-tremor active through hit-stop & 85ms flash window.
        let flash = (1.0 - reaction.flash_age / FLASH_SECS).clamp(0.0, 1.0);
        let tremor_env = if self.hit_stop > 0.0 { 1.0 } else { flash };
        let tremor_amp = (3.8 * reaction.strength).min(6.0) * tremor_env;
        let tremor = if tremor_amp > 0.01 {
            Vec2::new(
                (reaction.flash_age * 62.5 * std::f32::consts::TAU + 0.55).sin() * tremor_amp,
                (reaction.flash_age * 64.0 * std::f32::consts::TAU + 1.1).cos() * tremor_amp * 0.45,
            )
        } else {
            Vec2::ZERO
        };
        Pose {
            recoil: RECOIL_PX * reaction.strength * settle,
            tremor,
            squash: Vec2::new(
                1.0 - 0.12 * reaction.strength * wobble,
                1.0 + 0.09 * reaction.strength * wobble,
            ),
            flash,
            hurt: settle,
        }
    }

    /// 0..1 through the current hit reaction, or `None` when at rest.
    pub(super) fn reaction_progress(&self, target: Target) -> Option<f32> {
        let reaction = match target {
            Target::Enemy => self.enemy,
            Target::Hero => self.hero,
        };
        (reaction.strength > 0.0 && reaction.age < REACTION_SECS)
            .then(|| reaction.age / REACTION_SECS)
    }

    fn clear(&mut self) {
        *self = Self::default();
    }
}

/// When and how a damage number should appear.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Contact {
    pub delay: f32,
    pub crit: bool,
}

impl Contact {
    pub fn of(state: &BattleState, action: PlayerAction) -> Self {
        Self {
            delay: contact_delay(state, action),
            crit: is_crit(state.tactics.last_hit),
        }
    }

    pub fn at(delay: f32) -> Self {
        Self { delay, crit: false }
    }

    /// A companion's extra blow, one beat after the main contact.
    pub fn followup(self) -> Self {
        Self::at(self.delay + FOLLOWUP_GAP_SECS)
    }
}

/// The player's contact moment, measured from the confirmed command.
pub(super) fn contact_delay(state: &BattleState, action: PlayerAction) -> f32 {
    match action {
        PlayerAction::Attack | PlayerAction::Combo => {
            action_duration(action) * ATTACK_CONTACT_FRACTION
        }
        PlayerAction::Spell => state.spell_contact_secs,
        PlayerAction::Guard | PlayerAction::Item | PlayerAction::Flee => 0.0,
    }
}

/// Counter and pursuit blows are this battle's critical hits.
fn is_crit(hit: Hit) -> bool {
    hit.counter || hit.exposed
}

fn player_hit_weight(action: Option<PlayerAction>, hit: Hit) -> Weight {
    if is_crit(hit) || hit.broke {
        Weight::Crit
    } else if hit.resisted {
        Weight::Light
    } else {
        match action {
            Some(PlayerAction::Spell | PlayerAction::Combo) => Weight::Heavy,
            Some(PlayerAction::Attack) => Weight::Normal,
            _ => Weight::Light,
        }
    }
}

fn enemy_hit_weight(intent: EnemyIntent, damage: i32, max_hp: i32) -> Weight {
    if matches!(intent, EnemyIntent::Heavy | EnemyIntent::Special) || damage * 4 >= max_hp {
        Weight::Heavy
    } else if damage * 20 < max_hp {
        Weight::Light
    } else {
        Weight::Normal
    }
}

/// Turns this frame's HP losses into contact cues. Reads only resolved state.
fn cues_for(
    state: &BattleState,
    hero_hp: i32,
    hero_max_hp: i32,
    seen: (i32, i32),
    frame_dt: f32,
) -> [Option<HitCue>; 2] {
    let enemy_lost = seen.0 - state.enemy.hp;
    let hero_lost = seen.1 - hero_hp;
    let player_strike = state.phase == Phase::PlayerActing
        && matches!(
            state.player_action,
            Some(PlayerAction::Attack | PlayerAction::Spell | PlayerAction::Combo)
        );
    // Impact effects spawned this frame have already aged by this frame's
    // step, so the cue starts from the same point and fires with them.
    let elapsed = frame_dt;
    let player_delay = (state
        .player_action
        .map_or(0.0, |action| contact_delay(state, action))
        - elapsed)
        .max(0.0);
    let enemy_cue = (enemy_lost > 0).then(|| {
        let (weight, delay) = if player_strike {
            (
                player_hit_weight(state.player_action, state.tactics.last_hit),
                player_delay,
            )
        } else {
            (Weight::Light, 0.0)
        };
        HitCue {
            target: Target::Enemy,
            weight,
            damage: enemy_lost,
            kill: state.enemy.hp <= 0,
            delay,
        }
    });
    let hero_cue = (hero_lost > 0).then(|| {
        let (weight, delay) = if state.phase == Phase::EnemyActing && state.enemy_attacked {
            (
                enemy_hit_weight(state.executing_intent, hero_lost, hero_max_hp),
                (state.phase_duration * ENEMY_CONTACT_FRACTION - elapsed).max(0.0),
            )
        } else if player_strike {
            // Mirror backlash and self-inflicted costs land with the blow.
            (Weight::Light, player_delay)
        } else {
            (Weight::Light, 0.0)
        };
        HitCue {
            target: Target::Hero,
            weight,
            damage: hero_lost,
            kill: hero_hp <= 0,
            delay,
        }
    });
    [enemy_cue, hero_cue]
}

pub(super) fn advance(
    time: Res<Time>,
    mut feel: ResMut<BattleFeel>,
    mut hold: ResMut<AnimationHold>,
) {
    let real_dt = time.delta_secs();
    feel.advance(real_dt);
    hold.0 = real_dt - feel.dt;
}

pub(super) fn observe(
    state: Res<BattleState>,
    stats: Res<PlayerStats>,
    mut feel: ResMut<BattleFeel>,
) {
    let now = (state.enemy.hp, stats.hp);
    if let Some(seen) = feel.seen_hp {
        for cue in cues_for(&state, stats.hp, stats.max_hp, seen, feel.dt)
            .into_iter()
            .flatten()
        {
            feel.queue(cue);
        }
    }
    feel.seen_hp = Some(now);
}

pub(super) fn shake_camera(
    feel: Res<BattleFeel>,
    mut cameras: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
) {
    let offset = feel.shake_offset();
    let zoom_scale = feel.camera_zoom_scale();
    for (mut transform, mut projection) in &mut cameras {
        transform.translation.x = offset.x;
        transform.translation.y = offset.y;
        if let Projection::Orthographic(ortho) = projection.as_mut() {
            ortho.scale = zoom_scale;
        }
    }
}

/// Battle entry and exit both start from a clean slate: no queued contact,
/// frozen clock or camera offset/zoom may leak into another battle or screen.
pub(super) fn reset(
    mut feel: ResMut<BattleFeel>,
    mut hold: ResMut<AnimationHold>,
    mut cameras: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
) {
    feel.clear();
    hold.0 = 0.0;
    for (mut transform, mut projection) in &mut cameras {
        transform.translation.x = 0.0;
        transform.translation.y = 0.0;
        if let Projection::Orthographic(ortho) = projection.as_mut() {
            ortho.scale = 1.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{EncounterKind, tests::status_test_state};
    use super::*;

    fn cue(weight: Weight, delay: f32) -> HitCue {
        HitCue {
            target: Target::Enemy,
            weight,
            damage: 10,
            kill: false,
            delay,
        }
    }

    /// Runs `secs` of real time at a fixed frame length and returns
    /// (timeline seconds, first real time the contact was visible, peak shake).
    fn run(feel: &mut BattleFeel, frame: f32, secs: f32) -> (f32, Option<f32>, f32) {
        let mut timeline = 0.0;
        let mut contact_at = None;
        let mut peak: f32 = 0.0;
        let frames = (secs / frame).round() as u32;
        for index in 1..=frames {
            feel.advance(frame);
            timeline += feel.dt();
            if contact_at.is_none() && feel.pending_damage(Target::Enemy) == 0 {
                contact_at = Some(index as f32 * frame);
            }
            peak = peak.max(feel.shake_offset().length());
        }
        (timeline, contact_at, peak)
    }

    #[test]
    fn hit_stop_costs_the_same_time_at_30_60_and_144_fps() {
        for fps in [30.0_f32, 60.0, 144.0] {
            let mut feel = BattleFeel::default();
            feel.queue(cue(Weight::Heavy, 0.216));
            let frame = 1.0 / fps;
            let secs = (1.2 / frame).round() * frame;
            let (timeline, contact_at, _) = run(&mut feel, frame, secs);
            assert!(
                (secs - timeline - Weight::Heavy.hit_stop()).abs() < 1e-3,
                "{fps} fps froze {}s",
                secs - timeline
            );
            let contact_at = contact_at.expect("contact fired");
            assert!(
                (0.216..0.216 + frame + 1e-4).contains(&contact_at),
                "{fps} fps showed contact at {contact_at}"
            );
        }
    }

    #[test]
    fn a_contact_due_exactly_on_a_frame_boundary_fires_on_that_frame() {
        // 0.3s is nine 30 FPS frames, but nine f32 steps do not sum to 0.3.
        let mut feel = BattleFeel::default();
        feel.queue(cue(Weight::Normal, 0.3));
        for _ in 0..8 {
            feel.advance(1.0 / 30.0);
        }
        assert_eq!(feel.pending_damage(Target::Enemy), 10);
        feel.advance(1.0 / 30.0);
        assert_eq!(feel.pending_damage(Target::Enemy), 0);
    }

    #[test]
    fn one_long_frame_fires_the_contact_once_and_keeps_the_exact_stop() {
        let mut feel = BattleFeel::default();
        feel.queue(cue(Weight::Crit, 0.10));
        feel.advance(0.25);
        assert_eq!(feel.pending_damage(Target::Enemy), 0);
        assert!((feel.dt() - (0.25 - Weight::Crit.hit_stop())).abs() < 1e-5);
        feel.advance(0.25);
        assert!((feel.dt() - 0.25).abs() < 1e-6);
        feel.advance(0.25);
        assert_eq!(feel.shake_offset(), Vec2::ZERO);
        assert_eq!(feel.pose(Target::Enemy), Pose::REST);
    }

    #[test]
    fn stacked_impacts_never_exceed_the_stop_or_shake_caps() {
        let mut feel = BattleFeel::default();
        for index in 0..40 {
            feel.queue(HitCue {
                kill: true,
                ..cue(Weight::Crit, index as f32 * 0.004)
            });
        }
        let mut frozen = 0.0;
        let mut peak: f32 = 0.0;
        for _ in 0..240 {
            feel.advance(1.0 / 240.0);
            frozen += 1.0 / 240.0 - feel.dt();
            peak = peak.max(feel.shake_offset().length());
            assert!(feel.hit_stop <= MAX_HIT_STOP_SECS);
            assert!(feel.trauma <= 1.0);
        }
        assert!(peak <= SHAKE_MAX_PX * 1.2, "peak shake {peak}px");
        // 40 hits inside 0.16s may hold the scene, but never for seconds.
        assert!(frozen < 0.16 + MAX_HIT_STOP_SECS * 2.0, "froze {frozen}s");
    }

    #[test]
    fn a_second_impact_inside_the_gap_cannot_restart_the_shake() {
        let mut feel = BattleFeel::default();
        feel.queue(cue(Weight::Normal, 0.0));
        feel.advance(0.001);
        let first = feel.trauma;
        feel.queue(cue(Weight::Crit, 0.0));
        feel.advance(0.001);
        assert!(feel.trauma <= first);
        // Once the gap has passed a new impact shakes again.
        feel.advance(SHAKE_MIN_GAP_SECS + Weight::Crit.hit_stop());
        feel.queue(cue(Weight::Crit, 0.0));
        feel.advance(0.001);
        assert!(feel.trauma > first);
    }

    #[test]
    fn shake_and_reaction_return_to_rest() {
        let mut feel = BattleFeel::default();
        feel.queue(HitCue {
            kill: true,
            ..cue(Weight::Crit, 0.0)
        });
        feel.advance(1.0 / 60.0);
        assert!(feel.shake_offset().length() > 0.0);
        let pose = feel.pose(Target::Enemy);
        assert!(pose.recoil > 20.0 && pose.flash > 0.5 && pose.squash.x < 1.0);
        assert_eq!(feel.pose(Target::Hero), Pose::REST);
        for _ in 0..60 {
            feel.advance(1.0 / 60.0);
        }
        assert_eq!(feel.shake_offset(), Vec2::ZERO);
        assert_eq!(feel.pose(Target::Enemy), Pose::REST);
    }

    #[test]
    fn recoil_holds_through_hit_stop_while_the_flash_keeps_playing() {
        let mut feel = BattleFeel::default();
        feel.queue(cue(Weight::Crit, 0.0));
        feel.advance(0.001);
        let struck = feel.pose(Target::Enemy);
        feel.advance(0.06);
        let held = feel.pose(Target::Enemy);
        assert_eq!(feel.dt(), 0.0);
        assert_eq!(held.recoil, struck.recoil);
        assert!(held.flash < struck.flash);
    }

    #[test]
    fn heavier_hits_stop_shake_and_push_harder() {
        let weights = [Weight::Light, Weight::Normal, Weight::Heavy, Weight::Crit];
        for pair in weights.windows(2) {
            assert!(pair[0].hit_stop() < pair[1].hit_stop());
            assert!(pair[0].trauma() < pair[1].trauma());
            assert!(pair[0].reaction() < pair[1].reaction());
        }
        assert_eq!(Weight::Light.hit_stop(), 0.0);
        assert_eq!(Weight::Light.trauma(), 0.0);
        assert!(Weight::Crit.hit_stop() + KILL_EXTRA_STOP_SECS <= MAX_HIT_STOP_SECS);
    }

    #[test]
    fn only_lost_hp_produces_a_cue() {
        let mut state = status_test_state(EncounterKind::Random);
        state.player_action = Some(PlayerAction::Attack);
        // Nothing changed, a heal, and a guard turn: no feedback at all.
        assert!(
            cues_for(&state, 50, 100, (100, 50), 0.0)
                .iter()
                .all(Option::is_none)
        );
        assert!(
            cues_for(&state, 80, 100, (100, 50), 0.0)
                .iter()
                .all(Option::is_none)
        );
        state.enemy.hp = 120;
        assert!(
            cues_for(&state, 50, 100, (100, 50), 0.0)
                .iter()
                .all(Option::is_none)
        );

        let mut feel = BattleFeel::default();
        feel.queue(HitCue {
            damage: 0,
            ..cue(Weight::Crit, 0.0)
        });
        feel.advance(0.1);
        assert_eq!(feel.dt(), 0.1);
        assert_eq!(feel.shake_offset(), Vec2::ZERO);
        assert_eq!(feel.pose(Target::Enemy), Pose::REST);
    }

    #[test]
    fn sword_damage_waits_for_the_blade_and_counters_hit_hardest() {
        let mut state = status_test_state(EncounterKind::Random);
        super::super::start_player_acting(&mut state, PlayerAction::Attack);
        state.enemy.hp = 88;
        let [enemy, hero] = cues_for(&state, 50, 100, (100, 50), 0.0);
        let enemy = enemy.expect("enemy lost hp");
        assert!(hero.is_none());
        assert_eq!(enemy.damage, 12);
        assert_eq!(enemy.weight, Weight::Normal);
        assert!((enemy.delay - 0.72 * ATTACK_CONTACT_FRACTION).abs() < 1e-6);

        state.tactics.last_hit.counter = true;
        state.enemy.hp = 0;
        let [enemy, _] = cues_for(&state, 50, 100, (100, 50), 0.0);
        let enemy = enemy.unwrap();
        assert_eq!(enemy.weight, Weight::Crit);
        assert!(enemy.kill);

        state.tactics.last_hit = Hit {
            resisted: true,
            ..Hit::default()
        };
        state.enemy.hp = 95;
        let [enemy, _] = cues_for(&state, 50, 100, (100, 50), 0.0);
        assert_eq!(enemy.unwrap().weight, Weight::Light);
    }

    #[test]
    fn enemy_blows_land_at_the_lunge_peak_and_status_ticks_stay_light() {
        let mut state = status_test_state(EncounterKind::Random);
        state.phase = Phase::EnemyActing;
        state.phase_duration = 0.72;
        state.timer = 0.72;
        state.enemy_attacked = true;
        state.executing_intent = EnemyIntent::Heavy;
        let [enemy, hero] = cues_for(&state, 70, 100, (100, 80), 0.0);
        assert!(enemy.is_none());
        let hero = hero.expect("hero lost hp");
        assert_eq!(hero.weight, Weight::Heavy);
        assert!((hero.delay - 0.36).abs() < 1e-6);
        // The frame that resolved the blow is not waited for twice.
        let [_, hero] = cues_for(&state, 70, 100, (100, 80), 0.1);
        assert!((hero.unwrap().delay - 0.26).abs() < 1e-6);

        // Poison before the enemy acts: immediate, no stop, no shake.
        state.enemy_attacked = false;
        state.enemy.hp = 94;
        let [enemy, _] = cues_for(&state, 80, 100, (100, 80), 0.0);
        let enemy = enemy.unwrap();
        assert_eq!((enemy.weight, enemy.delay), (Weight::Light, 0.0));
    }

    #[test]
    fn pending_damage_is_hidden_until_contact_and_clear_drops_everything() {
        let mut feel = BattleFeel::default();
        feel.queue(cue(Weight::Heavy, 0.2));
        feel.advance(0.1);
        assert_eq!(feel.pending_damage(Target::Enemy), 10);
        assert_eq!(feel.pending_damage(Target::Hero), 0);
        feel.advance(0.11);
        assert_eq!(feel.pending_damage(Target::Enemy), 0);
        assert!(feel.hit_stop > 0.0 && feel.trauma > 0.0);

        feel.queue(cue(Weight::Heavy, 0.2));
        feel.seen_hp = Some((1, 1));
        feel.clear();
        assert_eq!(feel.pending_damage(Target::Enemy), 0);
        assert_eq!(feel.hit_stop, 0.0);
        assert_eq!(feel.shake_offset(), Vec2::ZERO);
        assert_eq!(feel.pose(Target::Enemy), Pose::REST);
        assert!(feel.seen_hp.is_none());
    }

    #[test]
    fn camera_zoom_punch_zooms_in_on_impact_and_recovers_zero_drift() {
        let mut feel = BattleFeel::default();
        assert_eq!(feel.camera_zoom_scale(), 1.0);
        feel.queue(HitCue {
            kill: true,
            ..cue(Weight::Crit, 0.0)
        });
        feel.advance(1.0 / 120.0);
        assert!(
            feel.camera_zoom_scale() < 0.94,
            "expected camera punch-in, got {}",
            feel.camera_zoom_scale()
        );
        for _ in 0..40 {
            feel.advance(1.0 / 60.0);
        }
        assert_eq!(feel.camera_zoom_scale(), 1.0, "zoom must settle zero-drift");
    }

    #[test]
    fn hitstop_victim_micro_tremor_vibrates_at_high_frequency_and_clears_at_rest() {
        let mut feel = BattleFeel::default();
        feel.queue(cue(Weight::Crit, 0.0));
        // Trigger contact on frame 1, then step 4ms and 8ms inside hit-stop.
        feel.advance(0.001);
        feel.advance(0.003);
        let p1 = feel.pose(Target::Enemy);
        feel.advance(0.008);
        let p2 = feel.pose(Target::Enemy);
        assert!(feel.is_hit_stopped());
        assert!(
            p1.tremor.x.abs() > 2.0,
            "expected strong tremor: {:?}",
            p1.tremor
        );
        assert!(
            p1.tremor.x * p2.tremor.x < 0.0,
            "62.5 Hz tremor should flip polarity within 8ms: {:?} vs {:?}",
            p1.tremor,
            p2.tremor
        );
        // Settle completely -> exact zero tremor.
        for _ in 0..60 {
            feel.advance(1.0 / 60.0);
        }
        assert_eq!(feel.pose(Target::Enemy).tremor, Vec2::ZERO);
        assert_eq!(feel.pose(Target::Enemy), Pose::REST);
    }
}

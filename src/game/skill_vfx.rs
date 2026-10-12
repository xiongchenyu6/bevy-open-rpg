//! 程序化二D技能演出。
//!
//! 每招由延迟生成的网格、精灵和动态光组合而成。通用动画器只负责时序、
//! 运动和消散;二十个 [`VfxStyle`] 各自拥有独立的生成脚本与轮廓。

use bevy::prelude::*;
use bevy_firefly::prelude::PointLight2d;

use super::lighting::{self, LightingAssets};
use super::roguelike::skill::VfxStyle;
use super::state::AppState;

#[derive(Resource)]
pub struct SkillVfxAssets {
    disc: Handle<Mesh>,
    ring: Handle<Image>,
    thick_ring: Handle<Image>,
    hex: Handle<Mesh>,
    hex_ring: Handle<Mesh>,
    diamond: Handle<Mesh>,
    triangle: Handle<Image>,
}

impl SkillVfxAssets {
    /// Dummy visual handles for CPU-only gameplay integration tests.
    #[cfg(test)]
    pub(crate) fn simulation() -> Self {
        Self {
            disc: Handle::default(),
            ring: Handle::default(),
            thick_ring: Handle::default(),
            hex: Handle::default(),
            hex_ring: Handle::default(),
            diamond: Handle::default(),
            triangle: Handle::default(),
        }
    }
}

impl FromWorld for SkillVfxAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        let ring = assets.load("effects/spell-ring.png");
        let thick_ring = assets.load("effects/spell-ring-wide.png");
        let triangle = assets.load("effects/spell-shard.png");
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        Self {
            disc: meshes.add(Circle::new(1.0)),
            ring,
            thick_ring,
            hex: meshes.add(RegularPolygon::new(1.0, 6)),
            hex_ring: meshes.add(RegularPolygon::new(1.0, 6).to_ring(0.13)),
            diamond: meshes.add(Rhombus::new(2.0, 2.0)),
            triangle,
        }
    }
}

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StatusVfxState {
    pub poison_stacks: u8,
    pub burn_turns: u8,
    pub frozen: bool,
    pub stunned: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Component)]
enum StatusAura {
    Poison,
    Burn,
    Freeze,
    Stun,
}

#[derive(Clone, Copy)]
enum Motion {
    Static {
        sway: Vec2,
        cycles: f32,
    },
    Linear {
        arc: f32,
        wave: f32,
        cycles: f32,
    },
    Orbit {
        center: Vec3,
        radius_start: f32,
        radius_end: f32,
        angle: f32,
        turns: f32,
        lift: f32,
    },
}

#[derive(Component, Clone, Copy)]
struct VfxNode {
    age: f32,
    delay: f32,
    duration: f32,
    repeating: bool,
    start: Vec3,
    end: Vec3,
    start_scale: Vec2,
    end_scale: Vec2,
    rotation: f32,
    spin: f32,
    fade_in: f32,
    fade_out: f32,
    pulse: f32,
    overshoot: f32,
    flicker: f32,
    seed: f32,
    color: Color,
    motion: Motion,
}

impl VfxNode {
    fn new(delay: f32, duration: f32, position: Vec3, color: Color) -> Self {
        Self {
            age: 0.0,
            delay,
            duration,
            repeating: false,
            start: position,
            end: position,
            start_scale: Vec2::ZERO,
            end_scale: Vec2::ONE,
            rotation: 0.0,
            spin: 0.0,
            fade_in: 0.10,
            fade_out: 0.28,
            pulse: 0.0,
            overshoot: 0.0,
            flicker: 0.0,
            seed: delay * 31.7 + duration * 7.3,
            color,
            motion: Motion::Static {
                sway: Vec2::ZERO,
                cycles: 0.0,
            },
        }
    }

    fn scales(mut self, start: Vec2, end: Vec2) -> Self {
        self.start_scale = start;
        self.end_scale = end;
        self
    }

    fn rotates(mut self, rotation: f32, spin: f32) -> Self {
        self.rotation = rotation;
        self.spin = spin;
        self
    }

    fn linear(mut self, end: Vec3, arc: f32, wave: f32, cycles: f32) -> Self {
        self.end = end;
        self.motion = Motion::Linear { arc, wave, cycles };
        self
    }

    fn orbit(mut self, center: Vec3, radii: Vec2, angle: f32, turns: f32, lift: f32) -> Self {
        self.motion = Motion::Orbit {
            center,
            radius_start: radii.x,
            radius_end: radii.y,
            angle,
            turns,
            lift,
        };
        self
    }

    fn sway(mut self, amount: Vec2, cycles: f32) -> Self {
        self.motion = Motion::Static {
            sway: amount,
            cycles,
        };
        self
    }

    fn envelope(mut self, fade_in: f32, fade_out: f32) -> Self {
        self.fade_in = fade_in;
        self.fade_out = fade_out;
        self
    }

    fn lively(mut self, pulse: f32, overshoot: f32, flicker: f32) -> Self {
        self.pulse = pulse;
        self.overshoot = overshoot;
        self.flicker = flicker;
        self
    }

    fn repeating(mut self) -> Self {
        self.repeating = true;
        self
    }
}

#[derive(Component)]
struct VfxLight {
    intensity: f32,
}

pub struct SkillVfxPlugin;

impl Plugin for SkillVfxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SkillVfxAssets>()
            .init_resource::<StatusVfxState>()
            .add_systems(
                Update,
                (sync_status_auras, update_vfx_nodes)
                    .chain()
                    .run_if(in_state(AppState::Battle)),
            );
    }
}

#[allow(clippy::too_many_arguments)]
pub fn spawn_skill_vfx(
    commands: &mut Commands,
    materials: &mut Assets<ColorMaterial>,
    assets: &SkillVfxAssets,
    lights: &LightingAssets,
    style: VfxStyle,
    caster: Vec3,
    target: Vec3,
) {
    let mut fx = Emitter {
        commands,
        materials,
        assets,
        lights,
    };
    match style {
        VfxStyle::SwordGleam => sword_gleam(&mut fx, caster, target),
        VfxStyle::CinderMeteor => cinder_meteor(&mut fx, caster, target),
        VfxStyle::FrostLance => frost_lance(&mut fx, caster, target),
        VfxStyle::StormChain => storm_chain(&mut fx, caster, target),
        VfxStyle::WindSpiral => wind_spiral(&mut fx, caster),
        VfxStyle::EarthRift => earth_rift(&mut fx, caster, target),
        VfxStyle::TidalDragon => tidal_dragon(&mut fx, caster, target),
        VfxStyle::ThornCage => thorn_cage(&mut fx, caster, target),
        VfxStyle::MoonHalo => moon_halo(&mut fx, caster),
        VfxStyle::SoulLantern => soul_lantern(&mut fx, caster, target),
        VfxStyle::KarmaLotus => karma_lotus(&mut fx, target),
        VfxStyle::GlacierCrown => glacier_crown(&mut fx, caster, target),
        VfxStyle::ThunderPrison => thunder_prison(&mut fx, target),
        VfxStyle::BladeStorm => blade_storm(&mut fx, target),
        VfxStyle::MountainAegis => mountain_aegis(&mut fx, caster),
        VfxStyle::WhaleBreath => whale_breath(&mut fx, caster, target),
        VfxStyle::WorldTreeBloom => world_tree_bloom(&mut fx, caster),
        VfxStyle::Starfall => starfall(&mut fx, target),
        VfxStyle::Eclipse => eclipse(&mut fx, target),
        VfxStyle::SwordArray => sword_array(&mut fx, caster, target),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusBurstKind {
    Poison,
    Burn,
    Freeze,
    Stun,
    Resist,
}

pub fn spawn_status_burst(
    commands: &mut Commands,
    materials: &mut Assets<ColorMaterial>,
    assets: &SkillVfxAssets,
    lights: &LightingAssets,
    kind: StatusBurstKind,
    target: Vec3,
) {
    let mut fx = Emitter {
        commands,
        materials,
        assets,
        lights,
    };
    match kind {
        StatusBurstKind::Poison => {
            fx.ring(
                rgba(0.48, 1.0, 0.18, 0.82),
                VfxNode::new(0.0, 0.52, target, rgba(0.48, 1.0, 0.18, 0.82))
                    .scales(Vec2::splat(30.0), Vec2::splat(128.0))
                    .lively(0.08, 0.18, 0.0),
            );
            for i in 0..9 {
                let a = i as f32 * 2.399;
                let start = target + Vec3::new(a.cos() * 44.0, a.sin() * 28.0, 0.0);
                fx.glow(
                    Vec2::splat(24.0 + (i % 3) as f32 * 7.0),
                    rgba(0.42, 0.92, 0.16, 0.68),
                    VfxNode::new(i as f32 * 0.025, 0.64, start, rgba(0.42, 0.92, 0.16, 0.68))
                        .scales(Vec2::splat(0.35), Vec2::splat(1.25))
                        .linear(start + Vec3::new(a.sin() * 22.0, 92.0, 0.0), 18.0, 8.0, 1.0),
                );
            }
        }
        StatusBurstKind::Burn => {
            fx.flash(target, 260.0, rgba(1.0, 0.22, 0.04, 0.58), 0.42);
            for i in 0..10 {
                let x = (i as f32 - 4.5) * 18.0;
                let start = target + Vec3::new(x, -46.0 + (i % 2) as f32 * 8.0, 0.0);
                fx.triangle(
                    rgba(1.0, 0.36 + (i % 3) as f32 * 0.12, 0.04, 0.90),
                    VfxNode::new(i as f32 * 0.018, 0.52, start, rgba(1.0, 0.42, 0.04, 0.90))
                        .scales(Vec2::new(8.0, 14.0), Vec2::new(18.0, 70.0))
                        .linear(start + Vec3::new(x * -0.14, 78.0, 0.0), 22.0, 7.0, 1.5)
                        .lively(0.12, 0.16, 0.2),
                );
            }
        }
        StatusBurstKind::Freeze => {
            fx.flash(target, 250.0, rgba(0.42, 0.88, 1.0, 0.58), 0.38);
            for i in 0..12 {
                let a = i as f32 * std::f32::consts::TAU / 12.0;
                let start = target + Vec3::new(a.cos() * 38.0, a.sin() * 30.0, 0.0);
                let end = target + Vec3::new(a.cos() * 145.0, a.sin() * 110.0, 0.0);
                fx.diamond(
                    rgba(0.70, 0.96, 1.0, 0.94),
                    VfxNode::new(i as f32 * 0.012, 0.48, start, rgba(0.70, 0.96, 1.0, 0.94))
                        .scales(Vec2::new(8.0, 22.0), Vec2::new(2.0, 8.0))
                        .linear(end, 18.0, 5.0, 1.0)
                        .rotates(-a, 1.4),
                );
            }
        }
        StatusBurstKind::Stun => {
            for i in 0..4 {
                let x = -70.0 + i as f32 * 46.0;
                let a = target + Vec3::new(x, 150.0, 0.0);
                let b = target + Vec3::new(x + 24.0, 20.0, 0.0);
                fx.line_between(
                    a,
                    b,
                    9.0,
                    rgba(0.72, 0.64, 1.0, 0.95),
                    i as f32 * 0.035,
                    0.44,
                    0.7,
                );
            }
            fx.flash(target, 235.0, rgba(0.58, 0.42, 1.0, 0.52), 0.34);
        }
        StatusBurstKind::Resist => {
            fx.hex_ring(
                rgba(0.84, 0.94, 1.0, 0.88),
                VfxNode::new(0.0, 0.52, target, rgba(0.84, 0.94, 1.0, 0.88))
                    .scales(Vec2::splat(150.0), Vec2::splat(76.0))
                    .rotates(0.0, -0.5),
            );
            fx.ring(
                rgba(0.38, 0.72, 1.0, 0.70),
                VfxNode::new(0.08, 0.48, target, rgba(0.38, 0.72, 1.0, 0.70))
                    .scales(Vec2::splat(120.0), Vec2::splat(182.0)),
            );
        }
    }
}

struct Emitter<'a, 'w, 's> {
    commands: &'a mut Commands<'w, 's>,
    materials: &'a mut Assets<ColorMaterial>,
    assets: &'a SkillVfxAssets,
    lights: &'a LightingAssets,
}

impl Emitter<'_, '_, '_> {
    fn mesh(&mut self, mesh: Handle<Mesh>, color: Color, node: VfxNode) -> Entity {
        self.commands
            .spawn((
                Mesh2d(mesh),
                MeshMaterial2d(self.materials.add(color)),
                Transform::from_translation(node.start).with_scale(Vec3::ZERO),
                node,
                DespawnOnExit(AppState::Battle),
            ))
            .id()
    }

    fn disc(&mut self, color: Color, node: VfxNode) -> Entity {
        self.mesh(self.assets.disc.clone(), color, node)
    }

    fn ring(&mut self, color: Color, node: VfxNode) -> Entity {
        self.textured(self.assets.ring.clone(), Vec2::splat(2.0), color, node)
    }

    fn thick_ring(&mut self, color: Color, node: VfxNode) -> Entity {
        self.textured(
            self.assets.thick_ring.clone(),
            Vec2::splat(2.0),
            color,
            node,
        )
    }

    fn hex(&mut self, color: Color, node: VfxNode) -> Entity {
        self.mesh(self.assets.hex.clone(), color, node)
    }

    fn hex_ring(&mut self, color: Color, node: VfxNode) -> Entity {
        self.mesh(self.assets.hex_ring.clone(), color, node)
    }

    fn diamond(&mut self, color: Color, node: VfxNode) -> Entity {
        self.mesh(self.assets.diamond.clone(), color, node)
    }

    fn triangle(&mut self, color: Color, node: VfxNode) -> Entity {
        self.textured(
            self.assets.triangle.clone(),
            Vec2::new(1.44, 2.0),
            color,
            node,
        )
    }

    fn textured(
        &mut self,
        image: Handle<Image>,
        size: Vec2,
        color: Color,
        node: VfxNode,
    ) -> Entity {
        self.commands
            .spawn((
                Sprite {
                    image,
                    custom_size: Some(size),
                    color,
                    ..default()
                },
                Transform::from_translation(node.start).with_scale(Vec3::ZERO),
                node,
                DespawnOnExit(AppState::Battle),
            ))
            .id()
    }

    fn glow(&mut self, size: Vec2, color: Color, node: VfxNode) -> Entity {
        self.commands
            .spawn((
                Sprite {
                    image: self.lights.orb.clone(),
                    color: Color::NONE,
                    custom_size: Some(size),
                    ..default()
                },
                Transform::from_translation(node.start).with_scale(Vec3::ZERO),
                VfxNode { color, ..node },
                DespawnOnExit(AppState::Battle),
            ))
            .id()
    }

    fn bar(&mut self, size: Vec2, color: Color, node: VfxNode) -> Entity {
        self.commands
            .spawn((
                Sprite::from_color(Color::NONE, size),
                Transform::from_translation(node.start).with_scale(Vec3::ZERO),
                VfxNode { color, ..node },
                DespawnOnExit(AppState::Battle),
            ))
            .id()
    }

    #[allow(clippy::too_many_arguments)]
    fn line_between(
        &mut self,
        a: Vec3,
        b: Vec3,
        width: f32,
        color: Color,
        delay: f32,
        duration: f32,
        flicker: f32,
    ) {
        let delta = b - a;
        let center = (a + b) * 0.5;
        let halo = color.with_alpha(color.alpha() * 0.34);
        self.glow(
            Vec2::new(delta.length(), width * 4.2),
            halo,
            VfxNode::new(delay, duration, center - Vec3::Z * 0.025, halo)
                .scales(Vec2::new(0.05, 0.30), Vec2::ONE)
                .rotates(delta.y.atan2(delta.x), 0.0)
                .lively(0.08, 0.12, flicker * 0.72)
                .envelope(0.05, 0.40),
        );
        self.bar(
            Vec2::new(delta.length(), width),
            color,
            VfxNode::new(delay, duration, center + Vec3::Z * 0.025, color)
                .scales(Vec2::new(0.05, 0.45), Vec2::ONE)
                .rotates(delta.y.atan2(delta.x), 0.0)
                .lively(0.06, 0.12, flicker)
                .envelope(0.06, 0.34),
        );
    }

    fn flash(&mut self, position: Vec3, size: f32, color: Color, duration: f32) {
        let entity = lighting::spawn_light(
            self.commands,
            self.lights,
            position - Vec3::Z * 0.5,
            size,
            color,
            AppState::Battle,
        );
        self.commands.entity(entity).insert((
            VfxNode::new(0.0, duration, position, color)
                .scales(Vec2::splat(0.22), Vec2::splat(1.18))
                .envelope(0.05, 0.62)
                .lively(0.08, 0.22, 0.0),
            VfxLight { intensity: 3.4 },
        ));
    }
}

fn sword_gleam(fx: &mut Emitter, caster: Vec3, target: Vec3) {
    let pale = rgba(0.72, 0.94, 1.0, 0.92);
    let white = rgba(1.0, 1.0, 1.0, 0.98);
    fx.ring(
        pale,
        VfxNode::new(0.0, 0.55, caster + Vec3::Y * 58.0, pale)
            .scales(Vec2::splat(18.0), Vec2::splat(74.0))
            .rotates(0.0, 1.1)
            .lively(0.08, 0.24, 0.0),
    );
    for i in 0..3 {
        let offset = Vec3::new(0.0, 30.0 + i as f32 * 13.0, i as f32 * 0.02);
        let start = caster + offset;
        let end = target + Vec3::new(0.0, 18.0 - i as f32 * 18.0, 0.0);
        fx.glow(
            Vec2::new(310.0 - i as f32 * 28.0, 18.0 - i as f32 * 3.0),
            if i == 1 { white } else { pale },
            VfxNode::new(0.12 + i as f32 * 0.07, 0.48, start, pale)
                .scales(Vec2::new(0.12, 0.55), Vec2::new(1.0, 0.9))
                .linear(end, 36.0 - i as f32 * 20.0, 8.0, 1.0)
                .rotates(-0.04 + i as f32 * 0.05, 0.0)
                .envelope(0.08, 0.46),
        );
    }
    for (i, rotation) in [0.68, -0.82].into_iter().enumerate() {
        fx.glow(
            Vec2::new(230.0, 18.0),
            white,
            VfxNode::new(0.39 + i as f32 * 0.04, 0.42, target, white)
                .scales(Vec2::new(0.18, 0.4), Vec2::new(1.15, 1.0))
                .rotates(rotation, 0.0)
                .lively(0.06, 0.16, 0.0),
        );
    }
    fx.ring(
        pale,
        VfxNode::new(0.42, 0.52, target, pale)
            .scales(Vec2::splat(28.0), Vec2::splat(150.0))
            .rotates(0.0, -0.45),
    );
    fx.flash(target, 250.0, pale, 0.42);
}

fn cinder_meteor(fx: &mut Emitter, _caster: Vec3, target: Vec3) {
    let ember = rgba(1.0, 0.18, 0.02, 0.94);
    let gold = rgba(1.0, 0.72, 0.12, 0.98);
    let start = target + Vec3::new(-330.0, 310.0, 0.0);
    for i in 0..9 {
        let p = i as f32 / 9.0;
        let pos = start.lerp(target, p);
        fx.glow(
            Vec2::splat(72.0 - i as f32 * 4.0),
            rgba(1.0, 0.22 + p * 0.34, 0.02, 0.34 + p * 0.35),
            VfxNode::new(i as f32 * 0.025, 0.52, pos, ember)
                .scales(Vec2::splat(0.25), Vec2::splat(1.15))
                .sway(Vec2::new(12.0, 8.0), 1.4),
        );
    }
    fx.hex(
        ember,
        VfxNode::new(0.0, 0.58, start, ember)
            .scales(Vec2::splat(24.0), Vec2::splat(46.0))
            .linear(target, 76.0, 6.0, 1.0)
            .rotates(0.2, 2.8)
            .lively(0.12, 0.16, 0.0),
    );
    for i in 0..12 {
        let angle = i as f32 * std::f32::consts::TAU / 12.0;
        let origin = target + Vec3::new(angle.cos() * 16.0, angle.sin() * 10.0, 0.0);
        let end = target + Vec3::new(angle.cos() * 150.0, angle.sin() * 102.0, 0.0);
        fx.diamond(
            if i % 2 == 0 { gold } else { ember },
            VfxNode::new(0.48 + i as f32 * 0.008, 0.55, origin, gold)
                .scales(Vec2::new(8.0, 14.0), Vec2::new(3.0, 7.0))
                .linear(end, 48.0, 5.0, 1.0)
                .rotates(-angle, 2.0),
        );
    }
    fx.thick_ring(
        ember,
        VfxNode::new(0.46, 0.70, target - Vec3::Y * 36.0, ember)
            .scales(Vec2::new(24.0, 10.0), Vec2::new(180.0, 72.0))
            .lively(0.08, 0.32, 0.0),
    );
    fx.flash(target, 420.0, rgba(1.0, 0.22, 0.02, 0.64), 0.58);
}

fn frost_lance(fx: &mut Emitter, caster: Vec3, target: Vec3) {
    let ice = rgba(0.34, 0.86, 1.0, 0.92);
    let core = rgba(0.92, 1.0, 1.0, 0.98);
    let hand = caster + Vec3::new(35.0, 66.0, 0.0);
    fx.disc(
        rgba(0.28, 0.76, 1.0, 0.34),
        VfxNode::new(0.0, 0.48, hand, ice)
            .scales(Vec2::splat(12.0), Vec2::splat(58.0))
            .lively(0.16, 0.24, 0.0),
    );
    fx.glow(
        Vec2::new(330.0, 24.0),
        core,
        VfxNode::new(0.10, 0.62, hand, core)
            .scales(Vec2::new(0.05, 0.4), Vec2::new(1.0, 1.0))
            .linear(target, 54.0, 5.0, 1.0)
            .rotates(0.02, 0.0),
    );
    for i in 0..10 {
        let p = (i as f32 + 1.0) / 11.0;
        let base = hand.lerp(target, p) + Vec3::new(0.0, -44.0 + (i % 2) as f32 * 13.0, 0.0);
        fx.triangle(
            if i % 3 == 0 { core } else { ice },
            VfxNode::new(0.12 + p * 0.38, 0.72, base, ice)
                .scales(
                    Vec2::new(3.0, 5.0),
                    Vec2::new(12.0 + p * 10.0, 38.0 + p * 74.0),
                )
                .rotates((i as f32 - 4.5) * 0.035, 0.0)
                .lively(0.05, 0.22, 0.0),
        );
    }
    fx.ring(
        core,
        VfxNode::new(0.48, 0.56, target, core)
            .scales(Vec2::splat(32.0), Vec2::splat(138.0))
            .rotates(0.0, 0.7),
    );
    fx.flash(target, 310.0, ice, 0.48);
}

fn storm_chain(fx: &mut Emitter, caster: Vec3, target: Vec3) {
    let violet = rgba(0.54, 0.38, 1.0, 0.90);
    let core = rgba(0.90, 0.94, 1.0, 0.98);
    let start = caster + Vec3::new(34.0, 76.0, 0.0);
    for strike in 0..3 {
        let delay = strike as f32 * 0.14;
        let mut points = vec![start];
        for i in 1..7 {
            let p = i as f32 / 7.0;
            let jitter = ((i * 13 + strike * 7) as f32 * 1.77).sin() * (34.0 - p * 10.0);
            points.push(start.lerp(target, p) + Vec3::Y * jitter);
        }
        points.push(target);
        for (i, pair) in points.windows(2).enumerate() {
            fx.line_between(
                pair[0],
                pair[1],
                if strike == 1 { 8.0 } else { 5.0 },
                if i % 2 == 0 { core } else { violet },
                delay + i as f32 * 0.012,
                0.34,
                0.9,
            );
        }
    }
    for i in 0..12 {
        let a = i as f32 * 2.17;
        let start = target + Vec3::new(a.cos() * 24.0, a.sin() * 20.0, 0.0);
        fx.glow(
            Vec2::new(36.0, 8.0),
            core,
            VfxNode::new(0.28 + i as f32 * 0.015, 0.52, start, core)
                .scales(Vec2::new(0.3, 0.5), Vec2::ONE)
                .linear(
                    start + Vec3::new(a.cos() * 132.0, a.sin() * 88.0, 0.0),
                    18.0,
                    8.0,
                    1.0,
                )
                .rotates(a, 0.0)
                .lively(0.0, 0.0, 0.55),
        );
    }
    fx.flash(target, 330.0, violet, 0.56);
}

fn wind_spiral(fx: &mut Emitter, caster: Vec3) {
    let mint = rgba(0.36, 1.0, 0.68, 0.82);
    let pale = rgba(0.84, 1.0, 0.92, 0.94);
    let center = caster + Vec3::Y * 56.0;
    for i in 0..16 {
        let angle = i as f32 * std::f32::consts::TAU / 16.0;
        fx.diamond(
            if i % 3 == 0 { pale } else { mint },
            VfxNode::new(i as f32 * 0.018, 0.90, center, mint)
                .scales(Vec2::new(4.0, 10.0), Vec2::new(10.0, 20.0))
                .orbit(center, Vec2::new(138.0, 18.0), angle, 1.35, 48.0)
                .rotates(angle, 2.6),
        );
    }
    for i in 0..3 {
        fx.ring(
            mint,
            VfxNode::new(
                i as f32 * 0.10,
                0.88,
                center - Vec3::Y * (36.0 - i as f32 * 32.0),
                mint,
            )
            .scales(
                Vec2::new(38.0, 14.0),
                Vec2::new(126.0 - i as f32 * 18.0, 42.0),
            )
            .rotates(0.0, if i % 2 == 0 { 1.8 } else { -1.5 })
            .lively(0.10, 0.25, 0.0),
        );
    }
    fx.flash(center, 255.0, mint, 0.65);
}

fn earth_rift(fx: &mut Emitter, caster: Vec3, target: Vec3) {
    let ochre = rgba(0.94, 0.58, 0.16, 0.90);
    let hot = rgba(1.0, 0.88, 0.42, 0.96);
    let start = caster + Vec3::new(30.0, -22.0, 0.0);
    let end = target - Vec3::Y * 58.0;
    let mut previous = start;
    for i in 1..9 {
        let p = i as f32 / 9.0;
        let point = start.lerp(end, p) + Vec3::Y * (((i * 17) as f32).sin() * 18.0);
        fx.line_between(
            previous,
            point,
            9.0,
            if i % 2 == 0 { hot } else { ochre },
            p * 0.30,
            0.72,
            0.0,
        );
        previous = point;
    }
    fx.line_between(previous, end, 10.0, hot, 0.30, 0.72, 0.0);
    for i in 0..7 {
        let x = (i as f32 - 3.0) * 34.0;
        let pos = end + Vec3::new(x, 6.0 + (i % 2) as f32 * 8.0, 0.0);
        fx.hex(
            if i % 2 == 0 { ochre } else { hot },
            VfxNode::new(0.28 + i as f32 * 0.035, 0.78, pos, ochre)
                .scales(
                    Vec2::new(6.0, 8.0),
                    Vec2::new(24.0, 54.0 + (3 - (i as i32 - 3).abs()) as f32 * 10.0),
                )
                .rotates(0.12 * (i as f32 - 3.0), 0.1)
                .lively(0.04, 0.30, 0.0),
        );
    }
    fx.thick_ring(
        ochre,
        VfxNode::new(0.34, 0.66, end, ochre).scales(Vec2::new(32.0, 12.0), Vec2::new(178.0, 58.0)),
    );
    fx.flash(target, 270.0, ochre, 0.44);
}

fn tidal_dragon(fx: &mut Emitter, caster: Vec3, target: Vec3) {
    let water = rgba(0.12, 0.58, 1.0, 0.78);
    let foam = rgba(0.76, 0.96, 1.0, 0.94);
    let start = caster + Vec3::new(30.0, 20.0, 0.0);
    for i in 0..18 {
        let p = i as f32 / 17.0;
        let pos =
            start.lerp(target, p) + Vec3::Y * ((p * std::f32::consts::TAU * 1.5).sin() * 62.0);
        fx.glow(
            Vec2::splat(48.0 + p * 42.0),
            if i % 4 == 0 { foam } else { water },
            VfxNode::new(p * 0.34, 0.76, pos, water)
                .scales(Vec2::splat(0.20), Vec2::splat(1.10))
                .linear(
                    pos + Vec3::new(72.0, (p * 13.0).sin() * 34.0, 0.0),
                    28.0,
                    9.0,
                    1.0,
                )
                .lively(0.08, 0.18, 0.0),
        );
    }
    fx.thick_ring(
        water,
        VfxNode::new(0.34, 0.82, target - Vec3::Y * 28.0, water)
            .scales(Vec2::new(36.0, 12.0), Vec2::new(196.0, 84.0))
            .rotates(0.0, 0.6)
            .lively(0.08, 0.28, 0.0),
    );
    for i in 0..9 {
        let a = i as f32 * std::f32::consts::PI / 8.0;
        let origin = target - Vec3::Y * 34.0;
        fx.glow(
            Vec2::new(18.0, 62.0),
            foam,
            VfxNode::new(0.40 + i as f32 * 0.018, 0.62, origin, foam)
                .scales(Vec2::new(0.4, 0.2), Vec2::ONE)
                .linear(
                    origin + Vec3::new(a.cos() * 160.0, a.sin() * 112.0, 0.0),
                    52.0,
                    6.0,
                    1.0,
                )
                .rotates(a - std::f32::consts::FRAC_PI_2, 0.0),
        );
    }
    fx.flash(target, 340.0, water, 0.58);
}

fn thorn_cage(fx: &mut Emitter, caster: Vec3, target: Vec3) {
    let vine = rgba(0.26, 0.86, 0.24, 0.88);
    let poison = rgba(0.66, 1.0, 0.18, 0.92);
    let start = caster + Vec3::new(34.0, -18.0, 0.0);
    let mut previous = start;
    for i in 1..10 {
        let p = i as f32 / 10.0;
        let point = start.lerp(target - Vec3::Y * 44.0, p)
            + Vec3::Y * ((p * std::f32::consts::TAU * 2.0).sin() * 22.0);
        fx.line_between(previous, point, 7.0, vine, p * 0.24, 0.82, 0.0);
        previous = point;
    }
    for i in 0..8 {
        let angle = i as f32 * std::f32::consts::TAU / 8.0;
        let base = target + Vec3::new(angle.cos() * 96.0, -48.0 + angle.sin() * 22.0, 0.0);
        fx.triangle(
            if i % 2 == 0 { poison } else { vine },
            VfxNode::new(0.28 + i as f32 * 0.025, 1.02, base, vine)
                .scales(
                    Vec2::new(5.0, 9.0),
                    Vec2::new(16.0, 108.0 + (i % 3) as f32 * 18.0),
                )
                .rotates(-angle.sin() * 0.24, 0.0)
                .lively(0.08, 0.32, 0.0),
        );
    }
    fx.hex_ring(
        poison,
        VfxNode::new(0.38, 0.90, target, poison)
            .scales(Vec2::splat(28.0), Vec2::splat(138.0))
            .rotates(0.0, 0.9)
            .lively(0.10, 0.22, 0.0),
    );
    fx.flash(target, 260.0, rgba(0.34, 0.94, 0.16, 0.46), 0.50);
}

fn moon_halo(fx: &mut Emitter, caster: Vec3) {
    let moon = rgba(1.0, 0.88, 0.38, 0.88);
    let pearl = rgba(1.0, 0.98, 0.82, 0.94);
    let center = caster + Vec3::Y * 56.0;
    fx.disc(
        rgba(1.0, 0.90, 0.48, 0.18),
        VfxNode::new(0.0, 1.12, center, moon)
            .scales(Vec2::splat(18.0), Vec2::splat(104.0))
            .lively(0.12, 0.34, 0.0),
    );
    fx.ring(
        pearl,
        VfxNode::new(0.02, 1.16, center, pearl)
            .scales(Vec2::splat(24.0), Vec2::splat(122.0))
            .rotates(0.0, 0.7)
            .lively(0.08, 0.38, 0.0),
    );
    fx.hex_ring(
        moon,
        VfxNode::new(0.16, 1.02, center, moon)
            .scales(Vec2::splat(32.0), Vec2::splat(148.0))
            .rotates(0.0, -0.42)
            .lively(0.06, 0.28, 0.0),
    );
    for i in 0..12 {
        let a = i as f32 * std::f32::consts::TAU / 12.0;
        fx.diamond(
            pearl,
            VfxNode::new(0.18 + i as f32 * 0.018, 0.86, center, pearl)
                .scales(Vec2::splat(3.0), Vec2::splat(9.0))
                .orbit(center, Vec2::new(52.0, 164.0), a, 0.32, 18.0)
                .rotates(a, 1.2),
        );
    }
    fx.flash(center, 290.0, moon, 0.62);
}

fn soul_lantern(fx: &mut Emitter, caster: Vec3, target: Vec3) {
    let soul = rgba(0.74, 0.24, 1.0, 0.82);
    let ghost = rgba(0.52, 1.0, 0.86, 0.90);
    fx.disc(
        rgba(0.20, 0.02, 0.28, 0.62),
        VfxNode::new(0.0, 0.92, target, soul)
            .scales(Vec2::splat(22.0), Vec2::splat(112.0))
            .lively(0.18, 0.18, 0.12),
    );
    fx.hex_ring(
        soul,
        VfxNode::new(0.02, 1.02, target, soul)
            .scales(Vec2::splat(38.0), Vec2::splat(128.0))
            .rotates(0.0, 1.4),
    );
    let end = caster + Vec3::new(24.0, 68.0, 0.0);
    for i in 0..14 {
        let angle = i as f32 * 2.399;
        let start = target + Vec3::new(angle.cos() * 74.0, angle.sin() * 62.0, 0.0);
        fx.glow(
            Vec2::splat(26.0 + (i % 4) as f32 * 6.0),
            if i % 3 == 0 { ghost } else { soul },
            VfxNode::new(0.22 + i as f32 * 0.026, 0.86, start, ghost)
                .scales(Vec2::splat(0.45), Vec2::splat(1.05))
                .linear(end, 90.0 + (i % 3) as f32 * 24.0, 18.0, 1.4)
                .lively(0.14, 0.12, 0.18),
        );
    }
    fx.ring(
        ghost,
        VfxNode::new(0.62, 0.56, end, ghost)
            .scales(Vec2::splat(22.0), Vec2::splat(94.0))
            .lively(0.08, 0.24, 0.0),
    );
    fx.flash(end, 220.0, ghost, 0.48);
}

fn karma_lotus(fx: &mut Emitter, target: Vec3) {
    let red = rgba(1.0, 0.10, 0.02, 0.88);
    let gold = rgba(1.0, 0.62, 0.08, 0.96);
    let base = target - Vec3::Y * 34.0;
    fx.disc(
        rgba(0.52, 0.02, 0.0, 0.46),
        VfxNode::new(0.0, 1.18, base, red)
            .scales(Vec2::new(12.0, 5.0), Vec2::new(132.0, 54.0))
            .lively(0.08, 0.28, 0.0),
    );
    for layer in 0..2 {
        for i in 0..8 {
            let a = i as f32 * std::f32::consts::TAU / 8.0 + layer as f32 * 0.39;
            let center =
                base + Vec3::new(a.cos() * (42.0 + layer as f32 * 28.0), a.sin() * 22.0, 0.0);
            fx.diamond(
                if layer == 0 { gold } else { red },
                VfxNode::new(
                    0.08 + layer as f32 * 0.12 + i as f32 * 0.014,
                    0.92,
                    center,
                    red,
                )
                .scales(
                    Vec2::new(3.0, 6.0),
                    Vec2::new(18.0, 48.0 - layer as f32 * 8.0),
                )
                .rotates(
                    a - std::f32::consts::FRAC_PI_2,
                    if layer == 0 { 0.8 } else { -0.6 },
                )
                .lively(0.10, 0.34, 0.08),
            );
        }
    }
    for i in 0..9 {
        let x = (i as f32 - 4.0) * 19.0;
        let origin = base + Vec3::new(x, 8.0, 0.0);
        fx.triangle(
            if i % 2 == 0 { gold } else { red },
            VfxNode::new(0.32 + i as f32 * 0.022, 0.76, origin, gold)
                .scales(
                    Vec2::new(6.0, 9.0),
                    Vec2::new(15.0, 92.0 + (i % 3) as f32 * 24.0),
                )
                .linear(origin + Vec3::new(x * -0.18, 46.0, 0.0), 18.0, 8.0, 1.0)
                .lively(0.08, 0.22, 0.2),
        );
    }
    fx.flash(target, 350.0, red, 0.62);
}

fn glacier_crown(fx: &mut Emitter, caster: Vec3, target: Vec3) {
    let blue = rgba(0.28, 0.72, 1.0, 0.84);
    let white = rgba(0.86, 1.0, 1.0, 0.96);
    let start_x = caster.x + 110.0;
    for i in 0..13 {
        let p = i as f32 / 12.0;
        let x = start_x.lerp(target.x + 116.0, p);
        let height = 40.0 + p.powf(1.65) * 150.0 + (i % 3) as f32 * 16.0;
        let base = Vec3::new(x, target.y - 72.0 + (i % 2) as f32 * 7.0, target.z);
        fx.triangle(
            if i % 3 == 0 { white } else { blue },
            VfxNode::new(0.04 + p * 0.48, 1.08 - p * 0.16, base, blue)
                .scales(Vec2::new(4.0, 5.0), Vec2::new(13.0 + p * 16.0, height))
                .rotates((i as f32 - 6.0) * 0.025, 0.0)
                .lively(0.05, 0.30, 0.0),
        );
        if i % 2 == 0 {
            fx.glow(
                Vec2::splat(62.0 + p * 28.0),
                rgba(0.54, 0.92, 1.0, 0.28),
                VfxNode::new(0.18 + p * 0.42, 1.10, base + Vec3::Y * 24.0, blue)
                    .scales(Vec2::splat(0.22), Vec2::splat(1.3))
                    .linear(base + Vec3::new(18.0, 112.0, 0.0), 20.0, 7.0, 1.0),
            );
        }
    }
    fx.thick_ring(
        white,
        VfxNode::new(0.46, 0.78, target - Vec3::Y * 52.0, white)
            .scales(Vec2::new(38.0, 12.0), Vec2::new(198.0, 66.0)),
    );
    fx.flash(target, 380.0, blue, 0.60);
}

fn thunder_prison(fx: &mut Emitter, target: Vec3) {
    let violet = rgba(0.58, 0.38, 1.0, 0.88);
    let white = rgba(0.94, 0.96, 1.0, 0.98);
    fx.thick_ring(
        violet,
        VfxNode::new(0.0, 1.14, target - Vec3::Y * 42.0, violet)
            .scales(Vec2::new(24.0, 8.0), Vec2::new(154.0, 52.0))
            .rotates(0.0, 1.0)
            .lively(0.08, 0.38, 0.44),
    );
    let mut tops = Vec::new();
    for i in 0..6 {
        let a = i as f32 * std::f32::consts::TAU / 6.0;
        let base = target + Vec3::new(a.cos() * 104.0, -48.0 + a.sin() * 25.0, 0.0);
        let top = base + Vec3::Y * (110.0 + (i % 2) as f32 * 20.0);
        tops.push(top);
        fx.line_between(
            base,
            top,
            7.0,
            if i % 2 == 0 { white } else { violet },
            0.16 + i as f32 * 0.035,
            0.88,
            0.78,
        );
    }
    for i in 0..tops.len() {
        fx.line_between(
            tops[i],
            tops[(i + 1) % tops.len()],
            5.0,
            violet,
            0.30 + i as f32 * 0.018,
            0.76,
            0.72,
        );
    }
    for strike in 0..3 {
        let x = (strike as f32 - 1.0) * 34.0;
        fx.line_between(
            target + Vec3::new(x - 36.0, 110.0, 0.0),
            target + Vec3::new(x + 12.0, -24.0, 0.0),
            11.0 - strike as f32 * 2.0,
            white,
            0.34 + strike as f32 * 0.12,
            0.42,
            0.92,
        );
    }
    fx.flash(target, 390.0, violet, 0.68);
}

fn blade_storm(fx: &mut Emitter, target: Vec3) {
    let mint = rgba(0.34, 1.0, 0.68, 0.84);
    let white = rgba(0.90, 1.0, 0.96, 0.96);
    for i in 0..18 {
        let a = i as f32 * std::f32::consts::TAU / 18.0;
        fx.glow(
            Vec2::new(94.0, 11.0),
            if i % 4 == 0 { white } else { mint },
            VfxNode::new(i as f32 * 0.018, 0.98, target, mint)
                .scales(Vec2::new(0.20, 0.35), Vec2::ONE)
                .orbit(
                    target,
                    Vec2::new(174.0, 18.0),
                    a,
                    1.18 + (i % 3) as f32 * 0.12,
                    (i % 4) as f32 * 12.0,
                )
                .rotates(a, 4.2)
                .lively(0.08, 0.16, 0.0),
        );
    }
    for i in 0..3 {
        fx.ring(
            if i == 1 { white } else { mint },
            VfxNode::new(0.22 + i as f32 * 0.10, 0.78, target, mint)
                .scales(
                    Vec2::splat(166.0 - i as f32 * 35.0),
                    Vec2::splat(34.0 + i as f32 * 12.0),
                )
                .rotates(0.0, if i % 2 == 0 { 1.6 } else { -1.9 }),
        );
    }
    fx.flash(target, 310.0, mint, 0.52);
}

fn mountain_aegis(fx: &mut Emitter, caster: Vec3) {
    let stone = rgba(0.82, 0.48, 0.16, 0.92);
    let rune = rgba(1.0, 0.84, 0.34, 0.96);
    let center = caster + Vec3::Y * 50.0;
    fx.hex(
        rgba(0.36, 0.16, 0.04, 0.62),
        VfxNode::new(0.0, 1.24, center, stone)
            .scales(Vec2::new(18.0, 22.0), Vec2::new(128.0, 156.0))
            .lively(0.04, 0.30, 0.0),
    );
    fx.hex_ring(
        rune,
        VfxNode::new(0.08, 1.18, center, rune)
            .scales(Vec2::new(24.0, 30.0), Vec2::new(142.0, 172.0))
            .rotates(0.0, 0.22)
            .lively(0.06, 0.32, 0.0),
    );
    for i in 0..7 {
        let x = (i as f32 - 3.0) * 34.0;
        let pos = center + Vec3::new(x, -84.0 + (i as f32 - 3.0).abs() * 9.0, 0.0);
        fx.diamond(
            if i % 2 == 0 { rune } else { stone },
            VfxNode::new(0.12 + (i as f32 - 3.0).abs() * 0.035, 1.02, pos, stone)
                .scales(
                    Vec2::new(5.0, 8.0),
                    Vec2::new(22.0, 58.0 + (3 - (i as i32 - 3).abs()) as f32 * 18.0),
                )
                .rotates((i as f32 - 3.0) * 0.12, 0.0)
                .lively(0.04, 0.28, 0.0),
        );
    }
    fx.thick_ring(
        rune,
        VfxNode::new(0.16, 0.88, center - Vec3::Y * 90.0, rune)
            .scales(Vec2::new(28.0, 8.0), Vec2::new(156.0, 38.0)),
    );
    fx.flash(center, 300.0, stone, 0.58);
}

fn whale_breath(fx: &mut Emitter, caster: Vec3, target: Vec3) {
    let deep = rgba(0.08, 0.42, 0.96, 0.74);
    let foam = rgba(0.62, 0.94, 1.0, 0.90);
    let start = caster + Vec3::new(26.0, 40.0, 0.0);
    for band in 0..4 {
        fx.glow(
            Vec2::new(360.0, 72.0 + band as f32 * 22.0),
            if band == 0 { foam } else { deep },
            VfxNode::new(
                band as f32 * 0.055,
                0.88,
                start - Vec3::Y * band as f32 * 10.0,
                deep,
            )
            .scales(Vec2::new(0.06, 0.28), Vec2::new(1.0, 1.0))
            .linear(
                target + Vec3::Y * (20.0 - band as f32 * 14.0),
                72.0 + band as f32 * 16.0,
                18.0,
                1.2,
            )
            .lively(0.12, 0.18, 0.0),
        );
    }
    fx.thick_ring(
        foam,
        VfxNode::new(0.32, 0.72, target, foam)
            .scales(Vec2::new(32.0, 16.0), Vec2::new(184.0, 92.0))
            .lively(0.08, 0.34, 0.0),
    );
    for i in 0..12 {
        let a = i as f32 * 2.399;
        let pos = target + Vec3::new(a.cos() * 86.0, a.sin() * 64.0, 0.0);
        fx.ring(
            foam,
            VfxNode::new(0.38 + i as f32 * 0.022, 0.78, pos, foam)
                .scales(Vec2::splat(4.0), Vec2::splat(16.0 + (i % 4) as f32 * 5.0))
                .linear(start, 84.0, 12.0, 1.0),
        );
    }
    fx.flash(target, 330.0, deep, 0.54);
}

fn world_tree_bloom(fx: &mut Emitter, caster: Vec3) {
    let green = rgba(0.28, 0.90, 0.30, 0.90);
    let leaf = rgba(0.70, 1.0, 0.48, 0.94);
    let gold = rgba(1.0, 0.90, 0.44, 0.90);
    let root = caster + Vec3::new(0.0, -48.0, 0.0);
    fx.bar(
        Vec2::new(26.0, 180.0),
        green,
        VfxNode::new(0.0, 1.22, root + Vec3::Y * 84.0, green)
            .scales(Vec2::new(0.35, 0.02), Vec2::ONE)
            .lively(0.05, 0.24, 0.0),
    );
    for i in 0..8 {
        let side = if i % 2 == 0 { -1.0 } else { 1.0 };
        let y = root.y + 52.0 + (i / 2) as f32 * 34.0;
        let a = Vec3::new(root.x, y, root.z);
        let b = a + Vec3::new(side * (62.0 + (i / 2) as f32 * 14.0), 32.0, 0.0);
        fx.line_between(a, b, 9.0, green, 0.18 + i as f32 * 0.035, 0.88, 0.0);
    }
    let crown = root + Vec3::Y * 170.0;
    for i in 0..20 {
        let a = i as f32 * 2.399;
        fx.diamond(
            if i % 5 == 0 { gold } else { leaf },
            VfxNode::new(0.24 + i as f32 * 0.018, 1.02, crown, leaf)
                .scales(Vec2::splat(3.0), Vec2::new(11.0, 18.0))
                .orbit(
                    crown,
                    Vec2::new(12.0, 112.0 + (i % 3) as f32 * 18.0),
                    a,
                    0.24,
                    -40.0,
                )
                .rotates(a, 1.2)
                .lively(0.10, 0.18, 0.0),
        );
    }
    fx.ring(
        gold,
        VfxNode::new(0.42, 0.72, caster + Vec3::Y * 54.0, gold)
            .scales(Vec2::splat(24.0), Vec2::splat(142.0))
            .lively(0.08, 0.32, 0.0),
    );
    fx.flash(caster + Vec3::Y * 72.0, 310.0, green, 0.62);
}

fn starfall(fx: &mut Emitter, target: Vec3) {
    let gold = rgba(1.0, 0.78, 0.22, 0.94);
    let white = rgba(1.0, 1.0, 0.88, 0.98);
    for i in 0..7 {
        let x = (i as f32 - 3.0) * 48.0;
        let start = target + Vec3::new(x - 170.0, 70.0 + (i % 3) as f32 * 20.0, 0.0);
        let end = target + Vec3::new(x * 0.55, -24.0 + (i % 2) as f32 * 18.0, 0.0);
        fx.glow(
            Vec2::new(190.0, 20.0),
            if i % 2 == 0 { white } else { gold },
            VfxNode::new(i as f32 * 0.10, 0.56, start, gold)
                .scales(Vec2::new(0.18, 0.44), Vec2::ONE)
                .linear(end, 48.0, 8.0, 1.0)
                .rotates(-0.92, 0.0)
                .lively(0.08, 0.12, 0.12),
        );
        fx.diamond(
            white,
            VfxNode::new(i as f32 * 0.10, 0.56, start, white)
                .scales(Vec2::new(7.0, 15.0), Vec2::new(12.0, 24.0))
                .linear(end, 48.0, 8.0, 1.0)
                .rotates(-0.92, 1.8)
                .lively(0.08, 0.16, 0.08),
        );
        fx.ring(
            gold,
            VfxNode::new(0.42 + i as f32 * 0.10, 0.56, end, gold)
                .scales(Vec2::splat(12.0), Vec2::splat(72.0 + i as f32 * 6.0))
                .lively(0.05, 0.24, 0.0),
        );
    }
    fx.thick_ring(
        white,
        VfxNode::new(1.02, 0.68, target, white)
            .scales(Vec2::splat(32.0), Vec2::splat(194.0))
            .rotates(0.0, 0.6)
            .lively(0.06, 0.30, 0.0),
    );
    fx.flash(target, 470.0, gold, 0.78);
}

fn eclipse(fx: &mut Emitter, target: Vec3) {
    let dark = rgba(0.08, 0.0, 0.12, 0.88);
    let violet = rgba(0.76, 0.20, 0.94, 0.88);
    let acid = rgba(0.50, 1.0, 0.16, 0.84);
    fx.disc(
        dark,
        VfxNode::new(0.0, 1.36, target, dark)
            .scales(Vec2::splat(20.0), Vec2::splat(116.0))
            .lively(0.10, 0.26, 0.14),
    );
    fx.thick_ring(
        violet,
        VfxNode::new(0.06, 1.30, target, violet)
            .scales(Vec2::splat(146.0), Vec2::splat(104.0))
            .rotates(0.0, -0.48)
            .lively(0.12, 0.18, 0.18),
    );
    for i in 0..12 {
        let a = i as f32 * std::f32::consts::TAU / 12.0;
        let outer = target + Vec3::new(a.cos() * 150.0, a.sin() * 108.0, 0.0);
        let inner = target + Vec3::new(a.cos() * 42.0, a.sin() * 30.0, 0.0);
        fx.line_between(
            outer,
            inner,
            8.0,
            if i % 3 == 0 { acid } else { violet },
            0.18 + i as f32 * 0.025,
            0.96,
            0.18,
        );
    }
    for i in 0..9 {
        let a = i as f32 * 2.399;
        let pos = target + Vec3::new(a.cos() * 72.0, a.sin() * 48.0, 0.0);
        fx.glow(
            Vec2::splat(34.0),
            acid,
            VfxNode::new(0.42 + i as f32 * 0.034, 0.82, pos, acid)
                .scales(Vec2::splat(0.25), Vec2::splat(1.15))
                .linear(pos + Vec3::new(a.sin() * 34.0, 104.0, 0.0), 22.0, 12.0, 1.0),
        );
    }
    fx.flash(target, 360.0, violet, 0.64);
}

fn sword_array(fx: &mut Emitter, caster: Vec3, target: Vec3) {
    let blue = rgba(0.58, 0.86, 1.0, 0.90);
    let white = rgba(1.0, 1.0, 1.0, 0.98);
    let center = target - Vec3::Y * 6.0;
    fx.ring(
        blue,
        VfxNode::new(0.0, 1.32, center, blue)
            .scales(Vec2::splat(22.0), Vec2::splat(148.0))
            .rotates(0.0, 1.1)
            .lively(0.08, 0.34, 0.0),
    );
    for i in 0..12 {
        let a = i as f32 * std::f32::consts::TAU / 12.0;
        let start = center + Vec3::new(a.cos() * 150.0, a.sin() * 116.0, 0.0);
        fx.glow(
            Vec2::new(118.0, 13.0),
            if i % 3 == 0 { white } else { blue },
            VfxNode::new(0.10 + i as f32 * 0.025, 1.04, start, blue)
                .scales(Vec2::new(0.22, 0.44), Vec2::ONE)
                .linear(center, 34.0 + (i % 3) as f32 * 20.0, 5.0, 1.0)
                .rotates(a + std::f32::consts::PI, 0.4)
                .envelope(0.08, 0.18)
                .lively(0.06, 0.14, 0.0),
        );
    }
    let hand = caster + Vec3::new(34.0, 64.0, 0.0);
    fx.glow(
        Vec2::new(520.0, 30.0),
        white,
        VfxNode::new(0.70, 0.62, hand, white)
            .scales(Vec2::new(0.02, 0.34), Vec2::ONE)
            .linear(center, 42.0, 4.0, 1.0)
            .lively(0.04, 0.18, 0.0),
    );
    fx.hex_ring(
        white,
        VfxNode::new(0.88, 0.62, center, white)
            .scales(Vec2::splat(32.0), Vec2::splat(136.0))
            .rotates(0.0, -1.2)
            .lively(0.04, 0.30, 0.0),
    );
    fx.flash(center, 490.0, blue, 0.72);
}

fn sync_status_auras(
    mut commands: Commands,
    status: Res<StatusVfxState>,
    existing: Query<(Entity, &StatusAura)>,
    assets: Res<SkillVfxAssets>,
    lights: Res<LightingAssets>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let active = [
        (StatusAura::Poison, status.poison_stacks > 0),
        (StatusAura::Burn, status.burn_turns > 0),
        (StatusAura::Freeze, status.frozen),
        (StatusAura::Stun, status.stunned),
    ];

    for (entity, aura) in &existing {
        if !active
            .iter()
            .any(|(kind, enabled)| kind == aura && *enabled)
        {
            commands.entity(entity).despawn();
        }
    }

    for (kind, enabled) in active {
        if enabled && !existing.iter().any(|(_, aura)| *aura == kind) {
            spawn_status_aura(
                &mut commands,
                &mut materials,
                &assets,
                &lights,
                kind,
                super::battle::ENEMY_POS + Vec3::new(0.0, 0.0, 2.2),
            );
        }
    }
}

fn spawn_status_aura(
    commands: &mut Commands,
    materials: &mut Assets<ColorMaterial>,
    assets: &SkillVfxAssets,
    lights: &LightingAssets,
    kind: StatusAura,
    target: Vec3,
) {
    let mut fx = Emitter {
        commands,
        materials,
        assets,
        lights,
    };
    match kind {
        StatusAura::Poison => {
            for i in 0..6 {
                let a = i as f32 * std::f32::consts::TAU / 6.0;
                let entity = fx.glow(
                    Vec2::splat(34.0 + (i % 3) as f32 * 8.0),
                    rgba(0.44, 0.94, 0.14, 0.46),
                    VfxNode::new(i as f32 * 0.12, 1.18, target, rgba(0.44, 0.94, 0.14, 0.46))
                        .scales(Vec2::splat(0.42), Vec2::splat(1.16))
                        .orbit(target, Vec2::new(62.0, 88.0), a, 0.44, 44.0)
                        .lively(0.18, 0.0, 0.08)
                        .repeating(),
                );
                fx.commands.entity(entity).insert(kind);
            }
        }
        StatusAura::Burn => {
            for i in 0..7 {
                let x = (i as f32 - 3.0) * 24.0;
                let pos = target + Vec3::new(x, -62.0, 0.0);
                let entity = fx.triangle(
                    rgba(1.0, 0.28 + (i % 3) as f32 * 0.14, 0.03, 0.66),
                    VfxNode::new(i as f32 * 0.10, 0.92, pos, rgba(1.0, 0.34, 0.03, 0.66))
                        .scales(
                            Vec2::new(7.0, 13.0),
                            Vec2::new(15.0, 62.0 + (i % 3) as f32 * 20.0),
                        )
                        .linear(pos + Vec3::new(x * -0.12, 56.0, 0.0), 16.0, 7.0, 1.0)
                        .lively(0.12, 0.14, 0.20)
                        .repeating(),
                );
                fx.commands.entity(entity).insert(kind);
            }
        }
        StatusAura::Freeze => {
            let entity = fx.thick_ring(
                rgba(0.36, 0.82, 1.0, 0.54),
                VfxNode::new(0.0, 1.22, target, rgba(0.36, 0.82, 1.0, 0.54))
                    .scales(Vec2::new(112.0, 74.0), Vec2::new(124.0, 82.0))
                    .rotates(0.0, 0.18)
                    .lively(0.08, 0.0, 0.0)
                    .repeating(),
            );
            fx.commands.entity(entity).insert(kind);
            for i in 0..6 {
                let a = i as f32 * std::f32::consts::TAU / 6.0;
                let pos = target + Vec3::new(a.cos() * 92.0, a.sin() * 72.0, 0.0);
                let entity = fx.triangle(
                    rgba(0.72, 0.96, 1.0, 0.68),
                    VfxNode::new(i as f32 * 0.08, 1.10, pos, rgba(0.72, 0.96, 1.0, 0.68))
                        .scales(Vec2::new(11.0, 34.0), Vec2::new(14.0, 52.0))
                        .rotates(-a * 0.4, 0.10)
                        .lively(0.10, 0.0, 0.0)
                        .repeating(),
                );
                fx.commands.entity(entity).insert(kind);
            }
        }
        StatusAura::Stun => {
            for i in 0..3 {
                let pos = target + Vec3::new((i as f32 - 1.0) * 44.0, 132.0, 0.0);
                let entity = fx.glow(
                    Vec2::new(72.0, 9.0),
                    rgba(0.68, 0.52, 1.0, 0.72),
                    VfxNode::new(i as f32 * 0.11, 0.66, pos, rgba(0.68, 0.52, 1.0, 0.72))
                        .scales(Vec2::new(0.35, 0.5), Vec2::ONE)
                        .rotates(if i % 2 == 0 { 0.62 } else { -0.72 }, 0.0)
                        .lively(0.10, 0.0, 0.68)
                        .repeating(),
                );
                fx.commands.entity(entity).insert(kind);
            }
        }
    }
}

fn update_vfx_nodes(
    time: Res<Time>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut nodes: Query<(
        Entity,
        &mut VfxNode,
        &mut Transform,
        Option<&mut Sprite>,
        Option<&MeshMaterial2d<ColorMaterial>>,
        Option<&mut PointLight2d>,
        Option<&VfxLight>,
    )>,
) {
    for (entity, mut node, mut transform, sprite, material, point_light, light) in &mut nodes {
        node.age += time.delta_secs();
        if node.age < node.delay {
            continue;
        }
        let elapsed = node.age - node.delay;
        if !node.repeating && elapsed >= node.duration {
            commands.entity(entity).despawn();
            continue;
        }
        let local = if node.repeating {
            elapsed.rem_euclid(node.duration)
        } else {
            elapsed
        };
        let p = (local / node.duration).clamp(0.0, 1.0);
        let eased = 1.0 - (1.0 - p).powi(3);
        let scale_p = if node.overshoot > 0.0 {
            let q = p - 1.0;
            1.0 + (node.overshoot + 1.0) * q.powi(3) + node.overshoot * q.powi(2)
        } else {
            eased
        };
        let pulse = 1.0 + node.pulse * (p * std::f32::consts::TAU * 2.0).sin();
        let scale = node.start_scale.lerp(node.end_scale, scale_p) * pulse;
        transform.scale = Vec3::new(scale.x.max(0.001), scale.y.max(0.001), 1.0);
        transform.rotation = Quat::from_rotation_z(node.rotation + node.spin * eased);

        transform.translation = match node.motion {
            Motion::Static { sway, cycles } => {
                let wave = (p * std::f32::consts::TAU * cycles + node.seed).sin();
                node.start + Vec3::new(sway.x * wave, sway.y * wave, 0.0)
            }
            Motion::Linear { arc, wave, cycles } => {
                let tangent = (node.end - node.start).truncate().normalize_or_zero();
                let normal = Vec2::new(-tangent.y, tangent.x);
                let bowed = arc * 4.0 * p * (1.0 - p);
                let wobble = wave * (p * std::f32::consts::TAU * cycles).sin();
                node.start.lerp(node.end, eased)
                    + Vec3::new(normal.x * wobble, bowed + normal.y * wobble, 0.0)
            }
            Motion::Orbit {
                center,
                radius_start,
                radius_end,
                angle,
                turns,
                lift,
            } => {
                let radius = radius_start.lerp(radius_end, eased);
                let theta = angle + turns * std::f32::consts::TAU * eased;
                center
                    + Vec3::new(
                        theta.cos() * radius,
                        theta.sin() * radius + lift * eased,
                        0.0,
                    )
            }
        };

        let fade_in = if node.fade_in <= 0.0 {
            1.0
        } else {
            (p / node.fade_in).clamp(0.0, 1.0)
        };
        let fade_out = if node.fade_out <= 0.0 {
            1.0
        } else {
            ((1.0 - p) / node.fade_out).clamp(0.0, 1.0)
        };
        let flicker = if node.flicker > 0.0 {
            1.0 - node.flicker
                * (0.5
                    + 0.5
                        * (elapsed * 47.0 + node.seed * 9.0).sin()
                        * (elapsed * 83.0 + node.seed).cos())
        } else {
            1.0
        };
        let alpha = node.color.alpha() * fade_in * fade_out * flicker;
        let color = node.color.with_alpha(alpha.clamp(0.0, 1.0));
        if let Some(mut sprite) = sprite {
            sprite.color = color;
        }
        if let Some(material) = material
            && let Some(mut material) = materials.get_mut(&material.0)
        {
            material.color = color;
        }
        if let (Some(mut point_light), Some(light)) = (point_light, light) {
            let srgba = node.color.to_srgba();
            point_light.color = Color::srgb(srgba.red, srgba.green, srgba.blue);
            point_light.intensity = light.intensity * fade_in * fade_out * flicker;
        }
    }
}

fn rgba(red: f32, green: f32, blue: f32, alpha: f32) -> Color {
    Color::srgba(red, green, blue, alpha)
}

#[cfg(test)]
mod tests {
    use crate::game::roguelike::skill::{ALL_SKILLS, skill};

    #[test]
    fn every_curated_skill_has_a_unique_vfx_script() {
        let styles: std::collections::HashSet<_> =
            ALL_SKILLS.into_iter().map(|id| skill(id).vfx).collect();
        assert_eq!(styles.len(), 20);
    }
}

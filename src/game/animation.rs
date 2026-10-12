use bevy::prelude::*;

use super::state::AppState;

#[derive(Clone)]
struct AtlasClipAsset {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

#[derive(Resource, Clone)]
pub struct AnimationAssets {
    hero_idle: AtlasClipAsset,
    hero_walk: AtlasClipAsset,
    hero_attack: AtlasClipAsset,
    monster: AtlasClipAsset,
    skill_vfx: AtlasClipAsset,
}

impl AnimationAssets {
    /// Dummy visual handles for CPU-only gameplay integration tests.
    #[cfg(test)]
    pub(crate) fn simulation() -> Self {
        let clip = AtlasClipAsset {
            image: Handle::default(),
            layout: Handle::default(),
        };
        Self {
            hero_idle: clip.clone(),
            hero_walk: clip.clone(),
            hero_attack: clip.clone(),
            monster: clip.clone(),
            skill_vfx: clip,
        }
    }

    pub fn load(assets: &AssetServer, layouts: &mut Assets<TextureAtlasLayout>) -> Self {
        Self {
            hero_idle: AtlasClipAsset {
                image: assets.load("actors/hero-motion.png"),
                layout: layouts.add(TextureAtlasLayout::from_grid(
                    UVec2::splat(256),
                    4,
                    6,
                    None,
                    None,
                )),
            },
            hero_walk: AtlasClipAsset {
                image: assets.load("actors/hero-motion.png"),
                layout: layouts.add(TextureAtlasLayout::from_grid(
                    UVec2::splat(256),
                    4,
                    6,
                    None,
                    None,
                )),
            },
            hero_attack: AtlasClipAsset {
                image: assets.load("actors/hero-motion.png"),
                layout: layouts.add(TextureAtlasLayout::from_grid(
                    UVec2::splat(256),
                    4,
                    6,
                    None,
                    None,
                )),
            },
            monster: AtlasClipAsset {
                image: assets.load("actors/monster-actions-sheet.png"),
                layout: layouts.add(TextureAtlasLayout::from_grid(
                    UVec2::splat(512),
                    8,
                    4,
                    None,
                    None,
                )),
            },
            skill_vfx: AtlasClipAsset {
                image: assets.load("effects/skill-vfx-sheet.png"),
                layout: layouts.add(TextureAtlasLayout::from_grid(
                    UVec2::splat(512),
                    4,
                    1,
                    None,
                    None,
                )),
            },
        }
    }

    fn asset(&self, clip: AnimationClip) -> &AtlasClipAsset {
        match clip {
            AnimationClip::HeroIdle => &self.hero_idle,
            AnimationClip::HeroWalk => &self.hero_walk,
            AnimationClip::HeroAttack => &self.hero_attack,
            AnimationClip::MonsterIdle | AnimationClip::MonsterAttack => &self.monster,
            AnimationClip::SkillImpact => &self.skill_vfx,
        }
    }

    pub fn sprite(&self, clip: AnimationClip, size: Vec2) -> Sprite {
        let asset = self.asset(clip);
        Sprite {
            image: asset.image.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: asset.layout.clone(),
                index: clip.first_frame(),
            }),
            custom_size: Some(size),
            ..default()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationClip {
    HeroIdle,
    HeroWalk,
    HeroAttack,
    MonsterIdle,
    MonsterAttack,
    SkillImpact,
}

impl AnimationClip {
    fn first_frame(self) -> usize {
        match self {
            Self::HeroIdle | Self::MonsterIdle | Self::SkillImpact => 0,
            Self::HeroWalk => 8,
            Self::HeroAttack | Self::MonsterAttack => 16,
        }
    }

    fn frame_count(self) -> usize {
        match self {
            Self::HeroIdle
            | Self::HeroWalk
            | Self::HeroAttack
            | Self::MonsterIdle
            | Self::MonsterAttack => 8,
            Self::SkillImpact => 4,
        }
    }

    fn fps(self) -> f32 {
        match self {
            Self::HeroIdle | Self::MonsterIdle => 6.0,
            Self::HeroWalk | Self::HeroAttack | Self::MonsterAttack => 12.0,
            Self::SkillImpact => 14.0,
        }
    }

    pub fn duration(self) -> f32 {
        self.frame_count() as f32 / self.fps()
    }
}

#[derive(Component)]
pub struct SpriteAnimation {
    clip: AnimationClip,
    frame: usize,
    elapsed: f32,
    repeat: bool,
}

impl SpriteAnimation {
    pub fn new(clip: AnimationClip) -> Self {
        Self {
            clip,
            frame: 0,
            elapsed: 0.0,
            repeat: true,
        }
    }

    pub fn once(clip: AnimationClip) -> Self {
        Self {
            clip,
            frame: 0,
            elapsed: 0.0,
            repeat: false,
        }
    }
}

#[derive(Component)]
pub struct Bobbing {
    base_y: f32,
    amplitude: f32,
    speed: f32,
    phase: f32,
}

impl Bobbing {
    pub fn new(base_y: f32, amplitude: f32, speed: f32, phase: f32) -> Self {
        Self {
            base_y,
            amplitude,
            speed,
            phase,
        }
    }
}

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (tick_sprite_animations, update_bobbing));
    }
}

pub fn spawn_animated_sprite(
    commands: &mut Commands,
    assets: &AnimationAssets,
    clip: AnimationClip,
    pos: Vec3,
    size: Vec2,
    exit_state: AppState,
) -> Entity {
    commands
        .spawn((
            assets.sprite(clip, size),
            SpriteAnimation::new(clip),
            Transform::from_translation(pos),
            DespawnOnExit(exit_state),
        ))
        .id()
}

pub fn set_clip(
    assets: &AnimationAssets,
    sprite: &mut Sprite,
    animation: &mut SpriteAnimation,
    clip: AnimationClip,
) {
    if animation.clip == clip {
        return;
    }

    let asset = assets.asset(clip);
    animation.clip = clip;
    animation.frame = 0;
    animation.elapsed = 0.0;
    animation.repeat = !matches!(
        clip,
        AnimationClip::HeroAttack | AnimationClip::MonsterAttack
    );
    sprite.image = asset.image.clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: asset.layout.clone(),
        index: clip.first_frame(),
    });
}

fn tick_sprite_animations(time: Res<Time>, mut query: Query<(&mut SpriteAnimation, &mut Sprite)>) {
    for (mut animation, mut sprite) in &mut query {
        animation.elapsed += time.delta_secs();
        let frame_time = 1.0 / animation.clip.fps();
        if animation.elapsed < frame_time {
            continue;
        }

        let steps = (animation.elapsed / frame_time).floor() as usize;
        animation.elapsed -= frame_time * steps as f32;
        let frame_count = animation.clip.frame_count();
        let next_frame = animation.frame + steps;
        animation.frame = if animation.repeat {
            next_frame % frame_count
        } else {
            next_frame.min(frame_count - 1)
        };

        if let Some(atlas) = sprite.texture_atlas.as_mut() {
            atlas.index = animation.clip.first_frame() + animation.frame;
        }
    }
}

fn update_bobbing(time: Res<Time>, mut query: Query<(&Bobbing, &mut Transform)>) {
    let t = time.elapsed_secs();
    for (bobbing, mut transform) in &mut query {
        transform.translation.y =
            bobbing.base_y + (t * bobbing.speed + bobbing.phase).sin() * bobbing.amplitude;
    }
}

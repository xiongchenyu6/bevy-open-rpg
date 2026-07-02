use bevy::prelude::*;
use bevy_firefly::prelude::*;
use bevy_fog_of_war::prelude::VisionSource;

use super::state::AppState;

pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FireflyPlugin);
    }
}

#[derive(Resource)]
pub struct LightingAssets {
    pub orb: Handle<Image>,
}

impl LightingAssets {
    pub fn load(assets: &AssetServer) -> Self {
        Self {
            orb: assets.load("effects/light_orb.png"),
        }
    }
}

pub fn camera_config() -> FireflyConfig {
    FireflyConfig {
        ambient_color: Color::srgb(0.86, 0.90, 1.0),
        ambient_brightness: 0.42,
        light_bands: None,
        soft_shadows: true,
        z_sorting: true,
        z_sorting_error_margin: 0.06,
        normal_mode: NormalMode::None,
        normal_attenuation: 0.85,
        combination_mode: CombinationMode::Multiply,
        lightmap_size: LightmapSize::Scaled(1.0),
        lightmap_filtering: true,
        enable_32bit_stencils: false,
    }
}

pub fn spawn_light(
    commands: &mut Commands,
    assets: &LightingAssets,
    pos: Vec3,
    size: f32,
    color: Color,
    exit_state: AppState,
) -> Entity {
    let srgba = color.to_srgba();
    let light_color = Color::srgb(srgba.red, srgba.green, srgba.blue);
    let radius = size * 0.72;
    let intensity = 0.65 + srgba.alpha * 3.0;
    let mut vision = VisionSource::circle(radius * 0.92);
    vision.transition_ratio = 0.34;

    commands
        .spawn((
            PointLight2d {
                color: light_color,
                intensity,
                radius,
                falloff: Falloff::linear(-0.28),
                core: LightCore::from_radius_boost((radius * 0.14).clamp(14.0, 58.0), 1.65),
                cast_shadows: true,
                ..default()
            },
            LightHeight((radius * 0.2).clamp(28.0, 110.0)),
            vision,
            Sprite {
                image: assets.orb.clone(),
                color,
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_translation(pos),
            DespawnOnExit(exit_state),
        ))
        .id()
}

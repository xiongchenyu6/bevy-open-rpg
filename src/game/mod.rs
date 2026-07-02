use bevy::prelude::*;

pub mod animation;
pub mod battle;
pub mod core;
pub mod explore;
pub mod fog;
pub mod lighting;
pub mod paperdoll;
pub mod quest;
pub mod state;

pub use core::{EncounterRate, Intent, PlayerStats, Rng};
pub use state::AppState;

use core::GameFont;
use lighting::LightingAssets;
use paperdoll::PaperdollAssets;
use quest::QuestLog;

/// Core game: state, shared resources, scene plugins. Deliberately does NOT
/// spawn a camera or read input — those differ between the desktop build and
/// the offscreen capture build, which both reuse this plugin.
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_paperdoll::PaperdollPlugin)
            .add_plugins(lighting::LightingPlugin);

        // Load the font here (AssetServer already exists, since DefaultPlugins
        // is added first) so `GameFont` is present before the initial
        // OnEnter(Explore) runs — a Startup system would apply too late.
        let assets = app.world().resource::<AssetServer>().clone();
        let font = assets.load("fonts/unifont.otf");
        let paperdoll_assets = PaperdollAssets::load(&assets);
        let lighting_assets = LightingAssets::load(&assets);
        let animation_assets = {
            let mut layouts = app.world_mut().resource_mut::<Assets<TextureAtlasLayout>>();
            animation::AnimationAssets::load(&assets, &mut layouts)
        };
        app.insert_resource(GameFont(font))
            .insert_resource(paperdoll_assets)
            .insert_resource(lighting_assets)
            .insert_resource(animation_assets)
            .init_state::<AppState>()
            .init_resource::<PlayerStats>()
            .init_resource::<Rng>()
            .init_resource::<Intent>()
            .init_resource::<EncounterRate>()
            .init_resource::<QuestLog>()
            .add_plugins(animation::AnimationPlugin)
            .add_plugins(paperdoll::PaperdollRuntimePlugin)
            .add_plugins(explore::ExplorePlugin)
            .add_plugins(battle::BattlePlugin)
            .add_plugins(fog::FogPlugin);
    }
}

/// Desktop wiring: a windowed camera plus keyboard → [`Intent`] each frame.
pub struct DesktopInputPlugin;

impl Plugin for DesktopInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(PreUpdate, gather_keyboard_intent);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, lighting::camera_config()));
}

fn gather_keyboard_intent(keys: Res<ButtonInput<KeyCode>>, mut intent: ResMut<Intent>) {
    intent.clear();

    intent.move_dir = if keys.pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::KeyW) {
        Some(IVec2::new(0, -1))
    } else if keys.pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::KeyS) {
        Some(IVec2::new(0, 1))
    } else if keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA) {
        Some(IVec2::new(-1, 0))
    } else if keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD) {
        Some(IVec2::new(1, 0))
    } else {
        None
    };

    intent.confirm = keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter);
    intent.up = keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyW);
    intent.down = keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyS);
}

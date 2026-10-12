use bevy::input::{InputSystems, keyboard::Key};
use bevy::prelude::*;

pub mod animation;
pub mod battle;
pub mod core;
pub mod cutout;
pub mod explore;
pub mod fog;
pub mod lighting;
pub mod paperdoll;
pub mod quest;
pub mod roguelike;
pub mod skill_vfx;
pub mod state;
pub mod ui;
pub mod ui_scroll;

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
            .add_plugins(lighting::LightingPlugin)
            .add_plugins(ui_scroll::UiScrollPlugin)
            .add_systems(Update, ui::scale_to_viewport)
            .add_systems(
                PostUpdate,
                ui::follow_contact_shadows.before(bevy::transform::TransformSystems::Propagate),
            );

        // Load the font here (AssetServer already exists, since DefaultPlugins
        // is added first) so `GameFont` is present before the initial
        // OnEnter(Explore) runs — a Startup system would apply too late.
        let assets = app.world().resource::<AssetServer>().clone();
        let font = assets.load("fonts/NotoSansCJKsc-Regular.otf");
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
            .add_plugins(skill_vfx::SkillVfxPlugin)
            .add_plugins(battle::BattlePlugin)
            .add_plugins(fog::FogPlugin)
            .add_plugins(roguelike::RoguelikePlugin);
    }
}

/// Desktop wiring: a windowed camera plus keyboard → [`Intent`] each frame.
pub struct DesktopInputPlugin;

impl Plugin for DesktopInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(PreUpdate, gather_keyboard_intent.after(InputSystems));
    }
}

fn spawn_camera(mut commands: Commands) {
    // Fixed 1280x720 world view scaled to the window, so the game fills any
    // window size instead of floating in a black frame.
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::Fixed {
                width: 1280.0,
                height: 720.0,
            },
            ..OrthographicProjection::default_2d()
        }),
        lighting::camera_config(),
    ));
}

fn gather_keyboard_intent(
    keys: Res<ButtonInput<KeyCode>>,
    logical: Res<ButtonInput<Key>>,
    mut intent: ResMut<Intent>,
) {
    intent.clear();
    // Virtual keyboards upload their own keymap: their physical code can be
    // Escape while their logical symbol is Space. Prefer recognized logical
    // controls so such a press does not also open the inventory.
    let recognized = |key: &Key| match key {
        Key::Space
        | Key::Enter
        | Key::Escape
        | Key::ArrowUp
        | Key::ArrowDown
        | Key::ArrowLeft
        | Key::ArrowRight => true,
        Key::Character(s) => matches!(s.to_ascii_lowercase().as_str(), "w" | "a" | "s" | "d" | " "),
        _ => false,
    };
    let physical = !logical
        .get_pressed()
        .chain(logical.get_just_pressed())
        .any(recognized);
    let held = |code: KeyCode, arrow: Key, letter: &str| {
        (physical && keys.pressed(code))
            || logical.pressed(arrow)
            || logical.pressed(Key::Character(letter.into()))
            || logical.pressed(Key::Character(letter.to_ascii_uppercase().into()))
    };
    let up = held(KeyCode::KeyW, Key::ArrowUp, "w") || (physical && keys.pressed(KeyCode::ArrowUp));
    let down =
        held(KeyCode::KeyS, Key::ArrowDown, "s") || (physical && keys.pressed(KeyCode::ArrowDown));
    let left =
        held(KeyCode::KeyA, Key::ArrowLeft, "a") || (physical && keys.pressed(KeyCode::ArrowLeft));
    let right = held(KeyCode::KeyD, Key::ArrowRight, "d")
        || (physical && keys.pressed(KeyCode::ArrowRight));
    intent.move_dir = if up {
        Some(IVec2::new(0, -1))
    } else if down {
        Some(IVec2::new(0, 1))
    } else if left {
        Some(IVec2::new(-1, 0))
    } else if right {
        Some(IVec2::new(1, 0))
    } else {
        None
    };
    intent.confirm = logical.just_pressed(Key::Space)
        || logical.just_pressed(Key::Enter)
        || logical.just_pressed(Key::Character(" ".into()))
        || (physical && (keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter)));
    intent.cancel =
        logical.just_pressed(Key::Escape) || (physical && keys.just_pressed(KeyCode::Escape));
    let edge = |code: KeyCode, arrow: Key, letter: &str| {
        (physical && keys.just_pressed(code))
            || logical.just_pressed(arrow)
            || logical.just_pressed(Key::Character(letter.into()))
            || logical.just_pressed(Key::Character(letter.to_ascii_uppercase().into()))
    };
    intent.up =
        edge(KeyCode::KeyW, Key::ArrowUp, "w") || (physical && keys.just_pressed(KeyCode::ArrowUp));
    intent.down = edge(KeyCode::KeyS, Key::ArrowDown, "s")
        || (physical && keys.just_pressed(KeyCode::ArrowDown));
}

#[cfg(test)]
mod desktop_input_tests {
    use super::*;
    use bevy::input::{ButtonState, InputPlugin, keyboard::KeyboardInput};
    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(InputPlugin)
            .insert_resource(Intent::default())
            .add_systems(PreUpdate, gather_keyboard_intent.after(InputSystems));
        app
    }
    fn message(app: &mut App, code: KeyCode, logical: Key, state: ButtonState) {
        let window = app.world_mut().spawn_empty().id();
        app.world_mut().write_message(KeyboardInput {
            key_code: code,
            logical_key: logical,
            state,
            text: None,
            repeat: false,
            window,
        });
    }
    #[test]
    fn virtual_space_on_unrelated_physical_escape_confirms_without_cancelling_same_frame() {
        let mut app = app();
        message(&mut app, KeyCode::Escape, Key::Space, ButtonState::Pressed);
        app.update();
        let intent = app.world().resource::<Intent>();
        assert!(intent.confirm);
        assert!(!intent.cancel);
        app.update();
        assert!(!app.world().resource::<Intent>().confirm);
        message(&mut app, KeyCode::Escape, Key::Space, ButtonState::Released);
        app.update();
        assert!(!app.world().resource::<Intent>().confirm);
    }
    #[test]
    fn logical_enter_escape_and_arrows_work_with_arbitrary_virtual_keycodes() {
        for (logical, expected) in [
            (Key::Enter, "confirm"),
            (Key::Escape, "cancel"),
            (Key::ArrowDown, "down"),
        ] {
            let mut app = app();
            message(&mut app, KeyCode::Digit1, logical, ButtonState::Pressed);
            app.update();
            let intent = app.world().resource::<Intent>();
            assert!(match expected {
                "confirm" => intent.confirm,
                "cancel" => intent.cancel,
                _ => intent.down,
            });
            if expected == "down" {
                assert_eq!(intent.move_dir, Some(IVec2::new(0, 1)));
            }
        }
    }
    #[test]
    fn uppercase_logical_wasd_moves_and_physical_layout_fallback_still_works() {
        for (code, logical) in [
            (KeyCode::Digit1, Key::Character("W".into())),
            (KeyCode::KeyW, Key::Character("z".into())),
        ] {
            let mut app = app();
            message(&mut app, code, logical.clone(), ButtonState::Pressed);
            app.update();
            assert_eq!(
                app.world().resource::<Intent>().move_dir,
                Some(IVec2::new(0, -1))
            );
            assert!(app.world().resource::<Intent>().up);
            app.update();
            assert!(!app.world().resource::<Intent>().up);
            message(&mut app, code, logical, ButtonState::Released);
            app.update();
            assert_eq!(app.world().resource::<Intent>().move_dir, None);
        }
    }
    #[test]
    fn press_and_release_in_one_frame_retains_confirm_edge() {
        let mut app = app();
        message(&mut app, KeyCode::Digit1, Key::Enter, ButtonState::Pressed);
        message(&mut app, KeyCode::Digit1, Key::Enter, ButtonState::Released);
        app.update();
        assert!(app.world().resource::<Intent>().confirm);
        app.update();
        assert!(!app.world().resource::<Intent>().confirm);
    }
}

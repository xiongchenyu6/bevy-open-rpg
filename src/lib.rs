use bevy::{
    prelude::*,
    window::{PresentMode, Window, WindowPlugin},
};

pub mod game;

pub use game::{AppState, DesktopInputPlugin, EncounterRate, GamePlugin, Intent};

pub fn run() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "御剑行 · 轮回 (Love RPG)".into(),
                        resolution: (1280u32, 720u32).into(),
                        present_mode: PresentMode::AutoVsync,
                        // Web: render into the page's #bevy-canvas and keep it
                        // sized to the viewport (the camera letterboxes 16:9).
                        canvas: Some("#bevy-canvas".into()),
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: true,
                        ..default()
                    }),
                    ..default()
                })
                // Assets ship without `.meta` sidecars; skip the probe so the
                // web build doesn't fire a 404 for every texture.
                .set(AssetPlugin {
                    meta_check: bevy::asset::AssetMetaCheck::Never,
                    ..default()
                }),
        )
        .add_plugins(GamePlugin)
        .add_plugins(DesktopInputPlugin)
        .run();
}

use bevy::{
    prelude::*,
    window::{PresentMode, Window, WindowPlugin},
};

pub mod game;

pub use game::{AppState, DesktopInputPlugin, EncounterRate, GamePlugin, Intent};

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "仙剑 · 御剑情缘 (Love RPG)".into(),
                resolution: (1280u32, 720u32).into(),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(GamePlugin)
        .add_plugins(DesktopInputPlugin)
        .run();
}

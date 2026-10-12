//! Scrolling for the active inventory or ending panel.
use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
};

#[derive(Component)]
pub struct ScrollPanel;

pub struct UiScrollPlugin;

impl Plugin for UiScrollPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, scroll_panels);
    }
}

fn scroll_panels(
    mut wheel: MessageReader<MouseWheel>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut panels: Query<(&mut ScrollPosition, &ComputedNode), With<ScrollPanel>>,
) {
    let mut delta = 0.0;
    for event in wheel.read() {
        delta -= event.y
            * match event.unit {
                MouseScrollUnit::Line => 42.0,
                MouseScrollUnit::Pixel => 1.0,
            };
    }
    for (mut position, node) in &mut panels {
        let viewport = node.size().y * node.inverse_scale_factor();
        let range =
            ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.0);
        let mut movement = delta;
        if keyboard.just_pressed(KeyCode::PageDown) {
            movement += viewport * 0.8;
        }
        if keyboard.just_pressed(KeyCode::PageUp) {
            movement -= viewport * 0.8;
        }
        position.y = (position.y + movement).clamp(0.0, range);
        if keyboard.just_pressed(KeyCode::Home) {
            position.y = 0.0;
        }
        if keyboard.just_pressed(KeyCode::End) {
            position.y = range;
        }
    }
}

//! Responsive presentation for the inventory; gameplay remains in scene.rs.
use crate::game::core::GameFont;
use bevy::prelude::*;

#[derive(Component)]
pub struct Panel;
#[derive(Component)]
pub struct DesktopHint;
#[derive(Component)]
pub struct Font(pub f32);

pub fn text_font(font: &GameFont, size: f32) -> impl Bundle {
    (font.text_font(size), Font(size))
}

pub fn layout(
    cameras: Query<&Camera, With<Camera2d>>,
    scale: Res<UiScale>,
    mut panels: Query<&mut Node, With<Panel>>,
    mut hints: Query<&mut Visibility, With<DesktopHint>>,
    mut fonts: Query<(&Font, &mut TextFont), With<Text>>,
) {
    let Some(size) = cameras.iter().find_map(Camera::logical_viewport_size) else {
        return;
    };
    let compact = size.x <= 900.0;
    let portrait = size.y > size.x;
    for mut hint in &mut hints {
        *hint = if compact {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
    for (base, mut font) in &mut fonts {
        font.font_size = if compact {
            base.0.clamp(14.0, 22.0) / scale.0.max(0.1)
        } else {
            base.0
        }
        .into();
    }
    for mut node in &mut panels {
        node.left = if compact && portrait {
            Val::Percent(4.0)
        } else {
            Val::Auto
        };
        node.right = Val::Percent(if compact && portrait {
            4.0
        } else if compact {
            8.0
        } else {
            6.0
        });
        node.width = if compact && portrait {
            Val::Auto
        } else {
            Val::Percent(if compact { 60.0 } else { 46.0 })
        };
        node.bottom = Val::Percent(if compact && !portrait {
            22.0
        } else if compact {
            12.0
        } else {
            8.0
        });
    }
}

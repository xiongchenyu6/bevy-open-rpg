//! Shared visual language: ink surfaces, quiet gold edges and readable type.
use bevy::prelude::*;

pub const INK: Color = Color::srgba(0.035, 0.055, 0.070, 0.96);
pub const SURFACE: Color = Color::srgba(0.065, 0.090, 0.105, 0.94);
pub const GOLD: Color = Color::srgb(0.85, 0.72, 0.46);
pub const EDGE: Color = Color::srgba(0.62, 0.55, 0.38, 0.40);
pub const TEXT: Color = Color::srgb(0.93, 0.91, 0.84);
pub const MUTED: Color = Color::srgb(0.58, 0.66, 0.67);
pub const HEALTH: Color = Color::srgb(0.72, 0.32, 0.29);
pub const SPIRIT: Color = Color::srgb(0.30, 0.65, 0.70);

pub fn scale_to_viewport(cameras: Query<&Camera, With<Camera2d>>, mut scale: ResMut<UiScale>) {
    if let Some(size) = cameras.iter().find_map(Camera::logical_viewport_size) {
        scale.0 = (size.x / 1280.0).min(size.y / 720.0).max(0.1);
    }
}

pub fn panel(mut node: Node) -> impl Bundle {
    node.border = UiRect::all(Val::Px(1.0));
    node.border_radius = BorderRadius::all(Val::Px(10.0));
    (
        node,
        BackgroundColor(INK),
        BorderColor::all(EDGE),
        BoxShadow(vec![ShadowStyle {
            color: Color::srgba(0.0, 0.0, 0.0, 0.35),
            x_offset: Val::Px(0.0),
            y_offset: Val::Px(6.0),
            spread_radius: Val::Px(0.0),
            blur_radius: Val::Px(18.0),
        }]),
    )
}

pub fn bar_track(width: Val, height: f32) -> impl Bundle {
    (
        Node {
            width,
            height: Val::Px(height),
            border_radius: BorderRadius::all(Val::Px(3.0)),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(SURFACE),
    )
}

pub fn bar_fill(color: Color) -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            border_radius: BorderRadius::all(Val::Px(3.0)),
            ..default()
        },
        BackgroundColor(color),
    )
}

/// A soft ellipse sits on the ground; it never bobs with a character's torso.
pub fn contact_shadow(
    commands: &mut Commands,
    assets: &AssetServer,
    feet: Vec3,
    width: f32,
    state: super::state::AppState,
) -> Entity {
    commands
        .spawn((
            Sprite {
                image: assets.load("effects/contact_shadow.png"),
                custom_size: Some(Vec2::new(width, width * 0.28)),
                ..default()
            },
            Transform::from_translation(feet),
            DespawnOnExit(state),
        ))
        .id()
}

#[derive(Component)]
pub struct ShadowAnchor {
    pub actor: Entity,
    pub offset: Vec2,
}

pub fn follow_contact_shadows(
    actors: Query<&Transform, Without<ShadowAnchor>>,
    mut shadows: Query<(&ShadowAnchor, &mut Transform)>,
) {
    for (anchor, mut shadow) in &mut shadows {
        if let Ok(actor) = actors.get(anchor.actor) {
            shadow.translation.x = actor.translation.x + anchor.offset.x;
            shadow.translation.y = actor.translation.y + anchor.offset.y;
        }
    }
}

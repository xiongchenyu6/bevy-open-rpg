//! Pointer and keyboard presentation for authored dialogue decisions.
use super::{RunState, event::RunDialogue};
use crate::game::{
    core::{GameFont, Intent, PlayerStats},
    ui as theme,
};
use bevy::prelude::*;

#[derive(Component)]
pub struct DialogueRoot;
#[derive(Component)]
pub(super) struct Portrait;
#[derive(Component, Clone, Copy)]
pub(super) enum Label {
    Title,
    Line,
    Preview,
    Hint,
    Continue,
    Option(usize),
}
#[derive(Component, Clone, Copy)]
pub(super) enum Action {
    Continue,
    Option(usize),
}

pub fn spawn(commands: &mut Commands, font: &GameFont, scope: impl Bundle) {
    commands
        .spawn((
            DialogueRoot,
            scope,
            theme::panel(Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(8.0),
                right: Val::Percent(8.0),
                bottom: Val::Px(28.0),
                column_gap: Val::Px(16.0),
                align_items: AlignItems::FlexStart,
                padding: UiRect::axes(Val::Px(26.0), Val::Px(20.0)),
                display: Display::None,
                ..default()
            }),
            GlobalZIndex(50),
        ))
        .with_children(|root| {
            root.spawn((
                Portrait,
                ImageNode::default(),
                Node {
                    width: Val::Px(120.0),
                    height: Val::Px(132.0),
                    flex_shrink: 0.0,
                    ..default()
                },
            ));
            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                row_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|column| {
                for (label, size, color) in [
                    (Label::Title, 20.0, theme::GOLD),
                    (Label::Line, 21.0, theme::TEXT),
                    (Label::Preview, 15.0, theme::MUTED),
                ] {
                    column.spawn((label, Text::new(""), font.text_font(size), TextColor(color)));
                }
                // Current authored conversations offer at most four choices; spare slots
                // keep new route mechanisms within the same pointer interaction contract.
                for index in 0..8 {
                    column
                        .spawn((
                            Button,
                            Action::Option(index),
                            Node {
                                width: Val::Percent(100.0),
                                min_height: Val::Px(32.0),
                                padding: UiRect::axes(Val::Px(12.0), Val::Px(5.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                border_radius: BorderRadius::all(Val::Px(5.0)),
                                display: Display::None,
                                ..default()
                            },
                            BackgroundColor(theme::SURFACE),
                            BorderColor::all(theme::EDGE),
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Label::Option(index),
                                Text::new(""),
                                font.text_font(18.0),
                                TextColor(theme::TEXT),
                            ));
                        });
                }
                column
                    .spawn((
                        Button,
                        Action::Continue,
                        Node {
                            padding: UiRect::axes(Val::Px(18.0), Val::Px(7.0)),
                            align_self: AlignSelf::FlexStart,
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(5.0)),
                            ..default()
                        },
                        BackgroundColor(theme::SURFACE),
                        BorderColor::all(theme::EDGE),
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Label::Continue,
                            Text::new("继续"),
                            font.text_font(18.0),
                            TextColor(theme::GOLD),
                        ));
                    });
                column.spawn((
                    Label::Hint,
                    Text::new(""),
                    font.text_font(13.0),
                    TextColor(theme::MUTED),
                ));
            });
        });
}

/// Only current visible actions can confirm; pointer leave never changes the
/// keyboard selection, and hovering merely previews rather than committing.
pub fn pointer_input(
    mut dialogue: ResMut<RunDialogue>,
    mut intent: ResMut<Intent>,
    buttons: Query<(&Action, &Interaction), Changed<Interaction>>,
) {
    if !dialogue.active {
        return;
    }
    for (action, interaction) in &buttons {
        if *interaction == Interaction::None {
            continue;
        }
        match *action {
            Action::Option(index) if dialogue.in_choice() && index < dialogue.options.len() => {
                dialogue.selected = index;
                dialogue.choice_error = None;
                if *interaction == Interaction::Pressed {
                    intent.confirm = true;
                }
            }
            Action::Continue if !dialogue.in_choice() => {
                if *interaction == Interaction::Pressed {
                    intent.confirm = true;
                }
            }
            _ => {}
        }
    }
}

/// Portrait windows have ample vertical room but must not inherit the tiny
/// landscape design scale. Keep local sizes in physical pixels, including DPI.
fn portrait_units(viewport: UVec2, ui_scale: f32, dpi: f32) -> Option<f32> {
    (viewport.y > viewport.x).then(|| 1.0 / (ui_scale * dpi).max(0.1))
}

fn label_size(label: Label, portrait: Option<f32>) -> f32 {
    let base: f32 = match label {
        Label::Title => 20.0,
        Label::Line => 21.0,
        Label::Preview => 15.0,
        Label::Hint => 13.0,
        Label::Continue | Label::Option(_) => 18.0,
    };
    portrait.map_or(base, |units| base.max(16.0) * units)
}

pub fn update(
    dialogue: Res<RunDialogue>,
    run: Option<Res<RunState>>,
    stats: Res<PlayerStats>,
    assets: Res<AssetServer>,
    cameras: Query<&Camera, With<Camera2d>>,
    scale: Res<UiScale>,
    mut root: Query<&mut Node, (With<DialogueRoot>, Without<Action>, Without<Portrait>)>,
    mut portrait: Query<
        (&mut ImageNode, &mut Node),
        (With<Portrait>, Without<DialogueRoot>, Without<Action>),
    >,
    mut buttons: Query<
        (&Action, &mut Node, &mut BackgroundColor, &mut BorderColor),
        (Without<DialogueRoot>, Without<Portrait>),
    >,
    mut labels: Query<(&Label, &mut Text, &mut TextColor, &mut TextFont)>,
) {
    let portrait_units = cameras.iter().find_map(|camera| {
        let size = camera.physical_viewport_size()?;
        portrait_units(size, scale.0, camera.target_scaling_factor().unwrap_or(1.0))
    });
    if let Ok(mut node) = root.single_mut() {
        let units = portrait_units.unwrap_or(1.0);
        node.left = Val::Percent(if portrait_units.is_some() { 4.0 } else { 8.0 });
        node.right = node.left;
        node.bottom = Val::Px(if portrait_units.is_some() {
            20.0 * units
        } else {
            28.0
        });
        node.padding = if portrait_units.is_some() {
            UiRect::axes(Val::Px(18.0 * units), Val::Px(16.0 * units))
        } else {
            UiRect::axes(Val::Px(26.0), Val::Px(20.0))
        };
        node.column_gap = Val::Px(if portrait_units.is_some() {
            12.0 * units
        } else {
            16.0
        });
        node.display = if dialogue.active {
            Display::Flex
        } else {
            Display::None
        };
    }
    if let Ok((mut image, mut node)) = portrait.single_mut() {
        node.width = Val::Px(portrait_units.map_or(120.0, |units| 72.0 * units));
        node.height = Val::Px(portrait_units.map_or(132.0, |units| 80.0 * units));
        node.display = if dialogue.portrait.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if let Some(path) = dialogue.portrait {
            if dialogue.is_changed() {
                image.image = assets.load(path);
            }
        }
    }
    let choosing = dialogue.active && dialogue.in_choice();
    for (action, mut node, mut background, mut border) in &mut buttons {
        let (visible, selected) = match *action {
            Action::Option(index) => (
                choosing && index < dialogue.options.len(),
                index == dialogue.selected,
            ),
            Action::Continue => (dialogue.active && !choosing, false),
        };
        let units = portrait_units.unwrap_or(1.0);
        node.min_height = match action {
            Action::Option(_) => Val::Px(if portrait_units.is_some() {
                44.0 * units
            } else {
                32.0
            }),
            Action::Continue => Val::Auto,
        };
        node.padding = match action {
            Action::Option(_) => UiRect::axes(
                Val::Px(12.0 * units),
                Val::Px(if portrait_units.is_some() {
                    8.0 * units
                } else {
                    5.0
                }),
            ),
            Action::Continue => UiRect::axes(Val::Px(18.0 * units), Val::Px(7.0 * units)),
        };
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
        background.0 = if selected {
            Color::srgb(0.17, 0.19, 0.16)
        } else {
            theme::SURFACE
        };
        *border = BorderColor::all(if selected { theme::GOLD } else { theme::EDGE });
    }
    for (label, mut text, mut color, mut font) in &mut labels {
        font.font_size = label_size(*label, portrait_units).into();
        text.0 = match *label {
            Label::Title => dialogue.title.clone(),
            Label::Line => dialogue
                .lines
                .get(dialogue.idx)
                .cloned()
                .unwrap_or_default(),
            Label::Preview if choosing => format!(
                "气血 {}/{} · 灵力 {}/{} · 钱财 {} 文 · 药水 {}\n{}",
                stats.hp,
                stats.max_hp,
                stats.mp,
                stats.max_mp,
                stats.gold,
                stats.potions,
                dialogue.choice_preview(run.as_deref())
            ),
            Label::Preview => String::new(),
            Label::Continue => "继续".into(),
            Label::Hint if choosing => "↑ ↓ 选择 · 空格 / 回车确认 · 点击选项".into(),
            Label::Hint => "空格 / 回车继续 · 点击继续".into(),
            Label::Option(index) => {
                color.0 = if index == dialogue.selected {
                    theme::GOLD
                } else {
                    theme::TEXT
                };
                dialogue.options.get(index).cloned().unwrap_or_default()
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn portrait_text_remains_readable_across_small_viewports_and_dpi_without_changing_landscape() {
        for (size, scale, dpi) in [
            (UVec2::new(720, 1280), 0.5625, 1.0),
            (UVec2::new(360, 800), 0.28125, 1.0),
            (UVec2::new(720, 1280), 0.28125, 2.0),
        ] {
            let units = portrait_units(size, scale, dpi);
            for label in [
                Label::Title,
                Label::Line,
                Label::Preview,
                Label::Hint,
                Label::Option(0),
                Label::Continue,
            ] {
                assert!(label_size(label, units) * scale * dpi >= 16.0);
            }
            // A 44px pointer target retains its physical extent at either DPI.
            assert!((44.0 * units.unwrap() * scale * dpi - 44.0).abs() < 0.01);
        }
        let landscape = portrait_units(UVec2::new(960, 540), 0.75, 1.0);
        assert!(landscape.is_none());
        assert_eq!(label_size(Label::Line, landscape), 21.0);
        assert_eq!(label_size(Label::Hint, landscape), 13.0);
    }

    fn app(choosing: bool) -> App {
        let mut app = App::new();
        let mut dialogue = RunDialogue::default();
        dialogue.open_event(0);
        dialogue.idx = if choosing {
            dialogue.lines.len() - 1
        } else {
            0
        };
        app.insert_resource(dialogue)
            .insert_resource(Intent::default())
            .add_systems(Update, pointer_input);
        app
    }
    #[test]
    fn hover_previews_press_confirms_and_pointer_leave_preserves_keyboard_selection() {
        let mut app = app(true);
        let button = app
            .world_mut()
            .spawn((Button, Action::Option(1), Interaction::Hovered))
            .id();
        app.update();
        assert_eq!(app.world().resource::<RunDialogue>().selected, 1);
        assert!(!app.world().resource::<Intent>().confirm);
        assert!(
            app.world()
                .resource::<RunDialogue>()
                .choice_preview(None)
                .contains("无消耗")
        );
        app.world_mut().entity_mut(button).insert(Interaction::None);
        app.world_mut().resource_mut::<RunDialogue>().selected = 0;
        app.update();
        assert_eq!(app.world().resource::<RunDialogue>().selected, 0);
        app.world_mut()
            .entity_mut(button)
            .insert(Interaction::Pressed);
        app.update();
        assert_eq!(app.world().resource::<RunDialogue>().selected, 1);
        assert!(app.world().resource::<Intent>().confirm);
    }
    #[test]
    fn reading_and_closed_dialogues_ignore_hidden_option_presses() {
        let mut app = app(false);
        app.world_mut()
            .spawn((Button, Action::Option(1), Interaction::Pressed));
        app.update();
        assert!(!app.world().resource::<Intent>().confirm);
        app.world_mut().resource_mut::<RunDialogue>().active = false;
        app.world_mut()
            .spawn((Button, Action::Continue, Interaction::Pressed));
        app.update();
        assert!(!app.world().resource::<Intent>().confirm);
    }
    #[test]
    fn continue_button_works_while_reading_but_cannot_commit_a_choice() {
        let mut app = app(false);
        let button = app
            .world_mut()
            .spawn((Button, Action::Continue, Interaction::Pressed))
            .id();
        app.update();
        assert!(app.world().resource::<Intent>().confirm);
        app.world_mut().resource_mut::<Intent>().clear();
        let last = app.world().resource::<RunDialogue>().lines.len() - 1;
        app.world_mut().resource_mut::<RunDialogue>().idx = last;
        app.world_mut().entity_mut(button).insert(Interaction::None);
        app.update();
        app.world_mut()
            .entity_mut(button)
            .insert(Interaction::Pressed);
        app.update();
        assert!(!app.world().resource::<Intent>().confirm);
    }
}

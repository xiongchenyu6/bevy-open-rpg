use bevy::prelude::*;

/// Top-level runtime states for the demo.
///
/// `Explore` owns the overworld map; `Battle` owns the turn-based combat scene.
/// Dialogue is handled as an overlay inside `Explore` (a resource flag), not a
/// separate state, so the map stays visible behind the text box.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum AppState {
    #[default]
    Explore,
    Battle,
}

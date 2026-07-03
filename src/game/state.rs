use bevy::prelude::*;

/// Top-level runtime states.
///
/// The roguelike run loop is: `Title` → `NodeMap` (pick a node) → `Battle` /
/// event overlay → `Reward` → back to `NodeMap` → … → `Ending` → `Title`.
///
/// `Explore` (free-roam overworld) and `Battle` predate the roguelike mode and
/// are kept intact: `Explore` is reachable for legacy capture presets, and
/// `Battle` is shared by both flows (a `RunState` resource tells it which flow
/// it belongs to).
///
/// Dialogue/event text is handled as an overlay resource inside its owning
/// state, not a separate state, so the scene stays visible behind the box.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum AppState {
    #[default]
    Title,
    NodeMap,
    /// Walkable tile-map scene for the node picked on the `NodeMap`.
    RunScene,
    Battle,
    Reward,
    Ending,
    Explore,
}

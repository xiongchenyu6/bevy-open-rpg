use bevy::prelude::*;

use super::{
    core::{MAP_H, MAP_W, TILE, tile_to_world},
    explore::{MapContent, PlayerPos},
    state::AppState,
};

const VISIBLE_RADIUS: f32 = 3.45;
const SEEN_ALPHA: f32 = 0.34;
const UNSEEN_ALPHA: f32 = 0.78;

#[derive(Resource)]
pub struct FogMemory {
    seen: Vec<bool>,
}

impl Default for FogMemory {
    fn default() -> Self {
        Self {
            seen: vec![false; (MAP_W * MAP_H) as usize],
        }
    }
}

impl FogMemory {
    pub fn clear(&mut self) {
        self.seen.fill(false);
    }
}

#[derive(Component)]
struct FogTile {
    col: i32,
    row: i32,
}

pub struct FogPlugin;

impl Plugin for FogPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FogMemory>().add_systems(
            Update,
            update_fog_overlay.run_if(in_state(AppState::Explore)),
        );
    }
}

pub fn spawn_explore_fog(commands: &mut Commands) {
    for row in 0..MAP_H {
        for col in 0..MAP_W {
            let p = tile_to_world(col, row);
            commands.spawn((
                FogTile { col, row },
                MapContent,
                Sprite::from_color(
                    Color::srgba(0.015, 0.012, 0.03, UNSEEN_ALPHA),
                    Vec2::splat(TILE),
                ),
                Transform::from_xyz(p.x, p.y, 45.0),
                DespawnOnExit(AppState::Explore),
            ));
        }
    }
}

fn update_fog_overlay(
    pos: Res<PlayerPos>,
    mut memory: ResMut<FogMemory>,
    mut tiles: Query<(&FogTile, &mut Sprite)>,
) {
    let radius2 = VISIBLE_RADIUS * VISIBLE_RADIUS;
    for (tile, mut sprite) in &mut tiles {
        let dx = (tile.col - pos.col) as f32;
        let dy = (tile.row - pos.row) as f32;
        let visible = dx * dx + dy * dy <= radius2;
        let idx = (tile.row * MAP_W + tile.col) as usize;
        if visible {
            memory.seen[idx] = true;
        }

        let alpha = if visible {
            0.0
        } else if memory.seen[idx] {
            SEEN_ALPHA
        } else {
            UNSEEN_ALPHA
        };
        sprite.color = Color::srgba(0.015, 0.012, 0.03, alpha);
    }
}

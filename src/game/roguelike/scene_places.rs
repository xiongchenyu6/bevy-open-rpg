//! Visible architecture and its matching ground footprint.
use super::*;

pub(super) struct Landmark {
    pub asset: &'static str,
    pub col: i32,
    pub row: i32,
    pub width: i32,
    pub depth: i32,
    pub blocking: bool,
}

impl Landmark {
    pub(super) fn blocks_column(&self, col: i32) -> bool {
        self.blocking && (self.asset != "celestial-arch" || (col - self.col).abs() >= 2)
    }
    pub(super) fn overhang_contains(&self, col: i32, row: i32) -> bool {
        self.blocking
            && col >= self.col - self.width / 2 - 1
            && col <= self.col + self.width / 2 + 1
            && row >= self.row - self.width
            && row <= self.row
    }
}

fn landmark(asset: &'static str, col: i32, row: i32, width: i32, depth: i32) -> Landmark {
    Landmark {
        asset,
        col,
        row,
        width,
        depth,
        blocking: true,
    }
}

pub(super) fn landmarks(map: MapKind, variant: usize) -> Vec<Landmark> {
    let phase = (variant % 16) as i32;
    let shift = phase % 3;
    let mut items = match map {
        MapKind::Village => vec![
            landmark("inn", 7, 7, 5, 3),
            landmark("pier", 23, 10, 4, 2),
            landmark("market", 16, 4, 3, 2),
        ],
        MapKind::Bamboo => vec![landmark("bamboo-pavilion", 10 + shift, 5, 4, 3)],
        MapKind::Cave => vec![landmark("crystals", 13 + shift, 6, 3, 2)],
        MapKind::MoonEchoCorridor => vec![
            landmark("moon-altar", 16, 5, 4, 3),
            landmark("crystals", 7, 11, 3, 2),
        ],
        MapKind::RiverTown => vec![
            landmark("market", 7, 4, 4, 2),
            landmark("bridge", 15 + shift, 8, 6, 2),
            landmark("pier", 23, 11, 4, 2),
        ],
        MapKind::RiverReedBed => vec![
            landmark("pier", 9, 6, 4, 2),
            landmark("bridge", 19, 10, 6, 2),
        ],
        MapKind::PlagueVillage => vec![
            landmark("medicine-hut", 6, 5, 4, 3),
            landmark("shrine", 21, 10, 4, 3),
        ],
        MapKind::PlagueShrinePath => vec![
            landmark("shrine", 15, 5, 5, 3),
            landmark("medicine-hut", 24, 11, 3, 2),
        ],
        MapKind::Capital => vec![
            landmark("capital-gate", 7, 7, 5, 3),
            landmark("market", 21, 10, 4, 2),
        ],
        MapKind::CapitalMansion => vec![
            landmark("manor", 15, 5, 6, 3),
            landmark("mirror-dais", 23, 11, 3, 2),
        ],
        MapKind::MansionMirrorGallery => vec![
            landmark("mirror-dais", 8, 5, 4, 3),
            landmark("mirror-dais", 21, 10, 4, 3),
        ],
        MapKind::SouthernRoad => vec![
            landmark("south-hut", 6, 5, 4, 3),
            landmark("thunder-drum", 21, 10, 4, 3),
        ],
        MapKind::ThunderDrumPath => vec![
            landmark("thunder-drum", 18, 8, 5, 3),
            landmark("south-hut", 6, 4, 3, 2),
        ],
        MapKind::DreamWaterway => vec![
            landmark("dream-bridge", 15 + shift, 6 + (phase / 3) % 3, 6, 2),
            landmark("moon-altar", 24, 5, 4, 3),
        ],
        MapKind::FinalSanctum => vec![
            landmark(
                if phase >= 8 {
                    "celestial-arch"
                } else if phase >= 4 {
                    "mirror-dais"
                } else {
                    "moon-altar"
                },
                15,
                5,
                5,
                3,
            ),
            landmark("crystals", 6 + shift, 11, 3, 2),
        ],
    };
    for item in &mut items {
        if matches!(item.asset, "bridge" | "dream-bridge") {
            item.blocking = false;
        }
    }
    items
}

/// Reserve the architectural footprint after terrain decoration. Collision and
/// visible buildings cannot drift independently on random revisits.
pub(super) fn reserve(tiles: &mut [Vec<Tile>], map: MapKind, variant: usize) {
    for item in landmarks(map, variant) {
        let left = item.col - item.width / 2;
        let right = left + item.width - 1;
        let top = item.row - item.depth + 1;
        if !item.blocking {
            for row in (top - 2).max(1)..=(item.row + 2).min(MAP_H - 2) {
                for col in item.col - 1..=item.col + 1 {
                    tiles[row as usize][col as usize] = Tile::Water;
                }
            }
        }
        for row in top - 1..=item.row + 1 {
            for col in left - 1..=right + 1 {
                if col <= 0 || col >= MAP_W - 1 || row <= 0 || row >= MAP_H - 1 {
                    continue;
                }
                let footprint = row >= top && row <= item.row && col >= left && col <= right;
                if !footprint
                    && (item.asset == "pier" || !item.blocking)
                    && tiles[row as usize][col as usize] == Tile::Water
                {
                    continue;
                }
                tiles[row as usize][col as usize] = if footprint && item.blocks_column(col) {
                    if item.asset == "pier" {
                        Tile::Water
                    } else {
                        Tile::Structure
                    }
                } else if footprint && !item.blocking && (col - item.col).abs() <= 1 {
                    Tile::Bridge
                } else {
                    Tile::Path
                };
            }
        }
    }
}

pub(super) fn spawn(
    commands: &mut Commands,
    assets: &AssetServer,
    scene: &RunSceneState,
    variant: usize,
) {
    for item in landmarks(scene.map, variant) {
        let point = tile_to_world(item.col, item.row);
        let size = TILE * (item.width as f32 + 0.4);
        let architecture = SceneArchitecture {
            foot_row: item.row,
            top_row: item.row - item.depth + 1,
            left_col: item.col - item.width / 2,
            right_col: item.col - item.width / 2 + item.width - 1,
            walkover: !item.blocking,
            shadow: false,
        };
        let shadow = theme::contact_shadow(
            commands,
            assets,
            Vec3::new(point.x, point.y + 8.0, 1.0),
            size * 0.70,
            AppState::RunScene,
        );
        commands.entity(shadow).insert(SceneArchitecture {
            shadow: true,
            ..architecture
        });
        commands.spawn((
            architecture,
            Sprite {
                image: assets.load(format!("props/landmark-{}.png", item.asset)),
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(point.x, point.y + size * 0.5 - TILE * 0.5, 7.0),
            DespawnOnExit(AppState::RunScene),
        ));
    }
}

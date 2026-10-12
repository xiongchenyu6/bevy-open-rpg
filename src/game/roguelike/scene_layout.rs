//! Map topology, location footprints and replay variation.
use super::*;

#[derive(Clone, Copy)]
struct StageTerrainProfile {
    wall_jitter: f32,
    wall_smoothing: usize,
    water_walks: i32,
    water_steps: i32,
    grass_walks: i32,
    grass_steps: i32,
}

fn stage_terrain_profile(map: MapKind) -> StageTerrainProfile {
    match map {
        MapKind::Village => StageTerrainProfile {
            wall_jitter: 0.04,
            wall_smoothing: 1,
            water_walks: 1,
            water_steps: 8,
            grass_walks: 4,
            grass_steps: 12,
        },
        MapKind::Bamboo => StageTerrainProfile {
            wall_jitter: 0.10,
            wall_smoothing: 2,
            water_walks: 1,
            water_steps: 8,
            grass_walks: 7,
            grass_steps: 16,
        },
        MapKind::Cave | MapKind::MoonEchoCorridor => StageTerrainProfile {
            wall_jitter: 0.08,
            wall_smoothing: 1,
            water_walks: 2,
            water_steps: 12,
            grass_walks: 3,
            grass_steps: 10,
        },
        MapKind::RiverTown => StageTerrainProfile {
            wall_jitter: 0.03,
            wall_smoothing: 1,
            water_walks: 2,
            water_steps: 15,
            grass_walks: 3,
            grass_steps: 12,
        },
        MapKind::RiverReedBed => StageTerrainProfile {
            wall_jitter: 0.07,
            wall_smoothing: 1,
            water_walks: 4,
            water_steps: 18,
            grass_walks: 7,
            grass_steps: 16,
        },
        MapKind::PlagueVillage | MapKind::PlagueShrinePath => StageTerrainProfile {
            wall_jitter: 0.07,
            wall_smoothing: 1,
            water_walks: 2,
            water_steps: 14,
            grass_walks: 5,
            grass_steps: 13,
        },
        MapKind::Capital | MapKind::CapitalMansion | MapKind::MansionMirrorGallery => {
            StageTerrainProfile {
                wall_jitter: 0.0,
                wall_smoothing: 0,
                water_walks: 1,
                water_steps: 8,
                grass_walks: 2,
                grass_steps: 9,
            }
        }
        MapKind::SouthernRoad | MapKind::ThunderDrumPath => StageTerrainProfile {
            wall_jitter: 0.08,
            wall_smoothing: 1,
            water_walks: 2,
            water_steps: 12,
            grass_walks: 7,
            grass_steps: 16,
        },
        MapKind::FinalSanctum | MapKind::DreamWaterway => StageTerrainProfile {
            wall_jitter: 0.09,
            wall_smoothing: 1,
            water_walks: 4,
            water_steps: 17,
            grass_walks: 3,
            grass_steps: 11,
        },
    }
}

fn architectural_blueprint(map: MapKind, variant: usize) -> Option<Vec<Vec<Tile>>> {
    use MapKind::*;
    if !matches!(
        map,
        Village
            | RiverTown
            | PlagueVillage
            | Capital
            | CapitalMansion
            | MansionMirrorGallery
            | PlagueShrinePath
            | SouthernRoad
            | ThunderDrumPath
            | DreamWaterway
            | FinalSanctum
    ) {
        return None;
    }
    let phase = variant % 16;
    let green = matches!(map, Village | PlagueVillage | SouthernRoad);
    let mut tiles =
        vec![vec![if green { Tile::Grass } else { Tile::Path }; MAP_W as usize]; MAP_H as usize];
    for row in 0..MAP_H as usize {
        for col in 0..MAP_W as usize {
            if row == 0 || col == 0 || row == MAP_H as usize - 1 || col == MAP_W as usize - 1 {
                tiles[row][col] = Tile::Wall;
            }
        }
    }
    match map {
        RiverTown | DreamWaterway => {
            let canal = 14 + (phase % 3) as usize;
            for row in 1..MAP_H as usize - 1 {
                for col in canal..canal + 3 {
                    tiles[row][col] = Tile::Water;
                }
            }
            // Two crossings create an actual route choice around the canal.
            for row in [4 + phase % 2, 10 + phase % 2] {
                for col in canal - 1..canal + 4 {
                    tiles[row][col] = Tile::Path;
                }
            }
        }
        Village => {
            for row in 9..14 {
                for col in 21..28 {
                    tiles[row][col] = Tile::Water;
                }
            }
        }
        PlagueVillage => {
            for row in 3..6 {
                for col in 20..25 {
                    tiles[row][col] = Tile::Water;
                }
            }
        }
        Capital | CapitalMansion | MansionMirrorGallery => {
            // Courtyard partitions with different doors on return visits.
            let door = 7 + phase % 4;
            for row in 2..MAP_H as usize - 2 {
                if row.abs_diff(door) > 1 {
                    tiles[row][11] = Tile::Wall;
                }
            }
        }
        FinalSanctum | PlagueShrinePath | ThunderDrumPath => {
            let aisle = 5 + phase % 5;
            for col in [9, 20] {
                for row in 2..MAP_H as usize - 2 {
                    if row.abs_diff(aisle) > 1 {
                        tiles[row][col] = Tile::Wall;
                    }
                }
            }
        }
        SouthernRoad => {
            for row in 2..MAP_H as usize - 2 {
                for col in 2..MAP_W as usize - 2 {
                    if (col + row / 3 + phase) % 11 < 2 {
                        tiles[row][col] = Tile::Wall;
                    }
                }
            }
        }
        _ => {}
    }
    // A broad approach reads as a road, not random decorative grass noise.
    for col in 1..MAP_W as usize - 1 {
        for row in 7..=9 {
            if tiles[row][col] != Tile::Water {
                tiles[row][col] = Tile::Path;
            }
        }
    }
    Some(tiles)
}

fn transform_stage_blueprint(
    map: MapKind,
    variant: usize,
    flip_x: bool,
    flip_y: bool,
) -> Vec<Vec<Tile>> {
    let source = MapData::build(map).tiles;
    let (w, h) = (MAP_W as usize, MAP_H as usize);
    let mut transformed = vec![vec![Tile::Wall; w]; h];
    for (row, output_row) in transformed.iter_mut().enumerate() {
        for (col, output) in output_row.iter_mut().enumerate() {
            let source_col = if flip_x { w - 1 - col } else { col };
            let source_row = if flip_y { h - 1 - row } else { row };
            *output = match source[source_row][source_col] {
                Tile::Npc | Tile::Portal => Tile::Path,
                tile => tile,
            };
        }
    }

    // Repeated visits retain the landmark topology while changing the approach
    // direction. Three shear phases keep the eleven-stage finale from cycling
    // through only four exact transforms.
    let shear_phase = (variant / 4) % 3;
    if shear_phase != 0 {
        for row in 1..h - 1 {
            transformed[row].rotate_left((row + variant) % (shear_phase + 2));
            transformed[row][0] = Tile::Wall;
            transformed[row][w - 1] = Tile::Wall;
        }
    }
    transformed
}

fn smooth_stage_walls(tiles: &mut [Vec<Tile>], rng: &mut Rng, profile: StageTerrainProfile) {
    let (w, h) = (MAP_W as usize, MAP_H as usize);
    let mut walls: Vec<Vec<bool>> = tiles
        .iter()
        .map(|row| row.iter().map(|tile| *tile == Tile::Wall).collect())
        .collect();

    if profile.wall_jitter > 0.0 {
        let snapshot = walls.clone();
        for row in 1..h - 1 {
            for col in 1..w - 1 {
                let adjacent = [
                    snapshot[row - 1][col],
                    snapshot[row + 1][col],
                    snapshot[row][col - 1],
                    snapshot[row][col + 1],
                ]
                .into_iter()
                .filter(|wall| *wall)
                .count();
                if (1..=3).contains(&adjacent) && rng.chance(profile.wall_jitter) {
                    walls[row][col] = !walls[row][col];
                }
            }
        }
    }

    for _ in 0..profile.wall_smoothing {
        let snapshot = walls.clone();
        for row in 1..h - 1 {
            for col in 1..w - 1 {
                let mut neighbours = 0;
                for dr in -1i32..=1 {
                    for dc in -1i32..=1 {
                        if snapshot[(row as i32 + dr) as usize][(col as i32 + dc) as usize] {
                            neighbours += 1;
                        }
                    }
                }
                walls[row][col] = neighbours >= 5;
            }
        }
    }

    for row in 0..h {
        for col in 0..w {
            if walls[row][col] {
                tiles[row][col] = Tile::Wall;
            } else if tiles[row][col] == Tile::Wall {
                tiles[row][col] = Tile::Path;
            }
        }
    }
}

fn paint_stage_walks(tiles: &mut [Vec<Tile>], rng: &mut Rng, kind: Tile, walks: i32, steps: i32) {
    for _ in 0..walks {
        let mut start = None;
        for _ in 0..80 {
            let candidate = (rng.range(2, MAP_W - 3), rng.range(2, MAP_H - 3));
            let tile = tiles[candidate.1 as usize][candidate.0 as usize];
            if matches!(tile, Tile::Path | Tile::Grass) {
                start = Some(candidate);
                break;
            }
        }
        let Some((mut col, mut row)) = start else {
            continue;
        };
        for _ in 0..steps {
            let tile = &mut tiles[row as usize][col as usize];
            if matches!(*tile, Tile::Path | Tile::Grass) {
                *tile = kind;
            }
            match rng.range(0, 3) {
                0 => col = (col + 1).min(MAP_W - 2),
                1 => col = (col - 1).max(1),
                2 => row = (row + 1).min(MAP_H - 2),
                _ => row = (row - 1).max(1),
            }
        }
    }
}

fn keep_largest_walkable_region(tiles: &mut [Vec<Tile>]) -> usize {
    let (w, h) = (MAP_W as usize, MAP_H as usize);
    let mut visited = vec![vec![false; w]; h];
    let mut largest = Vec::new();

    for start_row in 0..h {
        for start_col in 0..w {
            if visited[start_row][start_col] || !tiles[start_row][start_col].walkable() {
                continue;
            }
            let mut region = Vec::new();
            let mut queue = std::collections::VecDeque::from([(start_col, start_row)]);
            visited[start_row][start_col] = true;
            while let Some((col, row)) = queue.pop_front() {
                region.push((col, row));
                for (dc, dr) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                    let (next_col, next_row) = (col as i32 + dc, row as i32 + dr);
                    if next_col < 0 || next_row < 0 || next_col >= MAP_W || next_row >= MAP_H {
                        continue;
                    }
                    let (next_col, next_row) = (next_col as usize, next_row as usize);
                    if !visited[next_row][next_col] && tiles[next_row][next_col].walkable() {
                        visited[next_row][next_col] = true;
                        queue.push_back((next_col, next_row));
                    }
                }
            }
            if region.len() > largest.len() {
                largest = region;
            }
        }
    }

    let mut retained = vec![vec![false; w]; h];
    for &(col, row) in &largest {
        retained[row][col] = true;
    }
    for row in 0..h {
        for col in 0..w {
            if tiles[row][col].walkable() && !retained[row][col] {
                tiles[row][col] = Tile::Wall;
            }
        }
    }
    largest.len()
}

pub(super) fn stage_exploration_span(tiles: &[Vec<Tile>]) -> u16 {
    let map = MapData::generated(MapKind::Village, tiles.to_vec());
    let start = (1..MAP_W - 1).find_map(|col| {
        (1..MAP_H - 1)
            .find(|row| map.at(col, *row).walkable())
            .map(|row| (col, row))
    });
    let Some(start) = start else { return 0 };
    distance_field(&map, &[start])
        .into_iter()
        .flatten()
        .filter(|distance| *distance != u16::MAX)
        .max()
        .unwrap_or(0)
}

/// Build a route map from its authored biome blueprint, then soften and vary
/// it. This keeps all 15 places topologically distinct instead of applying one
/// cellular-automata layout to every chapter.
pub(super) fn generate_stage_tiles(map: MapKind, variant: usize, rng: &mut Rng) -> Vec<Vec<Tile>> {
    let (w, h) = (MAP_W as usize, MAP_H as usize);
    let mut profile = stage_terrain_profile(map);
    let architectural = architectural_blueprint(map, variant);
    if architectural.is_some() {
        profile.wall_jitter = 0.0;
        profile.wall_smoothing = 0;
        profile.water_walks = 0;
        profile.grass_walks = if matches!(
            map,
            MapKind::Village | MapKind::SouthernRoad | MapKind::PlagueVillage
        ) {
            1
        } else {
            0
        };
    }
    let flip_x = variant & 1 != 0;
    let flip_y = variant & 2 != 0;
    let blueprint =
        architectural.unwrap_or_else(|| transform_stage_blueprint(map, variant, flip_x, flip_y));
    let mut fallback = blueprint.clone();

    for _ in 0..20 {
        let mut tiles = blueprint.clone();
        smooth_stage_walls(&mut tiles, rng, profile);
        paint_stage_walks(
            &mut tiles,
            rng,
            Tile::Water,
            profile.water_walks,
            profile.water_steps,
        );
        paint_stage_walks(
            &mut tiles,
            rng,
            Tile::Grass,
            profile.grass_walks,
            profile.grass_steps,
        );

        // Isolated 1-3 cell water/grass specks read as texture mistakes.
        for kind in [Tile::Water, Tile::Grass] {
            let mut seen = vec![vec![false; w]; h];
            for r0 in 0..h {
                for c0 in 0..w {
                    if tiles[r0][c0] != kind || seen[r0][c0] {
                        continue;
                    }
                    let mut blob = vec![(c0, r0)];
                    let mut queue = std::collections::VecDeque::from([(c0, r0)]);
                    seen[r0][c0] = true;
                    while let Some((qc, qr)) = queue.pop_front() {
                        for (dc, dr) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                            let (nc, nr) = (qc as i32 + dc, qr as i32 + dr);
                            if nc < 0 || nr < 0 || nc >= w as i32 || nr >= h as i32 {
                                continue;
                            }
                            let (nc, nr) = (nc as usize, nr as usize);
                            if tiles[nr][nc] == kind && !seen[nr][nc] {
                                seen[nr][nc] = true;
                                blob.push((nc, nr));
                                queue.push_back((nc, nr));
                            }
                        }
                    }
                    if blob.len() <= 3 {
                        for (bc, br) in blob {
                            tiles[br][bc] = Tile::Path;
                        }
                    }
                }
            }
        }

        super::places::reserve(&mut tiles, map, variant);
        let open = keep_largest_walkable_region(&mut tiles);
        if open >= 150 && stage_exploration_span(&tiles) >= 24 {
            return tiles;
        }
        fallback = tiles;
    }

    keep_largest_walkable_region(&mut fallback);
    fallback
}

#[cfg(test)]
mod location_tests {
    use super::*;
    use crate::game::roguelike::{JOURNEY_MAPS, scene::places};

    #[test]
    fn all_journey_landmarks_have_matching_collision_and_reachable_approaches() {
        for (chapter, maps) in JOURNEY_MAPS.iter().enumerate() {
            for (stage, &map) in maps.iter().enumerate() {
                let variant = chapter * 16 + stage;
                let mut rng = Rng(0x28_5CE0 + variant as u64);
                let tiles = generate_stage_tiles(map, variant, &mut rng);
                let built = MapData::generated(map, tiles.clone());
                let start = (1..MAP_W - 1)
                    .find_map(|col| {
                        (1..MAP_H - 1)
                            .find(|&row| built.at(col, row).walkable())
                            .map(|row| (col, row))
                    })
                    .unwrap();
                let distances = distance_field(&built, &[start]);
                for landmark in places::landmarks(map, variant) {
                    let left = landmark.col - landmark.width / 2;
                    for row in landmark.row - landmark.depth + 1..=landmark.row {
                        for col in left..left + landmark.width {
                            assert_eq!(
                                built.at(col, row).walkable(),
                                !landmark.blocks_column(col),
                                "{chapter}-{stage} {} at{col},{row}",
                                landmark.asset
                            );
                        }
                    }
                    assert!(
                        [
                            (landmark.col, landmark.row + 1),
                            (left - 1, landmark.row),
                            (left + landmark.width, landmark.row)
                        ]
                        .iter()
                        .any(|&(col, row)| distances[row as usize][col as usize] != u16::MAX),
                        "{chapter}-{stage} {} has no approach",
                        landmark.asset
                    );
                    assert!(
                        std::path::Path::new(&format!(
                            "assets/props/landmark-{}.png",
                            landmark.asset
                        ))
                        .exists()
                    );
                }
            }
        }
    }
}

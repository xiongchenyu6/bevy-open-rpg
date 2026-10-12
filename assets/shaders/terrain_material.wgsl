#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct TerrainMaterialParams {
    floor_tint: vec4<f32>,
    grass_tint: vec4<f32>,
    wall_tint: vec4<f32>,
    water_tint: vec4<f32>,
    map: vec4<f32>,
    sample: vec4<f32>,
    phase: vec4<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var tile_ids: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var floor_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var floor_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var grass_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var grass_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var wall_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(6) var wall_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(7) var water_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(8) var water_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(9) var<uniform> material: TerrainMaterialParams;

fn terrain_id(coord: vec2<i32>) -> u32 {
    let max_coord = vec2<i32>(i32(material.map.x) - 1, i32(material.map.y) - 1);
    let sample_coord = clamp(coord, vec2<i32>(0, 0), max_coord);
    return u32(round(textureLoad(tile_ids, sample_coord, 0).r * 255.0));
}

// Paths and encounter grass are walkable; cliffs and water are distinct obstacles.
fn boundary_class(id: u32) -> u32 {
    // Architecture blocks movement like a wall, but uses ground underneath.
    let blocked = select(select(0u, id, id >= 2u), 2u, id == 4u);
    // A bridge displays the same water but its deck is traversable.
    return select(blocked, 0u, id == 5u);
}

fn ping_pong(value: f32, span: f32) -> f32 {
    let period = span * 2.0;
    let phase = value - floor(value / period) * period;
    return select(phase, period - phase, phase > span);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let tile_size = material.map.z;
    let grid = vec2<f32>(
        mesh.world_position.x / tile_size + (material.map.x - 1.0) * 0.5,
        (material.map.y - 1.0) * 0.5 - mesh.world_position.y / tile_size,
    );

    // Grid coordinates address tile centres. Ownership switches at the real
    // collision edge, never through a four-way alpha blend of unrelated art.
    let cell = vec2<i32>(floor(grid + vec2<f32>(0.5)));
    let cell_uv = fract(grid + vec2<f32>(0.5));
    let id = terrain_id(cell);
    let surface_class = boundary_class(id);
    let left_class = boundary_class(terrain_id(cell + vec2<i32>(-1, 0)));
    let right_class = boundary_class(terrain_id(cell + vec2<i32>(1, 0)));
    let top_class = boundary_class(terrain_id(cell + vec2<i32>(0, -1)));
    let bottom_class = boundary_class(terrain_id(cell + vec2<i32>(0, 1)));
    let edge_distance = min(min(
        select(1.0, cell_uv.x, surface_class != left_class),
        select(1.0, 1.0 - cell_uv.x, surface_class != right_class)), min(
        select(1.0, cell_uv.y, surface_class != top_class),
        select(1.0, 1.0 - cell_uv.y, surface_class != bottom_class)));
    let wall_distance = min(min(
        select(1.0, cell_uv.x, left_class == 2u),
        select(1.0, 1.0 - cell_uv.x, right_class == 2u)), min(
        select(1.0, cell_uv.y, top_class == 2u),
        select(1.0, 1.0 - cell_uv.y, bottom_class == 2u)));

    let warp = vec2<f32>(
        sin((grid.y + material.phase.y) * 0.73),
        cos((grid.x + material.phase.x) * 0.61),
    ) * material.phase.z;
    let sample_grid = grid + material.phase.xy + warp;
    let source = vec2<f32>(material.sample.y) + vec2<f32>(
        ping_pong(sample_grid.x * material.sample.w, material.sample.z),
        ping_pong(sample_grid.y * material.sample.w, material.sample.z),
    );
    let texture_uv = source / material.sample.x;

    // Pure surface art is resolution-independent and has no painted objects.
    let surface_uv = (source - vec2<f32>(material.sample.y)) / material.sample.z;
    // Samples remain outside divergent branches to preserve implicit derivatives.
    // Pure seamless ground repeats directly: reflecting cobbles created large
    // artificial diamond patterns across courtyards. A rotated world sample
    // keeps paving orientation independent of the collision grid.
    let ground_uv = fract(vec2<f32>(
        sample_grid.x * 0.966 - sample_grid.y * 0.259,
        sample_grid.x * 0.259 + sample_grid.y * 0.966,
    ) * (material.sample.w / material.sample.z));
    let floor_color = textureSample(floor_texture, floor_sampler, ground_uv) * material.floor_tint;
    let grass_color = textureSample(grass_texture, grass_sampler, ground_uv) * material.grass_tint;
    let wall_color = textureSample(wall_texture, wall_sampler, texture_uv) * material.wall_tint;
    let water_color = textureSample(water_texture, water_sampler, surface_uv) * material.water_tint;
    var color = select(floor_color, grass_color, id == 1u);
    color = select(color, wall_color, id == 2u);
    color = select(color, water_color, id == 3u || id == 5u);

    // Brush-like contact shading belongs to the existing surface; it cannot
    // introduce a fake island, rock or walkway from the adjacent texture.
    let grain = 0.85 + 0.15 * sin(grid.x * 29.3 + sin(grid.y * 17.1));
    let contact = (1.0 - smoothstep(0.0, material.map.w, edge_distance)) * grain;
    let wall_shadow = 1.0 - smoothstep(0.0, 0.18, wall_distance);
    if surface_class == 0u {
        color = vec4<f32>(color.rgb * (1.0 - wall_shadow * 0.25 - contact * 0.10), 1.0);
    } else if surface_class == 2u {
        color = vec4<f32>(color.rgb * (1.0 + contact * 0.18), 1.0);
    } else {
        color = vec4<f32>(color.rgb * (1.0 - contact * 0.38)
            + vec3<f32>(0.10, 0.17, 0.16) * contact * 0.25, 1.0);
    }
    return vec4<f32>(color.rgb, 1.0);
}

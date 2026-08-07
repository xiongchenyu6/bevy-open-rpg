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

fn id_weight(id: u32, expected: u32) -> f32 {
    return select(0.0, 1.0, id == expected);
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

    let base = vec2<i32>(floor(grid));
    let cell_uv = fract(grid);
    let blend_width = material.map.w;
    let edge_warp = vec2<f32>(
        sin((grid.y + material.phase.y) * 2.17)
            + 0.55 * sin((grid.y - material.phase.x) * 5.11),
        sin((grid.x + material.phase.x) * 2.43)
            + 0.55 * sin((grid.x + material.phase.y) * 4.79),
    ) * 0.075;
    let blend = smoothstep(
        vec2<f32>(0.5 - blend_width),
        vec2<f32>(0.5 + blend_width),
        cell_uv + edge_warp,
    );
    let corner_weights = vec4<f32>(
        (1.0 - blend.x) * (1.0 - blend.y),
        blend.x * (1.0 - blend.y),
        (1.0 - blend.x) * blend.y,
        blend.x * blend.y,
    );
    let ids = vec4<u32>(
        terrain_id(base),
        terrain_id(base + vec2<i32>(1, 0)),
        terrain_id(base + vec2<i32>(0, 1)),
        terrain_id(base + vec2<i32>(1, 1)),
    );

    let weights = vec4<f32>(
        dot(corner_weights, vec4<f32>(
            id_weight(ids.x, 0u), id_weight(ids.y, 0u),
            id_weight(ids.z, 0u), id_weight(ids.w, 0u),
        )),
        dot(corner_weights, vec4<f32>(
            id_weight(ids.x, 1u), id_weight(ids.y, 1u),
            id_weight(ids.z, 1u), id_weight(ids.w, 1u),
        )),
        dot(corner_weights, vec4<f32>(
            id_weight(ids.x, 2u), id_weight(ids.y, 2u),
            id_weight(ids.z, 2u), id_weight(ids.w, 2u),
        )),
        dot(corner_weights, vec4<f32>(
            id_weight(ids.x, 3u), id_weight(ids.y, 3u),
            id_weight(ids.z, 3u), id_weight(ids.w, 3u),
        )),
    );

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

    let floor_color = textureSample(floor_texture, floor_sampler, texture_uv) * material.floor_tint;
    let grass_color = textureSample(grass_texture, grass_sampler, texture_uv) * material.grass_tint;
    let wall_color = textureSample(wall_texture, wall_sampler, texture_uv) * material.wall_tint;
    let water_color = textureSample(water_texture, water_sampler, texture_uv) * material.water_tint;
    let color = floor_color * weights.x
        + grass_color * weights.y
        + wall_color * weights.z
        + water_color * weights.w;
    return vec4<f32>(color.rgb, 1.0);
}

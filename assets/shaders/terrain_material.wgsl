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

// Which painted surface a terrain id shows: 0 path, 1 grass, 2 cliff, 3 water.
// Architecture stands on path ground and a bridge shows the water under it.
fn surface_kind(id: u32) -> u32 {
    if id == 1u { return 1u; }
    if id == 2u { return 2u; }
    if id == 3u || id == 5u { return 3u; }
    return 0u;
}

// How much of each surface a corner cell contributes, as (path, grass, cliff, water).
fn surface_weight(cell: vec2<i32>, weight: f32) -> vec4<f32> {
    let kind = surface_kind(terrain_id(cell));
    return vec4<f32>(
        select(0.0, weight, kind == 0u),
        select(0.0, weight, kind == 1u),
        select(0.0, weight, kind == 2u),
        select(0.0, weight, kind == 3u),
    );
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

    // Grid coordinates address tile centres. Borders between surfaces follow
    // a wandering line near the collision edge and cross-fade over a narrow
    // band, so the map reads as painted ground rather than as square cells.
    let wander = vec2<f32>(
        sin(grid.y * 2.3 + sin(grid.x * 1.1) * 1.9),
        sin(grid.x * 2.7 + sin(grid.y * 1.3) * 1.7),
    ) * 0.15 + vec2<f32>(
        sin(grid.y * 7.1 + grid.x * 3.3),
        sin(grid.x * 6.3 - grid.y * 2.9),
    ) * 0.045;
    let border = grid + wander;
    let corner = vec2<i32>(floor(border));
    let band = material.map.w;
    let t = smoothstep(vec2<f32>(0.5 - band), vec2<f32>(0.5 + band), fract(border));
    let weights = surface_weight(corner, (1.0 - t.x) * (1.0 - t.y))
        + surface_weight(corner + vec2<i32>(1, 0), t.x * (1.0 - t.y))
        + surface_weight(corner + vec2<i32>(0, 1), (1.0 - t.x) * t.y)
        + surface_weight(corner + vec2<i32>(1, 1), t.x * t.y);

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
    var color = floor_color.rgb * weights.x + grass_color.rgb * weights.y
        + wall_color.rgb * weights.z + water_color.rgb * weights.w;

    // Shading lives in the cross-fade itself: a damp, slightly foamy bank
    // where land meets water and a soft shadow at the foot of a cliff.
    let grain = 0.85 + 0.15 * sin(grid.x * 29.3 + sin(grid.y * 17.1));
    let shore = 4.0 * weights.w * (1.0 - weights.w) * grain;
    let foot = 4.0 * weights.z * (1.0 - weights.z);
    color = color * (1.0 - shore * 0.30 - foot * 0.22)
        + vec3<f32>(0.10, 0.17, 0.16) * shore * 0.22;
    return vec4<f32>(color, 1.0);
}

//include:pbr.wgsl
//include:camera.wgsl

// Vertex shader
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@vertex
fn vs_main(
  @builtin(vertex_index) in_vertex_index: u32,
) -> @builtin(position) vec4<f32> {
    var quad_verts = array(
        vec4(-1.0, -1.0, 0.0, 1.0),
        vec4( 1.0, -1.0, 0.0, 1.0),
        vec4(-1.0,  1.0, 0.0, 1.0),
        vec4(-1.0,  1.0, 0.0, 1.0),
        vec4( 1.0, -1.0, 0.0, 1.0),
        vec4( 1.0,  1.0, 0.0, 1.0)
    );
    return quad_verts[in_vertex_index];
}

@group(1) @binding(0)
var texture_g_buffer_normal: texture_2d<f32>;
@group(1) @binding(1)
var texture_g_buffer_albedo: texture_2d<f32>;
@group(1) @binding(2)
var texture_g_buffer_emissive: texture_2d<f32>;
@group(1) @binding(3)
var texture_g_buffer_ao_roughness_metallic: texture_2d<f32>;
@group(1) @binding(4)
var texture_g_buffer_depth: texture_depth_2d;

@group(2) @binding(0)
var<uniform> lightsBuffer: LightsUniform;

@group(3) @binding(0)
var texture_shadow_map: texture_depth_2d;
@group(3) @binding(1)
var sampler_shadow_map: sampler_comparison;
@group(3) @binding(2)
var<uniform> light_as_camera: CameraUniform;

fn world_from_screen_coord(coord : vec2<f32>, depth_sample: f32) -> vec3<f32> {
    // reconstruct world-space position from the screen coordinate
    let pos_clip = vec4(coord.x * 2.0 - 1.0, (1.0 - coord.y) * 2.0 - 1.0, depth_sample, 1.0);
    let pos_world_w = camera.inv_view_proj * pos_clip;
    let pos_world = pos_world_w.xyz / pos_world_w.www;
    return pos_world;
}

// Fragment shader
@fragment
fn fs_main(@builtin(position) coord : vec4<f32>) -> @location(0) vec4<f32> {
    // normal vector (after normal mapping which was computed in write to g buffers shader)
    let normal = textureLoad(
        texture_g_buffer_normal,
        vec2<i32>(floor(coord.xy)),
        0
    ).xyz;

    // albedo texture
    let albedo = textureLoad(
        texture_g_buffer_albedo,
        vec2<i32>(floor(coord.xy)),
        0
    ).rgba;

    // emissive
    let emissive = textureLoad(
        texture_g_buffer_emissive,
        vec2<i32>(floor(coord.xy)),
        0
    ).rgba;

    // ao roughness metallic
    let ao_roughness_metallic = textureLoad(
        texture_g_buffer_ao_roughness_metallic,
        vec2<i32>(floor(coord.xy)),
        0
    ).rgba;
    let ao = ao_roughness_metallic.r;
    let roughness = ao_roughness_metallic.g;
    let metallic = ao_roughness_metallic.b;

    // sample from depth buffer
    let depth = textureLoad(
        texture_g_buffer_depth,
        vec2<i32>(floor(coord.xy)),
        0
    );

    // compute world position using depth buffer
    let depth_buffer_size = textureDimensions(texture_g_buffer_depth);
    let coord_uv = coord.xy / vec2<f32>(depth_buffer_size);
    let world_position = world_from_screen_coord(coord_uv, depth);

    // convert fragment position to light position view matrix, then convert XY of this to (0, 1) range
    var fragment_shadow_position_raw = light_as_camera.view_proj * vec4(world_position, 1.0);

    // shadow calculation
    let fragment_shadow_position = vec3(
        fragment_shadow_position_raw.xy * vec2(0.5, -0.5) + vec2(0.5),
        fragment_shadow_position_raw.z
    );
    // pcf filtering to compute visibility of shadow
    var visibility = 0.0;
    let shadow_depth_texture_size: f32 = vec2<f32>(textureDimensions(texture_shadow_map)).x;
    let one_over_shadow_depth_texture_size = 1.0 / shadow_depth_texture_size;
    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            let offset = vec2<f32>(vec2(x, y)) * one_over_shadow_depth_texture_size;
            visibility += textureSampleCompare(
                texture_shadow_map, sampler_shadow_map,
                fragment_shadow_position.xy + offset, fragment_shadow_position.z - 0.002
            );
        }
    }
    visibility /= 9.0;
    var is_outside_bounds = false;
    if (fragment_shadow_position_raw.y > 1.0) {
        visibility = 1.0;
        is_outside_bounds = true;
    }
    if (fragment_shadow_position_raw.y < -1.0) {
        visibility = 1.0;
        is_outside_bounds = true;
    }
    if (fragment_shadow_position_raw.x > 1.0) {
        visibility = 1.0;
        is_outside_bounds = true;
    }
    if (fragment_shadow_position_raw.x < -1.0) {
        visibility = 1.0;
        is_outside_bounds = true;
    }
    if (fragment_shadow_position_raw.z > 1.0) {
        visibility = 1.0;
        is_outside_bounds = true;
    }
    if (fragment_shadow_position_raw.z < 0.0) {
        visibility = 1.0;
        is_outside_bounds = true;
    }

    // depth >= 1 means nothing was there at this pixel
    if (depth >= 1.0) {
        discard;
    }

    // final color
    var final_color_rgb = compute_final_color(visibility, world_position, camera.position, normal, albedo, emissive, ao, roughness, metallic);

//    if (is_outside_bounds) {
//        final_color_rgb *= vec3(1.0, 0.0, 0.0);
//    }

    return vec4(final_color_rgb, 1.0);
}
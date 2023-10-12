//include:pbr.wgsl
//include:camera.wgsl
//include:shadow.wgsl
//include:skinning.wgsl

// Vertex shader
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(0) @binding(1)
var<uniform> boneTransformsUniform: BoneTransformsUniform;

@group(0) @binding(2)
var<uniform> lightsBuffer: LightsUniform;

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

// shadow cascades
@group(2) @binding(0)
var texture_shadow_map_0: texture_depth_2d;
@group(2) @binding(1)
var sampler_shadow_map_0: sampler_comparison;
@group(2) @binding(2)
var<uniform> light_as_camera_0: CameraUniform;

@group(2) @binding(3)
var texture_shadow_map_1: texture_depth_2d;
@group(2) @binding(4)
var sampler_shadow_map_1: sampler_comparison;
@group(2) @binding(5)
var<uniform> light_as_camera_1: CameraUniform;

@group(2) @binding(6)
var texture_shadow_map_2: texture_depth_2d;
@group(2) @binding(7)
var sampler_shadow_map_2: sampler_comparison;
@group(2) @binding(8)
var<uniform> light_as_camera_2: CameraUniform;

@group(2) @binding(9)
var texture_shadow_map_3: texture_depth_2d;
@group(2) @binding(10)
var sampler_shadow_map_3: sampler_comparison;
@group(2) @binding(11)
var<uniform> light_as_camera_3: CameraUniform;

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

    // calculate shadow_visibility
    var shadow_visibility = get_visibility_for_shadow(world_position, texture_shadow_map_0, sampler_shadow_map_0, light_as_camera_0);
    if (shadow_visibility >= 1.0) {
        shadow_visibility = get_visibility_for_shadow(world_position, texture_shadow_map_1, sampler_shadow_map_1, light_as_camera_1);
        if (shadow_visibility >= 1.0) {
            shadow_visibility = get_visibility_for_shadow(world_position, texture_shadow_map_2, sampler_shadow_map_2, light_as_camera_2);
            if (shadow_visibility >= 1.0) {
                shadow_visibility = get_visibility_for_shadow(world_position, texture_shadow_map_3, sampler_shadow_map_3, light_as_camera_3);
            }
        }
    }

    // depth >= 1 means nothing was there at this pixel
    if (depth >= 1.0) {
        discard;
    }

    // final color
    var final_color_rgb = compute_final_color(shadow_visibility, world_position, camera.position, normal, albedo, emissive, ao, roughness, metallic);

    return vec4(final_color_rgb, 1.0);
}
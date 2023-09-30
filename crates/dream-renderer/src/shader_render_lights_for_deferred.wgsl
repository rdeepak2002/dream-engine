struct CameraUniform {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>
};

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

struct Light {
  position: vec3<f32>,
  _padding1: u32,
  color: vec3<f32>,
  _padding2: u32,
}

struct LightsUniform {
  lights: array<Light, 4>
};

@group(2) @binding(0)
var<uniform> lightsBuffer: LightsUniform;

fn world_from_screen_coord(coord : vec2<f32>, depth_sample: f32) -> vec3<f32> {
    // reconstruct world-space position from the screen coordinate
    let pos_clip = vec4(coord.x * 2.0 - 1.0, (1.0 - coord.y) * 2.0 - 1.0, depth_sample, 1.0);
    let pos_world_w = camera.inv_view_proj * pos_clip;
    let pos_world = pos_world_w.xyz / pos_world_w.www;
    return pos_world;
}

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

    // depth >= 1 means nothing was there at this pixel
    if (depth >= 1.0) {
        discard;
    }

    // compute world position using depth buffer
    let depth_buffer_size = textureDimensions(texture_g_buffer_depth);
    let coord_uv = coord.xy / vec2<f32>(depth_buffer_size);
    let world_position = world_from_screen_coord(coord_uv, depth);

    // final color
    // TODO: use num_lights uniform variable
    var final_color_rgb = vec3(0., 0., 0.);
    for (var i = 0u; i < 4u; i += 1u) {
        let light = lightsBuffer.lights[i];
        let res = (albedo + emissive).rgb * light.color;
        final_color_rgb += res;
    }

    return vec4(final_color_rgb, 1.0);
}
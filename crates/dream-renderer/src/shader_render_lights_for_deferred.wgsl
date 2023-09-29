@vertex
fn vs_main(
  @builtin(vertex_index) in_vertex_index: u32,
) -> @builtin(position) vec4<f32> {
    if (in_vertex_index == 0u) {
        return vec4(vec2(-1.0, -1.0), 0.0, 1.0);
    } else if (in_vertex_index == 1u) {
        return vec4(vec2(1.0, -1.0), 0.0, 1.0);
    } else if (in_vertex_index == 2u) {
        return vec4(vec2(-1.0, 1.0), 0.0, 1.0);
    } else if (in_vertex_index == 3u) {
        return vec4(vec2(-1.0, 1.0), 0.0, 1.0);
    } else if (in_vertex_index == 4u) {
        return vec4(vec2(1.0, -1.0), 0.0, 1.0);
    } else {
        return vec4(vec2(1.0, 1.0), 0.0, 1.0);
    }
}

@group(0) @binding(0)
var texture_g_buffer_normal: texture_2d<f32>;
@group(0) @binding(1)
var sampler_g_buffer_normal: sampler;
@group(0) @binding(2)
var texture_g_buffer_albedo: texture_2d<f32>;
@group(0) @binding(3)
var sampler_g_buffer_albedo: sampler;
@group(0) @binding(4)
var texture_g_buffer_emissive: texture_2d<f32>;
@group(0) @binding(5)
var sampler_g_buffer_emissive: sampler;
@group(0) @binding(6)
var texture_g_buffer_ao_roughness_metallic: texture_2d<f32>;
@group(0) @binding(7)
var sampler_g_buffer_ao_roughness_metallic: sampler;
@group(0) @binding(8)
var texture_g_buffer_depth: texture_depth_2d;
//@group(0) @binding(9)
//var sampler_g_buffer_depth: sampler;

struct Light {
  position: vec3<f32>,
  _padding1: u32,
  color: vec3<f32>,
  _padding2: u32,
}

struct LightsUniform {
  lights: array<Light, 4>
};

@group(1) @binding(0)
var<uniform> lightsBuffer: LightsUniform;

//fn world_from_screen_coord(coord : vec2<f32>, depth_sample: f32) -> vec3<f32> {
//    // reconstruct world-space position from the screen coordinate
//    let posClip = vec4(coord.x * 2.0 - 1.0, (1.0 - coord.y) * 2.0 - 1.0, depth_sample, 1.0);
//    let posWorldW = camera.invViewProjectionMatrix * posClip;
//    let posWorld = posWorldW.xyz / posWorldW.www;
//    return posWorld;
//}

@fragment
fn fs_main(@builtin(position) coord : vec4<f32>) -> @location(0) vec4<f32> {
    let depth = textureLoad(
        texture_g_buffer_depth,
        vec2<i32>(floor(coord.xy)),
        0
    );

    if (depth >= 1.0) {
        discard;
    }

    let albedo = textureLoad(
        texture_g_buffer_albedo,
        vec2<i32>(floor(coord.xy)),
        0
    ).rgba;

    let light = lightsBuffer.lights[0];

    return vec4(albedo.rgb * light.color, 1.0);

//    let albedo = textureSample(texture_g_buffer_albedo, sampler_g_buffer_albedo, coord.xy * vec2(1.0/2000.0, 1.0/1000.0)).rgb;
//    return vec4(albedo, 1.0);
}
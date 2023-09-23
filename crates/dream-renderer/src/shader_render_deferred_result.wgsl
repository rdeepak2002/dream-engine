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

//@group(0) @binding(0)
//var texture_g_buffer_normal: texture_2d<f32>;
//@group(0) @binding(1)
//var sampler_g_buffer_normal: sampler;
//@group(0) @binding(2)
//var texture_g_buffer_albedo: texture_2d<f32>;
//@group(0) @binding(3)
//var sampler_g_buffer_albedo: sampler;
//@group(0) @binding(4)
//var texture_g_buffer_emissive: texture_2d<f32>;
//@group(0) @binding(5)
//var sampler_g_buffer_emissive: sampler;
//@group(0) @binding(6)
//var texture_g_buffer_ao_roughness_metallic: texture_2d<f32>;
//@group(0) @binding(7)
//var sampler_g_buffer_ao_roughness_metallic: sampler;

@group(0) @binding(0)
var texture_g_buffer_normal: texture_2d<f32>;
@group(0) @binding(1)
var texture_g_buffer_albedo: texture_2d<f32>;
@group(0) @binding(2)
var texture_g_buffer_emissive: texture_2d<f32>;
@group(0) @binding(3)
var texture_g_buffer_ao_roughness_metallic: texture_2d<f32>;

@fragment
fn fs_main(@builtin(position) coord : vec4<f32>) -> @location(0) vec4<f32> {
  let albedo = textureLoad(
    texture_g_buffer_albedo,
    vec2<i32>(floor(coord.xy)),
    0
  ).rgb;
  return vec4(albedo, 1.0);
}
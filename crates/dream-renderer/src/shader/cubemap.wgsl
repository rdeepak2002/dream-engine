//include:camera.wgsl
//include:pbr_structs.wgsl

struct VertexOutput {
  @builtin(position) position : vec4<f32>,
  @location(0) world_position : vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(0) @binding(1)
var<uniform> lightsBuffer: LightsUniform;

@vertex
fn vs_main(
    @builtin(vertex_index) id: u32,
) -> VertexOutput {
    let uv = vec2<f32>(vec2<u32>(
        id & 1u,
        (id >> 1u) & 1u,
    ));
    var out: VertexOutput;
    out.world_position = vec4(uv * 4.0 - 1.0, 1.0, 1.0);
    out.position = out.world_position;
    return out;
}

@group(1) @binding(0)
var cubemap_texture: texture_cube<f32>;
@group(1) @binding(1)
var cubemap_sampler: sampler;

fn aces_tone_map(hdr: vec3<f32>) -> vec3<f32> {
    let m1 = mat3x3(
        0.59719, 0.07600, 0.02840,
        0.35458, 0.90834, 0.13383,
        0.04823, 0.01566, 0.83777,
    );
    let m2 = mat3x3(
        1.60475, -0.10208, -0.00327,
        -0.53108,  1.10813, -0.07276,
        -0.07367, -0.00605,  1.07602,
    );
    let v = m1 * hdr;
    let a = v * (v + 0.0245786) - 0.000090537;
    let b = v * (0.983729 * v + 0.4329510) + 0.238081;
    return clamp(m2 * (a / b), vec3(0.0), vec3(1.0));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let view_pos_homogeneous = camera.inv_proj * in.world_position;
    let view_ray_direction = view_pos_homogeneous.xyz / view_pos_homogeneous.w;
    let ray_direction = normalize((camera.inv_view * vec4(view_ray_direction, 0.0)).xyz);
    let world_pos = ray_direction;

    var tex_sample = textureSample(cubemap_texture, cubemap_sampler, world_pos).rgb;
//    tex_sample.r = clamp(tex_sample.r, 0, 50);
//    tex_sample.g = clamp(tex_sample.g, 0, 50);
//    tex_sample.b = clamp(tex_sample.b, 0, 50);

    // tone mapping
    var mapped = tex_sample;
//    mapped = aces_tone_map(tex_sample);
//    let exposure = 1.0;
//    let mapped = vec3(1.0) - exp(-tex_sample * exposure);

    // gamma correction
//    let gamma = 2.2;
//    let gamma_corrected = pow(mapped, vec3(1.0 / gamma));
    let gamma_corrected = mapped;

    return vec4<f32>(gamma_corrected, 1.0);
}
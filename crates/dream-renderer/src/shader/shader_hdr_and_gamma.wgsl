struct VertexOutput {
  @builtin(position) position : vec4<f32>,
  @location(0) tex_coords : vec2<f32>,
}

@vertex
fn vs_main(
  @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var pos = array(
        vec2( 1.0,  1.0),
        vec2( 1.0, -1.0),
        vec2(-1.0, -1.0),
        vec2( 1.0,  1.0),
        vec2(-1.0, -1.0),
        vec2(-1.0,  1.0),
    );
    var uv = array(
        vec2(1.0, 0.0),
        vec2(1.0, 1.0),
        vec2(0.0, 1.0),
        vec2(1.0, 0.0),
        vec2(0.0, 1.0),
        vec2(0.0, 0.0),
    );
    var output : VertexOutput;
    output.position = vec4(pos[in_vertex_index], 0.0, 1.0);
    output.tex_coords = uv[in_vertex_index];
    return output;
}

@group(0) @binding(0)
var frame_texture: texture_2d<f32>;
@group(0) @binding(1)
var frame_texture_sampler: sampler;

@group(0) @binding(2)
var bloom_texture: texture_2d<f32>;
@group(0) @binding(3)
var bloom_texture_sampler: sampler;

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
    var hdr_color = textureSample(frame_texture, frame_texture_sampler, in.tex_coords).xyz;
    hdr_color += textureSample(bloom_texture, bloom_texture_sampler, in.tex_coords).xyz;

    // exposure tone mapping
    let exposure = 2.0;
    var mapped = aces_tone_map(hdr_color);
//    var mapped = vec3(1.0) - exp(-hdr_color * exposure);
//    var mapped = frame_color / (frame_color + vec3(1.0));
    // gamma correction
    let gamma = 2.2;
    mapped = pow(mapped, vec3(1.0 / gamma));

    return vec4(mapped, 1.0);
}
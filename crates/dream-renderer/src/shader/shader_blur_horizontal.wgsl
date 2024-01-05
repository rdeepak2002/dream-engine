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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let frame_color = textureSample(frame_texture, frame_texture_sampler, in.tex_coords).xyz;

    var blurScale = 2.0;
    var blurStrength = 1.0;
//    var blurScale = 1.5;
//    var blurStrength = 1.0;

    var weights: array<f32, 5> = array(0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216);
    var tex_offset: vec2<f32> = 1.0 / vec2<f32>(textureDimensions(frame_texture, 0)) * blurScale;
    var result: vec3<f32> = frame_color * weights[0];

    for(var i = 1; i < 5; i++) {
        result += textureSample(frame_texture, frame_texture_sampler, in.tex_coords + vec2<f32>(tex_offset.x * f32(i), 0.0)).xyz * weights[i] * blurStrength;
        result += textureSample(frame_texture, frame_texture_sampler, in.tex_coords - vec2<f32>(tex_offset.x * f32(i), 0.0)).xyz * weights[i] * blurStrength;
    }

    return vec4(result, 1.0);
}
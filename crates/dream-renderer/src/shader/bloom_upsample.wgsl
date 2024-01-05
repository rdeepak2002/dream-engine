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
var src_texture: texture_2d<f32>;
@group(0) @binding(1)
var src_texture_sampler: sampler;
@group(1) @binding(0)
var<uniform> filter_radius: f32;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	// source: https://github.com/JoeyDeVries/LearnOpenGL/blob/master/src/8.guest/2022/6.physically_based_bloom/6.new_downsample.fs

	// The filter kernel is applied with a radius, specified in texture
	// coordinates, so that the radius will vary across mip resolutions.
    let src_texel_size: vec2<f32> = 1.0 / vec2<f32>(textureDimensions(src_texture, 0));
	let x: f32 = filter_radius;
	let y: f32 = filter_radius;

	// Take 9 samples around current texel:
	// a - b - c
	// d - e - f
	// g - h - i
	// === ('e' is the current texel) ===
	let a = textureSample(src_texture, src_texture_sampler, vec2<f32>(in.tex_coords.x - x, in.tex_coords.y + y)).rgb;
	let b = textureSample(src_texture, src_texture_sampler, vec2<f32>(in.tex_coords.x,     in.tex_coords.y + y)).rgb;
	let c = textureSample(src_texture, src_texture_sampler, vec2<f32>(in.tex_coords.x + x, in.tex_coords.y + y)).rgb;

	let d = textureSample(src_texture, src_texture_sampler, vec2<f32>(in.tex_coords.x - x, in.tex_coords.y)).rgb;
	let e = textureSample(src_texture, src_texture_sampler, vec2<f32>(in.tex_coords.x,     in.tex_coords.y)).rgb;
	let f = textureSample(src_texture, src_texture_sampler, vec2<f32>(in.tex_coords.x + x, in.tex_coords.y)).rgb;

	let g = textureSample(src_texture, src_texture_sampler, vec2<f32>(in.tex_coords.x - x, in.tex_coords.y - y)).rgb;
	let h = textureSample(src_texture, src_texture_sampler, vec2<f32>(in.tex_coords.x,     in.tex_coords.y - y)).rgb;
	let i = textureSample(src_texture, src_texture_sampler, vec2<f32>(in.tex_coords.x + x, in.tex_coords.y - y)).rgb;

    var upsample: vec3<f32> = vec3(0.0, 0.0, 0.0);

	// Apply weighted distribution, by using a 3x3 tent filter:
	//  1   | 1 2 1 |
	// -- * | 2 4 2 |
	// 16   | 1 2 1 |
	upsample = e*4.0;
	upsample += (b+d+f+h)*2.0;
	upsample += (a+c+g+i);
	upsample *= 1.0 / 16.0;

    return vec4(upsample, 1.0);
}
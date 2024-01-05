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
var<uniform> mip_level: u32;

fn PowVec3(v: vec3<f32>, p: f32) -> vec3<f32> {
    return vec3(pow(v.x, p), pow(v.y, p), pow(v.z, p));
}

fn ToSRGB(v: vec3<f32>) -> vec3<f32> {
    let invGamma: f32 = 1.0 / 2.2;
    return PowVec3(v, invGamma);
}

fn sRGBToLuma(col: vec3<f32>) -> f32 {
    //return dot(col, vec3(0.2126f, 0.7152f, 0.0722f));
	return dot(col, vec3(0.299f, 0.587f, 0.114f));
}

fn KarisAverage(col: vec3<f32>) -> f32 {
	// Formula is 1 / (1 + luma)
	let luma: f32 = sRGBToLuma(ToSRGB(col)) * 0.25;
	return 1.0 / (1.0 + luma);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // source: https://github.com/JoeyDeVries/LearnOpenGL/blob/master/src/8.guest/2022/6.physically_based_bloom/6.new_upsample.fs
    let src_texel_size: vec2<f32> = 1.0 / vec2<f32>(textureDimensions(src_texture, 0));
	let x: f32 = src_texel_size.x;
	let y: f32 = src_texel_size.y;

	// Take 13 samples around current texel:
	// a - b - c
	// - j - k -
	// d - e - f
	// - l - m -
	// g - h - i
	// === ('e' is the current texel) ===
	let a = textureSample(src_texture, src_texture_sampler, vec2<f32>(in.tex_coords.x - 2.0*x, in.tex_coords.y + 2.0*y)).rgb;
	let b = textureSample(src_texture, src_texture_sampler, vec2<f32>(in.tex_coords.x,       in.tex_coords.y   + 2.0*y)).rgb;
	let c = textureSample(src_texture, src_texture_sampler, vec2<f32>(in.tex_coords.x + 2.0*x, in.tex_coords.y + 2.0*y)).rgb;

	let d = textureSample(src_texture, src_texture_sampler, vec2<f32>(in.tex_coords.x - 2.0*x, in.tex_coords.y)).rgb;
	let e = textureSample(src_texture, src_texture_sampler, vec2<f32>(in.tex_coords.x,         in.tex_coords.y)).rgb;
	let f = textureSample(src_texture, src_texture_sampler, vec2<f32>(in.tex_coords.x + 2.0*x, in.tex_coords.y)).rgb;

	let g = textureSample(src_texture, src_texture_sampler, vec2<f32>(in.tex_coords.x - 2.0*x, in.tex_coords.y - 2.0*y)).rgb;
	let h = textureSample(src_texture, src_texture_sampler, vec2<f32>(in.tex_coords.x,         in.tex_coords.y - 2.0*y)).rgb;
	let i = textureSample(src_texture, src_texture_sampler, vec2<f32>(in.tex_coords.x + 2.0*x, in.tex_coords.y - 2.0*y)).rgb;

	let j = textureSample(src_texture, src_texture_sampler, vec2<f32>(in.tex_coords.x - x, in.tex_coords.y + y)).rgb;
	let k = textureSample(src_texture, src_texture_sampler, vec2<f32>(in.tex_coords.x + x, in.tex_coords.y + y)).rgb;
	let l = textureSample(src_texture, src_texture_sampler, vec2<f32>(in.tex_coords.x - x, in.tex_coords.y - y)).rgb;
	let m = textureSample(src_texture, src_texture_sampler, vec2<f32>(in.tex_coords.x + x, in.tex_coords.y - y)).rgb;

    // Apply weighted distribution:
    // 0.5 + 0.125 + 0.125 + 0.125 + 0.125 = 1
    // a,b,d,e * 0.125
    // b,c,e,f * 0.125
    // d,e,g,h * 0.125
    // e,f,h,i * 0.125
    // j,k,l,m * 0.5
    // This shows 5 square areas that are being sampled. But some of them overlap,
    // so to have an energy preserving downsample we need to make some adjustments.
    // The weights are the distributed, so that the sum of j,k,l,m (e.g.)
    // contribute 0.5 to the final color output. The code below is written
    // to effectively yield this sum. We get:
    // 0.125*5 + 0.03125*4 + 0.0625*4 = 1
    var downsample: vec3<f32> = vec3(0.0, 0.0, 0.0);

	// Check if we need to perform Karis average on each block of 4 samples
    var groups = array(
        vec3(0.0,  0.0, 0.0),
        vec3(0.0,  0.0, 0.0),
        vec3(0.0,  0.0, 0.0),
        vec3(0.0,  0.0, 0.0),
        vec3(0.0,  0.0, 0.0)
    );

    if (mip_level == 0u) {
        // We are writing to mip 0, so we need to apply Karis average to each block
        // of 4 samples to prevent fireflies (very bright subpixels, leads to pulsating
        // artifacts).
        groups[0] = (a+b+d+e) * (0.125f/4.0f);
        groups[1] = (b+c+e+f) * (0.125f/4.0f);
        groups[2] = (d+e+g+h) * (0.125f/4.0f);
        groups[3] = (e+f+h+i) * (0.125f/4.0f);
        groups[4] = (j+k+l+m) * (0.5f/4.0f);
        groups[0] *= KarisAverage(groups[0]);
        groups[1] *= KarisAverage(groups[1]);
        groups[2] *= KarisAverage(groups[2]);
        groups[3] *= KarisAverage(groups[3]);
        groups[4] *= KarisAverage(groups[4]);
        downsample = groups[0]+groups[1]+groups[2]+groups[3]+groups[4];
        downsample.x = max(downsample.x, 0.0001f);
        downsample.y = max(downsample.y, 0.0001f);
        downsample.z = max(downsample.z, 0.0001f);
    } else {
        downsample = e*0.125;
        downsample += (a+c+g+i)*0.03125;
        downsample += (b+d+f+h)*0.0625;
        downsample += (j+k+l+m)*0.125;
    }

    return vec4(downsample, 1.0);
}
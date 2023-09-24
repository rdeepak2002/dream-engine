// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec4<f32>,
}

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec3<f32>,
    @location(3) bitangent: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
    out.normal = normalize(model.normal);
    out.tangent = normalize(model.tangent.xyz);
    out.bitangent = normalize(cross(model.normal, model.tangent.xyz) * model.tangent.w);
    return out;
}

// Fragment shader

struct MaterialFactors {
    base_color: vec3<f32>,
    _padding1: f32,
    emissive: vec3<f32>,
    _padding2: f32,
    metallic: f32,
    roughness: f32,
    alpha: f32,
    alpha_cutoff: f32,
};
@group(1) @binding(0)
var<uniform> material_factors: MaterialFactors;
// base color texture
@group(2) @binding(0)
var texture_base_color: texture_2d<f32>;
@group(2) @binding(1)
var sampler_base_color: sampler;
// metallic roughness texture
@group(2) @binding(2)
var texture_metallic_roughness: texture_2d<f32>;
@group(2) @binding(3)
var sampler_metallic_roughness: sampler;
// normal map texture
@group(2) @binding(4)
var texture_normal_map: texture_2d<f32>;
@group(2) @binding(5)
var sampler_normal_map: sampler;
// emissive texture
@group(2) @binding(6)
var texture_emissive: texture_2d<f32>;
@group(2) @binding(7)
var sampler_emissive: sampler;
// occlusion texture
@group(2) @binding(8)
var texture_occlusion: texture_2d<f32>;
@group(2) @binding(9)
var sampler_occlusion: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // base color
    let base_color_texture = textureSample(texture_base_color, sampler_base_color, in.tex_coords);
    let base_color_factor = vec4(material_factors.base_color, 1.0);
    let base_color = base_color_texture * base_color_factor;
    // emissive
    let emissive_texture = textureSample(texture_emissive, sampler_emissive, in.tex_coords);
    let emissive_factor = vec4(material_factors.emissive, 1.0);
    let emissive = emissive_texture * emissive_factor;
    // final color
    let final_color_no_alpha = base_color + emissive;
    let final_color_rgb = vec3(final_color_no_alpha.r, final_color_no_alpha.g, final_color_no_alpha.b);
    // transparency
    var alpha = material_factors.alpha;
    if (alpha <= material_factors.alpha_cutoff) {
        alpha = 0.0;
    }
    // compute normal using normal map
    let tbn = mat3x3<f32>(in.tangent, in.bitangent, in.normal);
    let normal_map_texture = textureSample(texture_normal_map, sampler_normal_map, in.tex_coords);
    var normal = normalize(normal_map_texture.rgb * 2.0 - 1.0);
    normal = normalize(tbn * normal);
    // TODO: remove this last line
    normal = in.normal;
    return vec4(final_color_rgb, alpha);
}


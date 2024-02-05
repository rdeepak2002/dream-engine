//include:pbr_structs.wgsl
//include:camera.wgsl
//include:model.wgsl

// Vertex shader
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(0) @binding(1)
var<uniform> lightsBuffer: LightsUniform;

//@group(2) @binding(0)
//var<uniform> boneTransformsUniform: BoneTransformsUniform;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) base_color_tex_coords: vec2<f32>,
    @location(1) metallic_roughness_tex_coords: vec2<f32>,
    @location(2) normal_tex_coords: vec2<f32>,
    @location(3) emissive_tex_coords: vec2<f32>,
    @location(4) occlusion_tex_coords: vec2<f32>,
    @location(5) normal: vec3<f32>,
    @location(6) tangent: vec3<f32>,
    @location(7) bitangent: vec3<f32>,
    @location(8) world_position: vec3<f32>,
    @location(9) color: vec4<f32>,
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

    var pos = vec4<f32>(model.position, 1.0);
    var nrm = model.normal;

    var totalPosition = vec4<f32>(0.0);
    var totalNormal = vec3<f32>(0.0);

    totalPosition = pos;
    totalNormal = nrm;

    totalNormal = normalize(totalNormal);

    var out: VertexOutput;

    // apply the correct UV coords
    var tex_transform = mat3x3<f32>(material_factors.base_color_tex_transform_0.xyz, material_factors.base_color_tex_transform_1.xyz, material_factors.base_color_tex_transform_2.xyz);
    out.base_color_tex_coords = model.tex_coords_0;
    if (material_factors.base_color_tex_coord == u32(1)) {
        out.base_color_tex_coords = model.tex_coords_1;
    }
    out.base_color_tex_coords = (tex_transform * vec3(out.base_color_tex_coords, 1.0)).xy;

    tex_transform = mat3x3<f32>(material_factors.metallic_roughness_tex_transform_0.xyz, material_factors.metallic_roughness_tex_transform_1.xyz, material_factors.metallic_roughness_tex_transform_2.xyz);
    out.metallic_roughness_tex_coords = model.tex_coords_0;
    if (material_factors.metallic_roughness_tex_coord == u32(1)) {
        out.metallic_roughness_tex_coords = model.tex_coords_1;
    }
    out.metallic_roughness_tex_coords = (tex_transform * vec3(out.metallic_roughness_tex_coords, 1.0)).xy;

    tex_transform = mat3x3<f32>(material_factors.normal_tex_transform_0.xyz, material_factors.normal_tex_transform_1.xyz, material_factors.normal_tex_transform_2.xyz);
    out.normal_tex_coords = model.tex_coords_0;
    if (material_factors.normal_tex_coord == u32(1)) {
        out.normal_tex_coords = model.tex_coords_1;
    }
    out.normal_tex_coords = (tex_transform * vec3(out.normal_tex_coords, 1.0)).xy;

    tex_transform = mat3x3<f32>(material_factors.emissive_tex_transform_0.xyz, material_factors.emissive_tex_transform_1.xyz, material_factors.emissive_tex_transform_2.xyz);
    out.emissive_tex_coords = model.tex_coords_0;
    if (material_factors.emissive_tex_coord == u32(1)) {
        out.emissive_tex_coords = model.tex_coords_1;
    }
    out.emissive_tex_coords = (tex_transform * vec3(out.emissive_tex_coords, 1.0)).xy;

    tex_transform = mat3x3<f32>(material_factors.occlusion_tex_transform_0.xyz, material_factors.occlusion_tex_transform_1.xyz, material_factors.occlusion_tex_transform_2.xyz);
    out.occlusion_tex_coords = model.tex_coords_0;
    if (material_factors.occlusion_tex_coord == u32(1)) {
        out.occlusion_tex_coords = model.tex_coords_1;
    }
    out.occlusion_tex_coords = (tex_transform * vec3(out.occlusion_tex_coords, 1.0)).xy;

    out.clip_position = camera.view_proj * model_matrix * totalPosition;
    out.normal = normalize((model_matrix * vec4(totalNormal, 0.0)).xyz);
    out.tangent = normalize((model_matrix * vec4(model.tangent.xyz, 0.0)).xyz);
    out.bitangent = model.tangent.w * normalize(cross(out.normal, out.tangent));
    out.color = model.color;
    return out;
}

// Fragment shader
struct GBufferOutput {
  @location(0) normal : vec4<f32>,
  @location(1) albedo : vec4<f32>,
  @location(2) emissive : vec4<f32>,
  @location(3) ao_roughness_metallic : vec4<f32>,
}

// base color texture
@group(1) @binding(0)
var texture_base_color: texture_2d<f32>;
@group(1) @binding(1)
var sampler_base_color: sampler;
// metallic roughness texture
@group(1) @binding(2)
var texture_metallic_roughness: texture_2d<f32>;
@group(1) @binding(3)
var sampler_metallic_roughness: sampler;
// normal map texture
@group(1) @binding(4)
var texture_normal_map: texture_2d<f32>;
@group(1) @binding(5)
var sampler_normal_map: sampler;
// emissive texture
@group(1) @binding(6)
var texture_emissive: texture_2d<f32>;
@group(1) @binding(7)
var sampler_emissive: sampler;
// occlusion texture
@group(1) @binding(8)
var texture_occlusion: texture_2d<f32>;
@group(1) @binding(9)
var sampler_occlusion: sampler;
@group(1) @binding(10)
var<uniform> material_factors: MaterialFactors;

@fragment
fn fs_main(in: VertexOutput) -> GBufferOutput {
    // texture transform (offset, scale, rotate)
    // base color
    let base_color_texture = textureSample(texture_base_color, sampler_base_color, in.base_color_tex_coords);
    let base_color_factor = vec4(material_factors.base_color, 1.0);
    let base_color = in.color * base_color_texture * base_color_factor;

    // compute normal using normal map
    let TBN = mat3x3<f32>(in.tangent, in.bitangent, in.normal);
    let normal_map_texture = textureSample(texture_normal_map, sampler_normal_map, in.normal_tex_coords);
    var normal = normal_map_texture.rgb * 2.0 - vec3(1.0, 1.0, 1.0);
    normal = normalize(TBN * normal);

    // emissive
    var emissive_texture = textureSample(texture_emissive, sampler_emissive, in.emissive_tex_coords);
    let emissive_factor = vec4(material_factors.emissive.rgb, 1.0);
    let emissive_strength = material_factors.emissive.w;
    let emissive = emissive_texture * emissive_factor * emissive_strength;

    // ambient occlusion
    let occlusion_texture = textureSample(texture_occlusion, sampler_occlusion, in.occlusion_tex_coords);
    let ao = vec4(occlusion_texture.r, occlusion_texture.r, occlusion_texture.r, 1.0);

    // metallic
    let metallic_roughness_texture = textureSample(texture_metallic_roughness, sampler_metallic_roughness, in.metallic_roughness_tex_coords);
    let metallic_factor = vec4(material_factors.metallic, material_factors.metallic, material_factors.metallic, 1.0);
    let metallic = vec4(metallic_roughness_texture.b, metallic_roughness_texture.b, metallic_roughness_texture.b, 1.0) * metallic_factor;

    // roughness
    let roughness_factor = vec4(material_factors.roughness, material_factors.roughness, material_factors.roughness, 1.0);
    let roughness = vec4(metallic_roughness_texture.g, metallic_roughness_texture.g, metallic_roughness_texture.g, 1.0) * roughness_factor;

    // transparency
    var alpha = material_factors.alpha;
    if (alpha <= material_factors.alpha_cutoff) {
        alpha = 0.0;
        discard;
    }
    if (base_color.a <= material_factors.alpha_cutoff) {
        alpha = 0.0;
        discard;
    }

    var output : GBufferOutput;
    output.normal = vec4(normal, 1.0);
    output.albedo = vec4(base_color.r, base_color.g, base_color.b, 1.0);
    output.emissive = vec4(emissive.r, emissive.g, emissive.b, 1.0);
    output.ao_roughness_metallic = vec4(ao.r, roughness.g, metallic.b, 1.0);

    // uncomment to debug each channel separately:
//    output.ao_roughness_metallic = vec4(ao.r, ao.g, ao.b, 1.0);
//    output.ao_roughness_metallic = vec4(roughness.r, roughness.g, roughness.b, 1.0);
//    output.ao_roughness_metallic = vec4(metallic.r, metallic.g, metallic.b, 1.0);

    return output;
}


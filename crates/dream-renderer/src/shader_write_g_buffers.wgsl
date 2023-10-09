//include:pbr.wgsl
//include:camera.wgsl
//include:model.wgsl
//include:skinning.wgsl

// Vertex shader
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> boneTransformsUniform: BoneTransformsUniform;

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

    var pos = vec4<f32>(model.position, 1.0);
    var nrm = model.normal;

    var totalPosition = vec4<f32>(0.0);
    var totalNormal = vec3<f32>(0.0);

    var boneIds = model.bone_ids;
    var weights = model.weights;
    var finalBonesMatrices = boneTransformsUniform.bone_transforms;

    for(var i = 0 ; i < 4 ; i++) {
        if (weights[0] + weights[1] + weights[2] + weights[3] <= 0.000001f) {
            // mesh is not skinned
            totalPosition = pos;
            totalNormal = nrm;
            break;
        }

        var localPosition: vec4<f32> = finalBonesMatrices[boneIds[i]] * vec4(model.position, 1.0f);
        totalPosition += localPosition * weights[i];

        var localNormal: vec3<f32> = (finalBonesMatrices[boneIds[i]] * vec4(model.normal, 0.0f)).xyz * weights[i];
        totalNormal += localNormal;
    }

    totalNormal = normalize(totalNormal);

    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_proj * model_matrix * totalPosition;
    out.normal = normalize((model_matrix * vec4(totalNormal, 0.0)).xyz);
    out.tangent = normalize((model_matrix * vec4(model.tangent.xyz, 0.0)).xyz);
    out.bitangent = normalize(cross(out.tangent, out.normal));
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
@group(2) @binding(10)
var<uniform> material_factors: MaterialFactors;

@group(3) @binding(0)
var<uniform> lightsBuffer: LightsUniform;

@fragment
fn fs_main(in: VertexOutput) -> GBufferOutput {
    // base color
    let base_color_texture = textureSample(texture_base_color, sampler_base_color, in.tex_coords);
    let base_color_factor = vec4(material_factors.base_color, 1.0);
    let base_color = base_color_texture * base_color_factor;
    // compute normal using normal map
    let TBN = mat3x3<f32>(in.tangent, in.bitangent, in.normal);
    let normal_map_texture = textureSample(texture_normal_map, sampler_normal_map, in.tex_coords);
    var normal = normal_map_texture.rgb * 2.0 - vec3(1.0, 1.0, 1.0);
    normal = normalize(TBN * normal);
    // emissive
    var emissive_texture = textureSample(texture_emissive, sampler_emissive, in.tex_coords);
    let emissive_factor = vec4(material_factors.emissive, 1.0);
    if (emissive_texture.r == 0.0 && emissive_texture.g == 0.0 && emissive_texture.b == 0.0) {
        if (emissive_factor.r > 0.0 || emissive_factor.g > 0.0 || emissive_factor.b > 0.0) {
            emissive_texture = vec4(1.0);
        }
    }
    let emissive = emissive_texture * emissive_factor;
    // ambient occlusion
    let occlusion_texture = textureSample(texture_occlusion, sampler_occlusion, in.tex_coords);
    let ao = vec4(occlusion_texture.r, occlusion_texture.r, occlusion_texture.r, 1.0);
    // metallic
    let metallic_roughness_texture = textureSample(texture_metallic_roughness, sampler_metallic_roughness, in.tex_coords);
    let metallic_factor = vec4(material_factors.metallic, material_factors.metallic, material_factors.metallic, 1.0);
    let metallic = vec4(metallic_roughness_texture.b, metallic_roughness_texture.b, metallic_roughness_texture.b, 1.0) * metallic_factor;
    // roughness
    let roughness_factor = vec4(material_factors.roughness, material_factors.roughness, material_factors.roughness, 1.0);
    let roughness = vec4(metallic_roughness_texture.g, metallic_roughness_texture.g, metallic_roughness_texture.g, 1.0) * roughness_factor;
    // transparency
    var alpha = material_factors.alpha;
    if (alpha <= material_factors.alpha_cutoff) {
        alpha = 0.0;
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


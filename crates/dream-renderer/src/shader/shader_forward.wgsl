//include:pbr.wgsl
//include:camera.wgsl
//include:model.wgsl
//include:skinning.wgsl
//include:shadow.wgsl

// Vertex shader
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(0) @binding(1)
var<uniform> boneTransformsUniform: BoneTransformsUniform;

@group(0) @binding(2)
var<uniform> lightsBuffer: LightsUniform;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec3<f32>,
    @location(3) bitangent: vec3<f32>,
    @location(4) world_position: vec3<f32>,
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
    out.world_position = (model_matrix * totalPosition).xyz;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_proj * model_matrix * totalPosition;
    out.normal = normalize((model_matrix * vec4(totalNormal, 0.0)).xyz);
    out.tangent = normalize((model_matrix * vec4(model.tangent.xyz, 0.0)).xyz);
    out.bitangent = normalize(cross(out.tangent, out.normal));
    return out;
}

// Fragment shader
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

// shadow cascades
@group(2) @binding(0)
var texture_shadow_map_0: texture_depth_2d;
@group(2) @binding(1)
var sampler_shadow_map_0: sampler_comparison;
@group(2) @binding(2)
var<uniform> light_as_camera_0: CameraUniform;

@group(2) @binding(3)
var texture_shadow_map_1: texture_depth_2d;
@group(2) @binding(4)
var sampler_shadow_map_1: sampler_comparison;
@group(2) @binding(5)
var<uniform> light_as_camera_1: CameraUniform;

@group(2) @binding(6)
var texture_shadow_map_2: texture_depth_2d;
@group(2) @binding(7)
var sampler_shadow_map_2: sampler_comparison;
@group(2) @binding(8)
var<uniform> light_as_camera_2: CameraUniform;

@group(2) @binding(9)
var texture_shadow_map_3: texture_depth_2d;
@group(2) @binding(10)
var sampler_shadow_map_3: sampler_comparison;
@group(2) @binding(11)
var<uniform> light_as_camera_3: CameraUniform;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // compute normal using normal map
    let TBN = mat3x3<f32>(in.tangent, in.bitangent, in.normal);
    let normal_map_texture = textureSample(texture_normal_map, sampler_normal_map, in.tex_coords);
    var normal = normal_map_texture.rgb * 2.0 - vec3(1.0, 1.0, 1.0);
    normal = normalize(TBN * normal);

    // albedo
    let base_color_texture = textureSample(texture_base_color, sampler_base_color, in.tex_coords);
    let base_color_factor = vec4(material_factors.base_color, 1.0);
    let albedo = base_color_texture * base_color_factor;

    // emissive
    var emissive_texture = textureSample(texture_emissive, sampler_emissive, in.tex_coords);
    let emissive_factor = vec4(material_factors.emissive, 1.0);
    let emissive = emissive_texture * emissive_factor;

    // ao
    let occlusion_texture = textureSample(texture_occlusion, sampler_occlusion, in.tex_coords);
    let ao = occlusion_texture.r;

    // roughness
    let metallic_roughness_texture = textureSample(texture_metallic_roughness, sampler_metallic_roughness, in.tex_coords);
    let roughness = metallic_roughness_texture.g * material_factors.roughness;

    // metallic
    let metallic = metallic_roughness_texture.b * material_factors.metallic;

    // transparency
    var alpha = material_factors.alpha;
    if (alpha <= material_factors.alpha_cutoff) {
        alpha = 0.0;
        discard;
    }
    if (albedo.a <= material_factors.alpha_cutoff) {
        alpha = 0.0;
        discard;
    }

    // world position
    let world_position = in.world_position;

    // calculate shadow_visibility
    var shadow_visibility = get_visibility_for_shadow(world_position, texture_shadow_map_0, sampler_shadow_map_0, light_as_camera_0);
    if (shadow_visibility >= 1.0) {
        shadow_visibility = get_visibility_for_shadow(world_position, texture_shadow_map_1, sampler_shadow_map_1, light_as_camera_1);
        if (shadow_visibility >= 1.0) {
            shadow_visibility = get_visibility_for_shadow(world_position, texture_shadow_map_2, sampler_shadow_map_2, light_as_camera_2);
            if (shadow_visibility >= 1.0) {
                shadow_visibility = get_visibility_for_shadow(world_position, texture_shadow_map_3, sampler_shadow_map_3, light_as_camera_3);
            }
        }
    }

    // final color
    var final_color_rgb = compute_final_color(shadow_visibility, world_position, camera.position, normal, albedo, emissive, ao, roughness, metallic);

    return vec4(final_color_rgb, alpha);
}


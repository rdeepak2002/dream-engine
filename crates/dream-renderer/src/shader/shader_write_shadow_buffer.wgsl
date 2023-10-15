//include:camera.wgsl
//include:model.wgsl
//include:skinning.wgsl
//include:pbr.wgsl

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(0) @binding(1)
var<uniform> boneTransformsUniform: BoneTransformsUniform;

@group(0) @binding(2)
var<uniform> lightsBuffer: LightsUniform;

// shadow camera
@group(1) @binding(0)
var<uniform> light_as_camera: CameraUniform;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>
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

    var totalPosition = vec4<f32>(0.0);

    var boneIds = model.bone_ids;
    var weights = model.weights;
    var finalBonesMatrices = boneTransformsUniform.bone_transforms;

    for(var i = 0 ; i < 4 ; i++) {
        if (weights[0] + weights[1] + weights[2] + weights[3] <= 0.000001f) {
            // mesh is not skinned
            totalPosition = pos;
            break;
        }

        var localPosition: vec4<f32> = finalBonesMatrices[boneIds[i]] * vec4(model.position, 1.0f);
        totalPosition += localPosition * weights[i];
    }

    var out: VertexOutput;
    out.position = light_as_camera.view_proj * model_matrix * totalPosition;
    out.tex_coords = model.tex_coords;

    return out;
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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // albedo
    let base_color_texture = textureSample(texture_base_color, sampler_base_color, in.tex_coords);
    let base_color_factor = vec4(material_factors.base_color, 1.0);
    let albedo = base_color_texture * base_color_factor;

    // transparency
    let alpha = material_factors.alpha;
    if (alpha <= material_factors.alpha_cutoff) {
        discard;
    }
    if (albedo.a <= material_factors.alpha_cutoff) {
        discard;
    }

    return albedo;
}
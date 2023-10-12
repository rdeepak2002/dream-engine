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

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> @builtin(position) vec4<f32> {
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

    return light_as_camera.view_proj * model_matrix * totalPosition;
}

@fragment
fn fs_main(@builtin(position) coord : vec4<f32>) -> @location(0) vec4<f32> {
    // TODO: discard if texture color alpha is <= 0 or <= cutoff
    return vec4(0.0, 1.0, 0.0, 1.0);
}
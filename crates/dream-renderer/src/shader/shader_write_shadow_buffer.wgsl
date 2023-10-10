//include:camera.wgsl
//include:model.wgsl
//include:skinning.wgsl

//struct Scene {
//  lightViewProjMatrix: mat4x4<f32>,
//  cameraViewProjMatrix: mat4x4<f32>,
//  lightPos: vec3<f32>,
//}

@group(0) @binding(0)
var<uniform> light_as_camera: CameraUniform;

@group(1) @binding(0)
var<uniform> boneTransformsUniform: BoneTransformsUniform;

@vertex
fn main(
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

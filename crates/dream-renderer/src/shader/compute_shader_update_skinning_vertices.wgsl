//include:model.wgsl
//include:skinning.wgsl

struct PrimitiveInfo {
  num_vertices: u32
};

// Vertex shader
@group(0) @binding(0)
var<uniform> primitiveInfo: PrimitiveInfo;
@group(1) @binding(0)
var<storage, read_write> finalBonesMatrices: array<mat4x4<f32>>;
@group(2) @binding(0)
var<storage, read_write> vertices: array<f32>;
@group(3) @binding(0)
var<storage, read_write> skinned_vertices: array<f32>;

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) global_invocation_id : vec3<u32>) {
    let idx = global_invocation_id.x;

    if (idx > primitiveInfo.num_vertices) {
        return;
    }

    let vertexInfoBytes = u32(20);

    let offsetPx = u32(0);
    let offsetPy = u32(1);
    let offsetPz = u32(2);

    let pxIdx = idx * vertexInfoBytes + offsetPx;
    let pyIdx = idx * vertexInfoBytes + offsetPy;
    let pzIdx = idx * vertexInfoBytes + offsetPz;

    let px = vertices[pxIdx];
    let py = vertices[pyIdx];
    let pz = vertices[pzIdx];

    let offsetNx = u32(5);
    let offsetNy = u32(6);
    let offsetNz = u32(7);

    let nxIdx = idx * vertexInfoBytes + offsetNx;
    let nyIdx = idx * vertexInfoBytes + offsetNy;
    let nzIdx = idx * vertexInfoBytes + offsetNz;

    let nx = vertices[nxIdx];
    let ny = vertices[nyIdx];
    let nz = vertices[nzIdx];

    let offsetTx = u32(8);
    let offsetTy = u32(9);
    let offsetTz = u32(10);

    let txIdx = idx * vertexInfoBytes + offsetTx;
    let tyIdx = idx * vertexInfoBytes + offsetTy;
    let tzIdx = idx * vertexInfoBytes + offsetTz;

    let tx = vertices[txIdx];
    let ty = vertices[tyIdx];
    let tz = vertices[tzIdx];

    let offsetBoneIdX = u32(12);
    let offsetBoneIdY = u32(13);
    let offsetBoneIdZ = u32(14);
    let offsetBoneIdW = u32(15);

    let boneIdXIdx = idx * vertexInfoBytes + offsetBoneIdX;
    let boneIdYIdx = idx * vertexInfoBytes + offsetBoneIdY;
    let boneIdZIdx = idx * vertexInfoBytes + offsetBoneIdZ;
    let boneIdWIdx = idx * vertexInfoBytes + offsetBoneIdW;

    let boneIdX = bitcast<u32>(vertices[boneIdXIdx]);
    let boneIdY = bitcast<u32>(vertices[boneIdYIdx]);
    let boneIdZ = bitcast<u32>(vertices[boneIdZIdx]);
    let boneIdW = bitcast<u32>(vertices[boneIdWIdx]);

    let offsetBoneWeightX = u32(16);
    let offsetBoneWeightY = u32(17);
    let offsetBoneWeightZ = u32(18);
    let offsetBoneWeightW = u32(19);

    let boneWeightXIdx = idx * vertexInfoBytes + offsetBoneWeightX;
    let boneWeightYIdx = idx * vertexInfoBytes + offsetBoneWeightY;
    let boneWeightZIdx = idx * vertexInfoBytes + offsetBoneWeightZ;
    let boneWeightWIdx = idx * vertexInfoBytes + offsetBoneWeightW;

    let boneWeightX = vertices[boneWeightXIdx];
    let boneWeightY = vertices[boneWeightYIdx];
    let boneWeightZ = vertices[boneWeightZIdx];
    let boneWeightW = vertices[boneWeightWIdx];

    var pos = vec4<f32>(px, py, pz, 1.0);
    var nrm = vec3<f32>(nx, ny, nz);
    var tn  = vec3<f32>(tx, ty, tz);

    var totalPosition = vec4<f32>(0.0);
    var totalNormal = vec3<f32>(0.0);
    var totalTangent = vec3<f32>(0.0);

    var boneIds = vec4<u32>(boneIdX, boneIdY, boneIdZ, boneIdW);
    var weights = vec4<f32>(boneWeightX, boneWeightY, boneWeightZ, boneWeightW);

    for(var i = 0 ; i < 4 ; i++) {
        if (weights[0] + weights[1] + weights[2] + weights[3] <= 0.000001f) {
            // mesh is not skinned
            totalPosition = pos;
            totalNormal = nrm;
            totalTangent = tn;
            break;
        }

        var localPosition: vec4<f32> = finalBonesMatrices[boneIds[i]] * vec4(px, py, pz, 1.0f) * weights[i];
        totalPosition += localPosition;

        var localNormal: vec3<f32> = (finalBonesMatrices[boneIds[i]] * vec4(nx, ny, nz, 0.0f)).xyz * weights[i];
        totalNormal += localNormal ;

        var localTangent: vec3<f32> = (finalBonesMatrices[boneIds[i]] * vec4(tx, ty, tz, 0.0)).xyz * weights[i];
        totalTangent += localTangent;
    }

    totalNormal = normalize(totalNormal);
    totalTangent = normalize(totalTangent);

    skinned_vertices[pxIdx] = totalPosition.x;
    skinned_vertices[pyIdx] = totalPosition.y;
    skinned_vertices[pzIdx] = totalPosition.z;

    skinned_vertices[nxIdx] = totalNormal.x;
    skinned_vertices[nyIdx] = totalNormal.y;
    skinned_vertices[nzIdx] = totalNormal.z;

    skinned_vertices[txIdx] = totalTangent.x;
    skinned_vertices[tyIdx] = totalTangent.y;
    skinned_vertices[tzIdx] = totalTangent.z;
}


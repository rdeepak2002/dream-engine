struct CameraUniform {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    inv_view: mat4x4<f32>,
    inv_proj: mat4x4<f32>,
};

struct VertexOutput {
  @builtin(position) position : vec4<f32>,
  @location(0) world_position : vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

//@group(0) @binding(1)
//var<uniform> lightsBuffer: LightsUniform;

@vertex
fn vs_main(
    @builtin(vertex_index) id: u32,
) -> VertexOutput {
    let uv = vec2<f32>(vec2<u32>(
        id & 1u,
        (id >> 1u) & 1u,
    ));
    var out: VertexOutput;
    out.world_position = vec4(uv * 4.0 - 1.0, 1.0, 1.0);
    out.position = out.world_position;
    return out;
}

@group(1) @binding(0)
var cubemap_texture: texture_cube<f32>;
@group(1) @binding(1)
var cubemap_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let PI: f32 = 3.14159265359;
    let view_pos_homogeneous = camera.inv_proj * in.world_position;
    let view_ray_direction = view_pos_homogeneous.xyz / view_pos_homogeneous.w;
    let ray_direction = normalize((camera.inv_view * vec4(view_ray_direction, 0.0)).xyz);
    let world_pos = ray_direction;

	// The world vector acts as the normal of a tangent surface
    // from the origin, aligned to WorldPos. Given this normal, calculate all
    // incoming radiance of the environment. The result of this radiance
    // is the radiance of light coming from -Normal direction, which is what
    // we use in the PBR shader to sample irradiance.
    let N: vec3<f32> = normalize(world_pos);
    var irradiance: vec3<f32> = vec3(0.0);

    // tangent space calculation from origin point
    var up: vec3<f32> = vec3(0.0, 1.0, 0.0);
    let right: vec3<f32> = normalize(cross(up, N));
    up = normalize(cross(N, right));

    let sampleDelta: f32 = 0.025;
    var nrSamples: f32 = 0.0;
    for(var phi: f32 = 0.0; phi < 2.0 * PI; phi += sampleDelta)
    {
        for(var theta: f32 = 0.0; theta < 0.5 * PI; theta += sampleDelta)
        {
            // spherical to cartesian (in tangent space)
            let tangentSample: vec3<f32> = vec3(sin(theta) * cos(phi),  sin(theta) * sin(phi), cos(theta));
            // tangent space to world
            let sampleVec: vec3<f32> = tangentSample.x * right + tangentSample.y * up + tangentSample.z * N;

            // TODO: should we tone_map this textureSample? - tbh idts cuz in the end for pbr equations we handle this
            irradiance += textureSample(cubemap_texture, cubemap_sampler, sampleVec).rgb * cos(theta) * sin(theta);
            nrSamples += 1.0;
        }
    }
    irradiance = PI * irradiance * (1.0 / nrSamples);

    return vec4(irradiance, 1.0);
}
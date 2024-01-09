struct CameraUniform {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
};

struct VertexOutput {
  @builtin(position) position : vec4<f32>,
  @location(0) world_position : vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@vertex
fn vs_main(
  @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var pos = array(
        // back face
        vec3(-1.0f, -1.0f, -1.0f), // bottom-left
        vec3( 1.0f,  1.0f, -1.0f), // top-right
        vec3( 1.0f, -1.0f, -1.0f), // bottom-right
        vec3( 1.0f,  1.0f, -1.0f), // top-right
        vec3(-1.0f, -1.0f, -1.0f), // bottom-left
        vec3(-1.0f,  1.0f, -1.0f), // top-left
        // front face
        vec3(-1.0f, -1.0f,  1.0f), // bottom-left
        vec3( 1.0f, -1.0f,  1.0f), // bottom-right
        vec3( 1.0f,  1.0f,  1.0f), // top-right
        vec3( 1.0f,  1.0f,  1.0f), // top-right
        vec3(-1.0f,  1.0f,  1.0f), // top-left
        vec3(-1.0f, -1.0f,  1.0f), // bottom-left
        // left face
        vec3(-1.0f,  1.0f,  1.0f), // top-right
        vec3(-1.0f,  1.0f, -1.0f), // top-left
        vec3(-1.0f, -1.0f, -1.0f), // bottom-left
        vec3(-1.0f, -1.0f, -1.0f), // bottom-left
        vec3(-1.0f, -1.0f,  1.0f), // bottom-right
        vec3(-1.0f,  1.0f,  1.0f), // top-right
        // right face
        vec3( 1.0f,  1.0f,  1.0f), // top-left
        vec3( 1.0f, -1.0f, -1.0f), // bottom-right
        vec3( 1.0f,  1.0f, -1.0f), // top-right
        vec3( 1.0f, -1.0f, -1.0f), // bottom-right
        vec3( 1.0f,  1.0f,  1.0f), // top-left
        vec3( 1.0f, -1.0f,  1.0f), // bottom-left
        // bottom face
        vec3(-1.0f, -1.0f, -1.0f), // top-right
        vec3( 1.0f, -1.0f, -1.0f), // top-left
        vec3( 1.0f, -1.0f,  1.0f), // bottom-left
        vec3( 1.0f, -1.0f,  1.0f), // bottom-left
        vec3(-1.0f, -1.0f,  1.0f), // bottom-right
        vec3(-1.0f, -1.0f, -1.0f), // top-right
        // top face
        vec3(-1.0f,  1.0f, -1.0f), // top-left
        vec3( 1.0f,  1.0f , 1.0f), // bottom-right
        vec3( 1.0f,  1.0f, -1.0f), // top-right
        vec3( 1.0f,  1.0f,  1.0f), // bottom-right
        vec3(-1.0f,  1.0f, -1.0f), // top-left
        vec3(-1.0f,  1.0f,  1.0f), // bottom-left
    );
    var output : VertexOutput;
    let world_position = vec4(pos[in_vertex_index], 1.0);
    output.position = camera.projection * camera.view * world_position;
    output.world_position = world_position;
    return output;
}

@group(1) @binding(0)
var equirectangular_map: texture_2d<f32>;
//@group(1) @binding(1)
//var equirectangular_map_sampler: sampler;

fn sampleSphericalMap(v: vec3<f32>) -> vec2<f32> {
    let invAtan = vec2<f32>(0.1591, 0.3183);
    var uv: vec2<f32> = vec2<f32>(atan2(v.z, v.x), asin(v.y));
    uv *= invAtan;
    uv += 0.5;
    return uv;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv: vec2<f32> = sampleSphericalMap(normalize(in.world_position.xyz));
    let color: vec3<f32> = textureLoad(
          equirectangular_map,
          vec2<i32>(uv.xy * vec2<f32>(textureDimensions(equirectangular_map))),
          0
    ).rgb;
    return vec4<f32>(color, 1.0);
}
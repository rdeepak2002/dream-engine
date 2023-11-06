@vertex
fn vs_main(
  @builtin(vertex_index) in_vertex_index: u32,
) -> @builtin(position) vec4<f32> {
    var quad_verts = array(
        vec4(-1.0, -1.0, 0.0, 1.0),
        vec4( 1.0, -1.0, 0.0, 1.0),
        vec4(-1.0,  1.0, 0.0, 1.0),
        vec4(-1.0,  1.0, 0.0, 1.0),
        vec4( 1.0, -1.0, 0.0, 1.0),
        vec4( 1.0,  1.0, 0.0, 1.0)
    );
    return quad_verts[in_vertex_index];
}

@group(0) @binding(0)
var frame_texture: texture_2d<f32>;

@fragment
fn fs_main(@builtin(position) coord : vec4<f32>) -> @location(0) vec4<f32> {
    let frame_color = textureLoad(
        frame_texture,
        vec2<i32>(floor(coord.xy)),
        0
    ).xyz;

    // exposure tone mapping
    let exposure = 1.0;
    var mapped = vec3(1.0) - exp(-frame_color * exposure);
//    var mapped = frame_color / (frame_color + vec3(1.0));
    // gamma correction
    let gamma = 1.0;
    mapped = pow(mapped, vec3(1.0 / gamma));

    return vec4(mapped, 1.0);
}
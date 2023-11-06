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

    let brightness = dot(frame_color, vec3(0.2126, 0.7152, 0.0722));
    var bright_color = vec3(0.0, 0.0, 0.0);
    if (brightness > 0.3) {
        bright_color = frame_color;
    }

    return vec4(bright_color, 1.0);
}
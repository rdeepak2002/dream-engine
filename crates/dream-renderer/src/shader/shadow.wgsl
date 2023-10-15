struct CascadeSettingsUniform {
    cascade_end: f32,
    bias: f32,
}

fn get_visibility_for_shadow(world_position: vec3<f32>, texture_shadow_map: texture_depth_2d, sampler_shadow_map: sampler_comparison, light_as_camera: CameraUniform) -> f32 {
    // convert fragment position to light position view matrix, then convert XY of this to (0, 1) range
    var fragment_shadow_position_raw = light_as_camera.view_proj * vec4(world_position, 1.0);

    // shadow calculation
    let fragment_shadow_position = vec3(
        fragment_shadow_position_raw.xy * vec2(0.5, -0.5) + vec2(0.5),
        fragment_shadow_position_raw.z
    );
    var visibility = 0.0;
    let shadow_depth_texture_size: f32 = vec2<f32>(textureDimensions(texture_shadow_map)).x;
    let one_over_shadow_depth_texture_size = 1.0 / shadow_depth_texture_size;
//    let bias = 0.002;
    let bias = 0.0;
    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            let offset = vec2<f32>(vec2(x, y)) * one_over_shadow_depth_texture_size;
            visibility += textureSampleCompare(
                texture_shadow_map, sampler_shadow_map,
                fragment_shadow_position.xy + offset, fragment_shadow_position.z - bias
            );
        }
    }
    visibility /= 9.0;
    var is_outside_bounds = false;
    if (fragment_shadow_position_raw.y > 1.0) {
        visibility = 1.0;
        is_outside_bounds = true;
    }
    if (fragment_shadow_position_raw.y < -1.0) {
        visibility = 1.0;
        is_outside_bounds = true;
    }
    if (fragment_shadow_position_raw.x > 1.0) {
        visibility = 1.0;
        is_outside_bounds = true;
    }
    if (fragment_shadow_position_raw.x < -1.0) {
        visibility = 1.0;
        is_outside_bounds = true;
    }
    if (fragment_shadow_position_raw.z > 1.0) {
        visibility = 1.0;
        is_outside_bounds = true;
    }
    if (fragment_shadow_position_raw.z < 0.0) {
        visibility = 1.0;
        is_outside_bounds = true;
    }
    return visibility;
}
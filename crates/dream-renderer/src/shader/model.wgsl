struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords_0: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec4<f32>,
    @location(4) bone_ids: vec4<u32>,
    @location(5) weights: vec4<f32>,
    @location(6) color: vec4<f32>,
    @location(7) tex_coords_1: vec2<f32>,
}

struct InstanceInput {
    @location(8)  model_matrix_0: vec4<f32>,
    @location(9)  model_matrix_1: vec4<f32>,
    @location(10) model_matrix_2: vec4<f32>,
    @location(11) model_matrix_3: vec4<f32>,
}

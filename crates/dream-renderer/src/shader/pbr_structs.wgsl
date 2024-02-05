const PI: f32 = 3.14159265359;
const LIGHT_TYPE_POINT: u32 = 0u;
const LIGHT_TYPE_DIRECTIONAL: u32 = 1u;

struct MaterialFactors {
    base_color: vec3<f32>,
    alpha: f32,
    emissive: vec4<f32>,
    metallic: f32,
    roughness: f32,
    alpha_cutoff: f32,
    // texture coordinates
    base_color_tex_coord: u32,
    metallic_roughness_tex_coord: u32,
    normal_tex_coord: u32,
    emissive_tex_coord: u32,
    occlusion_tex_coord: u32,
    // texture transform base color
    base_color_tex_transform_0: vec4<f32>,
    base_color_tex_transform_1: vec4<f32>,
    base_color_tex_transform_2: vec4<f32>,
    // texture transform metallic roughness
    metallic_roughness_tex_transform_0: vec4<f32>,
    metallic_roughness_tex_transform_1: vec4<f32>,
    metallic_roughness_tex_transform_2: vec4<f32>,
    // texture transform metallic roughness
    normal_tex_transform_0: vec4<f32>,
    normal_tex_transform_1: vec4<f32>,
    normal_tex_transform_2: vec4<f32>,
    // texture transform emissive
    emissive_tex_transform_0: vec4<f32>,
    emissive_tex_transform_1: vec4<f32>,
    emissive_tex_transform_2: vec4<f32>,
    // texture transform occlusion
    occlusion_tex_transform_0: vec4<f32>,
    occlusion_tex_transform_1: vec4<f32>,
    occlusion_tex_transform_2: vec4<f32>,
};

struct Light {
    position: vec3<f32>,
    radius: f32,
    color: vec3<f32>,
    _padding: u32,
    direction: vec3<f32>,
    light_type: u32,
}

struct LightsUniform {
  lights: array<Light, 4>
};
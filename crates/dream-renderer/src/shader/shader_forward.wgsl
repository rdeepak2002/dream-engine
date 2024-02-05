//include:pbr_structs.wgsl
//include:pbr.wgsl
//include:camera.wgsl
//include:model.wgsl
//include:shadow.wgsl

// Vertex shader
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

//@group(0) @binding(1)
//var<uniform> boneTransformsUniform: BoneTransformsUniform;

@group(0) @binding(1)
var<uniform> lightsBuffer: LightsUniform;

//@group(3) @binding(0)
//var<uniform> boneTransformsUniform: BoneTransformsUniform;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) base_color_tex_coords: vec2<f32>,
    @location(1) metallic_roughness_tex_coords: vec2<f32>,
    @location(2) normal_tex_coords: vec2<f32>,
    @location(3) emissive_tex_coords: vec2<f32>,
    @location(4) occlusion_tex_coords: vec2<f32>,
    @location(5) normal: vec3<f32>,
    @location(6) tangent: vec3<f32>,
    @location(7) bitangent: vec3<f32>,
    @location(8) world_position: vec3<f32>,
    @location(9) color: vec4<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    var pos = vec4<f32>(model.position, 1.0);
    var nrm = model.normal;

    var totalPosition = vec4<f32>(0.0);
    var totalNormal = vec3<f32>(0.0);

    totalPosition = pos;
    totalNormal = nrm;

    totalNormal = normalize(totalNormal);

    var out: VertexOutput;
    out.world_position = (model_matrix * totalPosition).xyz;

    // apply the correct UV coords
    var tex_transform = mat3x3<f32>(material_factors.base_color_tex_transform_0.xyz, material_factors.base_color_tex_transform_1.xyz, material_factors.base_color_tex_transform_2.xyz);
    out.base_color_tex_coords = model.tex_coords_0;
    if (material_factors.base_color_tex_coord == u32(1)) {
        out.base_color_tex_coords = model.tex_coords_1;
    }
    out.base_color_tex_coords = (tex_transform * vec3(out.base_color_tex_coords, 1.0)).xy;

    tex_transform = mat3x3<f32>(material_factors.metallic_roughness_tex_transform_0.xyz, material_factors.metallic_roughness_tex_transform_1.xyz, material_factors.metallic_roughness_tex_transform_2.xyz);
    out.metallic_roughness_tex_coords = model.tex_coords_0;
    if (material_factors.metallic_roughness_tex_coord == u32(1)) {
        out.metallic_roughness_tex_coords = model.tex_coords_1;
    }
    out.metallic_roughness_tex_coords = (tex_transform * vec3(out.metallic_roughness_tex_coords, 1.0)).xy;

    tex_transform = mat3x3<f32>(material_factors.normal_tex_transform_0.xyz, material_factors.normal_tex_transform_1.xyz, material_factors.normal_tex_transform_2.xyz);
    out.normal_tex_coords = model.tex_coords_0;
    if (material_factors.normal_tex_coord == u32(1)) {
        out.normal_tex_coords = model.tex_coords_1;
    }
    out.normal_tex_coords = (tex_transform * vec3(out.normal_tex_coords, 1.0)).xy;

    tex_transform = mat3x3<f32>(material_factors.emissive_tex_transform_0.xyz, material_factors.emissive_tex_transform_1.xyz, material_factors.emissive_tex_transform_2.xyz);
    out.emissive_tex_coords = model.tex_coords_0;
    if (material_factors.emissive_tex_coord == u32(1)) {
        out.emissive_tex_coords = model.tex_coords_1;
    }
    out.emissive_tex_coords = (tex_transform * vec3(out.emissive_tex_coords, 1.0)).xy;

    tex_transform = mat3x3<f32>(material_factors.occlusion_tex_transform_0.xyz, material_factors.occlusion_tex_transform_1.xyz, material_factors.occlusion_tex_transform_2.xyz);
    out.occlusion_tex_coords = model.tex_coords_0;
    if (material_factors.occlusion_tex_coord == u32(1)) {
        out.occlusion_tex_coords = model.tex_coords_1;
    }
    out.occlusion_tex_coords = (tex_transform * vec3(out.occlusion_tex_coords, 1.0)).xy;

    out.clip_position = camera.view_proj * model_matrix * totalPosition;
    out.normal = normalize((model_matrix * vec4(totalNormal, 0.0)).xyz);
    out.tangent = normalize((model_matrix * vec4(model.tangent.xyz, 0.0)).xyz);
    out.bitangent = model.tangent.w * normalize(cross(out.normal, out.tangent));
    out.color = model.color;
    return out;
}

// Fragment shader
// base color texture
@group(1) @binding(0)
var texture_base_color: texture_2d<f32>;
@group(1) @binding(1)
var sampler_base_color: sampler;
// metallic roughness texture
@group(1) @binding(2)
var texture_metallic_roughness: texture_2d<f32>;
@group(1) @binding(3)
var sampler_metallic_roughness: sampler;
// normal map texture
@group(1) @binding(4)
var texture_normal_map: texture_2d<f32>;
@group(1) @binding(5)
var sampler_normal_map: sampler;
// emissive texture
@group(1) @binding(6)
var texture_emissive: texture_2d<f32>;
@group(1) @binding(7)
var sampler_emissive: sampler;
// occlusion texture
@group(1) @binding(8)
var texture_occlusion: texture_2d<f32>;
@group(1) @binding(9)
var sampler_occlusion: sampler;
@group(1) @binding(10)
var<uniform> material_factors: MaterialFactors;

// shadow cascades
@group(2) @binding(0)
var texture_shadow_map_0: texture_depth_2d;
@group(2) @binding(1)
var sampler_shadow_map_0: sampler_comparison;
@group(2) @binding(2)
var<uniform> light_as_camera_0: CameraUniform;
@group(2) @binding(3)
var<uniform> cascade_settings_0: CascadeSettingsUniform;

@group(2) @binding(4)
var texture_shadow_map_1: texture_depth_2d;
@group(2) @binding(5)
var sampler_shadow_map_1: sampler_comparison;
@group(2) @binding(6)
var<uniform> light_as_camera_1: CameraUniform;
@group(2) @binding(7)
var<uniform> cascade_settings_1: CascadeSettingsUniform;

@group(2) @binding(8)
var texture_shadow_map_2: texture_depth_2d;
@group(2) @binding(9)
var sampler_shadow_map_2: sampler_comparison;
@group(2) @binding(10)
var<uniform> light_as_camera_2: CameraUniform;
@group(2) @binding(11)
var<uniform> cascade_settings_2: CascadeSettingsUniform;

@group(2) @binding(12)
var texture_shadow_map_3: texture_depth_2d;
@group(2) @binding(13)
var sampler_shadow_map_3: sampler_comparison;
@group(2) @binding(14)
var<uniform> light_as_camera_3: CameraUniform;
@group(2) @binding(15)
var<uniform> cascade_settings_3: CascadeSettingsUniform;

@group(3) @binding(0)
var irradiance_map: texture_cube<f32>;
@group(3) @binding(1)
var irradiance_map_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // texture transform (offset, scale, rotate)
    let tex_transform = mat3x3<f32>(material_factors.base_color_tex_transform_0.xyz, material_factors.base_color_tex_transform_1.xyz, material_factors.base_color_tex_transform_2.xyz);
    var tex_coords: vec2<f32> = vec2(0.0, 0.0);

    // compute normal using normal map
    let TBN = mat3x3<f32>(in.tangent, in.bitangent, in.normal);
    let normal_map_texture = textureSample(texture_normal_map, sampler_normal_map, in.normal_tex_coords);
    var normal = normal_map_texture.rgb * 2.0 - vec3(1.0, 1.0, 1.0);
    normal = normalize(TBN * normal);

    // albedo
    let base_color_texture = textureSample(texture_base_color, sampler_base_color, in.base_color_tex_coords);
    let base_color_factor = vec4(material_factors.base_color, 1.0);
    let albedo = in.color * base_color_texture * base_color_factor;

    // emissive
    var emissive_texture = textureSample(texture_emissive, sampler_emissive, in.emissive_tex_coords);
    let emissive_factor = vec4(material_factors.emissive.rgb, 1.0);
    let emissive_strength = material_factors.emissive.w;
    let emissive = emissive_texture * emissive_factor * emissive_strength;

    // ao
    let occlusion_texture = textureSample(texture_occlusion, sampler_occlusion, in.occlusion_tex_coords);
    let ao = occlusion_texture.r;

    // roughness
    let metallic_roughness_texture = textureSample(texture_metallic_roughness, sampler_metallic_roughness, in.metallic_roughness_tex_coords);
    let roughness = metallic_roughness_texture.g * material_factors.roughness;

    // metallic
    let metallic = metallic_roughness_texture.b * material_factors.metallic;

    // transparency
    var alpha = material_factors.alpha;
    if (alpha <= material_factors.alpha_cutoff) {
        alpha = 0.0;
        discard;
    }
    if (albedo.a <= material_factors.alpha_cutoff) {
        alpha = 0.0;
        discard;
    }

    // world position
    let world_position = in.world_position;

    // calculate shadow_visibility
    let depthValue = abs((camera.view * vec4(world_position, 1.0)).z);
    var debug_cascade_factor = vec3(1.0, 1.0, 1.0);
    var shadow_visibility = 1.0;
    let v1 = get_visibility_for_shadow(world_position, texture_shadow_map_0, sampler_shadow_map_0, light_as_camera_0, normal, cascade_settings_0);
    let v2 = get_visibility_for_shadow(world_position, texture_shadow_map_1, sampler_shadow_map_1, light_as_camera_1, normal, cascade_settings_1);
    let v3 = get_visibility_for_shadow(world_position, texture_shadow_map_2, sampler_shadow_map_2, light_as_camera_2, normal, cascade_settings_2);
    let v4 = get_visibility_for_shadow(world_position, texture_shadow_map_3, sampler_shadow_map_3, light_as_camera_3, normal, cascade_settings_3);
    if (depthValue <= cascade_settings_0.cascade_end) {
        shadow_visibility = v1;
//        debug_cascade_factor = vec3(1.0, 0.0, 0.0);
    } else if (depthValue <= cascade_settings_1.cascade_end) {
        shadow_visibility = v2;
//        debug_cascade_factor = vec3(0.0, 1.0, 0.0);
    } else if (depthValue <= cascade_settings_2.cascade_end) {
        shadow_visibility = v3;
//        debug_cascade_factor = vec3(0.0, 0.0, 1.0);
    } else if (depthValue <= cascade_settings_3.cascade_end) {
        shadow_visibility = v4;
//        debug_cascade_factor = vec3(1.0, 0.0, 1.0);
    }

    // final color
    var final_color_rgb = debug_cascade_factor * compute_final_color(shadow_visibility, world_position, camera.position, normal, albedo, emissive, ao, roughness, metallic);

    return vec4(final_color_rgb, alpha);
}


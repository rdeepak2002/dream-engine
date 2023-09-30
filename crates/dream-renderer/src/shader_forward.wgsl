// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec4<f32>,
}

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
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
    var out: VertexOutput;
    out.world_position = (model_matrix * vec4<f32>(model.position, 1.0)).xyz;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
//    var T = normalize((model_matrix * vec4(normalize(model.tangent.xyz), 0.0)).xyz);
//    let N = normalize((model_matrix * vec4(normalize(model.normal), 0.0)).xyz);
//    T = normalize(T - dot(T, N) * N);
//    let B = cross(N, T);
//    out.normal = N;
//    out.tangent = T;
//    out.bitangent = B;
    out.normal = normalize((model_matrix * vec4(model.normal, 0.0)).xyz);
    out.tangent = normalize((model_matrix * vec4(model.tangent.xyz, 0.0)).xyz);
    out.bitangent = normalize(cross(out.tangent, out.normal));
    return out;
}

// Fragment shader

struct MaterialFactors {
    base_color: vec3<f32>,
    _padding1: f32,
    emissive: vec3<f32>,
    _padding2: f32,
    metallic: f32,
    roughness: f32,
    alpha: f32,
    alpha_cutoff: f32,
};
@group(1) @binding(0)
var<uniform> material_factors: MaterialFactors;
// base color texture
@group(2) @binding(0)
var texture_base_color: texture_2d<f32>;
@group(2) @binding(1)
var sampler_base_color: sampler;
// metallic roughness texture
@group(2) @binding(2)
var texture_metallic_roughness: texture_2d<f32>;
@group(2) @binding(3)
var sampler_metallic_roughness: sampler;
// normal map texture
@group(2) @binding(4)
var texture_normal_map: texture_2d<f32>;
@group(2) @binding(5)
var sampler_normal_map: sampler;
// emissive texture
@group(2) @binding(6)
var texture_emissive: texture_2d<f32>;
@group(2) @binding(7)
var sampler_emissive: sampler;
// occlusion texture
@group(2) @binding(8)
var texture_occlusion: texture_2d<f32>;
@group(2) @binding(9)
var sampler_occlusion: sampler;

struct Light {
  position: vec3<f32>,
  radius: f32,
  color: vec3<f32>,
  _padding2: u32,
}

struct LightsUniform {
  lights: array<Light, 4>
};

@group(3) @binding(0)
var<uniform> lightsBuffer: LightsUniform;

fn compute_final_color(world_position: vec3<f32>, normal: vec3<f32>, albedo: vec4<f32>, emissive: vec4<f32>, ao: f32, roughness: f32, metallic: f32) -> vec3<f32> {
    // TODO: use num_lights uniform variable
    var result = vec3(0., 0., 0.);
    for (var i = 0u; i < 4u; i += 1u) {
        let light = lightsBuffer.lights[i];
        let position = world_position;
        let L = light.position.xyz - position;
        let distance = length(L);
        if (distance > light.radius) {
            continue;
        }
        let lambert = max(dot(normal, normalize(L)), 0.0);
        result += vec3<f32>(
            lambert * pow(1.0 - distance / light.radius, 2.0) * light.color * albedo.rgb
        );
    }
    return result;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // compute normal using normal map
    let TBN = mat3x3<f32>(in.tangent, in.bitangent, in.normal);
    let normal_map_texture = textureSample(texture_normal_map, sampler_normal_map, in.tex_coords);
    var normal = normal_map_texture.rgb * 2.0 - vec3(1.0, 1.0, 1.0);
    normal = normalize(TBN * normal);

    // albedo
    let base_color_texture = textureSample(texture_base_color, sampler_base_color, in.tex_coords);
    let base_color_factor = vec4(material_factors.base_color, 1.0);
    let albedo = base_color_texture * base_color_factor;

    // emissive
    let emissive_texture = textureSample(texture_emissive, sampler_emissive, in.tex_coords);
    let emissive_factor = vec4(material_factors.emissive, 1.0);
    let emissive = emissive_texture * emissive_factor;

    // ao
    let occlusion_texture = textureSample(texture_occlusion, sampler_occlusion, in.tex_coords);
    let ao = occlusion_texture.r;

    // roughness
    let metallic_roughness_texture = textureSample(texture_metallic_roughness, sampler_metallic_roughness, in.tex_coords);
    let roughness = metallic_roughness_texture.g * material_factors.roughness;

    // metallic
    let metallic = metallic_roughness_texture.b * material_factors.metallic;

    // transparency
    var alpha = material_factors.alpha;
    if (alpha <= material_factors.alpha_cutoff) {
        discard;
    }

    // world position
    let world_position = in.world_position;

    // final color
    var final_color_rgb = compute_final_color(world_position, normal, albedo, emissive, ao, roughness, metallic);

    return vec4(final_color_rgb, alpha);
}


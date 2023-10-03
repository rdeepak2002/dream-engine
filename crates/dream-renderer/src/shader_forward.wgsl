// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    position: vec3<f32>,
    _padding: f32,
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

const PI: f32 = 3.14159265359;

fn DistributionGGX(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
    let a: f32 = roughness*roughness;
    let a2: f32 = a*a;
    let NdotH: f32 = max(dot(N, H), 0.0);
    let NdotH2: f32 = NdotH*NdotH;

    let nom: f32 = a2;
    var denom: f32 = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return nom / denom;
}

fn GeometrySchlickGGX(NdotV: f32, roughness: f32) -> f32 {
    let r: f32 = (roughness + 1.0);
    let k: f32 = (r*r) / 8.0;

    let nom: f32   = NdotV;
    let denom: f32 = NdotV * (1.0 - k) + k;

    return nom / denom;
}

fn GeometrySmith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
    let NdotV: f32 = max(dot(N, V), 0.0);
    let NdotL: f32 = max(dot(N, L), 0.0);
    let ggx2: f32 = GeometrySchlickGGX(NdotV, roughness);
    let ggx1: f32 = GeometrySchlickGGX(NdotL, roughness);

    return ggx1 * ggx2;
}

fn fresnelSchlick(cosTheta: f32, F0: vec3<f32>) -> vec3<f32>{
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

fn compute_final_color(world_position: vec3<f32>, camera_position: vec3<f32>, normal: vec3<f32>, albedo: vec4<f32>, emissive: vec4<f32>, ao: f32, roughness: f32, metallic: f32) -> vec3<f32> {
    // TODO: use num_lights uniform variable
    var result = vec3(0., 0., 0.);

    var F0: vec3<f32> = vec3(0.04, 0.04, 0.04);
    F0 = mix(F0, albedo.rgb, vec3(metallic, metallic, metallic));
    let N = normalize(normal);

    for (var i = 0u; i < 4u; i += 1u) {
        let light = lightsBuffer.lights[i];

        var Lo: vec3<f32> = vec3(0.0, 0.0, 0.0);

        let lightPosition = light.position;
        let lightColor = light.color;

        // calculate per-light radiance
        let V: vec3<f32> = normalize(camera_position - world_position);
        let L: vec3<f32> = normalize(lightPosition - world_position);
        let H: vec3<f32> = normalize(V + L);
        let distance: f32 = length(lightPosition - world_position);
        let attenuation: f32 = 1.0 / (distance * distance);
        let radiance = lightColor * attenuation;

        // Cook-Torrance BRDF
        let NDF: f32 = DistributionGGX(N, H, roughness);
        let G: f32 = GeometrySmith(N, V, L, roughness);
        let F: vec3<f32> = fresnelSchlick(clamp(dot(H, V), 0.0, 1.0), F0);

        let numerator: vec3<f32> = NDF * G * F;
        let denominator: f32 = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001; // + 0.0001 to prevent divide by zero
        let specular: vec3<f32> = numerator / denominator;

        // kS is equal to Fresnel
        let kS: vec3<f32> = F;
        // for energy conservation, the diffuse and specular light can't
        // be above 1.0 (unless the surface emits light); to preserve this
        // relationship the diffuse component (kD) should equal 1.0 - kS.
        var kD: vec3<f32> = vec3(1.0, 1.0, 1.0) - kS;
        // multiply kD by the inverse metalness such that only non-metals
        // have diffuse lighting, or a linear blend if partly metal (pure metals
        // have no diffuse light).
        kD *= 1.0 - metallic;

        // scale light by NdotL
        let NdotL: f32 = max(dot(N, L), 0.0);

        // add to outgoing radiance Lo
        Lo += (kD * albedo.rgb / PI + specular) * radiance * NdotL;  // note that we already multiplied the BRDF by the Fresnel (kS) so we won't multiply by kS again

        result += Lo;
    }

    // TODO: make ambient light a uniform or iterate through all ambient lights

    let ambientIntensity = 0.3;
    let ambient: vec3<f32> = vec3(ambientIntensity, ambientIntensity, ambientIntensity) * albedo.rgb * ao;
    var color = result + ambient;

    if ((emissive.r > 0.0 || emissive.g > 0.0 || emissive.b > 0.0) && emissive.a > 0.0) {
        color = emissive.rgb;
    }

    // HDR tonemapping
    color = color / (color + vec3(1.0));
    // gamma correct
//    color = pow(color, vec3(1.0/2.2));

    return color;
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
    var final_color_rgb = compute_final_color(world_position, camera.position, normal, albedo, emissive, ao, roughness, metallic);

    return vec4(final_color_rgb, alpha);
}


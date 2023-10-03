struct CameraUniform {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    position: vec3<f32>,
    _padding: f32,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

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

@group(1) @binding(0)
var texture_g_buffer_normal: texture_2d<f32>;
@group(1) @binding(1)
var texture_g_buffer_albedo: texture_2d<f32>;
@group(1) @binding(2)
var texture_g_buffer_emissive: texture_2d<f32>;
@group(1) @binding(3)
var texture_g_buffer_ao_roughness_metallic: texture_2d<f32>;
@group(1) @binding(4)
var texture_g_buffer_depth: texture_depth_2d;

struct Light {
  position: vec3<f32>,
  radius: f32,
  color: vec3<f32>,
  _padding2: u32,
}

struct LightsUniform {
  lights: array<Light, 4>
};

@group(2) @binding(0)
var<uniform> lightsBuffer: LightsUniform;

fn world_from_screen_coord(coord : vec2<f32>, depth_sample: f32) -> vec3<f32> {
    // reconstruct world-space position from the screen coordinate
    let pos_clip = vec4(coord.x * 2.0 - 1.0, (1.0 - coord.y) * 2.0 - 1.0, depth_sample, 1.0);
    let pos_world_w = camera.inv_view_proj * pos_clip;
    let pos_world = pos_world_w.xyz / pos_world_w.www;
    return pos_world;
}

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

    let ambient: vec3<f32> = vec3(0.02, 0.02, 0.02) * albedo.rgb * ao;
    var color = result + ambient;

    // HDR tonemapping
    color = color / (color + vec3(1.0));
    // gamma correct
    color = pow(color, vec3(1.0/2.2));

    return color;
}

@fragment
fn fs_main(@builtin(position) coord : vec4<f32>) -> @location(0) vec4<f32> {
    // normal vector (after normal mapping which was computed in write to g buffers shader)
    let normal = textureLoad(
        texture_g_buffer_normal,
        vec2<i32>(floor(coord.xy)),
        0
    ).xyz;

    // albedo texture
    let albedo = textureLoad(
        texture_g_buffer_albedo,
        vec2<i32>(floor(coord.xy)),
        0
    ).rgba;

    // emissive
    let emissive = textureLoad(
        texture_g_buffer_emissive,
        vec2<i32>(floor(coord.xy)),
        0
    ).rgba;

    // ao roughness metallic
    let ao_roughness_metallic = textureLoad(
        texture_g_buffer_ao_roughness_metallic,
        vec2<i32>(floor(coord.xy)),
        0
    ).rgba;
    let ao = ao_roughness_metallic.r;
    let roughness = ao_roughness_metallic.g;
    let metallic = ao_roughness_metallic.b;

    // sample from depth buffer
    let depth = textureLoad(
        texture_g_buffer_depth,
        vec2<i32>(floor(coord.xy)),
        0
    );

    // depth >= 1 means nothing was there at this pixel
    if (depth >= 1.0) {
        discard;
    }

    // compute world position using depth buffer
    let depth_buffer_size = textureDimensions(texture_g_buffer_depth);
    let coord_uv = coord.xy / vec2<f32>(depth_buffer_size);
    let world_position = world_from_screen_coord(coord_uv, depth);

    // final color
    var final_color_rgb = compute_final_color(world_position, camera.position, normal, albedo, emissive, ao, roughness, metallic);

    return vec4(final_color_rgb, 1.0);
}
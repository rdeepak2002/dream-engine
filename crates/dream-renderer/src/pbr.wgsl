const PI: f32 = 3.14159265359;
const LIGHT_TYPE_POINT: u32 = 0u;
const LIGHT_TYPE_DIRECTIONAL: u32 = 1u;

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
        var L: vec3<f32> = vec3(0.0);
        if (light.light_type == LIGHT_TYPE_POINT) {
            L = normalize(lightPosition - world_position);
        }
        if (light.light_type == LIGHT_TYPE_DIRECTIONAL) {
            // TODO: verify
            L = normalize(light.direction);
        }
        let H: vec3<f32> = normalize(V + L);
        var radiance: vec3<f32> = vec3(0.0);
        if (light.light_type == LIGHT_TYPE_POINT) {
            let distance: f32 = length(lightPosition - world_position);
            let attenuation: f32 = 1.0 / pow(distance / light.radius + 1.0, 2.0);
            radiance = lightColor * attenuation;
        }
        if (light.light_type == LIGHT_TYPE_DIRECTIONAL) {
            // TODO: verify
            let attenuation: f32 = 1.0;
            radiance = lightColor * attenuation;
        }

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

    let ambientIntensity = 0.01;
    let ambient: vec3<f32> = vec3(ambientIntensity, ambientIntensity, ambientIntensity) * albedo.rgb * ao;
    var color = result + ambient;

    if ((emissive.r > 0.0 || emissive.g > 0.0 || emissive.b > 0.0) && emissive.a > 0.0) {
        color = emissive.rgb;
    }

    // HDR tonemapping
    let exposure: f32 = 4.0f;
    color = vec3(1.0) - exp(-color * exposure);
    // gamma correct
    let gamma: f32 = 1.2;
    color = pow(color, vec3(1.0 / gamma));

    return color;
}

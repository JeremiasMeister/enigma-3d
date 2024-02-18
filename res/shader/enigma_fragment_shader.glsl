#version 140

//uniforms
uniform float time;
uniform mat4 light_position;
uniform mat4 light_color;
uniform vec4 light_intensity;
uniform int light_amount;
uniform vec3 ambient_light_color;
uniform float ambient_light_intensity;
uniform float near; // Camera's near plane
uniform float far;  // Camera's far plane
uniform float shadow_far;  // Shadow's far plane
uniform samplerCube shadow_map0;
uniform samplerCube shadow_map1;
uniform samplerCube shadow_map2;
uniform samplerCube shadow_map3;

//attributes
in vec3 world_position;
in vec3 world_normal;
in vec3 view_direction;
in vec3 vertex_color;
in vec3 vertex_normal;
in vec2 vertex_texcoord;

//material properties
// material uniforms
uniform vec3 mat_color;
uniform sampler2D mat_albedo;
uniform sampler2D mat_normal;
uniform float mat_normal_strength;
uniform sampler2D mat_roughness;
uniform float mat_roughness_strength;
uniform sampler2D mat_metallic;
uniform float mat_metallic_strength;
uniform sampler2D mat_emissive;
uniform float mat_emissive_strength;
uniform float mat_transparency_strength;
uniform sampler2D skybox;

// fragment outputs
out vec4 color;

//constants
const float PI = 3.14159265359;

// Helper Functions for PBR
float remap(float value, float low1, float high1, float low2, float high2) {
    return low2 + (value - low1) * (high2 - low2) / (high1 - low1);
}

float CalculateShadow(samplerCube shadowMap, vec3 lightPosition){
    vec3 fragToLight = lightPosition - world_position;
    float distance = length(fragToLight);
    distance /= shadow_far;
    float shadowMapDepth = texture(shadowMap, normalize(fragToLight)).r;
    float shadow = 0.0;
    if (shadowMapDepth < distance ) {
        shadow = 1.0;
    }
    return remap(shadowMapDepth, 0.9, 1.0, 0.0, 1.0);
}


vec2 getSphereMapUV(vec3 dir) {
    float u = atan(dir.z, dir.x) / (2.0 * 3.14159265) + 0.5;
    float v = asin(dir.y) / 3.14159265 + 0.5;
    u = fract(u);
    v = fract(v);
    return vec2(u, v);
}

float DistributionGGX(vec3 N, vec3 H, float roughness) {
    float a = roughness * roughness;
    float a2 = a * a;
    float NdotH = max(dot(N, H), 0.0);
    float NdotH2 = NdotH * NdotH;

    float nom = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return nom / max(denom, 0.000001); // Prevent division by zero
}

float GeometrySchlickGGX(float NdotV, float roughness) {
    float r = (roughness + 1.0);
    float k = (r * r) / 8.0;

    float nom = NdotV;
    float denom = NdotV * (1.0 - k) + k;

    return nom / denom;
}

float GeometrySmith(vec3 N, vec3 V, vec3 L, float roughness) {
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx1 = GeometrySchlickGGX(NdotL, roughness);
    float ggx2 = GeometrySchlickGGX(NdotV, roughness);

    return ggx1 * ggx2;
}

vec3 fresnelSchlick(float cosTheta, vec3 F0) {
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}

// Main PBR calculation function
// PBR calculations including skybox lighting
vec4 calculatePBRColor(vec3 viewDir) {
    // Fetch material properties
    vec4 albedo_texel = texture(mat_albedo, vertex_texcoord);
    vec3 albedo = albedo_texel.rgb * mat_color;
    vec3 normal = normalize(vertex_normal + (texture(mat_normal, vertex_texcoord).rgb - 0.5) * mat_normal_strength);
    float roughness = max(texture(mat_roughness, vertex_texcoord).r, mat_roughness_strength);
    float metallic = max(texture(mat_metallic, vertex_texcoord).r, mat_metallic_strength);
    vec3 emissive = texture(mat_emissive, vertex_texcoord).rgb * mat_emissive_strength;

    // Calculate reflectance at normal incidence
    vec3 F0 = vec3(0.04);
    F0 = mix(F0, albedo, metallic);

    vec3 result = vec3(0.0);
    for(int i = 0; i < light_amount; i++) {
        vec3 lightDir = normalize(light_position[i].xyz - world_position);
        float distance = length(light_position[i].xyz - world_position);
        //float attenuation = 1.0 / (distance * distance);
        float attenuation = 1.0 / (distance * distance + 0.1 * distance + 0.01);
        vec3 radiance = light_color[i].xyz * light_intensity[i] * attenuation;

        vec3 halfDir = normalize(lightDir + viewDir);
        float NDF = DistributionGGX(normal, halfDir, roughness);
        float G = GeometrySmith(normal, viewDir, lightDir, roughness);
        vec3 F = fresnelSchlick(max(dot(halfDir, viewDir), 0.0), F0);

        vec3 kS = F;
        vec3 kD = vec3(1.0) - kS;
        kD *= 1.0 - metallic;

        float NdotL = max(dot(normal, lightDir), 0.0);

        vec3 numerator = NDF * G * F;
        float denominator = 4.0 * max(dot(normal, viewDir), 0.0) * NdotL + 0.0001;
        vec3 specular = numerator / denominator;

        vec3 ambient = ambient_light_color * ambient_light_intensity * albedo;
        vec3 diffuse = kD * albedo / PI;

        // Only direct light (diffuse + specular) is affected by shadows
        vec3 directLight = (diffuse + specular) * radiance * NdotL; // * (1.0 - shadow);

        // Accumulate result from this light
        result += ambient + directLight;
    }

    // Environmental reflection calculations
    vec3 reflectionVector = reflect(-viewDir, normal);
    vec2 uv = getSphereMapUV(reflectionVector);
    vec3 envReflection = texture(skybox, uv).rgb;
    vec3 fresnelEffect = fresnelSchlick(max(dot(viewDir, normal), 0.0), F0);
    vec3 envReflectionWithFresnel = envReflection * fresnelEffect * (1.0 - metallic) * (1.0 - roughness);

    // Combine PBR lighting with environmental reflection and emissive
    vec3 finalColor = result + emissive + envReflectionWithFresnel;

    return vec4(finalColor, albedo_texel.a * mat_transparency_strength);
}

void main() {

    float shadow0 = CalculateShadow(shadow_map0, light_position[0].xyz);
    float shadow1 = CalculateShadow(shadow_map1, light_position[1].xyz);
    float shadow2 = CalculateShadow(shadow_map2, light_position[2].xyz);
    float shadow3 = CalculateShadow(shadow_map3, light_position[3].xyz);

    float shadow = shadow0 * shadow1 * shadow2 * shadow3;

    vec4 pbrColor = calculatePBRColor(normalize(view_direction));
    color = vec4(shadow, shadow, shadow, 1.0);
}

#version 330 core

//uniforms
uniform float time;
uniform vec3 camera_position;
uniform mat4 light_position;
uniform mat4 light_direction;
uniform mat4 light_color;
uniform vec4 light_intensity;
uniform int light_amount;
uniform vec3 ambient_light_color;
uniform float ambient_light_intensity;

// shadow maps
uniform sampler2D shadow_map_0;
uniform sampler2D shadow_map_1;
uniform sampler2D shadow_map_2;
uniform sampler2D shadow_map_3;
uniform sampler2D shadow_point_0;
uniform sampler2D shadow_point_1;
uniform sampler2D shadow_point_2;
uniform sampler2D shadow_point_3;
uniform mat4 shadow_light_space_0;
uniform mat4 shadow_light_space_1;
uniform mat4 shadow_light_space_2;
uniform mat4 shadow_light_space_3;
uniform vec4 shadow_far_planes;
uniform vec4 light_cast_shadow;

//attributes
in vec3 world_position;
in vec3 view_direction;
in vec3 modelView_pos;
in vec3 object_position;
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
vec2 getSphereMapUV(vec3 dir) {
    float u = atan(dir.z, dir.x) / (2.0 * 3.14159265) + 0.5;
    float v = asin(dir.y) / 3.14159265 + 0.5;
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

float dir_shadow(sampler2D shadow_map, mat4 light_space, vec3 world_pos) {
    vec4 ls = light_space * vec4(world_pos, 1.0);
    vec3 proj = ls.xyz / ls.w;
    proj = proj * 0.5 + 0.5;
    if (proj.x < 0.0 || proj.x > 1.0 || proj.y < 0.0 || proj.y > 1.0 || proj.z < 0.0 || proj.z > 1.0) {
        return 1.0;
    }
    float stored = texture(shadow_map, proj.xy).r;
    return (proj.z - 0.0005 > stored) ? 0.0 : 1.0;
}

vec2 cube_atlas_uv(vec3 dir) {
    vec3 a = abs(dir);
    float s, t, major, col, row;
    if (a.x >= a.y && a.x >= a.z) {
        major = a.x;
        if (dir.x > 0.0) { col = 0.0; row = 0.0; s = -dir.z; t = -dir.y; }
        else              { col = 1.0; row = 0.0; s =  dir.z; t = -dir.y; }
    } else if (a.y >= a.x && a.y >= a.z) {
        major = a.y;
        if (dir.y > 0.0) { col = 0.0; row = 1.0; s =  dir.x; t =  dir.z; }
        else              { col = 1.0; row = 1.0; s =  dir.x; t = -dir.z; }
    } else {
        major = a.z;
        if (dir.z > 0.0) { col = 0.0; row = 2.0; s =  dir.x; t = -dir.y; }
        else              { col = 1.0; row = 2.0; s = -dir.x; t = -dir.y; }
    }
    vec2 face_uv = (vec2(s, t) / major + 1.0) * 0.5;
    return vec2((col + face_uv.x) * 0.5, (row + face_uv.y) / 3.0);
}

float point_shadow(sampler2D atlas, vec3 world_pos, vec3 light_pos, float far_plane) {
    vec3 dir = world_pos - light_pos;
    float current = length(dir) / far_plane;
    vec2 uv = cube_atlas_uv(dir);
    float stored = texture(atlas, uv).r;
    return (current - 0.0005 > stored) ? 0.0 : 1.0;
}

float compute_shadow(int i, vec3 world_pos, vec3 light_pos) {
    if (light_cast_shadow[i] < 0.5) return 1.0;
    bool is_dir = (light_direction[i].w == 1.0);
    if (i == 0) return is_dir ? dir_shadow(shadow_map_0, shadow_light_space_0, world_pos)
                              : point_shadow(shadow_point_0, world_pos, light_pos, shadow_far_planes[0]);
    if (i == 1) return is_dir ? dir_shadow(shadow_map_1, shadow_light_space_1, world_pos)
                              : point_shadow(shadow_point_1, world_pos, light_pos, shadow_far_planes[1]);
    if (i == 2) return is_dir ? dir_shadow(shadow_map_2, shadow_light_space_2, world_pos)
                              : point_shadow(shadow_point_2, world_pos, light_pos, shadow_far_planes[2]);
    if (i == 3) return is_dir ? dir_shadow(shadow_map_3, shadow_light_space_3, world_pos)
                              : point_shadow(shadow_point_3, world_pos, light_pos, shadow_far_planes[3]);
    return 1.0;
}

// Main PBR calculation function
// PBR calculations including skybox lighting
vec4 calculatePBRColor(vec3 viewDir) {
    // Fetch material properties
    vec4 albedo_texel = texture(mat_albedo, vertex_texcoord);
    float albedo_alpha = albedo_texel.a;
    vec3 albedo = albedo_texel.rgb * mat_color * vertex_color;
    vec3 normal = normalize(vertex_normal + (texture(mat_normal, vertex_texcoord).rgb - 0.5) * mat_normal_strength);
    float roughness = texture(mat_roughness, vertex_texcoord).r * mat_roughness_strength;
    float metallic = texture(mat_metallic, vertex_texcoord).r * mat_metallic_strength;
    vec3 emissive = texture(mat_emissive, vertex_texcoord).rgb * mat_emissive_strength;

    // Calculate reflectance at normal incidence
    vec3 F0 = vec3(0.04);
    F0 = mix(F0, albedo, metallic);

    vec3 ambient = ambient_light_color * ambient_light_intensity * albedo;
    vec3 result = ambient;
    for(int i = 0; i < light_amount; i++) {
        // Light calculations for each active light
        vec4 lightDirUniform = vec4(light_direction[i]);
        vec3 lightDir = normalize(light_position[i].xyz - world_position);
        if(lightDirUniform.w == 1.0) {
            lightDir = normalize(lightDirUniform.xyz);
        }
        vec3 halfDir = normalize(lightDir + viewDir);
        float distance = length(light_position[i].xyz - world_position);
        float attenuation = 1.0 / (distance * distance);
        vec3 radiance = light_color[i].xyz * light_intensity[i] * attenuation;

        // Cook-Torrance BRDF
        float NDF = DistributionGGX(normal, halfDir, roughness);
        float G = GeometrySmith(normal, viewDir, lightDir, roughness);
        vec3 F = fresnelSchlick(max(dot(halfDir, viewDir), 0.0), F0);

        vec3 kS = F;
        vec3 kD = vec3(1.0) - kS;
        kD *= 1.0 - metallic;

        float NdotL = max(dot(normal, lightDir), 0.0);

        // Combine terms
        vec3 numerator = NDF * G * F;
        float denominator = 4.0 * max(dot(normal, viewDir), 0.0) * NdotL + 0.0001;
        vec3 specular = numerator / denominator;

        vec3 diffuse = kD * albedo / PI;
        vec3 reflection = (diffuse + specular) * radiance * NdotL;

        float shadow = compute_shadow(i, world_position, light_position[i].xyz);
        result += reflection * shadow;
    }

    // Calculate reflection vector for environmental lighting
    vec3 reflectionVector = reflect(-viewDir, normal);
    vec2 uv = getSphereMapUV(reflectionVector);
    vec3 envReflection = textureLod(skybox, uv, roughness).rgb;

    // Apply fresnel effect to the environmental reflection
    vec3 fresnelEffect = fresnelSchlick(max(dot(normal, viewDir), 0.1), F0);
    vec3 envReflectionWithFresnel = envReflection * fresnelEffect * (1.0 - metallic);

    // Combine PBR lighting with environmental reflection
    vec3 finalColor = result + emissive + envReflectionWithFresnel;

    return vec4(finalColor, albedo_alpha * mat_transparency_strength);
}

void main() {
    color = calculatePBRColor(normalize(modelView_pos));
}

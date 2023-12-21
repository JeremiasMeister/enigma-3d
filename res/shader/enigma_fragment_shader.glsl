#version 140

//uniforms
uniform float time;
uniform vec3 light_position;
uniform vec3 light_color;
uniform float light_intensity;
uniform vec3 ambient_light_color;
uniform float ambient_light_intensity;

//attributes
in vec3 world_position;

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

// fragment outputs
out vec4 color;

// functions
vec3 light_direction(){
    return normalize(light_position - world_position);
}

float light_intensity_at_point(){
    float distance = length(light_position - world_position);
    return light_intensity / (distance * distance);
}

vec3 lambert_shading(){
    return (max(dot(light_direction(), vertex_normal), 0.0) * light_color * light_intensity_at_point());
}

void main() {
    vec4 albedo = texture(mat_albedo, vertex_texcoord) * vec4(mat_color, 1.0);
    color = albedo * vec4(lambert_shading(),1.0) + vec4((ambient_light_color * ambient_light_intensity),1.0);
}



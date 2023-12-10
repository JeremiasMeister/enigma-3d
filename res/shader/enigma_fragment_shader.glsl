#version 140

//uniforms
uniform float time;

//attributes
in vec3 vertex_color;
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

void main() {
    vec4 albedo = texture(mat_albedo, vertex_texcoord) * vec4(mat_color, 1.0);
    color = albedo;
}
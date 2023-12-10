#version 140

//uniforms
uniform float time;
uniform mat4 matrix;

//attributes
in vec3 position;
in vec2 texcoord;
in vec3 color;

out vec3 vertex_color;
out vec2 vertex_texcoord;

// material uniforms
uniform vec3 mat_color;
uniform sampler2D mat_albedo;
uniform sampler2D mat_normal;
uniform float mat_normal_strength;
uniform sampler2D mat_roughness;
uniform float mat_roughness_strength;
uniform sampler2D mat_metallic;
uniform float mat_metallic_strength;

void main() {
    vec3 pos = position;
    float movement = 0.2;
    pos.x += sin(time + pos.y) * movement;
    pos.y += cos(time + pos.x) * movement;
    pos *= 0.5;
    gl_Position = matrix * vec4(pos, 1.0);
    vertex_color = color;
    vertex_texcoord = texcoord;
}
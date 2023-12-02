#version 140

//uniforms
uniform float time;

//attributes
in vec3 vertex_color;
in vec2 vertex_texcoord;

out vec4 color;

void main() {
    color = vec4(vertex_color, 1.0);
}
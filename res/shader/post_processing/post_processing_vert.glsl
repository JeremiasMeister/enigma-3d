#version 140

in vec3 position;
in vec2 texcoord;
in vec3 color;
in vec3 normal;

out vec2 TEXCOORD;


void main() {
    TEXCOORD = texcoord;
    gl_Position = vec4(position, 1.0);
}
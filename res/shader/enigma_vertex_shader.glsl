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

void main() {
    vec3 pos = position;
    pos.x += sin(time + pos.y);
    pos.y += cos(time + pos.x);
    pos *= 0.5;
    gl_Position = matrix * vec4(pos, 1.0);
    vertex_color = color;
    vertex_texcoord = texcoord;
}
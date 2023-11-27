#version 140

//uniforms
uniform float time;


in vec3 position;

void main() {
    vec3 pos = position;
    pos.x += sin(time);
    gl_Position = vec4(pos, 1.0);
}
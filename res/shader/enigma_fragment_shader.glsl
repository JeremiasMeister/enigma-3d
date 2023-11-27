#version 140

//uniforms
uniform float time;

out vec4 color;


void main() {
    color = vec4(sin(time), sin(time*2), sin(time/2), 1.0);
}
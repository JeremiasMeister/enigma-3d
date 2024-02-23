#version 450 core

in vec2 TEXCOORD;

out vec4 color;

uniform sampler2D scene;
uniform sampler2D bloomBlur;

void main() {
    vec4 sceneColor = texture(scene, TEXCOORD);
    vec4 bloomColor = texture(bloomBlur, TEXCOORD);

    // Combine the scene with the bloom effect
    color = sceneColor + bloomColor;
}
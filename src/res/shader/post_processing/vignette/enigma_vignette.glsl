#version 450 core

in vec2 TEXCOORD;
out vec4 color;

uniform sampler2D scene;
uniform float intensity;
uniform float falloff;
uniform vec3 vignette_color;
uniform float opacity;

void main() {
    vec4 original_color = texture(scene, TEXCOORD);

    // Calculate distance from center
    vec2 position = TEXCOORD - 0.5;
    float dist = length(position);


    // Calculate vignette effect
    float vignette = smoothstep(1.0-intensity, (1.0-intensity) - falloff, dist);

    // Mix the vignette color with the original color
    vec3 output_color = mix(vignette_color, original_color.rgb, vignette);

    // Apply opacity
    color = vec4(mix(original_color.rgb, output_color, opacity), original_color.a);
}
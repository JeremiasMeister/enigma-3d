#version 450 core

in vec2 TEXCOORD;

out vec4 color;

uniform sampler2D scene;
uniform float threshold;

void main() {
    vec4 tex = texture(scene, TEXCOORD);
    float brightness = dot(tex.rgb, vec3(0.2126, 0.7152, 0.0722));
    if(brightness > threshold) { // Threshold for brightness
        color = tex;
    } else {
        color = vec4(0.0);
    }
}
#version 450 core

in vec2 TEXCOORD;
out vec4 color;

uniform sampler2D scene;
uniform bool horizontal;
uniform int iterations;

void main() {
    float weight = 1.0/iterations;
    vec2 tex_offset = 1.0 / textureSize(scene, 0); // gets size of single texel
    vec3 result = texture(scene, TEXCOORD).rgb * weight; // current fragment's contribution
    if(horizontal) {
        for(int i = 1; i < iterations; ++i) {
            result += texture(scene, TEXCOORD + vec2(tex_offset.x * i, 0.0)).rgb * weight;
            result += texture(scene, TEXCOORD - vec2(tex_offset.x * i, 0.0)).rgb * weight;
        }
    } else {
        for(int i = 1; i < iterations; ++i) {
            result += texture(scene, TEXCOORD + vec2(0.0, tex_offset.y * i)).rgb * weight;
            result += texture(scene, TEXCOORD - vec2(0.0, tex_offset.y * i)).rgb * weight;
        }
    }

    color = vec4(result, 1.0);
}
#version 140

in vec2 TEXCOORD;

out vec4 color;

uniform sampler2D scene;

void main() {
    vec4 c = texture(scene, TEXCOORD);
    float grayscale = dot(c.rgb, vec3(0.299, 0.587, 0.114));
    color = vec4(grayscale, grayscale, grayscale, c.a);
}
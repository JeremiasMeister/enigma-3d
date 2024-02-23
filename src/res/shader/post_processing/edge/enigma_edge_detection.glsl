#version 450 core

#define MIN_DEPTH 0.995

in vec2 TEXCOORD;
out vec4 color;

uniform sampler2D scene;
uniform sampler2D depth;
uniform float threshold;
uniform vec2 screenSize;
uniform vec3 outlineColor;
uniform float near; // Camera's near plane
uniform float far;  // Camera's far plane

float linearizeDepth(float depth) {
    float z = depth * 2.0 - 1.0; // Back to NDC
    return (2.0 * near * far) / (far + near - z * (far - near));
}

void main() {
    float depthValue = texture(depth, TEXCOORD).r;
    float depthLeft = texture(depth, TEXCOORD + vec2(-1.0 / screenSize.x, 0)).r;
    float depthRight = texture(depth, TEXCOORD + vec2(1.0 / screenSize.x, 0)).r;
    float depthUp = texture(depth, TEXCOORD + vec2(0, 1.0 / screenSize.y)).r;
    float depthDown = texture(depth, TEXCOORD + vec2(0, -1.0 / screenSize.y)).r;

    float linearDepth = linearizeDepth(depthValue);
    float linearDepthLeft = linearizeDepth(depthLeft);
    float linearDepthRight = linearizeDepth(depthRight);
    float linearDepthUp = linearizeDepth(depthUp);
    float linearDepthDown = linearizeDepth(depthDown);
    // Scale the depth value for visualization
    float scaledDepth = (linearDepth - near) / (far - near);
    float scaledDepthLeft = (linearDepthLeft - near) / (far - near);
    float scaledDepthRight = (linearDepthRight - near) / (far - near);
    float scaledDepthUp = (linearDepthUp - near) / (far - near);
    float scaledDepthDown = (linearDepthDown - near) / (far - near);

    float edge = 0.0;
    if (abs(scaledDepth - scaledDepthLeft) > threshold || abs(scaledDepth - scaledDepthRight) > threshold ||
    abs(scaledDepth - scaledDepthUp) > threshold || abs(scaledDepth - scaledDepthDown) > threshold) {
        edge = 1.0;
    }

    vec4 c = mix(texture(scene, TEXCOORD), vec4(outlineColor, 1.0), edge);

    color = c;
}
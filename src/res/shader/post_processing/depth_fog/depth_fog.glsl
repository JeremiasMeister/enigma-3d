#version 450 core

in vec2 TEXCOORD;
out vec4 color;

uniform sampler2D scene;
uniform sampler2D depth;
uniform vec3 fogColor;
uniform float near; // Camera's near plane
uniform float far;  // Camera's far plane
uniform float minDepth;
uniform float maxDepth;
uniform float fogCutoff;
uniform float opacity;

float linearizeDepth(float depth) {
    float z = depth * 2.0 - 1.0; // Back to NDC
    return (2.0 * near * far) / (far + near - z * (far - near));
}

float remapDepth(float depth) {
    if (depth > fogCutoff) {
        return 0.0; // Return 0 (black) for depths beyond maxDepth
    }
    // Remap the depth to the specified range
    return clamp((depth - minDepth) / (maxDepth - minDepth), 0.0, 1.0);
}

void main() {
    float depthValue = texture(depth, TEXCOORD).r;
    // Linearize the depth
    float linearDepth = linearizeDepth(depthValue);
    // Remap the linearized depth
    float remappedDepth = remapDepth(linearDepth);
    vec4 fog = vec4(fogColor.r, fogColor.g, fogColor.b, opacity) * remappedDepth;

    vec4 sceneColor = texture(scene, TEXCOORD);

    // Mix the scene color with the depth visualization
    color = mix(sceneColor, fog, remappedDepth);
}
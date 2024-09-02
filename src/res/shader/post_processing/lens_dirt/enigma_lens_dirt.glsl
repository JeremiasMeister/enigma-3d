#version 140

in vec2 TEXCOORD;
out vec4 color;

uniform sampler2D scene;
uniform sampler2D dirt_texture;
uniform float intensity;
uniform vec2 tile_scale;
uniform uvec2 screen_size;
uniform float light_sensitivity;

// Function to calculate perceived brightness
float get_brightness(vec3 color) {
    return dot(color, vec3(0.2126, 0.7152, 0.0722));
}

void main() {
    vec4 scene_color = texture(scene, TEXCOORD);

    // Calculate tiled texture coordinates
    vec2 tiled_coord = fract(TEXCOORD * screen_size / tile_scale);
    vec4 dirt_color = texture(dirt_texture, tiled_coord);

    // Calculate the scene brightness
    float scene_brightness = get_brightness(scene_color.rgb);

    // Adjust the dirt intensity based on scene brightness and light sensitivity
    float adjusted_intensity = intensity * pow(scene_brightness, light_sensitivity);

    // Blend the scene with the dirt texture
    vec3 blended = mix(scene_color.rgb, scene_color.rgb + dirt_color.rgb * intensity, adjusted_intensity * dirt_color.r);

    // Add a slight bloom effect to the dirt, also affected by scene brightness
    vec3 bloom = dirt_color.rgb * adjusted_intensity * 0.5 * scene_brightness;

    color = vec4(blended + bloom, scene_color.a);
}
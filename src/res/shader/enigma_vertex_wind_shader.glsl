#version 150

//uniforms
uniform float time;
uniform mat4 matrix;
uniform mat4 projection_matrix;
uniform mat4 view_matrix;
uniform mat4 model_matrix;

//attributes
in vec3 position;
in vec2 texcoord;
in vec3 normal;
in vec3 color;
in uint index;


out vec3 world_position;
out vec3 view_direction;

out vec3 vertex_color;
out vec3 vertex_normal;
out vec2 vertex_texcoord;

// material uniforms
uniform vec3 mat_color;
uniform sampler2D mat_albedo;
uniform sampler2D mat_normal;
uniform float mat_normal_strength;
uniform sampler2D mat_roughness;
uniform float mat_roughness_strength;
uniform sampler2D mat_metallic;
uniform float mat_metallic_strength;

uniform vec3 wind_direction = vec3(1.0, 0.0, 0.0); // Default to blowing along x-axis
uniform float wind_strength = 0.15;
uniform float wind_speed = 25.0;

void main() {
    // Calculate wind effect
    float height_factor = position.y; // Assuming Y is up
    float wind_effect = sin(time * wind_speed + position.x * 0.5 + position.z * 0.5) * wind_strength * height_factor;

    // Apply wind to position
    vec3 wind_offset = wind_direction * wind_effect;
    vec3 wind_pos = position + wind_offset;
    mat4 modelview = view_matrix * model_matrix;

    gl_Position = projection_matrix * modelview * vec4(wind_pos, 1.0);
    world_position = (modelview * vec4(position, 1.0)).xyz;
    vertex_normal = transpose(inverse(mat3(modelview))) * normal;
    view_direction = (view_matrix * vec4(0.0, 0.0, 0.0, 1.0)).xyz - world_position;

    vertex_color = color;
    vertex_texcoord = texcoord;
}
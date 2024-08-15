#version 330 core

//uniforms
uniform float time;
uniform vec3 camera_position;
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


out vec3 v_world_position;
out vec3 v_view_direction;
out vec3 v_modelView_pos;
out vec3 v_vertex_color;
out vec3 v_vertex_normal;
out vec2 v_vertex_texcoord;
out vec3 v_object_position;
out vec3 v_position;

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
uniform float wind_strength = 0.01;
uniform float wind_speed = 50.0;

// Pseudo-random function
float random(vec3 pos) {
    return fract(sin(dot(pos, vec3(12.9898, 78.233, 45.5432))) * 43758.5453);
}

void main() {
    v_object_position = vec3(model_matrix[3]);
    float random_offset = random(v_object_position) * 1000.0;

    // Calculate wind effect
    float height_factor = position.y; // Assuming Y is up
    float wind_effect = sin(time * wind_speed + position.x * 0.5 + position.z * 0.5 + random_offset) * wind_strength * height_factor;

    // Apply wind to position
    vec3 wind_offset = wind_direction * wind_effect;
    vec3 wind_pos = position + wind_offset;
    mat4 modelview = view_matrix * model_matrix;

    gl_Position = projection_matrix * modelview * vec4(wind_pos, 1.0);
    v_world_position = (model_matrix * vec4(position, 1.0)).xyz;
    v_vertex_normal = transpose(inverse(mat3(modelview))) * normal;
    v_view_direction = normalize(camera_position - v_world_position);
    v_modelView_pos = -(modelview * vec4(position, 1.0)).xyz;
    v_vertex_color = color;
    v_position = position;
    v_vertex_texcoord = texcoord;
}
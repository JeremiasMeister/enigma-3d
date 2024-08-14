#version 330 core

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


out vec3 v_world_position;
out vec3 v_view_direction;
out vec3 v_object_position;
out vec3 v_vertex_color;
out vec3 v_vertex_normal;
out vec2 v_vertex_texcoord;

// material uniforms
uniform vec3 mat_color;
uniform sampler2D mat_albedo;
uniform sampler2D mat_normal;
uniform float mat_normal_strength;
uniform sampler2D mat_roughness;
uniform float mat_roughness_strength;
uniform sampler2D mat_metallic;
uniform float mat_metallic_strength;

void main() {
    vec3 pos = position;
    float movement = 0.2;

    mat4 modelview = view_matrix * model_matrix;

    gl_Position = projection_matrix * modelview * vec4(position, 1.0);

    v_world_position = (modelview * vec4(position, 1.0)).xyz;
    v_vertex_normal = transpose(inverse(mat3(modelview))) * normal;
    v_view_direction = (view_matrix * vec4(0.0, 0.0, 0.0, 1.0)).xyz - v_world_position;
    v_object_position = vec3(model_matrix[3]);
    v_vertex_color = color;
    v_vertex_texcoord = texcoord;
}
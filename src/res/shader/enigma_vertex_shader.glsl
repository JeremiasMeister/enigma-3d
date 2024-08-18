#version 330 core

//uniforms
uniform float time;
uniform vec3 camera_position;
uniform mat4 matrix;
uniform mat4 projection_matrix;
uniform mat4 view_matrix;

//attributes
in vec3 position;
in vec2 texcoord;
in vec3 normal;
in vec3 color;
in uint index;
in mat4 model_matrix; // per instance attribute


out vec3 v_world_position;
out vec3 v_view_direction;
out vec3 v_modelView_pos;
out vec3 v_object_position;
out vec3 v_vertex_color;
out vec3 v_vertex_normal;
out vec2 v_vertex_texcoord;
out vec3 v_position;
out mat4 v_model_matrix;

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
    float movement = 0.2;

    mat4 modelview = view_matrix * model_matrix;

    gl_Position = projection_matrix * modelview * vec4(position, 1.0);
    v_model_matrix = model_matrix;
    v_position = position;
    v_world_position = (model_matrix * vec4(position, 1.0)).xyz;
    v_vertex_normal = transpose(inverse(mat3(modelview))) * normal;
    v_view_direction = normalize(v_world_position - camera_position);
    v_modelView_pos = -(modelview * vec4(position, 1.0)).xyz;
    v_object_position = vec3(model_matrix[3]);
    v_vertex_color = color;
    v_vertex_texcoord = texcoord;
}
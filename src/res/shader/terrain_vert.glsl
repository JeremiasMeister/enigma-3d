#version 330 core

in vec3 position;
in vec3 normal;
in vec3 color;

uniform mat4 model_matrix;
uniform mat4 view_matrix;
uniform mat4 projection_matrix;

out vec3 v_world_position;
out vec3 v_normal;
out vec3 v_color;

void main() {
    vec4 world_pos = model_matrix * vec4(position, 1.0);
    v_world_position = world_pos.xyz;
    v_normal = normalize(mat3(transpose(inverse(model_matrix))) * normal);
    v_color = color;
    gl_Position = projection_matrix * view_matrix * world_pos;
}

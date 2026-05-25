#version 330 core
uniform mat4 view_matrix;
uniform mat4 projection_matrix;

in vec3 position;
in vec2 uv;
in vec4 color;

out vec2 v_uv;
out vec4 v_color;

void main() {
    v_uv = uv;
    v_color = color;
    gl_Position = projection_matrix * view_matrix * vec4(position, 1.0);
}

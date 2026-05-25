#version 330 core

//uniforms
uniform float time;
uniform vec3 camera_position;
uniform float mat_transparency_strength;

//attributes
in vec3 world_position;
in vec3 view_direction;
in vec3 modelView_pos;
in vec3 object_position;
in vec3 vertex_color;
in vec3 vertex_normal;
in vec2 vertex_texcoord;

//material properties
// material uniforms
uniform vec3 mat_color;
uniform sampler2D mat_albedo;
uniform vec2 mat_uv_tiling;
uniform vec2 mat_uv_offset;

// fragment outputs
out vec4 color;

void main() {
    vec2 uv = vertex_texcoord * mat_uv_tiling + mat_uv_offset;
    vec4 tex = texture(mat_albedo, uv);
    color = vec4(tex.rgb * mat_color * vertex_color, tex.a * mat_transparency_strength);
}



#version 140

//uniforms
uniform float time;

//attributes
in vec3 world_position;
in vec3 view_direction;
in vec3 vertex_color;
in vec3 vertex_normal;
in vec2 vertex_texcoord;

//material properties
// material uniforms
uniform vec3 mat_color;
uniform sampler2D mat_albedo;

// fragment outputs
out vec4 color;

void main() {
    vec4 tex = texture(mat_albedo, vertex_texcoord);
    color = vec4(tex.rgb * mat_color * vertex_color, 1.0);
}



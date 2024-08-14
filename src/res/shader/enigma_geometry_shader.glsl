#version 330 core
#extension GL_EXT_geometry_shader : enable

layout(triangles) in;
layout(triangle_strip, max_vertices = 3) out;

// Input from vertex shader
in vec3 v_world_position[];
in vec3 v_view_direction[];
in vec3 v_object_position[];
in vec3 v_vertex_color[];
in vec3 v_vertex_normal[];
in vec2 v_vertex_texcoord[];

// Output to fragment shader (same names as input)
out vec3 world_position;
out vec3 view_direction;
out vec3 object_position;
out vec3 vertex_color;
out vec3 vertex_normal;
out vec2 vertex_texcoord;

void main() {
    for(int i = 0; i < 3; i++) {
        // Pass through the position
        gl_Position = gl_in[i].gl_Position;

        // Pass through all other attributes
        world_position = v_world_position[i];
        view_direction = v_view_direction[i];
        object_position = v_object_position[i];
        vertex_color = v_vertex_color[i];
        vertex_normal = v_vertex_normal[i];
        vertex_texcoord = v_vertex_texcoord[i];

        EmitVertex();
    }
    EndPrimitive();
}
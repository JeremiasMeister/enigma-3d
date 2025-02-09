#version 330 core

layout (triangles) in;
layout (triangle_strip, max_vertices = 128) out;

// Input from vertex shader
in vec3 v_position[];
in vec3 v_world_position[];
in vec3 v_view_direction[];
in vec3 v_modelView_pos[];
in vec3 v_object_position[];
in vec3 v_vertex_color[];
in vec3 v_vertex_normal[];
in vec2 v_vertex_texcoord[];
in mat4 v_model_matrix[];

// Output to fragment shader
out vec3 world_position;
out vec3 view_direction;
out vec3 modelView_pos;
out vec3 object_position;
out vec3 vertex_color;
out vec3 vertex_normal;
out vec2 vertex_texcoord;
out vec3 position;

uniform mat4 projection_matrix;
uniform mat4 view_matrix;
layout(std140) uniform BoneTransforms {
    mat4 bone_transforms[128];
};

const float cube_size = 0.3; // Size of the cube at each joint

void emitCube(vec3 center, vec3 color) {
    vec3 vertices[8];
    vertices[0] = center + vec3(-cube_size, -cube_size, -cube_size);
    vertices[1] = center + vec3(cube_size, -cube_size, -cube_size);
    vertices[2] = center + vec3(cube_size, cube_size, -cube_size);
    vertices[3] = center + vec3(-cube_size, cube_size, -cube_size);
    vertices[4] = center + vec3(-cube_size, -cube_size, cube_size);
    vertices[5] = center + vec3(cube_size, -cube_size, cube_size);
    vertices[6] = center + vec3(cube_size, cube_size, cube_size);
    vertices[7] = center + vec3(-cube_size, cube_size, cube_size);

    int faces[24] = int[](
        0, 1, 2, 3,  // Front
        5, 4, 7, 6,  // Back
        4, 0, 3, 7,  // Left
        1, 5, 6, 2,  // Right
        3, 2, 6, 7,  // Top
        4, 5, 1, 0   // Bottom
    );

    for (int f = 0; f < 6; ++f) {
        for (int v = 0; v < 4; ++v) {
            gl_Position = projection_matrix * view_matrix * vec4(vertices[faces[f * 4 + v]], 1.0);
            vertex_color = color;
            EmitVertex();
        }
        EndPrimitive();
    }
}

void main() {
    // Pass through the original triangle
    for(int i = 0; i < 3; i++) {
        gl_Position = gl_in[i].gl_Position;
        world_position = v_world_position[i];
        view_direction = v_view_direction[i];
        modelView_pos = v_modelView_pos[i];
        object_position = v_object_position[i];
        vertex_color = v_vertex_color[i];
        vertex_normal = v_vertex_normal[i];
        vertex_texcoord = v_vertex_texcoord[i];
        position = v_position[i];
        EmitVertex();
    }
    EndPrimitive();
    /*
    mat4 model_matrix = v_model_matrix[0]; // Assuming all vertices have the same model matrix
    for(int i = 0; i < 128; i++) {
        vec3 jointPosition = (bone_transforms[i] * vec4(v_object_position[0], 1)).xyz;
        emitCube(jointPosition, vec3(1.0, 0.0, 0.0)); // Red cube for each joint
    }
    */
}
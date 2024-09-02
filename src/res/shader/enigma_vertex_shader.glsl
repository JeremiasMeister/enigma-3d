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
in uvec4 bone_indices;
in vec4 bone_weights;
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

// skeletal animation uniforms
layout(std140) uniform BoneTransforms {
    mat4 bone_transforms[128];
};
uniform bool has_skeleton;

void main() {
    vec3 total_position = position;
    vec3 total_normal = normal;

    if (has_skeleton) {
        total_position = vec3(0.0);
        total_normal = vec3(0.0);

        for(int i = 0; i < 4; i++) {
            vec4 localPosition = bone_transforms[bone_indices[i]] * vec4(position, 1.0);
            total_position += (localPosition * bone_weights[i]).xyz;

            vec4 world_normal = bone_transforms[bone_indices[i]] * vec4(normal, 1.0);
            total_normal += normalize((world_normal * bone_weights[i]).xyz);
        }
    }
    mat4 modelview = view_matrix * model_matrix;
    gl_Position = projection_matrix * modelview * vec4(total_position, 1.0);
    v_model_matrix = model_matrix;
    v_position = total_position;
    v_world_position = (model_matrix * vec4(total_position, 1.0)).xyz;
    v_vertex_normal = transpose(inverse(mat3(modelview))) * total_normal;
    v_view_direction = normalize(v_world_position - camera_position);
    v_modelView_pos = -(modelview * vec4(total_position, 1.0)).xyz;
    v_object_position = vec3(model_matrix[3]);
    v_vertex_color = color;
    v_vertex_texcoord = texcoord;
}
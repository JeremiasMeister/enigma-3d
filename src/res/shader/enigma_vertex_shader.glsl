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

uniform bool has_skeleton;
layout(std140) uniform BoneTransforms {
    mat4 bone_transforms[128];
};

void main() {
    vec3 local_position = position;
    vec3 local_normal = normal;

    if (has_skeleton) {
        mat4 skin_matrix = bone_transforms[bone_indices.x] * bone_weights.x
                         + bone_transforms[bone_indices.y] * bone_weights.y
                         + bone_transforms[bone_indices.z] * bone_weights.z
                         + bone_transforms[bone_indices.w] * bone_weights.w;
        local_position = (skin_matrix * vec4(position, 1.0)).xyz;
        vec3 skinned_n = (skin_matrix * vec4(normal, 0.0)).xyz;
        local_normal = length(skinned_n) > 0.0001 ? normalize(skinned_n) : normal;
    }

    mat4 modelview = view_matrix * model_matrix;
    gl_Position = projection_matrix * modelview * vec4(local_position, 1.0);
    v_model_matrix = model_matrix;
    v_position = local_position;
    v_world_position = (model_matrix * vec4(local_position, 1.0)).xyz;
    v_vertex_normal = transpose(inverse(mat3(modelview))) * local_normal;
    v_view_direction = normalize(v_world_position - camera_position);
    v_modelView_pos = -(modelview * vec4(local_position, 1.0)).xyz;
    v_object_position = vec3(model_matrix[3]);
    v_vertex_color = color;
    v_vertex_texcoord = texcoord;
}
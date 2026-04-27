#version 330 core

in vec3 position;
in uvec4 bone_indices;
in vec4 bone_weights;
in mat4 model_matrix;

uniform mat4 light_space_matrix;
uniform bool has_skeleton;

layout(std140) uniform BoneTransforms {
    mat4 bone_transforms[128];
};

out vec3 v_world_pos;

void main() {
    vec3 local_pos = position;

    if (has_skeleton) {
        mat4 skin = bone_transforms[bone_indices.x] * bone_weights.x
                  + bone_transforms[bone_indices.y] * bone_weights.y
                  + bone_transforms[bone_indices.z] * bone_weights.z
                  + bone_transforms[bone_indices.w] * bone_weights.w;
        local_pos = (skin * vec4(position, 1.0)).xyz;
    }

    v_world_pos = (model_matrix * vec4(local_pos, 1.0)).xyz;
    gl_Position = light_space_matrix * model_matrix * vec4(local_pos, 1.0);
}

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
    vec4 total_position = vec4(position,1);
    vec3 total_normal = normal;
    float weight_sum = 0;
    if (has_skeleton) {
        for(int i = 0; i < 4; i++) {
            float weight = float(bone_weights[i]);
            if (weight > 0.0) {
                mat4 bone_transform = bone_transforms[bone_indices[i]];
                vec4 transformed_position = bone_transform * vec4(position, 1.0);
                total_position += weight * transformed_position;

                mat3 normal_transform = mat3(bone_transform);
                total_normal += weight * (normal_transform * normal);

                weight_sum += weight;
            }
        }

        if (weight_sum > 0.0) {
            total_position /= weight_sum;
            total_normal = normalize(total_normal);
        }
    }
    mat4 modelview = view_matrix * model_matrix;
    gl_Position = projection_matrix * modelview * total_position;
    v_model_matrix = model_matrix;
    v_position = total_position.xyz;
    v_world_position = (model_matrix * total_position).xyz;
    v_vertex_normal = transpose(inverse(mat3(modelview))) * total_normal;
    v_view_direction = normalize(v_world_position - camera_position);
    v_modelView_pos = -(modelview * total_position).xyz;
    v_object_position = vec3(model_matrix[3]);
    v_vertex_color = color;
    v_vertex_texcoord = texcoord;
}
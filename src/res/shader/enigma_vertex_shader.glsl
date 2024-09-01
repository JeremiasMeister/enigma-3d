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
uniform bool has_skeleton = false;

void main() {
    vec3 animated_position = position;
    vec3 animated_normal = normal;

    if (has_skeleton) {
        animated_position = vec3(0.0);
        animated_normal = vec3(0.0);

        // Correctly accessing bone transforms from the uniform buffer
        mat4 transform0 = bone_transforms[bone_indices.x];
        mat4 transform1 = bone_transforms[bone_indices.y];
        mat4 transform2 = bone_transforms[bone_indices.z];
        mat4 transform3 = bone_transforms[bone_indices.w];

        animated_position += (transform0 * vec4(position, 1.0)).xyz * bone_weights.x;
        animated_position += (transform1 * vec4(position, 1.0)).xyz * bone_weights.y;
        animated_position += (transform2 * vec4(position, 1.0)).xyz * bone_weights.z;
        animated_position += (transform3 * vec4(position, 1.0)).xyz * bone_weights.w;

        animated_normal += (mat3(transform0) * normal) * bone_weights.x;
        animated_normal += (mat3(transform1) * normal) * bone_weights.y;
        animated_normal += (mat3(transform2) * normal) * bone_weights.z;
        animated_normal += (mat3(transform3) * normal) * bone_weights.w;
    }

    mat4 modelview = view_matrix * model_matrix;
    gl_Position = projection_matrix * modelview * vec4(animated_position, 1.0);
    v_model_matrix = model_matrix;
    v_position = animated_position;
    v_world_position = (model_matrix * vec4(animated_position, 1.0)).xyz;
    v_vertex_normal = transpose(inverse(mat3(modelview))) * animated_normal;
    v_view_direction = normalize(v_world_position - camera_position);
    v_modelView_pos = -(modelview * vec4(animated_position, 1.0)).xyz;
    v_object_position = vec3(model_matrix[3]);
    v_vertex_color = color;
    v_vertex_texcoord = texcoord;
}
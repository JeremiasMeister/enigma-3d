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
in uint index;
in uvec4 bone_indices;
in vec4 bone_weights;
in mat4 model_matrix; // per instance attribute


out vec3 v_world_position;
out vec3 v_view_direction;
out vec3 v_modelView_pos;
out vec3 v_vertex_color;
out vec3 v_vertex_normal;
out vec2 v_vertex_texcoord;
out vec3 v_object_position;
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

uniform vec3 wind_direction = vec3(1.0, 0.0, 0.0); // Default to blowing along x-axis
uniform float wind_strength = 0.01;
uniform float wind_speed = 50.0;

// Pseudo-random function
float random(vec3 pos) {
    return fract(sin(dot(pos, vec3(12.9898, 78.233, 45.5432))) * 43758.5453);
}

void main() {
    v_object_position = vec3(model_matrix[3]);
    float random_offset = random(v_object_position) * 1000.0;

    vec3 animated_position = position;
    vec3 animated_normal = normal;

    if (has_skeleton) {
        animated_position = vec3(0.0);
        animated_normal = vec3(0.0);

        // Access bone transforms from the uniform buffer
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

    // Calculate wind effect
    float height_factor = animated_position.y; // Assuming Y is up
    float wind_effect = sin(time * wind_speed + animated_position.x * 0.5 + animated_position.z * 0.5 + random_offset) * wind_strength * height_factor;

    // Apply wind to position
    vec3 wind_offset = wind_direction * wind_effect;
    vec3 wind_pos = animated_position + wind_offset;
    mat4 modelview = view_matrix * model_matrix;

    gl_Position = projection_matrix * modelview * vec4(wind_pos, 1.0);
    v_world_position = (model_matrix * vec4(animated_position, 1.0)).xyz;
    v_vertex_normal = transpose(inverse(mat3(modelview))) * animated_normal;
    v_view_direction = normalize(camera_position - v_world_position);
    v_modelView_pos = -(modelview * vec4(animated_position, 1.0)).xyz;
    v_vertex_color = color;
    v_position = animated_position;
    v_model_matrix = model_matrix;
    v_vertex_texcoord = texcoord;
}
#version 150

//uniforms
uniform float time;
uniform mat4 matrix;
uniform mat4 projection_matrix;
uniform mat4 view_matrix;
uniform mat4 model_matrix;

//attributes
in vec3 position;
in vec2 texcoord;
in vec3 normal;
in vec3 color;
in uint index;


out vec3 world_position;
out vec3 world_normal;
out vec3 view_direction;

out vec3 vertex_color;
out vec3 vertex_normal;
out vec2 vertex_texcoord;

// material uniforms
uniform vec3 mat_color;
uniform sampler2D mat_albedo;
uniform sampler2D mat_normal;
uniform float mat_normal_strength;
uniform sampler2D mat_roughness;
uniform float mat_roughness_strength;
uniform sampler2D mat_metallic;
uniform float mat_metallic_strength;

void main() {
    vec3 pos = position;
    mat4 modelview = view_matrix * model_matrix;
    mat4 normal_matrix = transpose(inverse(model_matrix)); // Changed to model_matrix for world space normals

    gl_Position = projection_matrix * modelview * vec4(pos, 1.0);

    // For world position, use only the model_matrix
    world_position = (model_matrix * vec4(pos, 1.0)).xyz;

    // For world normal, use the normal_matrix calculated from the model_matrix
    world_normal = mat3(normal_matrix) * normal;

    vec3 camera_world_position = (inverse(view_matrix) * vec4(0.0, 0.0, 0.0, 1.0)).xyz;
    view_direction = camera_world_position - world_position;

    vertex_color = color;
    vertex_texcoord = texcoord;
}
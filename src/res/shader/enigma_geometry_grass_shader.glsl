#version 330 core

layout (triangles) in;
layout (triangle_strip, max_vertices = 60) out;

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

uniform float time;
uniform vec3 camera_position;
uniform mat4 projection_matrix;
uniform mat4 view_matrix;

const float BLADE_HEIGHT = 0.5;
const float BLADE_HEIGHT_RANDOM = 0.9;
const float BLADE_WIDTH = 0.1;
const int BLADE_SEGMENTS = 2;
const int BLADES_PER_TRIANGLE = 32;
const float MAX_OFFSET = 0.2;
const float SPREAD = 5.0;

uniform vec3 WIND_DIRECTION = vec3(1.0, 0.0, 1.0);
uniform float WIND_STRENGTH = 0.1;
uniform float WIND_SPEED = 50.0;
uniform float GRASS_DISTANCE = 50.0;

bool is_visible_and_in_grass_range(vec3 world_position, vec3 view_direction) {
    // Calculate the vector from the camera to the world position
    vec3 to_position = world_position - camera_position;
    // Calculate the distance to the world position
    float d = length(to_position);
    // Normalize the vector to the position
    vec3 to_position_normalized = to_position / d;
    // Calculate the dot product between the view direction and the direction to the position
    float dot_product = dot(view_direction, to_position_normalized);
    // Check if the dot product is positive (camera is facing the position)
    // and if the distance is greater than GRASS_DISTANCE
    return dot_product > 0 && d < GRASS_DISTANCE;
}

float random(vec2 st, float scale) {
    return fract(sin(dot(st.xy, vec2(12.9898,78.233))) * 43758.5453123) * scale;
}

float random_vec(vec3 pos) {
    return fract(sin(dot(pos, vec3(12.9898, 78.233, 45.5432))) * 43758.5453);
}

void emitGrassBlade(vec3 base_pos, vec3 normal, vec3 direction) {
    // Use the surface normal for up direction
    vec3 blade_up = normal;

    vec3 blade_forward = normalize(vec3(cross(direction, blade_up)));
    vec3 blade_right = normalize(cross(blade_up, blade_forward));

    mat4 modelview = view_matrix * v_model_matrix[0];
    float height_random = 1 + random_vec(base_pos) * BLADE_HEIGHT_RANDOM;
    for (int i = 0; i <= BLADE_SEGMENTS; i++) {
        float t = float(i) / float(BLADE_SEGMENTS);
        float height = t * BLADE_HEIGHT * height_random;
        float width = BLADE_WIDTH * (1.0 - t * 0.9); // Taper the width

        // Add some curvature
        float curve = sin(t * 3.14159 * 0.5) * 0.2;

        // Calculate wind effect
        float random_offset = random_vec(base_pos) * 0.05;
        float wind_effect = sin(time * WIND_SPEED + base_pos.x * 0.5 + base_pos.z * 0.5 + random_offset) * WIND_STRENGTH * t;

        vec3 offset = blade_right * curve + WIND_DIRECTION * wind_effect;

        vec3 left_pos = base_pos + blade_up * height - blade_right * width + offset;
        vec3 right_pos = base_pos + blade_up * height + blade_right * width + offset;

        gl_Position = projection_matrix * modelview * vec4(left_pos, 1.0);
        vertex_color = mix(vec3(0.0, 0.5, 0.0), vec3(0.0, 1.0, 0.0), t);
        vertex_texcoord = vec2(0, t);
        vertex_normal = blade_forward;
        world_position = left_pos;
        EmitVertex();

        gl_Position = projection_matrix * modelview * vec4(right_pos, 1.0);
        vertex_color = mix(vec3(0.0, 0.5, 0.0), vec3(0.0, 1.0, 0.0), t);
        vertex_texcoord = vec2(1, t);
        vertex_normal = blade_forward;
        world_position = right_pos;
        EmitVertex();
    }
    EndPrimitive();
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
        EmitVertex();
    }
    EndPrimitive();
    if(is_visible_and_in_grass_range(v_world_position[0], v_view_direction[0])){
        vec3 center = (v_position[0] + v_position[1] + v_position[2]) / 3.0;
        vec3 normal = normalize(v_vertex_normal[0] + v_vertex_normal[1] + v_vertex_normal[2]);
        for (int i = 0; i < BLADES_PER_TRIANGLE; i++) {
            vec2 rand = vec2(random(center.xy + float(i), SPREAD), random(center.yz + float(i), SPREAD));
            vec3 offset = vec3(rand.x, 0, rand.y) * MAX_OFFSET;
            vec3 base_pos = center + offset;
            vec3 world_pos = (v_model_matrix[0] * vec4(base_pos, 1.0)).xyz;
            vec3 blade_direction = normalize(world_pos - camera_position);
            blade_direction = vec3(blade_direction.x, base_pos.y, blade_direction.z);
            emitGrassBlade(base_pos, normal, blade_direction);
        }
    }
}
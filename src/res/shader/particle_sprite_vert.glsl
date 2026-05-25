#version 330 core
uniform mat4 view_matrix;
uniform mat4 projection_matrix;

// per-vertex (unit quad): position in [-0.5, 0.5]^2, uv in [0,1]^2
in vec3 corner;
in vec2 corner_uv;

// per-instance
in vec3 inst_position;
in float inst_size;
in vec4  inst_color;
in vec3  inst_velocity;
in float inst_rotation;
in vec4  inst_flipbook_uv;  // x,y = atlas offset; z,w = atlas frame size

uniform float velocity_stretch;

out vec2 v_uv;
out vec4 v_color;
out vec3 v_view_pos;
out vec4 v_flipbook_uv;

void main() {
    // Camera-aligned billboard from view matrix basis
    vec3 right = vec3(view_matrix[0][0], view_matrix[1][0], view_matrix[2][0]);
    vec3 up    = vec3(view_matrix[0][1], view_matrix[1][1], view_matrix[2][1]);

    // 2D rotation around screen-space normal
    float c = cos(inst_rotation);
    float s = sin(inst_rotation);
    vec2 rotated = vec2(c * corner.x - s * corner.y, s * corner.x + c * corner.y);

    // Velocity stretch (along projected velocity direction)
    vec3 vel_view = (view_matrix * vec4(inst_velocity, 0.0)).xyz;
    vec2 vel_screen = normalize(vec2(vel_view.x, vel_view.y) + vec2(1e-6, 0.0));
    float stretch_factor = 1.0 + velocity_stretch * length(inst_velocity);
    vec2 stretch_dir = vel_screen;
    vec2 perp = vec2(-stretch_dir.y, stretch_dir.x);
    vec2 stretched = stretch_dir * rotated.x * stretch_factor + perp * rotated.y;

    // Choose stretched layout only if velocity_stretch > 0; otherwise plain billboard
    vec2 use_pos = mix(rotated, stretched, step(0.001, velocity_stretch));

    vec3 world_pos = inst_position + right * (use_pos.x * inst_size) + up * (use_pos.y * inst_size);

    vec4 vp = view_matrix * vec4(world_pos, 1.0);
    v_view_pos = vp.xyz;
    v_uv = corner_uv;
    v_color = inst_color;
    v_flipbook_uv = inst_flipbook_uv;
    gl_Position = projection_matrix * vp;
}

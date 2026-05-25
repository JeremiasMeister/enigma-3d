#version 330 core
in vec2 v_uv;
in vec4 v_color;
in vec3 v_view_pos;
in vec4 v_flipbook_uv;

uniform sampler2D albedo;
uniform sampler2D scene_depth;
uniform float soft_fade_distance;  // 0 = disabled
uniform int   use_flipbook;        // 0 or 1
uniform vec2  screen_size;
uniform float near_plane;
uniform float far_plane;

out vec4 out_color;

float linearize_depth(float d) {
    float z = d * 2.0 - 1.0;
    return (2.0 * near_plane * far_plane) / (far_plane + near_plane - z * (far_plane - near_plane));
}

void main() {
    vec2 uv;
    if (use_flipbook == 1) {
        uv = v_flipbook_uv.xy + v_uv * v_flipbook_uv.zw;
    } else {
        uv = v_uv;
    }
    vec4 tex = texture(albedo, uv);
    vec3 color = tex.rgb * v_color.rgb;
    float alpha = tex.a * v_color.a;

    if (soft_fade_distance > 0.001) {
        vec2 screen_uv = gl_FragCoord.xy / screen_size;
        float scene_d  = texture(scene_depth, screen_uv).r;
        float scene_z  = linearize_depth(scene_d);
        float p_z      = -v_view_pos.z;
        float delta    = scene_z - p_z;
        alpha *= smoothstep(0.0, soft_fade_distance, delta);
    }

    if (alpha < 0.001) discard;
    out_color = vec4(color, alpha);
}

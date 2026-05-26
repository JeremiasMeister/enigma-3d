#version 330 core

in vec2 TEXCOORD;
out vec4 color;

uniform sampler2D scene;
uniform sampler2D depth_tex;

// Camera reconstruction (basis vectors + fov half-extents).
uniform vec3 cam_pos;
uniform vec3 cam_forward;
uniform vec3 cam_right;
uniform vec3 cam_up;
uniform float half_w;          // tan(fov/2) * aspect
uniform float half_h;          // tan(fov/2)

// Projection params for depth comparison.
uniform mat4 view_proj;        // projection * view, column-major

// Grid params.
uniform float extent;          // half-extent in world units; lines fade out by here
uniform float major_step;      // e.g. 10 — every Nth line renders as a "major" line
uniform vec4 minor_color;      // straight (non-premultiplied) RGBA
uniform vec4 major_color;
uniform vec4 x_axis_color;     // shown along world z = 0
uniform vec4 z_axis_color;     // shown along world x = 0

// Reconstructs the world point where the view ray for this pixel hits the
// Y = 0 plane. Returns false if the pixel doesn't intersect (above horizon or
// camera on the plane).
bool ray_hits_ground(vec2 ndc, out vec3 P, out float ray_t) {
    vec3 dir = normalize(cam_forward
                       + cam_right * (ndc.x * half_w)
                       + cam_up    * (ndc.y * half_h));
    if (abs(dir.y) < 1e-5) return false;
    float t = -cam_pos.y / dir.y;
    if (t <= 0.0) return false;
    P = cam_pos + dir * t;
    ray_t = t;
    return true;
}

// Anti-aliased line coverage: returns 1 on the line, 0 outside the pixel
// neighborhood, smooth in between. `coord` is the world coordinate to test
// (modular distance to the nearest line of `step`). `pixel_w` is the per-pixel
// world-space width along the relevant axis.
float line_alpha(float coord, float step, float pixel_w) {
    float d = abs(mod(coord + step * 0.5, step) - step * 0.5);
    float half_line = pixel_w * 0.5;     // 1-pixel-wide line
    return 1.0 - smoothstep(half_line, half_line + pixel_w, d);
}

// Same shape, but for a single line at exactly `coord == 0`.
float single_line_alpha(float coord, float pixel_w) {
    float d = abs(coord);
    float half_line = pixel_w * 0.5;
    return 1.0 - smoothstep(half_line, half_line + pixel_w, d);
}

// "Over" composite of a layer onto an accumulator (both straight alpha).
vec4 over(vec4 dst, vec4 src) {
    float a = src.a + dst.a * (1.0 - src.a);
    vec3 rgb = (src.rgb * src.a + dst.rgb * dst.a * (1.0 - src.a)) / max(a, 1e-5);
    return vec4(rgb, a);
}

void main() {
    vec4 scene_col = texture(scene, TEXCOORD);

    vec2 ndc = TEXCOORD * 2.0 - 1.0;

    vec3 P;
    float ray_t;
    if (!ray_hits_ground(ndc, P, ray_t)) {
        color = scene_col;
        return;
    }

    // Ground-distance fade — radial vignette around the camera's XZ.
    float ground_dist = length(P.xz - cam_pos.xz);
    float fade_t = clamp(ground_dist / extent, 0.0, 1.0);
    float dist_fade = 1.0 - fade_t * fade_t;
    if (dist_fade <= 0.0) {
        color = scene_col;
        return;
    }

    // Per-pixel world width along each axis (for anti-aliasing).
    float pw_x = fwidth(P.x);
    float pw_z = fwidth(P.z);

    // Minor grid lines: every 1 world unit, on both X and Z.
    float minor_a = max(line_alpha(P.x, 1.0, pw_x),
                        line_alpha(P.z, 1.0, pw_z));

    // Major grid lines: every `major_step` units.
    float major_a = max(line_alpha(P.x, major_step, pw_x),
                        line_alpha(P.z, major_step, pw_z));

    // World axes.
    float x_axis_a = single_line_alpha(P.z, pw_z); // line where z = 0 → +X axis (red)
    float z_axis_a = single_line_alpha(P.x, pw_x); // line where x = 0 → +Z axis (blue)

    // Layer order: minor at the bottom, then major, then world axes on top.
    vec4 grid = vec4(0.0);
    grid = over(grid, vec4(minor_color.rgb, minor_color.a * minor_a));
    grid = over(grid, vec4(major_color.rgb, major_color.a * major_a));
    grid = over(grid, vec4(x_axis_color.rgb, x_axis_color.a * x_axis_a));
    grid = over(grid, vec4(z_axis_color.rgb, z_axis_color.a * z_axis_a));

    grid.a *= dist_fade;

    if (grid.a <= 0.0) {
        color = scene_col;
        return;
    }

    // Depth occlusion: project P to clip space, convert to window depth, and
    // compare against the scene depth buffer. If the grid plane sits behind
    // already-rendered geometry, hide the grid there.
    vec4 P_clip = view_proj * vec4(P, 1.0);
    float grid_ndc_z = P_clip.z / P_clip.w;
    float grid_depth = grid_ndc_z * 0.5 + 0.5;
    float scene_depth = texture(depth_tex, TEXCOORD).r;
    if (grid_depth > scene_depth + 1e-5) {
        color = scene_col;
        return;
    }

    // Composite (straight-alpha over).
    color = vec4(mix(scene_col.rgb, grid.rgb, grid.a), 1.0);
}

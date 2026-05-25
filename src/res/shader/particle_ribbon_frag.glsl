#version 330 core
in vec2 v_uv;
in vec4 v_color;

uniform sampler2D albedo;
out vec4 out_color;

void main() {
    vec4 tex = texture(albedo, v_uv);
    vec4 c = vec4(tex.rgb * v_color.rgb, tex.a * v_color.a);
    if (c.a < 0.001) discard;
    out_color = c;
}

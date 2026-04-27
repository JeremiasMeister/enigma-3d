#version 330 core

in vec3 v_world_pos;

uniform vec3 light_pos;
uniform float far_plane;

out float frag_depth;

void main() {
    frag_depth = length(v_world_pos - light_pos) / far_plane;
}

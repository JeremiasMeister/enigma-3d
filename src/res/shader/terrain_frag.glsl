#version 330 core

in vec3 v_world_position;
in vec3 v_normal;
in vec3 v_color;

uniform mat4 light_position;
uniform mat4 light_color;
uniform vec4 light_intensity;
uniform int  light_amount;
uniform vec3 ambient_light_color;
uniform float ambient_light_intensity;

out vec4 frag_color;

void main() {
    vec3 normal = normalize(v_normal);
    vec3 diffuse = vec3(0.0);

    for (int i = 0; i < light_amount && i < 4; i++) {
        vec3 lpos  = light_position[i].xyz;
        vec3 lcol  = light_color[i].xyz;
        float lint = light_intensity[i];

        vec3 ldir  = normalize(lpos - v_world_position);
        float ndotl = max(dot(normal, ldir), 0.0);
        // Scale intensity so a typical sun (~2500) gives ~0.75 contribution
        diffuse += lcol * ndotl * clamp(lint / 3000.0, 0.0, 1.0);
    }

    vec3 ambient = ambient_light_color * ambient_light_intensity;
    frag_color = vec4((ambient + diffuse) * v_color, 1.0);
}

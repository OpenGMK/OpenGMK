#version 330 core

struct Light {
    bool enabled;
    bool is_point;
    vec3 pos;
    vec3 colour;
    float range;
};

uniform mat4 model;
uniform mat4 projection;

uniform bool lighting_enabled;
uniform bool gouraud_shading;
uniform bool gm81_normalize;
uniform vec3 ambient_colour;
uniform Light lights[8];
uniform bool light_enabled[8];
uniform bool light_is_point[8];
uniform vec3 light_pos[8];
uniform vec3 light_colour[8];
uniform float light_range[8];

layout (location = 0) in vec3 pos;
layout (location = 1) in vec4 blend;
layout (location = 2) in vec2 tex_coord;
layout (location = 3) in vec3 normal;
layout (location = 4) in vec4 atlas_xywh;

out vec2 frag_tex_coord;
out vec4 frag_atlas_xywh;
out vec4 frag_blend;
flat out vec4 frag_blend_flat;
out float fog_z;

void main() {
    vec4 world_pos = model * vec4(pos, 1.0);
    frag_tex_coord = tex_coord;
    frag_atlas_xywh = atlas_xywh;
    frag_blend = blend;
    frag_blend_flat = vec4(1.0);
    if (lighting_enabled) {
        vec3 light_col = vec3(0.0);
        vec3 new_normal = -(model * vec4(normal, 0.0)).xyz;
        if (gm81_normalize) {
            new_normal = normalize(new_normal);
        }
        for (int i = 0; i < 8; i++) {
            if (lights[i].enabled) {
                vec3 this_light_col = lights[i].colour;
                vec3 ray = lights[i].pos;
                if (lights[i].is_point) {
                    ray = world_pos.xyz - ray;
                    float dist = length(ray);
                    if (dist < lights[i].range) {
                        this_light_col /= 1 + (4 / lights[i].range) * dist;
                    } else {
                        this_light_col = vec3(0.0);
                    }
                }
                light_col += this_light_col * clamp(dot(normalize(ray), new_normal), 0.0, 1.0);
            }
        }
        if (gouraud_shading) {
            frag_blend.rgb *= light_col;
            frag_blend.rgb += ambient_colour;
        } else {
            frag_blend_flat.rgb *= light_col;
            frag_blend_flat.rgb += ambient_colour;
        }
    }
    gl_Position = projection * world_pos;
    fog_z = gl_Position.z;
}

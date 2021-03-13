#version 330 core

struct Light {
    vec4 pos; // padded vec3
    vec4 colour; // padded vec3
    bool enabled;
    bool is_point;
    float range;
};

layout(std140) uniform RenderState {
    // vertex shader
    mat4 model;
    mat4 viewproj;
    Light lights[8];
    vec4 ambient_colour; // padded vec3
    bool lighting_enabled;
    bool gouraud_shading;
    // frag shader
    bool repeat;
    bool lerp; // only used if repeat is on
    bool alpha_test;
    bool fog_enabled;
    float fog_begin;
    float fog_end;
    vec4 fog_colour; // padded vec3
};

uniform bool gm81_normalize; // this will never change so we don't need to include it in the state

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
                vec3 this_light_col = lights[i].colour.rgb;
                vec3 ray = lights[i].pos.xyz;
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
            frag_blend.rgb += ambient_colour.rgb;
        } else {
            frag_blend_flat.rgb *= light_col;
            frag_blend_flat.rgb += ambient_colour.rgb;
        }
    }
    gl_Position = viewproj * world_pos;
    fog_z = gl_Position.z;
}

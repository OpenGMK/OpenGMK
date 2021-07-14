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

uniform sampler2D tex; // can't put this in the state because it's opaque

in vec2 frag_tex_coord;
in vec4 frag_atlas_xywh;
in vec4 frag_blend;
flat in vec4 frag_blend_flat;
in float fog_z;

out vec4 out_colour;

void main() {
    vec2 tex_size = textureSize(tex, 0);
    vec2 sprite_coord;
    vec4 tex_col;
    // get colour from texture
    // keep in mind: the center of a pixel is where its colour is in full; we get the top-left as input
    if (repeat) {
        // get coordinate on sprite but wrapped around (note that fract(x) is equivalent to mod(x, 1.0) and is always positive)
        sprite_coord = fract(frag_tex_coord) * frag_atlas_xywh.zw;
        if (lerp) {
            // get the exact colour of each of the four pixels this coordinate is near, and mix them
            vec2 floor_coord = floor(sprite_coord - 0.5);
            
            vec2 topleft = (frag_atlas_xywh.xy + mod(floor_coord + 0.5, frag_atlas_xywh.zw)) / tex_size;
            vec2 botright = (frag_atlas_xywh.xy + mod(floor_coord + 1.5, frag_atlas_xywh.zw)) / tex_size;
            
            vec4 sampleTL = texture(tex, topleft);
            vec4 sampleTR = texture(tex, vec2(botright.x,topleft.y));
            vec4 sampleBL = texture(tex, vec2(topleft.x,botright.y));
            vec4 sampleBR = texture(tex, botright);
            
            vec2 factor = fract(sprite_coord + 0.5);
            vec4 mix_top = mix(sampleTL, sampleTR, factor.x);
            vec4 mix_bot = mix(sampleBL, sampleBR, factor.x);
            tex_col = mix(mix_top, mix_bot, factor.y);
        } else {
            // we've already done the wrapping, so clamp to center of edge pixels
            sprite_coord = clamp(sprite_coord, vec2(0.5), frag_atlas_xywh.zw - 0.5);
            tex_col = texture(tex, (frag_atlas_xywh.xy + sprite_coord) / tex_size);
        }
    } else {
        // clamp to center of edge pixels
        sprite_coord = clamp(frag_tex_coord * frag_atlas_xywh.zw, vec2(0.5), frag_atlas_xywh.zw - 0.5);
        tex_col = texture(tex, (frag_atlas_xywh.xy + sprite_coord) / tex_size);
    }
    vec4 colour = tex_col * frag_blend * frag_blend_flat;
    // apply fog
    if (fog_enabled) {
        float f = clamp((fog_end - fog_z) / (fog_end - fog_begin), 0, 1);
        colour.rgb = mix(fog_colour.rgb, colour.rgb, f);
    }
    // alpha test
    if (alpha_test && colour.a <= 0) {
        // discarding is bad for performance but blend modes make this a necessity
        discard;
    }
    out_colour = colour;
}

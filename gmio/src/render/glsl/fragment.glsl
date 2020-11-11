#version 330 core

uniform sampler2D tex;
uniform bool repeat;
uniform bool lerp; // only used if repeat is on
uniform bool alpha_test;
uniform bool fog_enabled;
uniform vec3 fog_colour;
uniform float fog_begin;
uniform float fog_end;

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
    if (repeat) {
        // fract(x) is equivalent to mod(x, 1.0) and is always positive
        sprite_coord = fract(frag_tex_coord) * frag_atlas_xywh.zw;
        if (lerp) {
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
            tex_col = texture(tex, (frag_atlas_xywh.xy + sprite_coord) / tex_size);
        }
    } else {
        sprite_coord = clamp(frag_tex_coord * frag_atlas_xywh.zw, vec2(0.5), frag_atlas_xywh.zw - 0.5);
        tex_col = texture(tex, (frag_atlas_xywh.xy + sprite_coord) / tex_size);
    }
    vec4 colour = tex_col * frag_blend * frag_blend_flat;
    // apply fog
    if (fog_enabled) {
        float f = clamp((fog_end - fog_z) / (fog_end - fog_begin), 0, 1);
        colour.rgb = (1-f) * fog_colour + f * colour.rgb;
    }
    // alpha test
    if (alpha_test && colour.a <= 0) {
        // discarding is bad for performance but blend modes make this a necessity
        discard;
    }
    out_colour = colour;
}

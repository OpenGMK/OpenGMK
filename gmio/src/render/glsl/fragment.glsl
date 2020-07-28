#version 330 core

uniform sampler2D tex;
uniform bool repeat;

in vec2 frag_tex_coord;
in vec4 frag_atlas_xywh;
in vec4 frag_blend;

out vec4 colour;

void main() {
    vec2 tex_size = textureSize(tex, 0);
    vec2 sprite_coord;
    if (repeat) {
        // TODO: lerp correctly on repeat
        sprite_coord = vec2(
            mod(frag_tex_coord.x, 1.0) * frag_atlas_xywh.z,
            mod(frag_tex_coord.y, 1.0) * frag_atlas_xywh.w
        );
    } else {
        sprite_coord = vec2(
            clamp(frag_tex_coord.x * frag_atlas_xywh.z, 0.0, frag_atlas_xywh.z - 1),
            clamp(frag_tex_coord.y * frag_atlas_xywh.w, 0.0, frag_atlas_xywh.w - 1)
        );
    }
    vec4 tex_col = texture(tex, vec2(
        (frag_atlas_xywh.x + sprite_coord.x) / tex_size.x,
        (frag_atlas_xywh.y + sprite_coord.y) / tex_size.y
    ));
    colour = tex_col * frag_blend;
}

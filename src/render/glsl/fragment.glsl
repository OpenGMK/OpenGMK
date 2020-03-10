#version 330 core

uniform sampler2D tex;

in vec2 frag_tex_coord;
in vec4 frag_atlas_xywh;
out vec4 colour;

void main() {
    vec2 tex_size = textureSize(tex, 0);
    colour = texture(tex, vec2(
        (frag_atlas_xywh.x + (frag_atlas_xywh.z * frag_tex_coord.x)) / tex_size.x,
        (frag_atlas_xywh.y + (frag_atlas_xywh.w * frag_tex_coord.y)) / tex_size.y
    ));
}

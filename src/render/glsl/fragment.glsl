#version 330 core
#extension GL_ARB_explicit_uniform_location : require

layout(location = 1) uniform sampler2D tex;

in vec2 frag_tex_coord;
in vec4 frag_atlas_xywh;
in vec3 frag_blend;
in float frag_alpha;

out vec4 colour;

void main() {
    vec2 tex_size = textureSize(tex, 0);
    vec4 tex_col = texture(tex, vec2(
        (frag_atlas_xywh.x + (frag_atlas_xywh.z * frag_tex_coord.x)) / tex_size.x,
        (frag_atlas_xywh.y + (frag_atlas_xywh.w * frag_tex_coord.y)) / tex_size.y
    ));
    colour = vec4(tex_col.x * frag_blend.x, tex_col.y * frag_blend.y, tex_col.z * frag_blend.z, tex_col.w * frag_alpha);
}

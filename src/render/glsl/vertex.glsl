#version 330 core
#extension GL_ARB_explicit_uniform_location : require

layout(location = 0) uniform mat4 projection;

layout (location = 0) in vec3 pos;
layout (location = 1) in mat4 model_view;
layout (location = 5) in vec2 tex_coord;
layout (location = 6) in vec4 atlas_xywh;
layout (location = 7) in vec3 blend;
layout (location = 8) in float alpha;

out vec2 frag_tex_coord;
out vec4 frag_atlas_xywh;
out vec3 frag_blend;
out float frag_alpha;

void main() {
    frag_tex_coord = tex_coord;
    frag_atlas_xywh = atlas_xywh;
    frag_blend = blend;
    frag_alpha = alpha;
    gl_Position = projection * model_view * vec4(pos.x, pos.y, pos.z, 1.0);
}

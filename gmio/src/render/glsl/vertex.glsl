#version 330 core

uniform mat4 projection;

layout (location = 0) in vec3 pos;
layout (location = 1) in vec4 blend;
layout (location = 2) in vec2 tex_coord;
layout (location = 3) in vec3 normal;
layout (location = 4) in vec4 atlas_xywh;

out vec2 frag_tex_coord;
out vec4 frag_atlas_xywh;
out vec4 frag_blend;

void main() {
    frag_tex_coord = tex_coord;
    frag_atlas_xywh = atlas_xywh;
    frag_blend = blend;
    gl_Position = projection * vec4(pos, 1.0);
}

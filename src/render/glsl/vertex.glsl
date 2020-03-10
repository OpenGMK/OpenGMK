#version 330 core

layout (location = 0) in vec3 pos;
in mat4 model_view;
in vec2 tex_coord;
in vec4 atlas_xywh;
out vec2 frag_tex_coord;
out vec4 frag_atlas_xywh;

uniform mat4 projection;

void main() {
    frag_tex_coord = tex_coord;
    frag_atlas_xywh = atlas_xywh;
    gl_Position = projection * model_view * vec4(pos.x, pos.y, pos.z, 1.0);
}

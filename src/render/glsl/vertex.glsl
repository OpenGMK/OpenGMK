#version 330 core

layout (location = 0) in vec3 pos;
in mat4 model_view;
in vec2 tex_coord;
out vec2 frag_tex_coord;

uniform mat4 projection;

void main() {
    frag_tex_coord = tex_coord;
    gl_Position = projection * model_view * vec4(pos.x, pos.y, pos.z, 1.0);
}

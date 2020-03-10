#version 330 core

uniform sampler2D tex;

in vec2 frag_tex_coord;
out vec4 colour;

void main() {
    colour = texture(tex, frag_tex_coord);
}

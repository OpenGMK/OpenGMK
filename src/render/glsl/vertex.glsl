#version 330 core

layout (location = 0) in vec3 pos;
in mat4 model_view;

uniform mat4 projection;

void main() {
    gl_Position = projection * model_view * vec4(pos.x, pos.y, pos.z, 1.0);
}

#version 330 core

layout (location = 0) in vec3 aPos;
in mat4 model_view;

uniform mat4 projection;

void main() {
    gl_Position = projection * model_view * vec4(aPos.x, aPos.y, aPos.z, 1.0);
}

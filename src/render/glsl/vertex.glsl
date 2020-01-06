#version 330 core

layout (location = 0) in vec3 aPos;
in mat4 project;

void main() {
    gl_Position = project * vec4(aPos.x, aPos.y, aPos.z, 1.0);
}

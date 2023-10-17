#version 330
in vec3 position;
in vec3 normal;
in vec2 texcoord;

out vec3 outColor;

uniform vec3 color;
uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

void main() {
    gl_Position = projection * view * model * vec4(position, 1.0);
    outColor = color;
}

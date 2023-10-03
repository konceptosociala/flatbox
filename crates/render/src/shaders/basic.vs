#version 330
in vec3 position;
in vec2 coordinate;

out vec2 texCoord;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

void main() {
    gl_Position = projection * view * model * vec4(position, 1.0);
    texCoord = coordinate;
}

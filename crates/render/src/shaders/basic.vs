#version 330
in vec3 position;
in vec3 normal;
in vec2 texcoord;

out vec2 textureCoordinate;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

void main() {
    gl_Position = projection * view * model * vec4(position, 1.0);
    textureCoordinate = texcoord;
}

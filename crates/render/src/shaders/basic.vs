#version 330
in vec2 position;
in vec2 coordinate;

out vec2 texCoord;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    texCoord = coordinate;
}

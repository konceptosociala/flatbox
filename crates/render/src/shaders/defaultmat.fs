#version 330
out vec4 FragColor;

in vec2 textureCoordinate;
in vec3 outColor;

void main() {
    FragColor = vec4(outColor, 1.0);
}

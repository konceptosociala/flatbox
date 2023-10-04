#version 330
out vec4 FragColor;

in vec2 textureCoordinate;

uniform sampler2D rustTexture;
uniform sampler2D wallTexture;

void main() {
    vec4 color0 = texture(rustTexture, textureCoordinate);
    vec4 color1 = texture(wallTexture, textureCoordinate);
    FragColor = color1;
}

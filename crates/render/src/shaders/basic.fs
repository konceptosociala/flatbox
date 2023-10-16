#version 330
out vec4 FragColor;

in vec2 textureCoordinate;

struct DefaultMaterial {
    sampler2D rustTexture;
    sampler2D wallTexture;
};

uniform DefaultMaterial material;

void main() {
    vec4 color0 = texture(material.rustTexture, textureCoordinate);
    vec4 color1 = texture(material.wallTexture, textureCoordinate);
    FragColor = color1;
}

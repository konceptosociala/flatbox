#version 330
out vec4 FragColor;

in vec2 texCoord;

uniform sampler2D rustTexture;
uniform sampler2D wallTexture;

void main() {
    vec4 color0 = texture(rustTexture, texCoord);
    vec4 color1 = texture(wallTexture, texCoord);
    FragColor = mix(color0, color1, 0.6) * color0.a;
}

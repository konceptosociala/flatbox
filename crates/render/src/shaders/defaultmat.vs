#version 330
in vec3 position;
in vec3 normal;
in vec2 texcoord;

out vec3 FragPos;
out vec3 Normal;
out vec2 TexCoord;

uniform mat4 model;
uniform mat4 inversed;
uniform mat4 view;
uniform mat4 projection;

void main() {
    FragPos = vec3(model * vec4(position, 1.0));
    Normal = mat3(transpose(inversed)) * normal;
    TexCoord = texcoord;
    
    gl_Position = projection * view * vec4(FragPos, 1.0);
}

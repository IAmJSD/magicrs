#version 330 core
in vec2 TexCoords;
out vec4 FragColor;
layout(position = 1) uniform sampler2D TextureSampler;
layout(position = 2) uniform float PixelSize;

void main() {
    vec2 texCoords = TexCoords * PixelSize;
    FragColor = texture(TextureSampler, texCoords);
}

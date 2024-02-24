#version 330 core
in vec2 TexCoords;
out vec4 FragColor;
layout(position = 1) uniform sampler2D TextureSampler;
layout(position = 2) uniform float PixelSize;

void main() {
    vec2 texelSize = 1.0 / textureSize(TextureSampler, 0);
    vec2 roundedTexCoords = TexCoords - mod(TexCoords, vec2(texelSize.x * PixelSize, texelSize.y * PixelSize)) + vec2(texelSize.x * PixelSize * 0.5, texelSize.y * PixelSize * 0.5);
    vec4 color = texture(textureSampler, roundedTexCoords);
    FragColor = color;
}

#version 330 core
in vec2 TexCoords;
out vec4 FragColor;
layout(position = 1) uniform sampler2D TextureSampler;
layout(position = 2) uniform float PixelSize;

void main() {
    // Calculate the size of each texel in texture coordinates
    vec2 texelSize = 1.0 / textureSize(TextureSampler, 0);

    // Calculate the coordinate of the center of the pixel block
    vec2 roundedTexCoords = TexCoords - mod(TexCoords, vec2(texelSize.x * PixelSize, texelSize.y * PixelSize)) + vec2(texelSize.x * PixelSize * 0.5, texelSize.y * PixelSize * 0.5);

    // Sample the color at the rounded coordinate
    vec4 color = texture(textureSampler, roundedTexCoords);

    // Output the pixelated color
    FragColor = color;
}

// Generic setup to all MagicCap editor shaders.
#version 330 core
in vec2 TexCoords;
out vec4 FragColor;
layout(position = 1) uniform sampler2D TextureSampler;
layout(position = 2) uniform float PixelSize;

// The entrypoint of the shader.
void main() {
    // The pixel size is used to calculate the texture coordinates.
    vec2 texCoords = TexCoords * PixelSize;

    // The texture is sampled and the color is output.
    FragColor = texture(TextureSampler, texCoords);
}

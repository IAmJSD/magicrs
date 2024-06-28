use image::RgbaImage;
use once_cell::sync::Lazy;
use super::gl_abstractions::GLTexture;

// Generates the texture pack.
fn generate_texture(dark: bool) -> RgbaImage {
    // TODO
    RgbaImage::new(1, 1)
}

// Defines the dark texture lazy container.
const DARK_TEXTURE: Lazy<RgbaImage> = Lazy::new(|| generate_texture(true));

// Defines the light texture lazy container.
const LIGHT_TEXTURE: Lazy<RgbaImage> = Lazy::new(|| generate_texture(false));

// Preloads the dark and light textures.
pub fn preload_textures() {
    let _ = &*DARK_TEXTURE;
    let _ = &*LIGHT_TEXTURE;
}

// Defines the texture pack for the region selector.
pub struct TexturePack {
    texture: GLTexture,
}

impl TexturePack {
    // Creates a new texture pack.
    pub fn new() -> Self {
        // Get a reference to the relevant texture.
        let texture = if dark_light::detect() == dark_light::Mode::Dark {
            &*DARK_TEXTURE
        } else {
            &*LIGHT_TEXTURE
        };

        // Create the texture pack.
        Self {
            texture: GLTexture::from_rgba(texture),
        }
    }
}

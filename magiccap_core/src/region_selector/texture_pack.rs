use image::RgbaImage;
use once_cell::sync::Lazy;
use super::{editors, gl_abstractions::GLTexture};

// Load Roboto from the frontend public folder.
static ROBOTO_REGULAR: &[u8] = include_bytes!("../../../frontend/public/Roboto-Regular.ttf");

// Defines the loaded font.
static LOADED_FONT: Lazy<rusttype::Font<'static>> = Lazy::new(|| {
    rusttype::Font::try_from_bytes(ROBOTO_REGULAR).unwrap()
});

// Defines the charset.
const CHARSET: &str = " ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()_+-=[]{}\\|;:'\",.<>/?`~";

// Generates the texture pack.
fn generate_texture(dark: bool) -> RgbaImage {
    // Figure out the width and height of the charset.
    let charset_width = CHARSET.chars().map(|c| {
        let glyph = LOADED_FONT.glyph(c).scaled(rusttype::Scale::uniform(24.0));
        glyph.h_metrics().advance_width
    }).sum::<f32>().ceil() as u32;
    let charset_height = 28;

    // Get the icon bytes and map them to RGBA.
    let icons = editors::create_editor_icons().iter().map(|icon| {
        image::load_from_memory(icon).unwrap().to_rgba8()
    }).collect::<Vec<_>>();

    // Calculate the width and height of the texture.
    let icons_height = icons.iter().map(|icon| icon.height()).max().unwrap();
    let height = icons_height.max(charset_height);
    let width = icons.iter().map(|icon| icon.width()).sum::<u32>() + charset_width;

    // Create the texture.
    // TODO
    RgbaImage::new(width, height)
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

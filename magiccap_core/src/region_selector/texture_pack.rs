use std::collections::HashMap;
use image::{GenericImage, Rgba, RgbaImage};
use once_cell::sync::Lazy;
use rusttype::{point, Scale};
use super::{editors, gl_abstractions::GLTexture};

// Load Roboto from the frontend public folder.
static ROBOTO_REGULAR: &[u8] = include_bytes!("../../../frontend/public/Roboto-Regular.ttf");

// Defines the dark textures.
static BLACK_HOVER: &[u8] = include_bytes!("textures/black_hover.png");
static BLACK_NO_HOVER: &[u8] = include_bytes!("textures/black_no_hover.png");

// Defines the light textures.
static WHITE_HOVER: &[u8] = include_bytes!("textures/white_hover.png");
static WHITE_NO_HOVER: &[u8] = include_bytes!("textures/white_no_hover.png");

// Defines the highlight texture.
static HIGHLIGHTED: &[u8] = include_bytes!("textures/highlighted.png");

// Defines the loaded font.
static LOADED_FONT: Lazy<rusttype::Font<'static>> = Lazy::new(|| {
    rusttype::Font::try_from_bytes(ROBOTO_REGULAR).unwrap()
});

// Defines the charset.
const CHARSET: &str = "? ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()_+-=[]{}\\|;:'\",.<>/`~";

// Generates the static textures.
fn generate_static_texture(dark: bool) -> RgbaImage {
    // Get the icon bytes and map them to RGBA.
    let icons = editors::create_editor_icons().iter().map(|icon| {
        image::load_from_memory(icon).unwrap().to_rgba8()
    }).collect::<Vec<_>>();

    // Get the menu textures.
    let menu_textures = if dark {
        vec![
            image::load_from_memory(BLACK_HOVER).unwrap().to_rgba8(),
            image::load_from_memory(BLACK_NO_HOVER).unwrap().to_rgba8(),
            image::load_from_memory(HIGHLIGHTED).unwrap().to_rgba8(),
        ]
    } else {
        vec![
            image::load_from_memory(WHITE_HOVER).unwrap().to_rgba8(),
            image::load_from_memory(WHITE_NO_HOVER).unwrap().to_rgba8(),
            image::load_from_memory(HIGHLIGHTED).unwrap().to_rgba8(),
        ]
    };

    // Calculate the width and height of the texture.
    let icons_height = icons.iter().map(|icon| icon.height()).max().unwrap();
    let menu_height = menu_textures.iter().map(|texture| texture.height()).max().unwrap();
    let height = icons_height.max(menu_height);
    let icons_width = icons.iter().map(|icon| icon.width()).sum::<u32>();
    let menu_width = menu_textures.iter().map(|texture| texture.width()).sum::<u32>();
    let width = icons_width + menu_width;

    // Create the texture.
    let mut rgba = RgbaImage::new(width, height);

    // Draw each menu item next to each other.
    let mut x = 0;
    for texture in &menu_textures {
        rgba.copy_from(texture, x, 0).unwrap();
        x += texture.width();
    }

    // Draw each icon next to each other.
    for icon in &icons {
        rgba.copy_from(icon, x, 0).unwrap();
        x += icon.width();
    }

    // Return the texture.
    rgba
}

// Defines information about the charset texture.
struct CharsetTexture {
    image: RgbaImage,
    x_offsets: HashMap<u8, (i32, i32)>,
}

// Generates the charset.
fn generate_charset(dark: bool) -> CharsetTexture {
    // Figure out the width and height of the charset.
    let charset_width = CHARSET.chars().map(|c| {
        let glyph = LOADED_FONT.glyph(c).scaled(rusttype::Scale::uniform(24.0));
        glyph.h_metrics().advance_width
    }).sum::<f32>().ceil() as u32;
    let charset_height = 28;

    // Create the charset with a background that is either black or white.
    let mut charset = RgbaImage::new(charset_width, charset_height);
    if dark {
        for pixel in charset.pixels_mut() {
            *pixel = Rgba([0, 0, 0, 255]);
        }
    } else {
        for pixel in charset.pixels_mut() {
            *pixel = Rgba([255, 255, 255, 255]);
        }
    }

    // Preallocate a vector for the x offsets.
    let mut x_offsets = HashMap::with_capacity(CHARSET.len());

    // Prepare the font drawing scale and position.
    let scale = Scale::uniform(24.0);
    let v_metrics = LOADED_FONT.v_metrics(scale);
    let offset_y = v_metrics.ascent;

    // Draw each character in the charset.
    let mut x_offset = 0.0;
    for c in CHARSET.bytes() {
        let glyph: rusttype::PositionedGlyph = LOADED_FONT.glyph(c as char).scaled(scale).positioned(point(x_offset, offset_y));

        // Draw the glyph into the image.
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            glyph.draw(|x, y, v| {
                let x = x as i32 + bounding_box.min.x;
                let y = y as i32 + bounding_box.min.y;
                if x >= 0 && x < charset_width as i32 && y >= 0 && y < charset_height as i32 {
                    let x = x as u32;
                    let y = y as u32;
                    if v.round() != 0.0 {
                        let color = if dark {
                            [255, 255, 255, 255] // White font on dark background
                        } else {
                            [0, 0, 0, 255] // Black font on light background
                        };
                        charset.put_pixel(x, y, Rgba(color));
                    }
                }
            });
        }

        // Store the x offset.
        let aw = glyph.unpositioned().h_metrics().advance_width;
        x_offsets.insert(c, (x_offset as i32, aw as i32));

        // Advance the x_offset for the next glyph.
        x_offset += aw;
    }

    // Return the charset and offsets.
    CharsetTexture {
        image: charset,
        x_offsets,
    }
}

// Defines the dark texture lazy container.
const DARK_TEXTURE: Lazy<(RgbaImage, CharsetTexture)> = Lazy::new(|| (generate_static_texture(true), generate_charset(true)));

// Defines the light texture lazy container.
const LIGHT_TEXTURE: Lazy<(RgbaImage, CharsetTexture)> = Lazy::new(|| (generate_static_texture(false), generate_charset(false)));

// Preloads the dark and light textures.
pub fn preload_textures() {
    let _ = &*DARK_TEXTURE;
    let _ = &*LIGHT_TEXTURE;
}

// Defines the texture pack for the region selector.
pub struct TexturePack {
    static_texture: GLTexture,
    charset_texture: GLTexture,
    charset_offsets: HashMap<u8, (i32, i32)>,
}

impl TexturePack {
    // Creates a new texture pack.
    pub fn new() -> Self {
        // Get a reference to the relevant texture.
        let (static_texture, charset_texture) = if dark_light::detect() == dark_light::Mode::Dark {
            &*DARK_TEXTURE
        } else {
            &*LIGHT_TEXTURE
        };

        // Create the texture pack.
        Self {
            static_texture: GLTexture::from_rgba(static_texture),
            charset_texture: GLTexture::from_rgba(&charset_texture.image),
            charset_offsets: charset_texture.x_offsets.clone(),
        }
    }

    // Get the length of the text specified.
    pub fn text_length(&self, text: &str) -> i32 {
        text.bytes().map(|c| {
            self.charset_offsets.get(&c).unwrap_or(
                self.charset_offsets.get(&('?' as u8)).unwrap()
            ).1
        }).sum()
    }

    // Write text at a position. Marked as unsafe due to OpenGL usage.
    pub unsafe fn write_text(&self, text: &str, mut rel_x: i32, rel_y: i32, screen_height: i32) {
        // Load content into the framebuffer.
        gl::FramebufferTexture2D(
            gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D, self.charset_texture.texture, 0
        );

        // Handle writing to the screen.
        macro_rules! render_char {
            ($char:expr) => {
                // Get the character X offset/width in the character map.
                let (offset, char_width) = self.charset_offsets.get($char).unwrap_or(
                    // Fallback to the question mark if the character is not found.
                    self.charset_offsets.get(&('?' as u8)).unwrap()
                ).clone();

                // Blit the framebuffer to the screen.
                gl::BlitFramebuffer(
                    offset, 0, offset + char_width, 28,
                    rel_x, screen_height - rel_y, rel_x + char_width as i32, screen_height - rel_y - 28,
                    gl::COLOR_BUFFER_BIT, gl::NEAREST,
                );

                // Add the character width to the relative X position.
                rel_x += char_width as i32;
            };
        }
        render_char!(&(' ' as u8));
        for c in text.bytes() {
            render_char!(&c);
        }
        render_char!(&(' ' as u8));

        // Stop Rust whining about rel_x being unused.
        let _ = rel_x;
    }
}

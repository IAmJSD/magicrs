use image::RgbaImage;
use crate::region_selector::gl_abstractions::GLTexture;
use super::{Editor, EditorFactory, EditorRegion};

// The divisor for the pixelate effect.
const PIXELATE_DIVISOR: usize = 20;

// Defines the pixelate editor.
struct Pixelate {}
impl Editor for Pixelate {
    fn click(&mut self, _: i32, _: i32) -> Option<EditorRegion> { None }

    fn render(
        &self, screenshot: &GLTexture, _: u32, screen_h: u32,
        texture_w: u32, texture_h: u32, texture_x: i32, texture_y: i32,
    ) {
        // Load in the chunk of the texture that we want to pixelate.
        let mut pixels = vec![0; (texture_w * texture_h * 4) as usize];
        unsafe {
            let texture = screenshot.texture;
            gl::FramebufferTexture2D(
                gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D, texture, 0
            );
            gl::ReadPixels(
                texture_x, texture_y, texture_w as i32, texture_h as i32,
                gl::RGBA, gl::UNSIGNED_BYTE, pixels.as_mut_ptr() as *mut _
            );
        }

        // Iterate through the image data.
        let mut image = RgbaImage::from_raw(
            texture_w, texture_h, pixels).unwrap();
        let mut pixel = *image.get_pixel(0, 0);
        let pixel_iter = image.pixels_mut();
        for (i, pixel_ptr) in pixel_iter.enumerate() {
            if i % PIXELATE_DIVISOR == 0 {
                pixel = *pixel_ptr;
            }
            *pixel_ptr = pixel;
        }

        // Make a new texture from the pixelated image.
        let texture = GLTexture::from_rgba(&image);

        // Bind the pixelated texture.
        unsafe {
            gl::FramebufferTexture2D(
                gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D, texture.texture, 0
            );
        };

        // Blit the pixelated image.
        unsafe {
            gl::BlitFramebuffer(
                0, 0, texture_w as i32, texture_h as i32,
                texture_x, screen_h as i32 - texture_y, texture_x + texture_w as i32,
                (screen_h as i32 - (texture_y + texture_h as i32)) as i32,
                gl::COLOR_BUFFER_BIT, gl::NEAREST
            );
        }

        // Free the pixelated texture.
        drop(texture);
    }
}

// Defines the factory for the pixelate editor.
pub struct PixelateFactory {}
impl EditorFactory for PixelateFactory {
    fn new() -> Self {
        PixelateFactory {}
    }

    fn description(&self) -> &'static str {
        "Pixelates the region specified."
    }

    fn create_editor(&mut self) -> Box<dyn Editor> {
        Box::new(Pixelate {})
    }
}

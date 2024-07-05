use image::{imageops, RgbaImage};
use crate::region_selector::gl_abstractions::GLTexture;
use super::{Editor, EditorFactory, EditorRegion};

// Defines the blur editor.
struct Blur {
    cache: Option<(u32, u32, i32, i32, GLTexture)>,
}
impl Editor for Blur {
    fn click(&mut self, _: i32, _: i32) -> Option<EditorRegion> { None }

    fn render(
        &mut self, screenshot: &GLTexture, _: u32, screen_h: u32,
        texture_w: u32, texture_h: u32, texture_x: i32, texture_y: i32,
    ) {
        // Handle if the cache is a hit.
        if let Some((a, b, c, d, gl_tex)) = &self.cache {
            if *a == texture_w && *b == texture_h && *c == texture_x && *d == texture_y {
                // Since this was a hit, we can just blit the texture and then return.
                unsafe {
                    gl::FramebufferTexture2D(
                        gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0,
                        gl::TEXTURE_2D, gl_tex.texture, 0
                    );
                    gl::BlitFramebuffer(
                        0, 0, texture_w as i32, texture_h as i32,
                        texture_x, screen_h as i32 - texture_y, texture_x + texture_w as i32,
                        (screen_h as i32 - (texture_y + texture_h as i32)) as i32,
                        gl::COLOR_BUFFER_BIT, gl::NEAREST
                    );
                }
                return;
            }
        }

        // Load in the chunk of the texture that we want to blur.
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

        // Get as a RGBA image.
        let image = RgbaImage::from_raw(
            texture_w, texture_h, pixels).unwrap();

        // Blur the underlying image.
        let image = imageops::blur(&image, 10.0);

        // Make a new texture from the blurred image.
        let texture = GLTexture::from_rgba(&image);

        // Bind the blurred texture.
        unsafe {
            gl::FramebufferTexture2D(
                gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D, texture.texture, 0
            );
        };

        // Blit the blurred image.
        unsafe {
            gl::BlitFramebuffer(
                0, 0, texture_w as i32, texture_h as i32,
                texture_x, screen_h as i32 - texture_y, texture_x + texture_w as i32,
                (screen_h as i32 - (texture_y + texture_h as i32)) as i32,
                gl::COLOR_BUFFER_BIT, gl::NEAREST
            );
        }

        // Save the cache.
        self.cache = Some((texture_w, texture_h, texture_x, texture_y, texture));
    }
}

// Defines the factory for the blur editor.
pub struct BlurFactory {}
impl EditorFactory for BlurFactory {
    fn new() -> Self {
        BlurFactory {}
    }

    fn description(&self) -> &'static str {
        "Blurs the region specified."
    }

    fn create_editor(&mut self) -> Box<dyn Editor> {
        Box::new(Blur {cache: None})
    }
}

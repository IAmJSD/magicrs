use super::{Editor, EditorFactory, EditorRegion};
use crate::region_selector::{engine::RegionSelectorContext, gl_abstractions::GLTexture};
use image::RgbaImage;

// Defines the pixelate editor.
struct Pixelate {
    cache: Option<(u32, u32, i32, i32, GLTexture)>,
}
impl Editor for Pixelate {
    fn click(&mut self, _: i32, _: i32) -> Option<Option<EditorRegion>> {
        None
    }

    fn render(
        &mut self,
        screenshot: &GLTexture,
        _: u32,
        screen_h: u32,
        texture_w: u32,
        texture_h: u32,
        texture_x: i32,
        texture_y: i32,
    ) {
        // Handle if the cache is a hit.
        if let Some((a, b, c, d, gl_tex)) = &self.cache {
            if *a == texture_w && *b == texture_h && *c == texture_x && *d == texture_y {
                // Since this was a hit, we can just blit the texture and then return.
                unsafe {
                    gl::FramebufferTexture2D(
                        gl::READ_FRAMEBUFFER,
                        gl::COLOR_ATTACHMENT0,
                        gl::TEXTURE_2D,
                        gl_tex.texture,
                        0,
                    );
                    gl::BlitFramebuffer(
                        0,
                        0,
                        texture_w as i32,
                        texture_h as i32,
                        texture_x,
                        screen_h as i32 - texture_y,
                        texture_x + texture_w as i32,
                        (screen_h as i32 - (texture_y + texture_h as i32)) as i32,
                        gl::COLOR_BUFFER_BIT,
                        gl::NEAREST,
                    );
                }
                return;
            }
        }

        // Load in the chunk of the texture that we want to pixelate.
        let mut pixels = vec![0; (texture_w * texture_h * 4) as usize];
        unsafe {
            let texture = screenshot.texture;
            gl::FramebufferTexture2D(
                gl::READ_FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                texture,
                0,
            );
            gl::ReadPixels(
                texture_x,
                texture_y,
                texture_w as i32,
                texture_h as i32,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                pixels.as_mut_ptr() as *mut _,
            );
        }

        // Get as a RGBA image.
        let mut image = RgbaImage::from_raw(texture_w, texture_h, pixels).unwrap();

        // Pixelate the underlying image.
        let pixelation_size = 20;
        for y in (0..texture_h).step_by(pixelation_size) {
            for x in (0..texture_w).step_by(pixelation_size) {
                let mut r_total = 0u32;
                let mut g_total = 0u32;
                let mut b_total = 0u32;
                let mut a_total = 0u32;
                let mut count = 0;

                for yy in y..(y + pixelation_size as u32).min(texture_h) {
                    for xx in x..(x + pixelation_size as u32).min(texture_w) {
                        let pixel = image.get_pixel(xx, yy).0;
                        r_total += pixel[0] as u32;
                        g_total += pixel[1] as u32;
                        b_total += pixel[2] as u32;
                        a_total += pixel[3] as u32;
                        count += 1;
                    }
                }

                let avg_pixel = image::Rgba([
                    (r_total / count) as u8,
                    (g_total / count) as u8,
                    (b_total / count) as u8,
                    (a_total / count) as u8,
                ]);

                for yy in y..(y + pixelation_size as u32).min(texture_h) {
                    for xx in x..(x + pixelation_size as u32).min(texture_w) {
                        image.put_pixel(xx, yy, avg_pixel);
                    }
                }
            }
        }

        // Make a new texture from the pixelated image.
        let texture = GLTexture::from_rgba(&image);

        // Bind the pixelated texture.
        unsafe {
            gl::FramebufferTexture2D(
                gl::READ_FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                texture.texture,
                0,
            );
        };

        // Blit the pixelated image.
        unsafe {
            gl::BlitFramebuffer(
                0,
                0,
                texture_w as i32,
                texture_h as i32,
                texture_x,
                screen_h as i32 - texture_y,
                texture_x + texture_w as i32,
                (screen_h as i32 - (texture_y + texture_h as i32)) as i32,
                gl::COLOR_BUFFER_BIT,
                gl::NEAREST,
            );
        }

        // Save the cache.
        self.cache = Some((texture_w, texture_h, texture_x, texture_y, texture));
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

    fn create_editor(&mut self, _: &mut RegionSelectorContext) -> Box<dyn Editor> {
        Box::new(Pixelate { cache: None })
    }
}

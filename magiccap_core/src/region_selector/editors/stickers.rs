use crate::region_selector::{engine::RegionSelectorContext, gl_abstractions::GLTexture};
use super::{Editor, EditorRegion, EditorFactory};

// Defines the sticker structure.
struct Sticker {
    texture: Option<GLTexture>,
}

// Implement the sticker loading functionality.
impl Sticker {
    fn load_sticker(&mut self) -> Option<EditorRegion> {
        // Get the file path.
        let res = native_dialog::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "bmp", "gif", "tiff", "webp"])
            .show_open_single_file()
            .unwrap();

        // Open as a image.
        let img = match res {
            Some(path) => match image::open(path) {
                Ok(img) => img,
                Err(_) => return None,
            },
            None => return None,
        };

        // Get the width and height of the image.
        let (w, h) = (img.width(), img.height());

        // Convert to a RGBA image.
        let rgba_img = img.to_rgba8();

        // Create a texture from the image.
        self.texture = Some(GLTexture::from_rgba(&rgba_img));

        // Get the width and height but scaled to under 720p.
        let (w, h) = if w > 1280 || h > 720 {
            let scale = 720.0 / h as f32;
            (1280, (h as f32 * scale) as u32)
        } else {
            (w, h)
        };

        // Return the region.
        Some(EditorRegion {width: w, height: h})
    }
}

// Implement the editor handlers.
impl Editor for Sticker {
    fn click(&mut self, _: i32, _: i32) -> Option<Option<EditorRegion>> {
        Some(self.load_sticker())
    }

    fn render(
        &mut self, _: &GLTexture, _: u32, window_h: u32,
        texture_w: u32, texture_h: u32, texture_x: i32, texture_y: i32,
    ) {
        let texture_id = match &self.texture {
            Some(t) => t.texture,
            None => return,
        };
        let y1 = window_h as i32 - texture_y - texture_h as i32;
        let y0 = y1 + texture_h as i32;
        unsafe {
            gl::FramebufferTexture2D(
                gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D, texture_id, 0
            );
            gl::BlitFramebuffer(
                0, 0, texture_w as i32, texture_h as i32,
                texture_x, y0, texture_x + texture_w as i32, y1,
                gl::COLOR_BUFFER_BIT, gl::NEAREST,
            )
        }
    }
}

// Defines the sticker factory.
pub struct StickerFactory {}
impl EditorFactory for StickerFactory {
    fn new() -> Self {
        StickerFactory {}
    }

    fn description(&self) -> &'static str {
        "Puts a sticker on the screen."
    }

    fn create_editor(&mut self, _: &mut RegionSelectorContext) -> Box<dyn Editor> {
        Box::new(Sticker {texture: None})
    }
}

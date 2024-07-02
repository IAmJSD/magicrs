use crate::region_selector::gl_abstractions::{
    copy_texture, GLShaderProgram, GLTexture,
};
use super::{Editor, EditorFactory, EditorRegion};

// Defines the pixelate editor.
struct Pixelate {}
impl Editor for Pixelate {
    fn click(&mut self, _: i32, _: i32) -> Option<EditorRegion> { None }

    fn render(
        &self, screenshot: &GLTexture, window_w: u32, window_h: u32,
        texture_w: u32, texture_h: u32, texture_x: i32, texture_y: i32,
    ) {
        // Blit ourselves to the screen.
        
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

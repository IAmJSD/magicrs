use std::sync::Arc;
use crate::region_selector::{engine::RegionSelectorContext, gl_abstractions::GLTexture};
use super::{Editor, EditorFactory, EditorRegion};

// Defines a rectangle editor.
pub struct Rectangle {
    color: Arc<(u8, u8, u8)>,
}
impl Editor for Rectangle {
    fn click(&mut self, _: i32, _: i32) -> Option<EditorRegion> { None }

    fn render(
        &mut self, _: &GLTexture, _: u32, screen_h: u32,
        texture_w: u32, texture_h: u32, texture_x: i32, texture_y: i32,
    ) {
        // Turn the color into a float where 1 is u8::MAX.
        let (r, g, b) = self.color.as_ref();
        let r = *r as f32 / u8::MAX as f32;
        let g = *g as f32 / u8::MAX as f32;
        let b = *b as f32 / u8::MAX as f32;

        // Defines the scizzor.
        unsafe {
            gl::Enable(gl::SCISSOR_TEST);
            gl::Scissor(
                texture_x, screen_h as i32 - texture_y - texture_h as i32,
                texture_w as i32, texture_h as i32,
            );
            gl::ClearColor(r, g, b, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::Disable(gl::SCISSOR_TEST);
        }
    }
}

// Defines the factory for the rectangle editor.
pub struct RectangleFactory {}
impl EditorFactory for RectangleFactory {
    fn new() -> Self {
        RectangleFactory {}
    }

    fn description(&self) -> &'static str {
        "Puts a rectangle on the screen."
    }

    fn create_editor(&mut self, ctx: &mut RegionSelectorContext) -> Box<dyn Editor> {
        Box::new(Rectangle {color: ctx.color_selection.clone()})
    }
}

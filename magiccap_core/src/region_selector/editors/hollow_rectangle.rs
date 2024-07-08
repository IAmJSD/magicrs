use crate::region_selector::{engine::RegionSelectorContext, gl_abstractions::GLTexture};
use super::{Editor, EditorFactory, EditorRegion};

// Defines a hollow rectangle editor.
pub struct HollowRectangle {
    color: (u8, u8, u8),
}
impl Editor for HollowRectangle {
    fn click(&mut self, _: i32, _: i32) -> Option<Option<EditorRegion>> { None }

    fn render(
        &mut self, _: &GLTexture, _: u32, screen_h: u32,
        texture_w: u32, texture_h: u32, texture_x: i32, texture_y: i32,
    ) {
        // Turn the color into a float where 1 is u8::MAX.
        let r = self.color.0 as f32 / u8::MAX as f32;
        let g = self.color.1 as f32 / u8::MAX as f32;
        let b = self.color.2 as f32 / u8::MAX as f32;

        // Defines the scizzor regions.
        let scizzors = [
            // Top region.
            (texture_x, screen_h as i32 - texture_y - texture_h as i32, texture_w, 1),
    
            // Bottom region.
            (texture_x, screen_h as i32 - texture_y, texture_w, 1),

            // Left region.
            (texture_x, screen_h as i32 - texture_y - texture_h as i32, 1, texture_h),

            // Right region.
            (texture_x + texture_w as i32 - 1, screen_h as i32 - texture_y - texture_h as i32, 1, texture_h),
        ];
        unsafe {
            gl::Enable(gl::SCISSOR_TEST);
            for scizzor in scizzors.iter() {
                gl::Scissor(scizzor.0, scizzor.1, scizzor.2 as i32, scizzor.3 as i32);
                gl::ClearColor(r, g, b, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }
            gl::Disable(gl::SCISSOR_TEST);
        }
    }
}

// Defines the factory for the hollow rectangle editor.
pub struct HollowRectangleFactory {}
impl EditorFactory for HollowRectangleFactory {
    fn new() -> Self {
        HollowRectangleFactory {}
    }

    fn description(&self) -> &'static str {
        "Puts a hollow rectangle on the screen."
    }

    fn create_editor(&mut self, ctx: &mut RegionSelectorContext) -> Box<dyn Editor> {
        let read_guard = ctx.color_selection.read().unwrap();
        Box::new(HollowRectangle {color: (
            read_guard.0, read_guard.1, read_guard.2,
        )})
    }
}

use crate::region_selector::gl_abstractions::{GLShaderProgram, GLTexture};
use super::{Editor, EditorFactory};

// Defines the factory for the pixelate editor.
pub struct PixelateFactory {
    program: GLShaderProgram,
}
impl EditorFactory for PixelateFactory {
    fn new() -> Self {
        let mut program = GLShaderProgram::new();
        program.compile_vertex_shader(
            include_str!("shaders/vertex/pixelate.vert").to_string(),
            "pixelate.vert",
        );
        program.compile_fragment_shader(
            include_str!("shaders/fragment/pixelate.frag").to_string(),
            "pixelate.frag",
        );
        PixelateFactory { program }
    }

    fn create_editor(&mut self) -> Box<dyn Editor> {
        Box::new(Pixelate { program_id: self.program.program })
    }
}

// Defines the pixelate editor.
pub struct Pixelate {
    program_id: u32,
}
impl Editor for Pixelate {
    fn click(&self, _: i32, _: i32) -> Option<image::RgbaImage> { None }

    fn render(
        &self, screenshot: &GLTexture, window_w: u32, window_h: u32,
        texture_w: u32, texture_h: u32, texture_x: i32, texture_y: i32,
    ) {
        // Use the program.
        unsafe { gl::UseProgram(self.program_id) };

        // TODO
    }
}

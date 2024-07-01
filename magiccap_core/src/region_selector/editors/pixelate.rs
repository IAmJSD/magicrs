use crate::region_selector::gl_abstractions::{
    copy_texture, GLShaderProgram, GLTexture,
};
use super::{Editor, EditorFactory, EditorRegion};

// Defines the pixelate editor.
struct Pixelate {
    program_id: u32,
}
impl Editor for Pixelate {
    fn click(&mut self, _: i32, _: i32) -> Option<EditorRegion> { None }

    fn render(
        &self, screenshot: &GLTexture, window_w: u32, window_h: u32,
        texture_w: u32, texture_h: u32, texture_x: i32, texture_y: i32,
    ) {
        // Create the texture by copying from the other OpenGL texture.
        let texture = copy_texture(screenshot, texture_w, texture_h, texture_x, texture_y);

        // Use the program.
        unsafe { gl::UseProgram(self.program_id) };

        // Create a framebuffer and use the shader program.
        let mut framebuffer = 0;
        unsafe { gl::GenFramebuffers(1, &mut framebuffer) };
        unsafe { gl::BindFramebuffer(gl::READ_FRAMEBUFFER, framebuffer) };
        unsafe {
            gl::FramebufferTexture2D(
                gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D,
                texture.texture, 0)
        };

        // Blit the framebuffer.
        unsafe {
            gl::BlitFramebuffer(
                0, 0, texture_w as i32, texture_h as i32,
                0, window_h as i32, window_w as i32, 0,
                gl::COLOR_BUFFER_BIT, gl::NEAREST
            );
        }

        // Explicitly drop the texture so we don't use-after-free.
        drop(texture);
    }
}

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
        program.link();
        PixelateFactory { program }
    }

    fn description(&self) -> &'static str {
        "Pixelates the region specified."
    }

    fn create_editor(&mut self) -> Box<dyn Editor> {
        Box::new(Pixelate { program_id: self.program.program })
    }
}

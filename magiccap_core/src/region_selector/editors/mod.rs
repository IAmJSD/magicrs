use once_cell::unsync::Lazy;
use super::gl_abstractions::GLTexture;

// Defines the editor context.
pub trait EditorContext {
    // Sets up the editor context.
    fn new() -> Self
    where
        Self: Sized;

    // Creates a editor from the context.
    fn create_editor(&mut self) -> Box<dyn Editor>;
}

// Defines the editor trait.
pub trait Editor<T = Box<dyn EditorContext>> {
    // Sets up the editor.
    fn new(context: T) -> Self
    where
        Self: Sized;

    // If this returns a value, turns this editor from a draggable one to a
    // click controlled one.
    fn click(&self, x: i32, y: i32) -> Option<image::RgbaImage>;

    // Renders the editor.
    fn render(
        &self, screenshot: &GLTexture, window_w: u32, window_h: u32,
        texture_w: u32, texture_h: u32, texture_x: i32, texture_y: i32,
    );
}

// Creates the editor vector. The vector should be in the order that the editors
// are set.
pub fn create_editor_vec<'a>() -> Vec<Lazy<Box<dyn EditorContext + 'a>>> {
    Vec::new()
}

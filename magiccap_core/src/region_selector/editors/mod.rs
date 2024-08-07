mod blur;
mod fastblur_rgba;
mod hollow_rectangle;
mod pixelate;
mod rectangle;
mod stickers;

use super::{engine::RegionSelectorContext, gl_abstractions::GLTexture};
use once_cell::unsync::Lazy;

// TODO: Add text

// Defines the editor factory.
pub trait EditorFactory {
    // Creates a new instance of the editor factory.
    fn new() -> Self
    where
        Self: Sized;

    // Defines the description of the editor.
    fn description(&self) -> &'static str;

    // Creates a new instance of the editor.
    fn create_editor(&mut self, ctx: &mut RegionSelectorContext) -> Box<dyn Editor>;
}

// Defines the editor region.
pub struct EditorRegion {
    pub width: u32,
    pub height: u32,
}

// Defines an editor made by a factory.
pub trait Editor {
    // If this returns a value, turns this editor from a draggable one to a
    // click controlled one.
    fn click(&mut self, x: i32, y: i32) -> Option<Option<EditorRegion>>;

    // Renders the editor.
    fn render(
        &mut self,
        screenshot: &GLTexture,
        window_w: u32,
        window_h: u32,
        texture_w: u32,
        texture_h: u32,
        texture_x: i32,
        texture_y: i32,
    );
}

// Creates the editor vector. The vector should be in the order that the editors are set.
pub fn create_editor_vec() -> Vec<Lazy<Box<dyn EditorFactory>>> {
    vec![
        Lazy::new(|| Box::new(blur::BlurFactory::new())),
        Lazy::new(|| Box::new(pixelate::PixelateFactory::new())),
        Lazy::new(|| Box::new(hollow_rectangle::HollowRectangleFactory::new())),
        Lazy::new(|| Box::new(rectangle::RectangleFactory::new())),
        Lazy::new(|| Box::new(stickers::StickerFactory::new())),
    ]
}

// Create the editor icons.
pub fn create_editor_icons() -> Vec<&'static [u8]> {
    // Defines the macro to handle the weird path.
    macro_rules! include_texture {
        ($filename:expr) => {
            include_bytes!(concat!("../textures/", $filename))
        };
    }

    // Return the icons.
    vec![
        // Crosshair is always the first icon.
        include_texture!("cursor.png"),
        // Defines the editor icons.
        include_texture!("blur.png"),
        include_texture!("pixelate.png"),
        include_texture!("hollow_rectangle.png"),
        include_texture!("rectangle.png"),
        include_texture!("sticker.png"),
    ]
}

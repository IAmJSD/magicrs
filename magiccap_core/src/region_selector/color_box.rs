use super::{
    color_picker::open_color_picker, engine::RegionSelectorContext, gl_abstractions::GLTexture,
    texture_pack::LOADED_FONT,
};
use image::RgbImage;
use rusttype::{point, Scale};

// Defines the margin from the top right corner.
const MARGIN: i32 = 10;

// Defines the width/height of the color box.
const WIDTH: i32 = 140;
const HEIGHT: i32 = 50;

// Generates the small texture that will be drawn on the color box.
pub fn render_texture(r: u8, g: u8, b: u8) -> GLTexture {
    // Make a RGB texture of width*height with the RGB values.
    let mut data = vec![0; (WIDTH * HEIGHT * 3) as usize];
    for i in 0..WIDTH * HEIGHT {
        data[i as usize * 3] = r;
        data[i as usize * 3 + 1] = g;
        data[i as usize * 3 + 2] = b;
    }

    // Check the lightness of the color.
    let lightness = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
    let is_light = lightness > 128.0;

    // Render the text. LOADED_FONT is truetype Roboto.
    let font = &*LOADED_FONT;
    let scale = Scale::uniform(15.0);
    let mut start_point = point(6.0, (HEIGHT - 15) as f32 - scale.y.ceil());

    // Create a new image.
    let mut img: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> =
        RgbImage::from_vec(WIDTH as u32, HEIGHT as u32, data).unwrap();

    // Draw the text onto the image.
    macro_rules! draw_text {
        ($text:expr) => {
            for glyph in font.layout($text, scale, start_point) {
                if let Some(bounding_box) = glyph.pixel_bounding_box() {
                    glyph.draw(|x, y, v| {
                        let x = x as i32 + bounding_box.min.x;
                        let y = y as i32 + bounding_box.min.y;
                        if x >= 0 && x < WIDTH && y >= 0 && y < HEIGHT {
                            let pixel = img.get_pixel_mut(x as u32, y as u32);
                            let alpha = (v * 255.0) as u8;
                            let inv_alpha = 255 - alpha;

                            let fg = if is_light { [0, 0, 0] } else { [255, 255, 255] };

                            pixel.0[0] = ((alpha as u16 * fg[0] as u16
                                + inv_alpha as u16 * pixel.0[0] as u16)
                                / 255 as u16) as u8;
                            pixel.0[1] = ((alpha as u16 * fg[1] as u16
                                + inv_alpha as u16 * pixel.0[1] as u16)
                                / 255 as u16) as u8;
                            pixel.0[2] = ((alpha as u16 * fg[2] as u16
                                + inv_alpha as u16 * pixel.0[2] as u16)
                                / 255 as u16) as u8;
                        }
                    });
                }
            }
        };
    }
    draw_text!("Press C or left click to");
    start_point.y += 15.0;
    draw_text!("change editor color");

    // Return the texture.
    GLTexture::from_rgb(&img)
}

// Draws the color box.
pub unsafe fn draw_color_box(ctx: &mut RegionSelectorContext, width: i32, height: i32) {
    // Get the lock guard.
    let guard = ctx.color_selection.read().unwrap();

    // Bind the framebuffer to the texture.
    gl::FramebufferTexture2D(
        gl::READ_FRAMEBUFFER,
        gl::COLOR_ATTACHMENT0,
        gl::TEXTURE_2D,
        guard.3.texture,
        0,
    );

    // Get the start X/Y.
    let x = width - WIDTH - MARGIN;
    let y = MARGIN;

    // BLit the color box.
    gl::BlitFramebuffer(
        0,
        HEIGHT,
        WIDTH,
        0,
        x,
        height - y - HEIGHT,
        x + WIDTH,
        height - y,
        gl::COLOR_BUFFER_BIT,
        gl::LINEAR,
    );
}

// Handles if the color box is clicked.
pub fn handle_color_box_click(
    ctx: &mut RegionSelectorContext,
    x: i32,
    y: i32,
    window_w: i32,
) -> bool {
    if x < window_w - WIDTH - MARGIN || x > window_w - MARGIN {
        return false;
    }
    if y < MARGIN || y > MARGIN + HEIGHT {
        return false;
    }

    // Clone the color Arc.
    let color_arc = ctx.color_selection.clone();

    // Open the color selector.
    open_color_picker(move |(r, g, b)| {
        let mut write_guard = color_arc.write().unwrap();
        *write_guard = (r, g, b, render_texture(r, g, b));
    });
    true
}

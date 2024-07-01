use super::engine::RegionSelectorContext;

// Defines the gap between the cursor and the magnifier.
const GAP: i32 = 15;

// Defines the size of the plucked content.
// TODO: play with these constants
const PLUCKED_SIZE: i32 = 50;

// Defines the size of the magnifier.
const MAGNIFIER_SIZE: i32 = 125;

// Renders the magnifier.
pub unsafe fn render_magnifier(
    ctx: &mut RegionSelectorContext, index: usize, cursor_x: i32, cursor_y: i32,
    width: i32, height: i32,
) {
    let (mut x, mut y) = (cursor_x, cursor_y);

    // Defines the pixel location text.
    let text = format!("X: {} | Y: {}", cursor_x, cursor_y);

    // Get the width of the text.
    let text_width = ctx.texture_pack.text_length(text.as_str());

    // https://www.youtube.com/watch?v=LNBjMRvOB5M
    let total_x = MAGNIFIER_SIZE.max(text_width);
    let go_west = x + GAP + total_x > width;
    if go_west {
        // Make the magnifier render to the left if otherwise it would go off the screen.
        x -= GAP + total_x;
    } else {
        // Add a gap between the cursor and the magnifier.
        x += GAP;
    }

    // Figure out the X offsets for both the text and magnifier.
    let mag_x_offset: i32;
    let text_x_offset: i32;
    if MAGNIFIER_SIZE > text_width {
        mag_x_offset = 0;
        text_x_offset = (MAGNIFIER_SIZE - text_width) / 2;
    } else {
        mag_x_offset = (text_width - MAGNIFIER_SIZE) / 2;
        text_x_offset = 0;
    }

    const TEXT_GAP: i32 = 36;
    let above = y + TEXT_GAP + GAP + MAGNIFIER_SIZE > height;
    if above {
        // Make the magnifier render above the cursor if otherwise it would go off the screen.
        y -= GAP + MAGNIFIER_SIZE + TEXT_GAP;
    } else {
        // Add a gap between the cursor and the magnifier.
        y += GAP;
    }

    // Defines the dest X/Y.
    let x0 = x + mag_x_offset;
    let y0 = height - y - MAGNIFIER_SIZE;
    let x1 = x + mag_x_offset + MAGNIFIER_SIZE;
    let y1 = height - y;

    // Draw a black box where the magnifier will be. This involves clearing the specific area.
    gl::Scissor(x0, y0, MAGNIFIER_SIZE, MAGNIFIER_SIZE);
    gl::Enable(gl::SCISSOR_TEST);
    gl::ClearColor(0.0, 0.0, 0.0, 1.0);
    gl::Clear(gl::COLOR_BUFFER_BIT);
    gl::Disable(gl::SCISSOR_TEST);

    // Bind the framebuffer to the texture used for the un-dark content.
    gl::FramebufferTexture2D(
        gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0,
        gl::TEXTURE_2D, ctx.gl_screenshots[index].texture, 0
    );

    // Blit a stretched version of the content around the cursor to the magnifier.
    gl::BlitFramebuffer(
        cursor_x - PLUCKED_SIZE, cursor_y + PLUCKED_SIZE, cursor_x + PLUCKED_SIZE,
        cursor_y - PLUCKED_SIZE, x0, y0, x1, y1,
        gl::COLOR_BUFFER_BIT, gl::NEAREST,
    );

    // Set the Y position to 8px under the magnifier.
    y += MAGNIFIER_SIZE + 8;

    // Draw the text.
    ctx.texture_pack.write_text(
        text.as_str(), x + text_x_offset,
        y, height,
    );
}

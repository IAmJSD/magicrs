use super::engine::RegionSelectorContext;

// Renders the magnifier.
pub unsafe fn render_magnifier(
    ctx: &mut RegionSelectorContext, index: usize, cursor_x: i32, cursor_y: i32,
    width: i32, height: i32,
) {
    let (mut x, mut y) = (cursor_x, cursor_y);

    if x + 10 + 100 > width {
        // Make the magnifier render to the left if otherwise it would go off the screen.
        x -= 10 + 100;
    } else {
        // Add a gap between the cursor and the magnifier.
        x += 10;
    }

    if y + 10 + 100 > height {
        // Make the magnifier render above the cursor if otherwise it would go off the screen.
        y -= 10 + 100;
    } else {
        // Add a gap between the cursor and the magnifier.
        y += 10;
    }

    // Bind the framebuffer to the texture used for the un-dark content.
    gl::FramebufferTexture2D(
        gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0,
        gl::TEXTURE_2D, ctx.gl_screenshots[index].texture, 0
    );

    // Blit a stretched version of the content around the cursor to the magnifier.
    gl::BlitFramebuffer(
        cursor_x - 10, cursor_y + 10, cursor_x + 10, cursor_y - 10,
        x, height - y - 100, x + 100, height - y,
        gl::COLOR_BUFFER_BIT, gl::NEAREST,
    );
}

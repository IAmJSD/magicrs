use super::{engine::RegionSelectorContext, window_find::get_nearest_window};

// Renders the line around the window that will be captured if the user clicks.
pub unsafe fn render_window_line(
    ctx: &mut RegionSelectorContext, index: usize, cursor_x: i32, cursor_y: i32,
) {
    // Get the nearest window.
    let (monitor, nearest_window) = get_nearest_window(
        ctx, cursor_x, cursor_y, index);

    // Handle if the nearest window is set.
    if let Some(window) = nearest_window {
        // Get the window X/Y/W/H.
        let x = window.x();
        let y = window.y();
        let w = window.width() as i32;
        let h = window.height() as i32;

        // Get the X/Y relative to the display.
        let x = x - monitor.x();
        let y = y - monitor.y();

        // Bind the framebuffer to the vertical striped texture.
        gl::FramebufferTexture2D(
            gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D, ctx.striped_tex_h.texture, 0
        );

        // Blit the left and right lines.
        let (_, screen_h) = ctx.glfw_windows[index].get_size();
        gl::BlitFramebuffer(
            0, 0, 1, h,
            x - 1, screen_h - y, x, screen_h - y - h,
            gl::COLOR_BUFFER_BIT, gl::NEAREST,
        );
        gl::BlitFramebuffer(
            0, 0, 1, h,
            x + w, screen_h - y, x + w + 1, screen_h - y - h,
            gl::COLOR_BUFFER_BIT, gl::NEAREST,
        );

        // Bind the framebuffer to the horizontal striped texture.
        gl::FramebufferTexture2D(
            gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D, ctx.striped_tex_w.texture, 0
        );

        // Blit the top and bottom lines.
        gl::BlitFramebuffer(
            0, 0, w, 1,
            x, screen_h - y, x + w, screen_h - y - 1,
            gl::COLOR_BUFFER_BIT, gl::NEAREST,
        );
        gl::BlitFramebuffer(
            0, 0, w, 1,
            x, screen_h - y - h, x + w, screen_h - y - h - 1,
            gl::COLOR_BUFFER_BIT, gl::NEAREST,
        );
    }
}

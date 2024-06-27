use super::engine::RegionSelectorContext;

// Renders the line around the window that will be captured if the user clicks.
pub unsafe fn render_window_line(
    ctx: &mut RegionSelectorContext, index: usize, cursor_x: i32, cursor_y: i32,
) {
    // Get windows within the monitor this is on.
    let monitor = &ctx.setup.monitors[index];
    let windows = ctx.setup.windows.iter()
        .filter(|w| w.current_monitor().id() == monitor.id());

    // Get the un-relative cursor position.
    let (mut cursor_x, mut cursor_y) = (cursor_x, cursor_y);
    cursor_y = monitor.height() as i32 - cursor_y;
    cursor_x += monitor.x();

    // Get the window nearest to the cursor.
    let mut nearest_window = None;
    let mut nearest_distance = std::f64::MAX;
    for window in windows {
        // Get the window X/Y/W/H.
        let x = window.x();
        let y = window.y();
        let w = window.width() as i32;
        let h = window.height() as i32;

        // Get the distance to the window.
        let distance = if cursor_x < x {
            (x - cursor_x).pow(2) as f64
        } else if cursor_x > x + w {
            (cursor_x - (x + w)).pow(2) as f64
        } else if cursor_y < y {
            (y - cursor_y).pow(2) as f64
        } else if cursor_y > y + h {
            (cursor_y - (y + h)).pow(2) as f64
        } else {
            0.0
        };

        // If the distance is less than the nearest distance, set the nearest window.
        if distance < nearest_distance {
            nearest_window = Some(window);
            nearest_distance = distance;
        }
    }

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

        // Bind the framebuffer.
        gl::FramebufferTexture2D(
            gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D, ctx.striped_tex_w.texture, 0
        );

        // Blit the left and right lines.
        // TODO
    }
}

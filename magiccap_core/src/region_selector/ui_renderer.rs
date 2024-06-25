use glfw::{Context, Window};
use super::{
    engine::RegionSelectorContext,
    gl_abstractions::GLTexture
};

// Draw the background.
unsafe fn draw_background(
    texture: &GLTexture, texture_w: i32, texture_h: i32,
    window_w: i32, window_h: i32
) {
    // Create a framebuffer.
    let mut framebuffer = 0;
    gl::GenFramebuffers(1, &mut framebuffer);
    gl::BindFramebuffer(gl::READ_FRAMEBUFFER, framebuffer);
    gl::FramebufferTexture2D(
        gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0,
        gl::TEXTURE_2D, texture.texture, 0
    );

    // Blit the framebuffer.
    gl::BlitFramebuffer(
        0, 0, texture_w, texture_h,
        0, window_h, window_w, 0,
        gl::COLOR_BUFFER_BIT, gl::NEAREST
    );

    // Delete the framebuffer.
    gl::DeleteFramebuffers(1, &framebuffer);
}

// Renders the decorations.
unsafe fn render_decorations(
    ctx: &mut RegionSelectorContext, window: &mut Window, index: usize,
) {
    // Get the width and height of the window.
    let (width, height) = window.get_size();

    // Get the cursor X and Y.
    let (cursor_x, cursor_y) = window.get_cursor_pos();

    // If the cursor is within the window, render the crosshair.
    let within = cursor_x >= 0.0 && cursor_x < width as f64 && cursor_y >= 0.0 && cursor_y < height as f64;
    if within {
        // Get the cursor position relative to the window.
        let (cursor_x, cursor_y) = (cursor_x as i32, height as i32 - cursor_y as i32);

        // Get if the crosshair should be dark.
        let crosshair_dark = ctx.light_detectors[index].get_lightness(cursor_x as u32, cursor_y as u32);

        // Load the crosshair texture.
        let mut framebuffer = 0;
        gl::GenFramebuffers(1, &mut framebuffer);
        gl::BindFramebuffer(gl::READ_FRAMEBUFFER, framebuffer);
        if crosshair_dark {
            gl::FramebufferTexture2D(
                gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D, ctx.black_texture.texture, 0
            );
        } else {
            gl::FramebufferTexture2D(
                gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D, ctx.white_texture.texture, 0
            );
        }

        // Blit the framebuffer through the cursor position.
        gl::BlitFramebuffer(
            // Pull in a line the width of the display.
            0, 0, width, 1,

            // Place it at the cursor position so it goes through the cursor.
            0, cursor_y, width, cursor_y + 1,

            // Copy the color buffer.
            gl::COLOR_BUFFER_BIT, gl::NEAREST,
        );
        gl::BlitFramebuffer(
            // Pull in a line the height of the display.
            0, 0, height, 1,

            // Place it at the cursor position so it goes through the cursor.
            cursor_x, 0, cursor_x + 1, height,

            // Copy the color buffer.
            gl::COLOR_BUFFER_BIT, gl::NEAREST,
        );

        // Delete the framebuffer.
        gl::DeleteFramebuffers(1, &framebuffer);
    }
}

// Handles iterating or jumping right to a index.
pub fn iter_windows_or_jump(
    ctx: &mut RegionSelectorContext, index: Option<usize>,
    closure: &dyn Fn(&mut RegionSelectorContext, &mut Window, usize)
) {
    // Use unsafe to get the mutable reference. This is safe because we know that the context will outlive
    // the mutable reference.
    let ctx2 = unsafe { &mut *(&mut *ctx as *mut RegionSelectorContext) };

    // Handle if the index is set.
    if let Some(index) = index {
        // Get the window.
        let window = &mut ctx2.glfw_windows[index];

        // Call the closure with separate mutable references.
        closure(ctx, window, index);
        return;
    }

    // Iterate through the screenshots.
    for (i, window) in ctx2.glfw_windows.iter_mut().enumerate() {
        // Call the closure with separate mutable references.
        closure(ctx, window, i);
    }
}

// Renders the UI. This is marked as unsafe because it uses OpenGL.
pub unsafe fn region_selector_render_ui(
    ctx: &mut RegionSelectorContext, with_decorations: bool, window_index: Option<usize>,
) {
    iter_windows_or_jump(ctx, window_index, &|ctx, window, i| {
        // Set the window as the current context.
        window.make_current();

        // Set the viewport.
        let (width, height) = window.get_size();
        gl::Viewport(0, 0, width, height);

        // Clear the buffer.
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);

        // Render the background.
        let screenshot_non_darkened = &ctx.gl_screenshots[i];
        let screenshot = if with_decorations {
            &ctx.gl_screenshots_darkened[i]
        } else { screenshot_non_darkened };
        let (texture_w, texture_h) = ctx.image_dimensions[i];
        draw_background(
            screenshot, texture_w as i32, texture_h as i32,
            width, height
        );

        // Render the editors.
        for editor in ctx.active_editors.iter_mut() {
            if editor.display_index == i {
                editor.editor.render(
                    screenshot_non_darkened, width as u32, height as u32,
                    editor.width, editor.height,
                    editor.x, editor.y
                );
            }
        }

        // If decorations should be rendered, render them.
        if with_decorations { render_decorations(ctx, window, i) }

        // Flush the buffer.
        gl::Flush();

        // Swap the buffer with the current window.
        window.swap_buffers();
    })
}

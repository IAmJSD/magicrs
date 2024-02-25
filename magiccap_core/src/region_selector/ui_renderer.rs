use glfw::{Context, Window};
use super::{
    engine::RegionSelectorContext,
    gl_abstractions::GLTexture
};

// Draw the image background.
unsafe fn draw_image_background(texture: &GLTexture, w: u32, h: u32) {
    // Bind the texture.
    gl::BindTexture(gl::TEXTURE_2D, texture.texture);

    // Set the texture parameters.
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

    // Set the texture wrapping.
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

    // Draw the quad.
    gl::BeginTransformFeedback(gl::QUADS);
    gl::TexCoordP2ui(0, 0);
    gl::VertexP2ui(0, 0);
    gl::TexCoordP2ui(1, 0);
    gl::VertexP2ui(w, 0);
    gl::TexCoordP2ui(1, 1);
    gl::VertexP2ui(w, h);
    gl::TexCoordP2ui(0, 1);
    gl::VertexP2ui(0, h);
    gl::EndTransformFeedback();

    // Flush the buffer.
    gl::Flush();
}

// Handles iterating or jumping right to a index.
fn iter_screenshots_or_jump(
    ctx: &mut RegionSelectorContext, index: Option<u32>,
    closure: &dyn Fn(&mut RegionSelectorContext, &mut Window, &GLTexture)
) {
    // Use unsafe to get the mutable reference. This is safe because we know that the context will outlive
    // the mutable reference.
    let ctx2 = unsafe { &mut *(&mut *ctx as *mut RegionSelectorContext) };

    // Handle if the index is set.
    if let Some(index) = index {
        // Get the window and screenshot.
        let screenshot = &ctx2.gl_screenshots[index as usize];
        let window = &mut ctx2.glfw_windows[index as usize];

        // Call the closure with separate mutable references.
        closure(ctx, window, screenshot);
        return;
    }

    // Iterate through the screenshots.
    for (_, (window, screenshot)) in ctx2.glfw_windows.iter_mut().zip(&ctx2.gl_screenshots).enumerate() {
        // Call the closure with separate mutable references.
        closure(ctx, window, screenshot);
    }
}

// Renders the UI. This is marked as unsafe because it uses OpenGL.
pub unsafe fn region_selector_render_ui(
    ctx: &mut RegionSelectorContext, with_decorations: bool, window_index: Option<u32>
) {
    // Enable textures and blending.
    gl::Enable(gl::TEXTURE_2D);
    gl::Enable(gl::BLEND);

    // Iterate through the screenshots.
    iter_screenshots_or_jump(ctx, window_index, &|ctx, window, screenshot| {
        // Set the viewport.
        let (width, height) = window.get_size();
        gl::Viewport(0, 0, width, height);

        // Render the image background.
        draw_image_background(screenshot, width as u32, height as u32);

        // Swap the buffer.
        window.swap_buffers();
    });

    // Disable textures and blending.
    gl::Disable(gl::TEXTURE_2D);
    gl::Disable(gl::BLEND);
}

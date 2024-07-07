use glfw::Window;
use super::{
    engine::{EditorUsage, RegionSelectorContext},
    ui_renderer::region_selector_render_ui, Region, RegionCapture,
};

// Handles capturing the region.
pub fn region_capture(
    ctx: &mut RegionSelectorContext, i: usize, x: i32, y: i32, w: i32, h: i32,
    window: &mut Window,
) -> Option<RegionCapture> {
    // Set the current selection to None since otherwise it will break Escape.
    ctx.active_selection = None;

    // Return immediately if either the width or height is 0. This will panic the application.
    if w == 0 || h == 0 {
        return None;
    }

    // Handle if a editor is selected.
    let ctx2 = unsafe { &mut *(&mut *ctx as *mut _) };
    if let Some(editor_index) = ctx.editor_index {
        let editor = ctx.editors[editor_index].create_editor(ctx2);
        let active_editor = EditorUsage {
            x,
            y,
            width: w as u32,
            height: h as u32,
            editor,
            display_index: i,
        };
        ctx.active_editors.push(active_editor);
        return None;
    }

    // Render the window without decorations.
    unsafe {
        region_selector_render_ui(ctx, false, Some(i))
    };

    // Grab the image.
    let mut buffer = vec![0u8; (w * h * 4) as usize];
    unsafe {
        // We need to do this brainfuck maths because OpenGL flips the Y axis. If you are
        // less dyslexic than me and can make this work nicer, feel free.
        let (_, screen_h) = window.get_size();
        gl::ReadPixels(
            x, screen_h - h - y, w, h, gl::RGBA,
            gl::UNSIGNED_BYTE, buffer.as_mut_ptr() as *mut _
        );
    }

    // Create the result.
    let res = RegionCapture {
        image: image::RgbaImage::from_raw(w as u32, h as u32, buffer).unwrap(),
        monitor: ctx.setup.monitors[i].clone(),
        relative_region: Region {
            x,
            y,
            width: w as u32,
            height: h as u32,
        },
    };

    // Mark the selector as closed.
    window.set_should_close(true);

    // Return the result.
    Some(res)
}

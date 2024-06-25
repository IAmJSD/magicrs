use glfw::{Action, Key};
use super::{
    engine::{EditorUsage, RegionSelectorContext, SendSyncBypass},
    ui_renderer::region_selector_render_ui, Region, RegionCapture
};

// Handles the fullscreen key being pressed.
fn fullscreen_key(ctx: &mut RegionSelectorContext, shift_held: bool) -> Option<RegionCapture> {
    // Find the window the mouse is on.
    let mut active_window = &ctx.glfw_windows[0];
    let mut active_index = 0;
    for (index, window) in ctx.glfw_windows.iter().enumerate() {
        let (x, y) = window.get_cursor_pos();
        let (w, h) = window.get_size();
        if x >= 0.0 && x < w as f64 && y >= 0.0 && y < h as f64 {
            active_window = &window;
            active_index = index;
            break;
        }
    }
    let (width, height) = active_window.get_size();

    // Handle if shift is held with a tool selected.
    if ctx.editor_index.is_some() && shift_held {
        let index = ctx.editor_index.unwrap();
        let (width, height) = active_window.get_size();
        let editor = ctx.editors[index].create_editor();
        let active_editor = EditorUsage {
            x: 0,
            y: 0,
            editor,
            width: width as u32,
            height: height as u32, 
            display_index: active_index,
        };
        ctx.active_editors.push(active_editor);
        return None;
    }

    // Render the window without decorations.
    unsafe {
        region_selector_render_ui(
            ctx, false, Some(active_index),
        );
    }

    // Pull the OpenGL buffer of the window.
    let mut buffer = vec![0; (width * height * 4) as usize];
    unsafe {
        gl::ReadPixels(
            0, 0, width, height, gl::RGBA, gl::UNSIGNED_BYTE, buffer.as_mut_ptr() as *mut std::ffi::c_void
        );
    }

    // Create the image with the buffer inside.
    let image = image::RgbaImage::from_raw(width as u32, height as u32, buffer).unwrap();

    // Return the region capture.
    Some(RegionCapture {
        monitor: ctx.setup.monitors[active_index].clone(),
        relative_region: Region {
            x: 0,
            y: 0,
            width: width as u32,
            height: height as u32,
        },
        image,
    })
}

// Handles the mouse left button being pushed.
fn mouse_left_push(ctx: &mut RegionSelectorContext, i: i32) {
    // TODO
}

// Handles the mouse left button being released.
fn mouse_left_release(ctx: &mut RegionSelectorContext, i: i32) -> Option<RegionCapture> {
    // TODO
    None
}

// Defines when a number key is hit. This function is a bit special since we repeat it a lot so we render the UI in here.
fn number_key_hit(ctx: &mut RegionSelectorContext, number: u8) {
    // Return early if editors are off.
    if !ctx.setup.show_editors { return; }

    if number == 1 {
        // If the key is 1, go ahead and remove the tool.
        ctx.editor_index = None;
    } else {
        // Check if the number is greater than the number of editors.
        let number_u = number as usize - 1;
        if number_u > ctx.editors.len() {
            return;
        }
        ctx.editor_index = Some(number_u - 1);
    }
}

// Defines an IO event loop send.
pub fn region_selector_io_event_sent(
    ctx: &mut RegionSelectorContext,
    event: glfw::WindowEvent,
    current_index: i32,
){
    match event {
        // Handle either aborting the selection or closing the window when esc is hit.
        glfw::WindowEvent::Key(Key::Escape, _, Action::Release, _) => {
            if ctx.active_selection.is_some() {
                // Remove the active selection.
                ctx.active_selection = None;
                return;
            }

            // All the other windows and this should die.
            for window in &mut ctx.glfw_windows {
                window.set_should_close(true);
            }
        },

        // Handle the fullscreen key.
        glfw::WindowEvent::Key(Key::F, _, Action::Release, modifiers) => match fullscreen_key(
            ctx, modifiers.contains(glfw::Modifiers::Shift)
        ) {
            Some(x) => {
                // Write the result and kill the windows.
                ctx.result = Some(x);
                for window in &mut ctx.glfw_windows {
                    window.set_should_close(true);
                }
            },
            None => {},
        },

        // Handle Cmd or Ctrl + Z.
        glfw::WindowEvent::Key(Key::Z, _, Action::Release, mods) => {
            let modifier = if cfg!(target_os = "macos") {
                glfw::Modifiers::Super
            } else {
                glfw::Modifiers::Control
            };
            if mods.contains(modifier) {
                // Pop off the last editor.
                ctx.active_editors.pop();
            }
        },

        // Handle mouse left clicks.
        glfw::WindowEvent::MouseButton(glfw::MouseButtonLeft, Action::Press, _) => {
            mouse_left_push(ctx, current_index);
        },
        glfw::WindowEvent::MouseButton(glfw::MouseButtonLeft, Action::Release, _) => match mouse_left_release(ctx, current_index) {
            Some(x) => {
                // Write the result and kill the windows.
                ctx.result = Some(x);
                for window in &mut ctx.glfw_windows {
                    window.set_should_close(true);
                }
            },
            None => {},
        },

        // Handle 1-9 being hit.
        glfw::WindowEvent::Key(Key::Num1, _, Action::Release, _) => {
            number_key_hit(ctx, 1)
        },
        glfw::WindowEvent::Key(Key::Num2, _, Action::Release, _) => {
            number_key_hit(ctx, 2)
        },
        glfw::WindowEvent::Key(Key::Num3, _, Action::Release, _) => {
            number_key_hit(ctx, 3)
        },
        glfw::WindowEvent::Key(Key::Num4, _, Action::Release, _) => {
            number_key_hit(ctx, 4)
        },
        glfw::WindowEvent::Key(Key::Num5, _, Action::Release, _) => {
            number_key_hit(ctx, 5)
        },
        glfw::WindowEvent::Key(Key::Num6, _, Action::Release, _) => {
            number_key_hit(ctx, 6)
        },
        glfw::WindowEvent::Key(Key::Num7, _, Action::Release, _) => {
            number_key_hit(ctx, 7)
        },
        glfw::WindowEvent::Key(Key::Num8, _, Action::Release, _) => {
            number_key_hit(ctx, 8)
        },
        glfw::WindowEvent::Key(Key::Num9, _, Action::Release, _) => {
            number_key_hit(ctx, 9)
        },

        // Sinkhole other events.
        _ => {},
    }

}

// Defines the event loop handler for the region selector.
pub fn region_selector_event_loop_handler(
    ctx: &mut Box<SendSyncBypass<RegionSelectorContext>>
) -> Option<Option<RegionCapture>> {
    // Convert the container into a mutable reference.
    let ctx = ctx.as_mut().as_mut();

    // Wait for the events.
    ctx.glfw.wait_events();

    // Go through the windows.
    for window in &mut ctx.glfw_windows {
        if window.should_close() {
            // Terminate through the result saved to the context.
            return Some(ctx.result.take());
        }
    }

    // Render the UI.
    unsafe {
        region_selector_render_ui(ctx, true, None);
    }

    // Return none since we don't want to close the window.
    None
}

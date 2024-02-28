use glfw::{Action, Key};
use super::{
    engine::{RegionSelectorContext, SendSyncBypass},
    RegionCapture,
    ui_renderer::region_selector_render_ui,
};

// Handles the fullscreen key being pressed.
fn handle_fullscreen_key(ctx: &mut RegionSelectorContext) -> Option<RegionCapture> {
    // TODO
    None
}

// Handles the mouse being moved.
fn mouse_move(ctx: &mut RegionSelectorContext, i: i32, x: f64, y: f64) {
    // TODO
}

// Handles the mouse left button being pushed.
fn handle_mouse_left_push(ctx: &mut RegionSelectorContext, i: i32) {
    // TODO
}

// Handles the mouse left button being released.
fn handle_mouse_left_release(ctx: &mut RegionSelectorContext, i: i32) -> Option<RegionCapture> {
    // TODO
    None
}

// Defines when a number key is hit. This function is a bit special since we repeat it
// a lot so we render the UI in here.
fn number_key_hit(ctx: &mut RegionSelectorContext, number: u8) {
    if number == 1 {
        // If the key is 1, go ahead and remove the tool.
        ctx.editor_index = None;
    } else {
        // Check if the number is greater than the number of editors.
        let number_u = number as usize;
        if number_u > ctx.editors.len() {
            return;
        }
        ctx.editor_index = Some(number_u - 1);
    }

    // Re-render the UI.
    unsafe {
        region_selector_render_ui(
            ctx, true, None,
        );
    }
}

// Defines the event loop handler for the region selector.
pub fn region_selector_event_loop_handler(
    ctx: &mut Box<SendSyncBypass<RegionSelectorContext>>
) -> Option<Option<RegionCapture>> {
    // Convert the container into a mutable reference.
    let ctx = ctx.as_mut().as_mut();

    // Go through the windows.
    for window in &ctx.glfw_windows {
        if window.should_close() {
            // All the other windows should die too.
            return Some(None);
        }
    }

    // Poll the events.
    ctx.glfw.poll_events();
    let mut window_index = 0;
    for events in &ctx.glfw_events {
        let current_index = window_index;
        window_index += 1;
        for (_, event) in glfw::flush_messages(events) {
            match event {
                // Handle either aborting the selection or closing the window when esc is hit.
                glfw::WindowEvent::Key(Key::Escape, _, Action::Release, _) => {
                    if ctx.active_selection.is_some() {
                        // Remove the active selection and re-render the UI.
                        ctx.active_selection = None;
                        return unsafe {
                            region_selector_render_ui(
                                ctx, true, None,
                            );
                            None
                        };
                    }

                    // All the other windows should die too.
                    return Some(None);
                },

                // Handle the fullscreen key.
                glfw::WindowEvent::Key(Key::F, _, Action::Release, _) => match handle_fullscreen_key(ctx) {
                    Some(x) => return Some(Some(x)),
                    None => {
                        // Re-render the UI.
                        unsafe {
                            region_selector_render_ui(
                                ctx, true, None,
                            );
                        }

                        // Return none since we just want to re-render the UI.
                        return None;
                    },
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

                        // Re-render the UI.
                        unsafe {
                            region_selector_render_ui(
                                ctx, true, None,
                            );
                        }

                        // Return none since the UI was re-rendered.
                        return None;
                    }
                },
    
                // Handle mouse movement.
                glfw::WindowEvent::CursorPos(x, y) => {
                    // Handle the mouse move.
                    mouse_move(ctx, current_index, x, y);

                    // Re-render the UI.
                    unsafe {
                        region_selector_render_ui(
                            ctx, true, None,
                        );
                    };

                    // Return none since we just want to re-render the UI.
                    return None;
                },
    
                // Handle mouse left clicks.
                glfw::WindowEvent::MouseButton(glfw::MouseButtonLeft, Action::Press, _) => {
                    handle_mouse_left_push(ctx, current_index);

                    // In so many cases that it makes sense to do it in all, we want to re-render the UI.
                    unsafe {
                        region_selector_render_ui(
                            ctx, true, None,
                        );
                    }

                    return None;
                },
                glfw::WindowEvent::MouseButton(glfw::MouseButtonLeft, Action::Release, _) => match handle_mouse_left_release(ctx, current_index) {
                    Some(x) => return Some(Some(x)),
                    None => {
                        // Re-render the UI.
                        unsafe {
                            region_selector_render_ui(
                                ctx, true, None,
                            );
                        }

                        // Return none since we just want to re-render the UI.
                        return None;
                    },
                },

                // Handle 1-9 being hit.
                glfw::WindowEvent::Key(Key::Num1, _, Action::Release, _) => {
                    number_key_hit(ctx, 1); return None;
                },
                glfw::WindowEvent::Key(Key::Num2, _, Action::Release, _) => {
                    number_key_hit(ctx, 2); return None;
                },
                glfw::WindowEvent::Key(Key::Num3, _, Action::Release, _) => {
                    number_key_hit(ctx, 3); return None;
                },
                glfw::WindowEvent::Key(Key::Num4, _, Action::Release, _) => {
                    number_key_hit(ctx, 4); return None;
                },
                glfw::WindowEvent::Key(Key::Num5, _, Action::Release, _) => {
                    number_key_hit(ctx, 5); return None;
                },
                glfw::WindowEvent::Key(Key::Num6, _, Action::Release, _) => {
                    number_key_hit(ctx, 6); return None;
                },
                glfw::WindowEvent::Key(Key::Num7, _, Action::Release, _) => {
                    number_key_hit(ctx, 7); return None;
                },
                glfw::WindowEvent::Key(Key::Num8, _, Action::Release, _) => {
                    number_key_hit(ctx, 8); return None;
                },
                glfw::WindowEvent::Key(Key::Num9, _, Action::Release, _) => {
                    number_key_hit(ctx, 9); return None;
                },

                // Sinkhole other events.
                _ => {},
            }
        }
    }

    // No event was handled.
    None
}

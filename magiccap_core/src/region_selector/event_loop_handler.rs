use glfw::{Action, Key};
use super::{
    engine::{RegionSelectorContext, SendSyncBypass},
    RegionCapture,
    ui_renderer::region_selector_render_ui,
};

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
    for events in &ctx.glfw_events {
        for (_, event) in glfw::flush_messages(events) {
            match event {
                // Handle either aborting the selection or closing the window when esc is hit.
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
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
    
                // TODO: implement other keys

                // Sinkhole other events.
                _ => {},
            }
        }
    }

    // Return none if nothing interesting happened.
    None
}

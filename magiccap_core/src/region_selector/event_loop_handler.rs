use glfw::{Action, Key};
use super::{
    engine::{RegionSelectorContext, SendSyncBypass},
    RegionCapture,
}; 

// Defines the event loop handler for the region selector.
pub fn region_selector_event_loop_handler(
    ctx: &mut SendSyncBypass<RegionSelectorContext>
) -> Option<Option<RegionCapture>> {
    // Convert the container into a mutable reference.
    let ctx = ctx.as_mut();

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
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    // End the event loop.
                    return Some(None);
                }
                // TODO: Fix this event.
                // TODO: other events.
                _ => {},
            }
        }
    }

    // Return none if nothing interesting happened.
    None
}

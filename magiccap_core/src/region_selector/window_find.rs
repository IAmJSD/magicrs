use super::engine::RegionSelectorContext;
use xcap::{Monitor, Window};

#[cfg(not(target_os = "linux"))]
pub fn get_nearest_window(
    ctx: &mut RegionSelectorContext,
    rel_cursor_x: i32,
    rel_cursor_y: i32,
    index: usize,
) -> (&Monitor, Option<&Window>) {
    // Get windows within the monitor this is on.
    let monitor = &ctx.setup.monitors[index];
    let windows = ctx
        .setup
        .windows
        .iter()
        .filter(|w| w.current_monitor().id() == monitor.id());

    // Get the un-relative cursor position.
    let (mut cursor_x, mut cursor_y) = (rel_cursor_x, rel_cursor_y);
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

        // If the distance is less than the nearest distance, process this window.
        if distance < nearest_distance {
            nearest_window = Some(window);
            nearest_distance = distance;
        }
    }

    // Return the nearest window.
    (monitor, nearest_window)
}

#[cfg(target_os = "linux")]
pub fn get_nearest_window(
    ctx: &mut RegionSelectorContext,
    _: i32,
    _: i32,
    index: usize,
) -> (&Monitor, Option<&Window>) {
    // Display order is too unreliable on Linux :(
    (ctx.setup.monitors.get(index).unwrap(), None)
}

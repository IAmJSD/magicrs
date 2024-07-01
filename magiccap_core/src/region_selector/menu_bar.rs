use glfw::Window;
use super::engine::RegionSelectorContext;

// Get the gap from the top for the menu bar.
fn get_top_gap() -> i32 {
    // TODO: notches
    20
}

// Draw the menu bar.
pub unsafe fn draw_menu_bar(ctx: &mut RegionSelectorContext, window: &mut Window, index: usize) {
    // TODO
}

// Check if the cursor is within the menu bar.
pub fn within_menu_bar(ctx: &RegionSelectorContext, rel_x: i32, rel_y: i32, screen_w: i32) -> bool {
    // Get the editors count and add 1 for the icon count.
    let icons_count = ctx.editors.len() as i32 + 1;

    // Add the width of the curves here.
    let curve_w = if icons_count == 1 { 25 } else { 50 };

    // Calculate the total width of the menu bar.
    let menu_width = (icons_count * 50) + curve_w;

    // Get the bounds of the menu bar on the window.
    let half_sw = screen_w / 2;
    let half_mw = menu_width / 2;
    let x0 = half_sw - half_mw;
    let x1 = half_sw + half_mw;

    // Check if the cursor is within the menu bar.
    let y0 = get_top_gap();
    let y1 = y0 + 50;

    // Return the calculations.
    x0 <= rel_x && rel_x <= x1 && y0 <= rel_y && rel_y <= y1
}

// Check if the cursor is within the menu bar and if so handle it.
pub fn menu_bar_click(ctx: &mut RegionSelectorContext, rel_x: i32, rel_y: i32) {
    // TODO
}

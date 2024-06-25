use glfw::Window;
use super::engine::RegionSelectorContext;

// Draw the menu bar.
pub unsafe fn draw_menu_bar(ctx: &mut RegionSelectorContext, window: &mut Window, index: usize) {
    // TODO
}

// Check if the cursor is within the menu bar.
pub fn within_menu_bar(ctx: &RegionSelectorContext, rel_x: i32, rel_y: i32) -> bool {
    // TODO
    false
}

// Check if the cursor is within the menu bar and if so handle it.
pub fn menu_bar_click(ctx: &mut RegionSelectorContext, rel_x: i32, rel_y: i32) {
    // TODO
}

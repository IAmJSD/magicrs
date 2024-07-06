use super::engine::RegionSelectorContext;

// Handles the cursor being mouse down within a editors crop margin. Within a branch that
// excludes the menu bar. Returns false if the cursor is not within the crop margin.
pub fn handle_active_editor_drag_start(
    ctx: &mut RegionSelectorContext, index: usize, rel_x: i32, rel_y: i32,
) -> bool {
    // TODO
    false
}

// Flushes any updates to editors.
pub fn flush_editor_updates(
    ctx: &mut RegionSelectorContext, index: usize, cursor_x: f64, cursor_y: f64,
    width: i32, height: i32,
) {
    // TODO
}

// Render lines for the editors.
pub fn render_editor_resize_lines(ctx: &RegionSelectorContext, index: usize) {
    // TODO
}

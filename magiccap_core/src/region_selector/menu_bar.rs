use super::engine::RegionSelectorContext;

// Get the gap from the top for the menu bar.
fn get_top_gap() -> i32 {
    // TODO: notches
    20
}

// Find the index of the icon that the cursor is over.
fn get_editor_index(
    icons_count: usize,
    rel_x: i32,
    rel_y: i32,
    screen_w: i32,
    top_gap: i32,
) -> Option<usize> {
    // Calculate the total width of the menu bar.
    let menu_width = (icons_count * 50) - 1;

    // Get the bounds of the menu bar on the window.
    let half_sw = screen_w / 2;
    let half_mw = menu_width as i32 / 2;
    let x0 = half_sw - half_mw;
    let x1 = half_sw + half_mw;

    // Check if the cursor is within the menu bar.
    let y1 = top_gap + 50;
    if x0 <= rel_x && rel_x <= x1 && top_gap <= rel_y && rel_y <= y1 {
        // Get the relative X.
        let rel_x = rel_x - x0;

        // Divide by the icon width.
        return Some(rel_x as usize / 50);
    }

    // We aren't in range.
    None
}

// Draw the menu bar.
pub unsafe fn draw_menu_bar(
    ctx: &mut RegionSelectorContext,
    cursor_x: i32,
    cursor_y: i32,
    screen_w: i32,
    screen_h: i32,
) {
    // Get the index of the editor.
    let icons_count = ctx.editors.len() + 1;
    let top_gap = get_top_gap();
    let hovering = get_editor_index(icons_count, cursor_x, cursor_y, screen_w, top_gap);

    // Get the X position of the menu bar.
    let half_sw = screen_w / 2;
    let half_mw = (icons_count * 50) as i32 / 2;
    let x = half_sw - half_mw;

    // Render the menu bar.
    let selected = match ctx.editor_index {
        Some(i) => i + 1,
        None => 0,
    };
    ctx.texture_pack
        .render_menu_bar(selected, hovering, x, top_gap, screen_h);

    // Render the description.
    if let Some(i) = hovering {
        // Defines the X position of the description.
        let x = if i == 0 { x } else { x + (i as i32 * 50) };

        // Get the description.
        let desc = if i == 0 {
            "The selector for a region of the screen."
        } else {
            ctx.editors[i - 1].description()
        };

        // Render the description.
        ctx.texture_pack.write_text(desc, x, top_gap + 55, screen_h);
    }
}

// Check if the cursor is within the menu bar.
pub fn within_menu_bar(ctx: &RegionSelectorContext, rel_x: i32, rel_y: i32, screen_w: i32) -> bool {
    // Get the editors count and add 1 for the icon count.
    let icons_count = ctx.editors.len() as i32 + 1;

    // Calculate the total width of the menu bar.
    let menu_width = icons_count * 50;

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
pub fn menu_bar_click(ctx: &mut RegionSelectorContext, rel_x: i32, rel_y: i32, screen_w: i32) {
    // Get which index was clicked.
    let icons_count = ctx.editors.len() + 1;
    let top_gap = get_top_gap();
    let i = match get_editor_index(icons_count, rel_x, rel_y, screen_w, top_gap) {
        Some(i) => i,
        None => return,
    };

    // Write the appropriate editor index.
    if i == 0 {
        ctx.editor_index = None;
    } else {
        ctx.editor_index = Some(i - 1);
    }
}

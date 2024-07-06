use std::collections::HashSet;
use super::engine::{EditorResizerElement, EditorUsage, RegionSelectorContext};

// Check if a overlaps b.
fn overlaps(a: &EditorUsage, b: &EditorUsage) -> bool {
    // Check if a is to the left of b.
    if (a.x + a.width as i32) < b.x {
        return false;
    }

    // Check if a is to the right of b.
    if a.x > b.x + b.width as i32 {
        return false;
    }

    // Check if a is above b.
    if (a.y + a.height as i32) < b.y {
        return false;
    }

    // Check if a is below b.
    if a.y > b.y + b.height as i32 {
        return false;
    }

    // If none of the above are true, then a and b overlap.
    true
}

// Gets the visible editors on a display.
fn get_visible_editors<'a>(ctx: &'a mut RegionSelectorContext, index: usize) -> Vec<(usize, &'a mut EditorUsage)> {
    // Get the editors active on the display specified. It is reversed since the last created will be
    // at the end of the list.
    let editors = ctx.active_editors.iter_mut().enumerate()
        .filter(|(_, editor)| editor.display_index == index).rev().collect::<Vec<_>>();

    // Remove any editors that are not visible.
    let mut overlapped_editors = HashSet::new();
    for (i, (_, editor)) in editors.iter().enumerate() {
        // If this editor is marked as not visible, skip it.
        if overlapped_editors.contains(&i) {
            continue;
        }

        // Check if this editor overlaps any other editor.
        let second_iter = editors.iter().enumerate().skip(i + 1);
        for (j, (_, other_editor)) in second_iter {
            // If the other editor is marked as not visible, skip it.
            if overlapped_editors.contains(&j) {
                continue;
            }

            // Check if the editor overlaps the other editor.
            if overlaps(editor, other_editor) {
                overlapped_editors.insert(j);
            }
        }
    }

    // Remove the overlapped editors and return the result.
    editors.into_iter().enumerate()
        .filter(|(i, _)| !overlapped_editors.contains(&i)).map(|(_, e)| e).collect()
} 

// Defines the selector size.
const SELECTOR_WIDTH: i32 = 10;
const SELECTOR_HEIGHT: i32 = 10;

// Handles the cursor being mouse down within a editors crop margin. Within a branch that
// excludes the menu bar. Returns false if the cursor is not within the crop margin.
pub fn handle_active_editor_drag_start(
    ctx: &mut RegionSelectorContext, index: usize, rel_x: i32, rel_y: i32,
) -> bool {
    // Get the editors active on the display specified.
    let ctx2 = unsafe { &mut *(ctx as *mut _) };
    let editors = get_visible_editors(ctx2, index);

    // Check if the cursor is on any of the resize points.
    let half_sw = SELECTOR_WIDTH / 2;
    let half_sh = SELECTOR_HEIGHT / 2;
    for (editor_index, editor) in editors {
        // Get the editor X/Y/W/H.
        let x = editor.x;
        let y = editor.y;
        let w = editor.width as i32;
        let h = editor.height as i32;

        // Check if there is a midpoint and if so if the cursor is within it.
        if w > 25 && h > 25 {
            let (mid_x, mid_y) = (x + w / 2, y + h / 2);
            if rel_x >= mid_x - half_sw && rel_x < mid_x + half_sw &&
                rel_y >= mid_y - half_sh && rel_y < mid_y + half_sh {
                // We are in the midpoint. Write the center point.
                ctx.editor_dragged = Some((editor_index, EditorResizerElement::Centre));
                return true;
            }
        }

        // Check the rest of the points.
        let points = [
            (x, y, EditorResizerElement::TopLeft),
            (x + w, y, EditorResizerElement::TopRight),
            (x, y + h, EditorResizerElement::BottomLeft),
            (x + w, y + h, EditorResizerElement::BottomRight),
        ];
        for (x, y, element) in points.iter() {
            if rel_x >= x - half_sw && rel_x < x + half_sw &&
                rel_y >= y - half_sh && rel_y < y + half_sh {
                // We are in this point. Write the point.
                ctx.editor_dragged = Some((editor_index, element.clone()));
                return true;
            }
        }
    }

    // We aren't in any editor.
    false
}

// Flushes any updates to editors.
pub fn flush_editor_updates(
    ctx: &mut RegionSelectorContext, index: usize, cursor_x: f64, cursor_y: f64,
    width: i32, height: i32,
) {
    // Get the editor index and element.
    let (editor_index, element) = match ctx.editor_dragged {
        Some(e) => e,
        None => return,
    };

    // Get the editor.
    let editor = match ctx.active_editors.get_mut(editor_index) {
        Some(e) => e,
        None => {
            // The editor was removed.
            ctx.editor_dragged = None;
            return;
        },
    };

    // Make sure that the editor is within the same display.
    if editor.display_index != index {
        ctx.editor_dragged = None;
        return;
    }

    // Get the cursor position relative to the window.
    let (cursor_x, cursor_y) = (cursor_x.floor() as i32, cursor_y.floor() as i32);

    // Match the element.
    match element {
        EditorResizerElement::Centre => {
            // Get the editor X/Y/W/H.
            let w = editor.width as i32;
            let h = editor.height as i32;

            // Calculate the new X/Y.
            let new_x = cursor_x - w / 2;
            let new_y = cursor_y - h / 2;

            // Update the editor.
            editor.x = new_x;
            editor.y = new_y;
        },

        // This is all copilot, requires extensive testing.
        EditorResizerElement::TopLeft => {
            // Get the editor X/Y/W/H.
            let w = editor.width as i32;
            let h = editor.height as i32;

            // Calculate the new X/Y.
            let new_x = cursor_x;
            let new_y = cursor_y;

            // Update the editor.
            editor.x = new_x;
            editor.y = new_y;
            editor.width = (editor.x + w - new_x) as u32;
            editor.height = (editor.y + h - new_y) as u32;
        },
        EditorResizerElement::TopRight => {
            // Get the editor X/Y/W/H.
            let w = editor.width as i32;
            let h = editor.height as i32;

            // Calculate the new X/Y.
            let new_x = cursor_x - w;
            let new_y = cursor_y;

            // Update the editor.
            editor.x = new_x;
            editor.y = new_y;
            editor.width = (cursor_x - new_x) as u32;
            editor.height = (editor.y + h - new_y) as u32;
        },
        EditorResizerElement::BottomLeft => {
            // Get the editor X/Y/W/H.
            let w = editor.width as i32;
            let h = editor.height as i32;

            // Calculate the new X/Y.
            let new_x = cursor_x;
            let new_y = cursor_y - h;

            // Update the editor.
            editor.x = new_x;
            editor.y = new_y;
            editor.width = (editor.x + w - new_x) as u32;
            editor.height = (cursor_y - new_y) as u32;
        },
        EditorResizerElement::BottomRight => {
            // Get the editor X/Y/W/H.
            let w = editor.width as i32;
            let h = editor.height as i32;

            // Calculate the new X/Y.
            let new_x = cursor_x - w;
            let new_y = cursor_y - h;

            // Update the editor.
            editor.x = new_x;
            editor.y = new_y;
            editor.width = (cursor_x - new_x) as u32;
            editor.height = (cursor_y - new_y) as u32;
        },
    }
}

// Draws the editor resize points.
unsafe fn draw_editor_resize_points(ctx: &mut RegionSelectorContext, index: usize, editor: &EditorUsage, screen_h: i32) {
    // Get all the relevant points.
    let (x, y, w, h) = (editor.x, editor.y, editor.width as i32, editor.height as i32);
    let (x, y, x2, y2) = (x - 1, y - 1, x + w, y + h);
    let (mid_x, mid_y) = (x + w / 2, y + h / 2);

    // Call the light detector for each point.
    let light_detector = &mut ctx.light_detectors[index];
    let (
        top_left_light, top_right_light, bottom_left_light, bottom_right_light,
        center_light,
    ) = (
        light_detector.get_lightness(x as u32, y as u32),
        light_detector.get_lightness(x2 as u32, y as u32),
        light_detector.get_lightness(x as u32, y2 as u32),
        light_detector.get_lightness(x2 as u32, y2 as u32),
        light_detector.get_lightness(mid_x as u32, mid_y as u32),
    );

    // Get the half width and height.
    let half_sw = SELECTOR_WIDTH / 2;
    let half_sh = SELECTOR_HEIGHT / 2;

    // Handle the centre point.
    if w > 25 && h > 25 {
        // Get the color based on the lightness.
        let color_f = if center_light { 0.0 } else { 1.0 };

        // Draw the center point.
        gl::Scissor(mid_x - half_sw, screen_h - mid_y - half_sh, SELECTOR_WIDTH, SELECTOR_HEIGHT);
        gl::Enable(gl::SCISSOR_TEST);
        gl::ClearColor(color_f, color_f, color_f, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
        gl::Disable(gl::SCISSOR_TEST);    
    }

    // Handle the rest of the points.
    let points = [
        (x, y, top_left_light),
        (x2, y, top_right_light),
        (x, y2, bottom_left_light),
        (x2, y2, bottom_right_light),
    ];
    for (x, y, light) in points.iter() {
        // Get the color based on the lightness.
        let color_f = if *light { 0.0 } else { 1.0 };

        // Draw the point.
        gl::Scissor(*x - half_sw, screen_h - *y - half_sh, SELECTOR_WIDTH, SELECTOR_HEIGHT);
        gl::Enable(gl::SCISSOR_TEST);
        gl::ClearColor(color_f, color_f, color_f, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
        gl::Disable(gl::SCISSOR_TEST);
    }
}

// Render lines for the editors.
pub unsafe fn render_editor_resize_lines(ctx: &mut RegionSelectorContext, index: usize, screen_h: i32) {
    // Get the editors active on the display specified.
    let (w_texture, h_texture) = (ctx.striped_tex_w.texture, ctx.striped_tex_h.texture);
    let ctx2 = unsafe { &mut *(ctx as *mut _) };
    let editors = get_visible_editors(ctx2, index);

    // Bind the framebuffer to the vertical striped texture.
    gl::FramebufferTexture2D(
        gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0,
        gl::TEXTURE_2D, h_texture, 0,
    );

    // Render the vertical lines.
    for (_, editor) in &editors {
        // Get the editor X/Y/W/H.
        let x = editor.x;
        let y = editor.y;
        let w = editor.width as i32;
        let h = editor.height as i32;

        // Blit the left and right lines.
        gl::BlitFramebuffer(
            0, 0, 1, h,
            x - 1, screen_h - y, x, screen_h - y - h,
            gl::COLOR_BUFFER_BIT, gl::NEAREST,
        );
        gl::BlitFramebuffer(
            0, 0, 1, h,
            x + w, screen_h - y, x + w + 1, screen_h - y - h,
            gl::COLOR_BUFFER_BIT, gl::NEAREST,
        );
    }

    // Bind the framebuffer to the horizontal striped texture.
    gl::FramebufferTexture2D(
        gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0,
        gl::TEXTURE_2D, w_texture, 0,
    );

    // Render the horizontal lines.
    for (_, editor) in &editors {
        // Get the editor X/Y/W/H.
        let x = editor.x;
        let y = editor.y;
        let w = editor.width as i32;
        let h = editor.height as i32;

        // Blit the top and bottom lines.
        gl::BlitFramebuffer(
            0, 0, w, 1,
            x, screen_h - y, x + w, screen_h - y - 1,
            gl::COLOR_BUFFER_BIT, gl::NEAREST,
        );
        gl::BlitFramebuffer(
            0, 0, w, 1,
            x, screen_h - y - h, x + w, screen_h - y - h - 1,
            gl::COLOR_BUFFER_BIT, gl::NEAREST,
        );
    }

    // Render the selection points for the editors.
    for (_, editor) in editors {
        draw_editor_resize_points(ctx, index, editor, screen_h);
    }
}

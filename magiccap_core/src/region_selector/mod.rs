mod color_box;
mod color_picker;
mod editor_resizers;
mod engine;
mod event_loop_handler;
mod gl_abstractions;
mod light_detector;
mod texture_pack;
mod ui_renderer;
mod image_manipulation_simd;
mod editors;
mod magnifier;
mod menu_bar;
mod region_selected;
mod window_find;
mod window_line;

use std::sync::atomic::{AtomicBool, Ordering};
use xcap::{Monitor, Window};
use crate::database::get_config_option;
use self::engine::RegionSelectorSetup;

// Export out the texture pack preloader.
pub use texture_pack::preload_textures;

// Only one selector can be open at a time.
static SELECTOR_OPENED: AtomicBool = AtomicBool::new(false);

// Defines a region.
pub struct Region {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

// Defines the result of a region capture.
pub struct RegionCapture {
    pub monitor: Monitor,
    pub windows: Vec<Window>,
    pub relative_region: Region,
    pub image: image::RgbaImage,
}

// Opens the region selector. This is the one API that is exposed to the outside world.
pub fn open_region_selector(show_editors: bool) -> Option<RegionCapture> {
    // Check if the selector is already open.
    if SELECTOR_OPENED.swap(true, Ordering::Relaxed) {
        return None;
    }

    // Get all the monitors.
    let mut monitors = match Monitor::all() {
        Ok(monitors) => monitors,
        Err(_) => {
            SELECTOR_OPENED.store(false, Ordering::Relaxed);
            return None;
        },
    };

    // Get all the windows.
    let windows = match Window::all() {
        Ok(windows) => windows,
        Err(_) => {
            SELECTOR_OPENED.store(false, Ordering::Relaxed);
            return None;
        },
    };

    // Pop off the last monitor for now. That is because we do not need to
    // make a thread for screenshotting the last monitor since we can use
    // the current thread.
    let last_monitor = match monitors.pop() {
        Some(m) => m,
        None => {
            // I have zero clue how you got here without a display. Return though.
            SELECTOR_OPENED.store(false, Ordering::Relaxed);
            return None;
        },
    };

    // Build the threads to capture the images.
    let mut threads = Vec::with_capacity(monitors.len());
    for monitor in &monitors {
        let monitor = monitor.clone();
        threads.push(std::thread::spawn(move || {
            match monitor.capture_image() {
                Ok(image) => Some(image),
                Err(_) => None,
            }
        }));
    }

    // Capture the last monitor in the current thread.
    let last_image = match last_monitor.capture_image() {
        Ok(image) => image,
        Err(_) => {
            SELECTOR_OPENED.store(false, Ordering::Relaxed);
            return None;
        },
    };

    // Push the monitor back onto the list.
    monitors.push(last_monitor);

    // Figure out if we should show the magnifier. Do it here since the other threads can work while we are doing this.
    let show_magnifier = get_config_option("show_magnifier").
        unwrap_or(serde_json::Value::Bool(true)).as_bool().unwrap();

    // Figure out the default color for the editors. Do it here since the other threads can work while we are doing this.
    let mut default_color: (u8, u8, u8) = (u8::MAX, 0, 0);
    if let Some(color_result) = get_config_option("default_editor_color") {
        // Check this is an array.
        if let Some(color_array) = color_result.as_array() {
            // Check the length of the array.
            if color_array.len() == 3 {
                // Check the values of the array.
                if let (Some(r), Some(g), Some(b)) = (color_array[0].as_u64(), color_array[1].as_u64(), color_array[2].as_u64()) {
                    // Check the values are in the correct range.
                    if r <= u8::MAX as u64 && g <= u8::MAX as u64 && b <= u8::MAX as u64 {
                        default_color = (r as u8, g as u8, b as u8);
                    }
                }
            }
        }
    }

    // Wait for all the threads to finish.
    let mut screenshots = Vec::with_capacity(threads.len() + 1);
    for thread in threads {
        screenshots.push(match thread.join().unwrap() {
            Some(image) => image,
            None => {
                SELECTOR_OPENED.store(false, Ordering::Relaxed);
                return None;
            },
        });
    }
    screenshots.push(last_image);

    // Call the engine.
    let result = engine::invoke(Box::new(RegionSelectorSetup {
        monitors, windows, show_editors, show_magnifier, default_color,
    }), &mut screenshots);

    // Make sure to set the selector opened to false.
    SELECTOR_OPENED.store(false, Ordering::Relaxed);

    // Return the result.
    result
}

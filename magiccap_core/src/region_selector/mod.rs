mod engine;

use std::sync::atomic::{AtomicBool, Ordering};
use xcap::{Monitor, Window};
use self::engine::RegionSelectorSetup;

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
    pub region: Region,
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

    // Wait for all the threads to finish.
    let mut screenshots = Vec::with_capacity(threads.len());
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
        monitors, windows, show_editors, images: screenshots,
    }));

    // Make sure to set the selector opened to false.
    SELECTOR_OPENED.store(false, Ordering::Relaxed);

    // Return the result.
    result
}

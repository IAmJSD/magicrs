mod engine;

use std::{ffi::CString, sync::atomic::{AtomicBool, Ordering}};
use include_dir::{include_dir, Dir};
use xcap::{Monitor, Window};

// Only one selector can be open at a time.
static SELECTOR_OPENED: AtomicBool = AtomicBool::new(false);

// Defines the shaders folder.
static SHADERS_FOLDER: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/region_selector/fragments");

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

// Handles the region result.
fn handle_region_result_t(
    ptr: *const self::engine::region_result_t,
    monitors: Vec<Monitor>
) -> RegionCapture {
    let rusty = unsafe { ptr.as_ref() }.unwrap();

    // Get RGBA as a image::RgbaImage.
    let image = image::RgbaImage::from_raw(
        rusty.w, rusty.h, unsafe {
            std::slice::from_raw_parts(rusty.rgba, rusty.rgba_len)
        }.to_vec()
    ).unwrap();

    // Get the monitor.
    let monitor = monitors[rusty.display_index as usize].clone();

    // Return the result.
    RegionCapture {
        monitor,
        region: Region {
            x: rusty.coordinate.x,
            y: rusty.coordinate.y,
            width: rusty.w,
            height: rusty.h,
        },
        image,
    }
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

    // Get the fragments.
    let mut fragments = SHADERS_FOLDER.files().map(
        |f| self::engine::gl_fragment_t {
            data: Some(CString::new(f.contents()).unwrap()),
            name: Some(CString::new(
                f.path().file_name().unwrap().to_str().unwrap().split(".").
                    next().unwrap()
            ).unwrap()),
            gl_object: 0,
        }
    ).collect::<Vec<_>>();
    fragments.push(self::engine::gl_fragment_t {
        data: None, name: None, gl_object: 0,
    });

    // Call the engine and return the result.
    let coordinates = monitors.iter().map(
        |m| self::engine::region_coordinate_t {
            x: m.x(),
            y: m.y(),
        }
    ).collect::<Vec<_>>();
    let screenshot_pointers = screenshots.iter().map(|s| self::engine::screenshot_t {
        data: s.as_ptr(),
        w: s.width() as usize,
        h: s.height() as usize,
    }).collect::<Vec<_>>();
    let result = unsafe {
        self::engine::region_selector_open(
            monitors.len(), coordinates.as_ptr(),
            screenshot_pointers.as_ptr(), fragments.as_ptr(),
            show_editors
        )
    };
    drop(coordinates);
    drop(screenshots);
    drop(screenshot_pointers);
    drop(fragments);

    // Check if the result is null.
    SELECTOR_OPENED.store(false, Ordering::Relaxed);
    match result.is_null() {
        true => None,
        false => Some(handle_region_result_t(result, monitors))
    }
}

use std::{io::Cursor, path::PathBuf};
use crate::{
    clipboard_actions::{self, CaptureFile}, database, notification, ocr,
    region_selector::open_region_selector, search_indexing, statics::run_thread,
    uploaders, utils::get_filename,
};
use image::{DynamicImage, RgbaImage};
use xcap::{Monitor, Window};

// Handles writing captures to the filesystem, uploading them to the internet, and injecting them into the clipboard.
// Also handles any errors within the process.
fn post_capture_flow(
    ext: &str, notification_content: &str, data: Vec<u8>,
    thread_callback: Option<Box<dyn FnOnce(i64) + Send>>,
) {
    // Generate a file name.
    let filename = match get_filename(match database::get_config_option("filename_format") {
        Some(format) => match format.as_str() {
            Some(f) => Some(f.to_string()),
            None => return notification::send_dialog_message(
                "The filename format in the configuration database is not a valid string. Please file a bug!",
            ),
        },
        None => None,
    }) {
        Ok(filename) => format!("{}.{}", filename, ext),
        Err(e) => {
            notification::send_dialog_message(&e);
            return;
        },
    };

    // Handle writing to the filesystem.
    let save_capture = match database::get_config_option("save_capture") {
        Some(x) => x.as_bool().unwrap_or(false),

        // By default, save the capture.
        None => true,
    };
    let mut fp_result = "".to_string();
    if save_capture {
        // Get the folder to write to.
        let folder_path = match database::get_config_option("folder_path") {
            Some(folder_path) => {
                match folder_path.as_str() {
                    Some(fp) => PathBuf::from(fp),
                    None => return notification::send_dialog_message(
                        "The folder path in the configuration database is not a valid string. Please file a bug!",
                    ),
                }
            },
            None => {
                // Get the ~/Pictures/MagicCap folder.
                home::home_dir().unwrap().join("Pictures").join("MagicCap")
            },
        };

        // Create the folder if it does not exist.
        match std::fs::create_dir_all(&folder_path) {
            Ok(_) => {},
            Err(e) => {
                // Log this as a capture failure.
                database::insert_failed_capture(&filename, None);

                // Notify the user and stop the flow.
                notification::send_dialog_message(&format!("Failed to create the folder: {}", e));
                return;
            },
        }

        // Get the file path.
        let fp = folder_path.join(&filename);
        drop(folder_path);

        // Write the file.
        match std::fs::write(&fp, &data) {
            Ok(_) => {},
            Err(e) => {
                // Log this as a capture failure.
                database::insert_failed_capture(&filename, None);

                // Notify the user and stop the flow.
                notification::send_dialog_message(&format!("Failed to write the file to the filesystem: {}", e));
                return;
            },
        }
        fp_result = fp.to_str().unwrap().to_string();
    }

    // Get the uploader type.
    let uploader_type = match database::get_config_option("uploader_type") {
        Some(type_or_null) => {
            match type_or_null.as_str() {
                Some(type_) => type_.to_string(),
                None => "imgur".to_string(),
            }
        },

        // Defaults to uploading to imgur.
        None => "imgur".to_string(),
    };

    // Check if we should upload.
    let upload_capture = match database::get_config_option("upload_capture") {
        Some(x) => x.as_bool().unwrap_or(false),

        // By default, upload the capture.
        None => true,
    };

    // If uploading is on, upload the file.
    let mut url_result: Option<String> = None;
    let mut capture_success = true;
    if upload_capture {
        match uploaders::call_uploader(
            &uploader_type, Box::new(Cursor::new(data.clone())), filename.as_str(),
        ) {
            Ok(u) => url_result = Some(u),
            Err(e) => {
                // This is a capture failure.
                capture_success = false;

                // Log this as a capture failure.
                database::insert_failed_capture(&filename, Some(&fp_result));

                // Notify the user but do not stop the flow for uploader errors.
                notification::send_dialog_message(&e);
            },
        }
    }

    // Handle the clipboard flow.
    let scratch_str: String;
    let url_str = match url_result {
        Some(url) => {
            scratch_str = url;
            Some(scratch_str.as_str())
        },
        None => None,
    };
    clipboard_actions::handle_clipboard_action(match save_capture {
        true => Some(&fp_result),
        false => None,
    }, url_str, Some(CaptureFile {
        file_name: filename.clone(),
        content: data,
    }));

    // If this capture was successful, push a notification and write to the database.
    if capture_success {
        // The order here matters. The notification can block forever on some systems.
        let capture_id = database::insert_successful_capture(filename.as_str(), Some(&fp_result), url_str);
        if let Some(thread_callback) = thread_callback {
            run_thread(move || thread_callback(capture_id));
        }
        notification::send_notification(notification_content, url_str, match save_capture {
            true => Some(&fp_result),
            false => None,
        });
    }
}

// Handles search indexing a RGBA region.
fn search_indexing_rgba(image: RgbaImage, windows: Vec<Window>, capture_id: i64) {
    // Convert the image to a RGB image.
    let image = DynamicImage::ImageRgba8(image).to_rgb8();
    let text = ocr::scan_text(image);

    // Insert the capture into the index.
    search_indexing::insert_capture(capture_id, text, windows.iter().map(
        |w| w.title().to_string()).collect());
}
macro_rules! search_indexing_rgba_callback {
    ($rgba:ident, $windows:ident) => {
        Some(Box::new(move |id| search_indexing_rgba($rgba, $windows, id)))
    };
}

// Handle doing region captures.
pub fn region_capture() {
    let (image, windows) = match open_region_selector(true) {
        Some(result) => (result.image, result.windows),
        None => return,
    };

    // Convert the result to a PNG.
    let mut data: Vec<u8> = Vec::new();
    image.write_to(&mut Cursor::new(&mut data), image::ImageFormat::Png).unwrap();

    post_capture_flow(
        "png", "Region capture successful.", data,
        search_indexing_rgba_callback!(image, windows),
    )
}

// Handle doing GIF captures.
pub fn gif_capture() {
    // TODO
}

// Handle doing MP4 captures.
pub fn video_capture() {
    // TODO
}

// Handles uploading files from the clipboard.
pub fn clipboard_capture() {
    // TODO
}

// Handle doing fullscreen captures. This will capture all of the displays in order.
pub fn fullscreen_capture() {
    let monitors = Monitor::all().unwrap();

    // Find the lowest/highest X and Y.
    let mut lowest_x = 0;
    let mut lowest_y = 0;
    let mut highest_x = 0;
    let mut highest_y = 0;
    for monitor in &monitors {
        let x = monitor.x();
        let y = monitor.y();
        let w = monitor.width() * monitor.scale_factor() as u32;
        let h = monitor.height() * monitor.scale_factor() as u32;

        if lowest_x > x { lowest_x = x }
        if lowest_y > y { lowest_y = y }
        let top_right_x = x + w as i32;
        let bottom_left_y = y + h as i32;
        if top_right_x > highest_x { highest_x = top_right_x }
        if bottom_left_y > highest_y { highest_y = bottom_left_y }
    }

    // Handle normalising the smallest values.
    let x_transform = lowest_x * -1;
    let y_transform = lowest_y * -1;

    // Make the canvas it is all in.
    let width = ((highest_x - lowest_x) + x_transform) as u32;
    let height = ((highest_y - lowest_y) + y_transform) as u32;
    let mut canvas = image::RgbaImage::new(width, height);

    for monitor in monitors {
        // Capture the display.
        let capture = monitor.capture_image().unwrap();

        // Get the position of the display.
        let x = monitor.x() + x_transform;
        let y = monitor.y() + y_transform;

        // Draw the capture onto the canvas.
        image::imageops::overlay(&mut canvas, &capture, x as i64, y as i64);
    }

    // Convert the canvas to a vector.
    let mut vec: Vec<u8> = Vec::new();
    canvas.write_to(&mut Cursor::new(&mut vec), image::ImageFormat::Png).unwrap();

    // Handle the post capture flow.
    let windows = Window::all().unwrap();
    post_capture_flow(
        "png", "Fullscreen capture successful.", vec,
        search_indexing_rgba_callback!(canvas, windows),
    );
}

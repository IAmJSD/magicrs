use crate::clipboard_actions;
use crate::notification;
use crate::statics::run_thread;
use crate::uploaders;
use crate::database;
use crate::mainthread::main_thread_async;

// Wraps the safe call to the uploader with a C-compatible function.
unsafe extern "C" fn call_uploader(name_ptr: *const u8, name_len: usize, path_ptr: *const u8, path_len: usize) {
    // Get the name and path.
    let name = std::slice::from_raw_parts(name_ptr, name_len);
    let path = std::slice::from_raw_parts(path_ptr, path_len);

    // Convert the name and path to strings.
    let name = std::str::from_utf8(name).unwrap().to_string();
    let path_str = std::str::from_utf8(path).unwrap().to_string();

    // Call the uploader in a new thread.
    run_thread(move || {
        // Create a reader for the path.
        let reader: Box<dyn std::io::Read + Send + Sync> = Box::new(
            std::fs::File::open(&path_str).unwrap()
        );

        let filename = path_str.split(path::MAIN_SEPARATOR).last().unwrap();
        match uploaders::call_uploader(&name, reader, filename) {
            Ok(url) => {
                // Write a successful "capture".
                notification::send_notification(
                    "The file was uploaded successfully.", Some(&url), None,
                );
                database::insert_successful_capture(filename, Some(&path_str), Some(&url));

                // Handle the clipboard flow.
                clipboard_actions::handle_clipboard_action(Some(&path_str), Some(&url), None);
            }
            Err(e) => {
                // Write a failed "capture".
                database::insert_failed_capture(filename, Some(&path_str));
                notification::send_dialog_message(&e);
            },
        }
    });
}

// Defines the quit handler.
extern "C" fn quit_handler() {
    run_thread(|| {
        // Check the kill switch.
        if crate::statics::KILL_SWITCH.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }

        // Call the application unload function.
        crate::unload::application_unload();

        // Quit the application.
        std::process::exit(0);
    });
}

// Defines the config open handler.
extern "C" fn config_open() {
    // Check the kill switch.
    if crate::statics::KILL_SWITCH.load(std::sync::atomic::Ordering::Relaxed) {
        return;
    }

    // Open the config.
    crate::config::open_config();
}

#[cfg(target_os = "macos")]
use std::ffi::c_int;
use std::path;

#[cfg(target_os = "macos")]
extern "C" fn capture_type_clicked(type_: c_int) {
    run_thread(move || {
        // Check the kill switch.
        if crate::statics::KILL_SWITCH.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }

        match type_ {
            0 => crate::capture::region_capture(),
            1 => crate::capture::fullscreen_capture(),
            2 => crate::capture::gif_capture(),
            3 => crate::capture::video_capture(),
            4 => crate::capture::clipboard_capture(),
            _ => panic!("Unknown capture type."),
        }
    });
}

// Defines the function to create the tray on macOS.
#[cfg(target_os = "macos")]
fn tray_main_thread() {
    use std::ffi::CString;
    use crate::macos_delegate;
    use crate::macos;
    use objc::{class, runtime::Object, sel, msg_send, sel_impl};
    use objc_id::{Id, ShareId};

    // Get the tray ID.
    let app = macos_delegate::app();
    let mut tray_id = app.delegate.tray_id.lock().unwrap();

    // Delete the tray if it exists.
    if tray_id.is_some() {
        let tray_id_curently = tray_id.take().unwrap();
        unsafe {
            let system_id: ShareId<Object> = msg_send![class!(NSStatusBar), systemStatusBar];
            let _: () = msg_send![system_id, removeStatusItem: tray_id_curently];
        };
    }

    // Get the default uploader.
    let default_uploader = match database::get_config_option("uploader_type") {
        // If the default option is set, use it.
        Some(option_json) => {
            let cpy = option_json.clone();
            cpy.to_string()
        },

        // imgur is the default uploader within the application if none is set.
        None => "imgur".to_owned(),
    };

    // Create all the uploader items.
    let mut uploader_items: Vec<macos::UploaderItem> =  Vec::with_capacity(uploaders::UPLOADERS.len());
    for (id, uploader) in uploaders::UPLOADERS.iter() {
        // Check if this is the default uploader.
        let is_default = id == &default_uploader;

        // Get all of the configuration options for this uploader from the database.
        let db_options = database::get_uploader_config_items(id.as_str());

        // Check that all required options are set.
        let mut all_required_options_set = true;
        for (name, option) in &uploader.options {
            if option.is_required() && !db_options.contains_key(name) {
                all_required_options_set = false;
                break;
            }
        }

        // If all required options are set, add the uploader to the list.
        if all_required_options_set {
            // I couldn't figure out how to get this to work without a memory leak. Fuck it, it is
            // a tiny amount of memory per reload which will be rare.
            let name = Box::leak(Box::new(CString::new(uploader.name.clone()).unwrap()));
            let id = Box::leak(Box::new(CString::new(id.as_str()).unwrap()));

            // Create the uploader item.
            let item = macos::UploaderItem {
                name: name.as_ptr(),
                id: id.as_ptr(),
                default_uploader: is_default,
            };
            uploader_items.push(item);
        }
    }

    // Defines the capture items.
    let capture_items: [macos::CaptureType; 5] = [
        macos::CaptureType {
            name: Box::leak(Box::new(std::ffi::CString::new("Region Capture").unwrap())).as_ptr(),
            type_: 0,
        },
        macos::CaptureType {
            name: Box::leak(Box::new(std::ffi::CString::new("Fullscreen Capture").unwrap())).as_ptr(),
            type_: 1,
        },
        macos::CaptureType {
            name: Box::leak(Box::new(std::ffi::CString::new("GIF Capture").unwrap())).as_ptr(),
            type_: 2,
        },
        macos::CaptureType {
            name: Box::leak(Box::new(std::ffi::CString::new("Video Capture").unwrap())).as_ptr(),
            type_: 3,
        },
        macos::CaptureType {
            name: Box::leak(Box::new(std::ffi::CString::new("Clipboard Capture").unwrap())).as_ptr(),
            type_: 4,
        },
    ];

    // Create the tray.
    let tray_id_usize = unsafe {
        macos::create_tray(
            uploader_items.as_ptr(), uploader_items.len(),
            capture_items.as_ptr(), capture_items.len(),
            call_uploader, quit_handler, capture_type_clicked,
            config_open,
        )
    };
    drop(capture_items);

    // Turn the tray ID into an object.
    let id: Id<Object> = unsafe { Id::from_ptr(tray_id_usize as *mut Object) };

    // Set the tray ID.
    *tray_id = Some(id);
}

// (Re)-loads the tray. Required when there are changes to initialization, uploaders, or shortcuts.
pub fn load_tray() {
    // Call the main thread async function with a handler that is on the main thread. We do not
    // care if this is instant, so no need to block.
    main_thread_async(tray_main_thread);
}

use crate::clipboard_actions;
use crate::notification;
use crate::statics::run_thread;
use crate::uploaders;
use crate::database;
use crate::mainthread::main_thread_async;

// TODO: update tray on Linux
// TODO: reminder to add success message to webview, in the wrong place I know, but hey, I run this shit

// Defines the safe call which is wrapped.
fn safely_upload(name: &str, path: &str) {
    // Create a reader for the path.
    let reader: Box<dyn std::io::Read + Send + Sync> = Box::new(
        std::fs::File::open(path).unwrap()
    );

    let filename = path.split(path::MAIN_SEPARATOR).last().unwrap();
    match uploaders::call_uploader(&name, reader, filename) {
        Ok(url) => {
            // Write a successful "capture".
            notification::send_notification(
                "The file was uploaded successfully.", Some(&url), None,
            );
            database::insert_successful_capture(filename, Some(path), Some(&url));

            // Handle the clipboard flow.
            clipboard_actions::handle_clipboard_action(Some(path), Some(&url), None);
        }
        Err(e) => {
            // Write a failed "capture".
            database::insert_failed_capture(filename, Some(path));
            notification::send_dialog_message(&e);
        },
    };
}

// Wraps the safe call to the uploader with a C-compatible function.
#[no_mangle]
#[cfg(target_os = "macos")]
unsafe extern "C" fn call_uploader(name_ptr: *const u8, name_len: usize, path_ptr: *const u8, path_len: usize) {
    // Get the name and path.
    let name = std::slice::from_raw_parts(name_ptr, name_len);
    let path = std::slice::from_raw_parts(path_ptr, path_len);

    // Convert the name and path to strings.
    let name = std::str::from_utf8(name).unwrap().to_string();
    let path_str = std::str::from_utf8(path).unwrap().to_string();

    // Call the uploader in a new thread.
    run_thread(move || safely_upload(&name, &path_str));
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

// Handles tray_icon imports which are used as types on Linux.
#[cfg(target_os = "linux")]
use tray_icon::menu::{Menu, MenuItem};

// Handles a muda event on Linux.
#[cfg(target_os = "linux")]
use muda::MenuEvent;

// We need HashMap's on Linux for the menu items.
#[cfg(target_os = "linux")]
use std::collections::HashMap;

// Defines a map of menu items to handlers on Linux.
#[cfg(target_os = "linux")]
static mut MENU_ITEM_HANDLERS: Option<HashMap<String, Box<dyn Fn() + 'static>>> = None;

// Handles the menu events on Linux.
#[cfg(target_os = "linux")]
fn menu_event(event: MenuEvent) {
    let m = unsafe { MENU_ITEM_HANDLERS.as_ref().unwrap() };
    if let Some(hn) = m.get(event.id.0.as_str()) {
        hn();
    }
}

// Create a individual menu item on Linux.
#[cfg(target_os = "linux")]
fn new_menu_item(name: &str, enabled: bool, hn: Box<dyn Fn() + 'static>) -> MenuItem {
    // Get the menu item handlers. We don't need to be thread safe since this is only called on the main thread.
    let menu_item_handlers = unsafe { MENU_ITEM_HANDLERS.as_mut().unwrap() };

    // Create the menu item.
    let item = tray_icon::menu::MenuItem::new(name, enabled, None);

    // Add the handler to the map.
    let id = item.id().0.clone();
    menu_item_handlers.insert(id, hn);

    // Return a reference to the item.
    item
}

// Defines a macro to make a reference to a menu item on Linux.
#[cfg(target_os = "linux")]
macro_rules! menu_item {
    ($name:expr, $enabled:expr, $hn:expr) => {
        &new_menu_item($name, $enabled, $hn)
    };
}

// Defines a macro to make a separator on Linux.
#[cfg(target_os = "linux")]
macro_rules! separator {
    () => {
        &muda::PredefinedMenuItem::separator()
    };
}

// Handle doing a file path upload on Linux.
#[cfg(target_os = "linux")]
fn do_upload_fp(uploader_id: &str) {
    // Open the file dialog.
    let file_path = match native_dialog::FileDialog::new().show_open_single_file() {
        Ok(Some(fp)) => fp,
        _ => return,
    }.as_path().to_str().unwrap().to_string();

    // Run a thread to upload the file.
    let id_cpy = uploader_id.to_string();
    run_thread(move || safely_upload(&id_cpy, &file_path));
}

// Creates the menu items on Linux.
#[cfg(target_os = "linux")]
fn create_or_update_menu(menu: &mut Box<Menu>) {
    use muda::Submenu;

    // Wipe all menu items or make the map they all live in.
    match unsafe { MENU_ITEM_HANDLERS.as_mut() } {
        Some(m) => {
            let len = menu.items().len();
            for _ in 0..len {
                menu.remove_at(0);
            }
            m.clear();
        }
        None => {
            let map = HashMap::new();
            unsafe { MENU_ITEM_HANDLERS = Some(map); }
        }
    }

    // Add the uploaders to a submenu.
    let uploaders_menu = Submenu::new("Upload to...", true);
    for (id, uploader) in uploaders::UPLOADERS.iter() {
        // Check if this is the default uploader.
        let is_default = id == &database::get_config_option("uploader_type").unwrap_or_else(
            || serde_json::Value::String("imgur".to_owned()));

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
            // Get the label for the uploader.
            let label = format!("Upload to {}{}", uploader.name, if is_default { " (Default)" } else { "" });

            // Create the menu item.
            let id_cpy = id.clone();
            uploaders_menu.append(
                menu_item!(label.as_str(), true, Box::new(move || do_upload_fp(id_cpy.as_str()))),
            ).unwrap();
        }
    }

    // Add all of the items to the menu.
    menu.append_items(&[
        menu_item!("Region Capture", true, Box::new(|| run_thread(crate::capture::region_capture))),
        menu_item!("Fullscreen Capture", true, Box::new(|| run_thread(crate::capture::fullscreen_capture))),
        menu_item!("GIF Capture", true, Box::new(|| run_thread(crate::capture::gif_capture))),
        menu_item!("Video Capture", true, Box::new(|| run_thread(crate::capture::video_capture))),
        menu_item!("Clipboard Capture", true, Box::new(|| run_thread(crate::capture::clipboard_capture))),
        separator!(),
        &uploaders_menu,
        separator!(),
        menu_item!("Captures/Config", true, Box::new(|| config_open())),
        menu_item!("Quit", true, Box::new(|| quit_handler())),
    ]).unwrap();
}

// Defines the tray menu dynamic handler on Linux. This allows it to be upgraded over time without restarting the application.
#[cfg(target_os = "linux")]
fn local_dynamic_menu_handler(event: MenuEvent) {
    use crate::linux_shared::app;

    let current_hn = app().menu_event.read().unwrap().clone().unwrap();
    current_hn(event);
}

// Defines the function to create the tray on Linux.
#[cfg(target_os = "linux")]
fn tray_main_thread() {
    use crate::linux_shared::app;
    use tray_icon::{menu::MenuEvent, Icon, TrayIconBuilder, TrayIconEvent};

    // Defines the static taskbar icon.
    static TRAY_ICON: &[u8] = include_bytes!("../../assets/taskbar.png");

    // Turn the tray icon into RGBA.
    let rgba = image::load_from_memory(TRAY_ICON).unwrap().to_rgba8();

    // Set the event handler for this instance of the library.
    let mut event_handler_w = app().menu_event.write().unwrap();
    event_handler_w.replace(&menu_event);

    // Handle fetching the tray icon.
    let mut write_guard = app().tray_menu.write().unwrap();
    match write_guard.as_mut() {
        // Handle the tray already being loaded.
        Some(menu) => {
            // We just want to update the menu.
            create_or_update_menu(menu);
        },

        // Handle the first load of the application.
        None => {
            // Set the event handlers.
            MenuEvent::set_event_handler(Some(|event| {
                main_thread_async(move || local_dynamic_menu_handler(event));
            }));
            TrayIconEvent::set_event_handler(Some(move |_| {}));

            // Create the menu since this is first run.
            let mut menu = Box::new(Menu::new());
            create_or_update_menu(&mut menu);

            // Chuu chuu! Here comes the language abuse! We need to do this since this is a special deployment
            // method since this is a library that can be updated, and when it is updated we need to update the
            // menu.
            let menu_ref = &mut menu;
            let menu_ref_cpy = unsafe {
                std::mem::transmute::<&mut Box<Menu>, &'static mut Box<Menu>>(menu_ref)
            };

            // Create the tray icon since this is the first run. Tell Rust to leak it since even after
            // updates, we do not want to remove it, and we certainly do not want to drop it with the
            // abuse we just did.
            Box::leak(Box::new(TrayIconBuilder::new()
                .with_menu(menu)
                .with_icon(Icon::from_rgba(rgba.to_vec(), rgba.width(), rgba.height()).unwrap())
                .with_tooltip("MagicCap")
                .build()
                .unwrap()));

            // Update the write guard with our abused menu reference.
            write_guard.replace(menu_ref_cpy);
        },
    }
}

// (Re)-loads the tray. Required when there are changes to initialization, uploaders, or shortcuts.
pub fn load_tray() {
    // Call the main thread async function with a handler that is on the main thread. We do not
    // care if this is instant, so no need to block.
    main_thread_async(tray_main_thread);
}

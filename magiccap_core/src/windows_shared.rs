use crate::{hotkeys::HotkeyWrapper, mainthread::main_event_loop, reload, statics::run_thread};
use once_cell::sync::OnceCell;
use std::sync::RwLock;
use tray_icon::menu::{Menu, MenuEvent};
use windows::Win32::System::Threading::GetCurrentThreadId;

// Defines the structure for a shared application.
pub struct SharedApplication {
    pub main_thread_id: u32,
    pub wv_controller: Option<(webview2::Controller, native_windows_gui::Window)>,
    pub tray_menu: RwLock<Option<&'static mut Box<Menu>>>,
    pub menu_event: RwLock<Option<&'static dyn Fn(MenuEvent)>>,
    pub hotkey_wrapper: HotkeyWrapper,
}

// Defines the public variable.
static mut SHARED_APPLICATION: OnceCell<&'static mut SharedApplication> = OnceCell::new();

// Defines the shared application object.
pub fn app() -> &'static mut SharedApplication {
    unsafe { SHARED_APPLICATION.get_mut().unwrap() }
}

// The main entrypoint for setting up the application.
pub fn application_init() {
    // Set up native windows gui.
    native_windows_gui::init().unwrap();

    // Create the shared application box.
    let leaky_box = Box::leak(Box::new(SharedApplication {
        main_thread_id: unsafe { GetCurrentThreadId() },
        wv_controller: None,
        tray_menu: RwLock::new(None),
        menu_event: RwLock::new(None),
        hotkey_wrapper: HotkeyWrapper::new(),
    }));
    let ptr = leaky_box as *mut SharedApplication;
    unsafe {
        let _ = SHARED_APPLICATION.set(&mut *ptr);
    }

    // Set the MAGICCAP_INTERNAL_MEMORY_ADDRESS env var.
    std::env::set_var(
        "MAGICCAP_INTERNAL_MEMORY_ADDRESS",
        (ptr as usize).to_string(),
    );

    // In a thread, launch the application_reload function. This is because it can cause problems
    // if it blocks the main thread.
    run_thread(reload::application_reload);

    // Call the main event loop.
    main_event_loop();
}

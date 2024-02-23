use crate::{config::{MagicCapConfigDelegate, setup_window}, macos, reload, statics::run_thread};
use std::{cell::OnceCell, sync::{Mutex, RwLock}};
use cacao::{
    appkit::{window::{Window, WindowDelegate}, App, AppDelegate},
    webview::WebView,
};
use objc::runtime::Object;
use objc_id::Id;

pub struct ConfigWindow {
    pub content: WebView<MagicCapConfigDelegate>
}

impl WindowDelegate for ConfigWindow {
    const NAME: &'static str = "MagicCapConfigWindowDelegate";

    fn did_load(&mut self, window: Window) {
        window.set_content_view(&self.content);
        setup_window(&window);

        // Make sure it is shown in the dock and focused.
        unsafe { macos::transform_process_type(true); };

        // Make sure the window is focused.
        window.make_key_and_order_front();
    }

    fn should_close(&self) -> bool {
        app().delegate.webview.write().unwrap().take();
        unsafe { macos::transform_process_type(false); };
        true
    }
}

#[derive(Default)]
pub struct MagicCapAppDelegate {
    pub tray_id: Mutex<Option<Id<Object>>>,
    pub webview: RwLock<Option<Window<ConfigWindow>>>,
}

impl AppDelegate for MagicCapAppDelegate {
    fn did_finish_launching(&self) {
        // Take a screenshot to trigger the permission checks in macOS.
        #[cfg(target_os = "macos")]
        match xcap::Monitor::all() {
            Ok(monitors) => {
                if monitors.len() != 0 {
                    let monitor = &monitors[0];
                    let _ = monitor.capture_image();
                }
            },
            Err(_) => (),
        }

        // Call the notification center hook handler.
        unsafe { macos::hook_notif_center() };

        // Hide in the dock.
        unsafe { macos::transform_process_type(false); };

        // In a thread, launch the application_reload function. This is because
        // it can cause problems if it blocks the main thread.
        run_thread(reload::application_reload);
    }
}

// Get the global app reference in a safe way after initialization.
#[inline]
pub fn app() -> &'static App<MagicCapAppDelegate> {
    unsafe { APP.get().unwrap() }
}

// This is global because it is used as the source of truth in macOS.
static mut APP: OnceCell<App<MagicCapAppDelegate>> = OnceCell::new();

pub unsafe fn application_init() {
    // Spawn the application and store it in APP.
    APP.set(
        App::new("org.magiccap.magiccap", MagicCapAppDelegate::default())
    ).unwrap();
    app().run();
}

pub unsafe fn application_hydrate() {
    // Check if APP is set. If it is, application_init has already been called.
    if APP.get().is_some() {
        // We are hydrated. Return now.
        return;
    }

    // Get the app from Objective-C and store it in APP.
    panic!("TODO: hydrate");
}

use std::sync::{mpsc::{self, Sender}, RwLock};
use once_cell::sync::OnceCell;
use webkit2gtk::WebView;
use tray_icon::TrayIcon;
use crate::{reload, statics::run_thread};

// Defines the callback type.
type Callback = Box<dyn FnOnce() + Send + 'static>;

// Defines a wrapper to fake something being safe to send.
pub struct FakeSend<T> {
    pub value: T,
}
unsafe impl<T> Send for FakeSend<T> {}
unsafe impl<T> Sync for FakeSend<T> {}

// Defines the structure for a shared application.
struct SharedApplication {
    pub main_thread_writer: Sender<Callback>,
    pub webview: RwLock<Option<FakeSend<WebView>>>,
    pub tray_icon: RwLock<Option<TrayIcon>>,
}

// Defines the public variable.
static mut SHARED_APPLICATION: OnceCell<&'static mut SharedApplication> = OnceCell::new();

// Defines the shared application object.
pub fn app() -> &'static mut SharedApplication { 
    unsafe { SHARED_APPLICATION.get_mut().unwrap() }
}

// The main entrypoint for setting up the application.
pub fn application_init() {
    // Create a channel.
    let (tx, rx) = mpsc::channel::<Callback>();

    // Create the shared application box.
    let leaky_box = Box::leak(Box::new(SharedApplication {
        main_thread_writer: tx,
        webview: RwLock::new(None),
        tray_icon: RwLock::new(None),
    }));
    let ptr = leaky_box as *mut SharedApplication;
    unsafe { SHARED_APPLICATION.set(&mut *ptr); }

    // Set the MAGICCAP_INTERNAL_MEMORY_ADDRESS env var.
    std::env::set_var("MAGICCAP_INTERNAL_MEMORY_ADDRESS", (ptr as usize).to_string());

    // In a thread, launch the application_reload function. This is because
    // it can cause problems if it blocks the main thread.
    run_thread(reload::application_reload);

    // Keep consuming the main thread.
    for cb in rx { cb(); }
}

pub fn application_hydrate() {
    unsafe {
        // Check if SHARED_APPLICATION is set. If it is, application_init has already been called.
        if SHARED_APPLICATION.get().is_some() {
            // We are hydrated. Return now.
            return;
        }
    }

    // Get the MAGICCAP_INTERNAL_MEMORY_ADDRESS env var.
    let mem_addr = std::env::var("MAGICCAP_INTERNAL_MEMORY_ADDRESS").unwrap()
        .parse::<usize>().unwrap();

    // Turn it into a pointer.
    unsafe {
        SHARED_APPLICATION.set(
            (mem_addr as *mut SharedApplication).as_mut().unwrap()
        );
    }

    // In a thread, launch the application_reload function. This is because
    // it can cause problems if it blocks the main thread.
    run_thread(reload::application_reload);
}
